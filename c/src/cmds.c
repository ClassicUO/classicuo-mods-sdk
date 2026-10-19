/* Commands: each call goes straight to the host's command buffer through the running
 * system's `commands` param (applied after the callback returns). A spawn returns the
 * real entity id. Components go on through the entity builder of cuo:modding/components. */
#include <string.h>

#include "internal.h"

typedef tinyecs_modding_ecs_borrow_commands_t ecs_cmds_t;

static ecs_cmds_t H(cuo_cmds *c)
{
    return (ecs_cmds_t){ cuo__typed_cmds(c).__handle };
}

size_t cuo_cmds_len(const cuo_cmds *c)
{
    return c->count;
}

cuo_comp cuo_child_of(cuo_entity parent)
{
    cuo_ChildOf dto = { .parent = parent };
    return cuo_ChildOf_comp(&dto);
}

cuo_entity cuo_spawn(cuo_cmds *c, const cuo_comp *comps, size_t n)
{
    if (!n)
        return tinyecs_modding_ecs_method_commands_spawn(H(c));
    cuo__own_eb b = cuo_modding_components_spawn(cuo__typed_cmds(c));
    cuo_entity e = cuo_modding_components_method_entity_builder_id(cuo_modding_components_borrow_entity_builder(b));
    cuo__eb_push(b, comps, n);
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
    if (n)
        cuo__eb_push(cuo_modding_components_entity_of(cuo__typed_cmds(c), e), comps, n);
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
