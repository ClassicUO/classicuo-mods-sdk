/* Command buffer: streamed straight into the flatcc builder as the mod records
 * commands (a CommandBuffer root with one open cmds union vector), so nothing is
 * buffered twice.
 *
 * Entities: a spawn returns CUO_PENDING | temp id. Temp ids are unique for the mod's
 * lifetime, so a placeholder kept across runs still names one spawn; mod_spawned
 * records the real ids, and cuo_resolve swaps them in. In a command an unresolved
 * placeholder is the wire ref -(temp_id) - 1 (abi/mod-abi.fbs). */
#include <stdlib.h>
#include <string.h>

#include "internal.h"

struct cuo_cmds {
    flatcc_builder_t *B;
    size_t count;
};

static cuo_cmds instance;
static uint32_t next_temp;

/* real id + 1 per temp id (0 = not resolved yet); temp ids are dense from 0. */
static uint64_t *resolved;
static size_t nresolved;

cuo_cmds *cuo__cmds_instance(void)
{
    return &instance;
}

cuo_entity cuo_resolve(cuo_entity e)
{
    if (!(e & CUO_PENDING))
        return e;
    uint32_t temp = (uint32_t)e;
    return temp < nresolved && resolved[temp] ? resolved[temp] - 1 : e;
}

static int64_t wire_ref(cuo_entity e)
{
    e = cuo_resolve(e);
    return (e & CUO_PENDING) ? -(int64_t)(uint32_t)e - 1 : (int64_t)e;
}

void cuo__cmds_begin(cuo_cmds *c)
{
    c->B = cuo__builder();
    c->count = 0;
}

static flatcc_builder_t *open_cmd(cuo_cmds *c)
{
    if (c->count++ == 0) {
        ModAbi_CommandBuffer_start_as_root(c->B);
        ModAbi_CommandBuffer_cmds_start(c->B);
    }
    return c->B;
}

uint64_t cuo__cmds_finish(cuo_cmds *c)
{
    if (c->count == 0)
        return 0;
    ModAbi_CommandBuffer_cmds_end(c->B);
    ModAbi_CommandBuffer_end_as_root(c->B);
    return cuo__pack_builder(c->B);
}

size_t cuo_cmds_len(const cuo_cmds *c)
{
    return c->count;
}

static ModAbi_CompValue_ref_t comp_value(flatcc_builder_t *B, cuo_comp comp)
{
    /* Not ModAbi_CompValue_create: it fails on a null data ref, i.e. on a marker.
     * Encoding stays at its default (Json). */
    ModAbi_CompValue_start(B);
    ModAbi_CompValue_type_id_add(B, comp.type_id);
    if (comp.data.len)
        ModAbi_CompValue_data_create(B, comp.data.ptr, comp.data.len);
    return ModAbi_CompValue_end(B);
}

static ModAbi_CompValue_vec_ref_t comp_vec(flatcc_builder_t *B, const cuo_comp *comps, size_t n)
{
    ModAbi_CompValue_vec_start(B);
    for (size_t i = 0; i < n; i++)
        ModAbi_CompValue_vec_push(B, comp_value(B, comps[i]));
    return ModAbi_CompValue_vec_end(B);
}

cuo_comp cuo_comp_bytes(uint16_t type_id, cuo_bytes json)
{
    cuo_comp c = { type_id, json };
    return c;
}

cuo_comp cuo_comp_json(uint16_t type_id, const char *json)
{
    return cuo_comp_bytes(type_id, cuo_str_bytes(json));
}

cuo_comp cuo_comp_marker(uint16_t type_id)
{
    cuo_comp c = { type_id, { NULL, 0 } };
    return c;
}

cuo_comp cuo_child_of(cuo_entity parent)
{
    /* The placeholder of a parent spawned in this run is fine: the host maps it. */
    cuo_ChildOfDto dto = { .parent = cuo_resolve(parent) };
    return cuo_ChildOfDto_comp(&dto);
}

cuo_entity cuo_spawn(cuo_cmds *c, const cuo_comp *comps, size_t n)
{
    flatcc_builder_t *B = open_cmd(c);
    uint32_t temp = next_temp++;
    ModAbi_CommandBuffer_cmds_SpawnCmd_push_start(B);
    ModAbi_SpawnCmd_temp_id_add(B, temp);
    if (n)
        ModAbi_SpawnCmd_comps_add(B, comp_vec(B, comps, n));
    ModAbi_CommandBuffer_cmds_SpawnCmd_push_end(B);
    return CUO_PENDING | temp;
}

