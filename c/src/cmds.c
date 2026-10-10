/* Commands: each call goes straight to the host's command buffer through the running
 * system's `commands` param (applied after the callback returns). A spawn returns the
 * real entity id. */
#include <string.h>

#include "internal.h"

typedef cuo_wit_tuple2_type_path_json_t pair_t;

/* The running export's `commands`, counted. An export without one cannot record. */
static tinyecs_modding_ecs_borrow_commands_t H(cuo_cmds *c)
{
    if (c->handle.__handle < 0)
        cuo__trap(cuo_fmt("cuo: '%s' recorded a command but its export in wit/world.wit takes no "
                          "`commands: commands`",
                          c->system));
    c->count++;
    return c->handle;
}

/* The JSON comps; *typed = how many go through the entity builder instead. */
static tinyecs_modding_ecs_bundle_t bundle(const cuo_comp *comps, size_t n, size_t *typed)
{
    pair_t *p = cuo_alloc((n ? n : 1) * sizeof *p);
    size_t k = 0;
    *typed = 0;
    for (size_t i = 0; i < n; i++) {
        if (comps[i].typed) {
            ++*typed;
            continue;
        }
        p[k].f0 = cuo__wstr(cuo_type_path(comps[i].type_id));
        p[k].f1 = cuo__wbytes(comps[i].data);
        k++;
    }
    tinyecs_modding_ecs_bundle_t b = { p, k };
    return b;
}

size_t cuo_cmds_len(const cuo_cmds *c)
{
    return c->count;
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
    cuo_ChildOfDto dto = { .parent = parent };
    return cuo_ChildOfDto_typed(&dto);
}

/* Typed comps: the entity builder, after one commands.insert of the JSON ones. */
cuo_entity cuo_spawn(cuo_cmds *c, const cuo_comp *comps, size_t n)
{
    size_t typed;
    tinyecs_modding_ecs_bundle_t b = bundle(comps, n, &typed);
    if (!typed)
        return tinyecs_modding_ecs_method_commands_spawn(H(c), &b);
    cuo_entity e;
    int32_t eb = cuo__eb_spawn(H(c), &e);
    if (b.len)
        tinyecs_modding_ecs_method_commands_insert(H(c), e, &b);
    cuo__eb_push(eb, comps, n);
    return e;
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
    size_t typed;
    tinyecs_modding_ecs_bundle_t b = bundle(comps, n, &typed);
    if (b.len || !typed)
        tinyecs_modding_ecs_method_commands_insert(H(c), e, &b);
    if (typed)
        cuo__eb_push(cuo__eb_of(H(c), e), comps, n);
}

void cuo_insert1(cuo_cmds *c, cuo_entity e, cuo_comp comp)
{
    cuo_insert(c, e, &comp, 1);
}

void cuo_remove(cuo_cmds *c, cuo_entity e, const uint16_t *type_ids, size_t n)
{
    cuo_wit_string_t *paths = cuo_alloc((n ? n : 1) * sizeof *paths);
    for (size_t i = 0; i < n; i++)
        paths[i] = cuo__wstr(cuo_type_path(type_ids[i]));
    cuo_wit_list_type_path_t l = { paths, n };
    tinyecs_modding_ecs_method_commands_remove(H(c), e, &l);
}

void cuo_despawn(cuo_cmds *c, cuo_entity e)
{
    tinyecs_modding_ecs_method_commands_despawn(H(c), e);
}

void cuo_resource_set(cuo_cmds *c, cuo_comp value)
{
    if (value.typed)
        cuo__trap("cuo: cuo_resource_set takes a JSON payload (cuo_X_comp), not cuo_X_typed");
    const cuo__params *ps = c->params;
    for (size_t i = 0; i < ps->n; i++) {
        const cuo__pval *v = &ps->v[i];
        if (v->tag == CUO__K_RES && v->mut && v->type_id == value.type_id) {
            c->count++;
            cuo_wit_string_t json = cuo__wbytes(value.data);
            tinyecs_modding_ecs_method_res_set((tinyecs_modding_ecs_borrow_res_t){ v->handle }, &json);
            return;
        }
    }
    cuo_set_resource(c, cuo_type_path(value.type_id), value.data);
}

void cuo_set_resource(cuo_cmds *c, const char *path, cuo_bytes json)
{
    cuo_wit_string_t p = cuo__wstr(path);
    cuo_wit_string_t v = cuo__wbytes(json);
    tinyecs_modding_ecs_method_commands_set_resource(H(c), &p, &v);
}

void cuo_emit(cuo_cmds *c, const char *event_path, uint64_t entity, cuo_bytes json)
{
    (void)entity;
    cuo_wit_string_t path = cuo__wstr(event_path);
    cuo_wit_string_t v = cuo__wbytes(json);
    tinyecs_modding_ecs_method_commands_send(H(c), &path, &v);
}
