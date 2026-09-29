/* Command buffer: streamed straight into the flatcc builder as the mod records
 * commands (a CommandBuffer root with one open cmds union vector), so nothing is
 * buffered twice. Temp entity refs are -(temp_id) - 1 (abi/mod-abi.fbs). */
#include <stdlib.h>
#include <string.h>

#include "internal.h"

typedef struct pending_name {
    uint32_t temp_id;
    char *name;
} pending_name;

struct cuo_cmds {
    flatcc_builder_t *B;
    size_t count;
    uint32_t next_temp;
    pending_name *names;
    size_t nnames, capnames;
};

static cuo_cmds instance;

/* Names minted by the LAST buffer handed to the host; mod_spawned resolves them. */
static pending_name *stash;
static size_t nstash;

cuo_cmds *cuo__cmds_instance(void)
{
    return &instance;
}

static void free_names(pending_name *v, size_t n)
{
    for (size_t i = 0; i < n; i++)
        free(v[i].name);
    free(v);
}

void cuo__cmds_begin(cuo_cmds *c)
{
    c->B = cuo__builder();
    c->count = 0;
    c->next_temp = 0;
    c->nnames = 0;
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
    uint64_t packed = cuo__pack_builder(c->B);
    /* Replace (not append): the host resolves each buffer before the next call. */
    if (c->nnames) {
        free_names(stash, nstash);
        stash = c->names;
        nstash = c->nnames;
        c->names = NULL;
        c->nnames = c->capnames = 0;
    }
    return packed;
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

cuo_eref cuo_spawn(cuo_cmds *c, const cuo_comp *comps, size_t n)
{
    flatcc_builder_t *B = open_cmd(c);
    uint32_t temp = c->next_temp++;
    ModAbi_CommandBuffer_cmds_SpawnCmd_push_start(B);
    ModAbi_SpawnCmd_temp_id_add(B, temp);
    if (n)
        ModAbi_SpawnCmd_comps_add(B, comp_vec(B, comps, n));
    ModAbi_CommandBuffer_cmds_SpawnCmd_push_end(B);
    return -(cuo_eref)temp - 1;
}

cuo_eref cuo_spawn_named(cuo_cmds *c, const char *name, const cuo_comp *comps, size_t n)
{
    cuo_eref e = cuo_spawn(c, comps, n);
    size_t len = strlen(name) + 1;
    pending_name p = { c->next_temp - 1, memcpy(malloc(len), name, len) };
    if (c->nnames == c->capnames) {
        c->capnames = c->capnames ? c->capnames * 2 : 16;
        c->names = realloc(c->names, c->capnames * sizeof(*c->names));
    }
    c->names[c->nnames++] = p;
    return e;
}

cuo_eref cuo_spawn_child(cuo_cmds *c, cuo_eref parent, const char *name, const cuo_comp *comps, size_t n)
{
    cuo_eref e = name ? cuo_spawn_named(c, name, comps, n) : cuo_spawn(c, comps, n);
    cuo_add_child(c, parent, e, CUO_APPEND);
    return e;
}

void cuo_insert(cuo_cmds *c, cuo_eref e, const cuo_comp *comps, size_t n)
{
    flatcc_builder_t *B = open_cmd(c);
    ModAbi_CommandBuffer_cmds_InsertCmd_push_start(B);
    ModAbi_InsertCmd_entity_add(B, e);
    if (n)
        ModAbi_InsertCmd_comps_add(B, comp_vec(B, comps, n));
    ModAbi_CommandBuffer_cmds_InsertCmd_push_end(B);
}

void cuo_insert1(cuo_cmds *c, cuo_eref e, cuo_comp comp)
{
    cuo_insert(c, e, &comp, 1);
}

void cuo_remove(cuo_cmds *c, cuo_eref e, const uint16_t *type_ids, size_t n)
{
    flatcc_builder_t *B = open_cmd(c);
    ModAbi_CommandBuffer_cmds_RemoveCmd_push_start(B);
    ModAbi_RemoveCmd_entity_add(B, e);
    if (n)
        ModAbi_RemoveCmd_type_ids_create(B, type_ids, n);
    ModAbi_CommandBuffer_cmds_RemoveCmd_push_end(B);
}

void cuo_despawn(cuo_cmds *c, cuo_eref e)
{
    flatcc_builder_t *B = open_cmd(c);
    ModAbi_CommandBuffer_cmds_DespawnCmd_push_start(B);
    ModAbi_DespawnCmd_entity_add(B, e);
    ModAbi_CommandBuffer_cmds_DespawnCmd_push_end(B);
}

bool cuo_despawn_named(cuo_cmds *c, const char *name)
{
    uint64_t id;
    if (!cuo__map_remove(&cuo__named, name, &id))
        return false;
    cuo_despawn(c, CUO_E(id));
    return true;
}

void cuo_add_child(cuo_cmds *c, cuo_eref parent, cuo_eref child, uint32_t index)
{
    flatcc_builder_t *B = open_cmd(c);
    ModAbi_CommandBuffer_cmds_AddChildCmd_push_start(B);
    ModAbi_AddChildCmd_parent_add(B, parent);
    ModAbi_AddChildCmd_child_add(B, child);
    ModAbi_AddChildCmd_index_add(B, index);
    ModAbi_CommandBuffer_cmds_AddChildCmd_push_end(B);
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
    ModAbi_EmitEventCmd_entity_add(B, entity);
    if (json.len)
        ModAbi_EmitEventCmd_data_create(B, json.ptr, json.len);
    ModAbi_CommandBuffer_cmds_EmitEventCmd_push_end(B);
}

void cuo_consume_mouse(cuo_cmds *c, uint8_t button)
{
    flatcc_builder_t *B = open_cmd(c);
    ModAbi_CommandBuffer_cmds_ConsumeMouseCmd_push_start(B);
    ModAbi_ConsumeMouseCmd_button_add(B, button);
    ModAbi_CommandBuffer_cmds_ConsumeMouseCmd_push_end(B);
}

void cuo_consume_key(cuo_cmds *c, uint32_t key)
{
    flatcc_builder_t *B = open_cmd(c);
    ModAbi_CommandBuffer_cmds_ConsumeKeyCmd_push_start(B);
    ModAbi_ConsumeKeyCmd_key_add(B, key);
    ModAbi_CommandBuffer_cmds_ConsumeKeyCmd_push_end(B);
}

__attribute__((export_name("mod_spawned"))) void mod_spawned(uint32_t ptr, uint32_t len)
{
    (void)len;
    if (!nstash)
        return;
    ModAbi_SpawnedInput_table_t in = ModAbi_SpawnedInput_as_root((const void *)(uintptr_t)ptr);
    ModAbi_SpawnResolved_vec_t v = in ? ModAbi_SpawnedInput_spawned(in) : NULL;
    for (size_t i = 0, n = ModAbi_SpawnResolved_vec_len(v); i < n; i++) {
        ModAbi_SpawnResolved_table_t sr = ModAbi_SpawnResolved_vec_at(v, i);
        uint32_t temp = ModAbi_SpawnResolved_temp_id(sr);
        for (size_t j = 0; j < nstash; j++)
            if (stash[j].temp_id == temp)
                cuo__map_put(&cuo__named, stash[j].name, ModAbi_SpawnResolved_entity(sr));
    }
    free_names(stash, nstash);
    stash = NULL;
    nstash = 0;
}