cuo_entity cuo_spawn_child(cuo_cmds *c, cuo_entity parent, const cuo_comp *comps, size_t n)
{
    cuo_comp *all = cuo_alloc((n + 1) * sizeof *all);
    memcpy(all, comps, n * sizeof *all);
    all[n] = cuo_child_of(parent);
    return cuo_spawn(c, all, n + 1);
}

void cuo_insert(cuo_cmds *c, cuo_entity e, const cuo_comp *comps, size_t n)
{
    flatcc_builder_t *B = open_cmd(c);
    ModAbi_CommandBuffer_cmds_InsertCmd_push_start(B);
    ModAbi_InsertCmd_entity_add(B, wire_ref(e));
    if (n)
        ModAbi_InsertCmd_comps_add(B, comp_vec(B, comps, n));
    ModAbi_CommandBuffer_cmds_InsertCmd_push_end(B);
}

void cuo_insert1(cuo_cmds *c, cuo_entity e, cuo_comp comp)
{
    cuo_insert(c, e, &comp, 1);
}

void cuo_remove(cuo_cmds *c, cuo_entity e, const uint16_t *type_ids, size_t n)
{
    flatcc_builder_t *B = open_cmd(c);
    ModAbi_CommandBuffer_cmds_RemoveCmd_push_start(B);
    ModAbi_RemoveCmd_entity_add(B, wire_ref(e));
    if (n)
        ModAbi_RemoveCmd_type_ids_create(B, type_ids, n);
    ModAbi_CommandBuffer_cmds_RemoveCmd_push_end(B);
}

void cuo_despawn(cuo_cmds *c, cuo_entity e)
{
    flatcc_builder_t *B = open_cmd(c);
    ModAbi_CommandBuffer_cmds_DespawnCmd_push_start(B);
    ModAbi_DespawnCmd_entity_add(B, wire_ref(e));
    ModAbi_CommandBuffer_cmds_DespawnCmd_push_end(B);
}

void cuo_resource_set(cuo_cmds *c, cuo_comp value)
{
    flatcc_builder_t *B = open_cmd(c);
    ModAbi_CommandBuffer_cmds_ResourceSetCmd_push_start(B);
    ModAbi_ResourceSetCmd_value_add(B, comp_value(B, value));
    ModAbi_CommandBuffer_cmds_ResourceSetCmd_push_end(B);
}

void cuo_emit(cuo_cmds *c, const char *event_path, uint64_t entity, cuo_bytes json)
{
    flatcc_builder_t *B = open_cmd(c);
    ModAbi_CommandBuffer_cmds_EmitEventCmd_push_start(B);
    ModAbi_EmitEventCmd_event_name_create_str(B, event_path);
    ModAbi_EmitEventCmd_entity_add(B, cuo_resolve(entity));
    if (json.len)
        ModAbi_EmitEventCmd_data_create(B, json.ptr, json.len);
    ModAbi_CommandBuffer_cmds_EmitEventCmd_push_end(B);
}

__attribute__((export_name("mod_spawned"))) void mod_spawned(uint32_t ptr, uint32_t len)
{
    (void)len;
    ModAbi_SpawnedInput_table_t in = ModAbi_SpawnedInput_as_root((const void *)(uintptr_t)ptr);
    ModAbi_SpawnResolved_vec_t v = in ? ModAbi_SpawnedInput_spawned(in) : NULL;
    for (size_t i = 0, n = ModAbi_SpawnResolved_vec_len(v); i < n; i++) {
        ModAbi_SpawnResolved_table_t sr = ModAbi_SpawnResolved_vec_at(v, i);
        uint32_t temp = ModAbi_SpawnResolved_temp_id(sr);
        if (temp >= nresolved) {
            size_t cap = nresolved ? nresolved : 64;
            while (cap <= temp)
                cap *= 2;
            resolved = realloc(resolved, cap * sizeof *resolved);
            memset(resolved + nresolved, 0, (cap - nresolved) * sizeof *resolved);
            nresolved = cap;
        }
        resolved[temp] = ModAbi_SpawnResolved_entity(sr) + 1;
    }
}
