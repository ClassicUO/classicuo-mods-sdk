/* Shared between the SDK's translation units. Not part of the mod-facing API. */
#ifndef CUO_INTERNAL_H
#define CUO_INTERNAL_H

#include "cuo/cuo.h"

_Noreturn void cuo__trap(const char *msg);

/* call-scoped scratch */
void cuo__scratch_reset(void);
void *cuo__scratch_grow(void *p, size_t old_n, size_t new_n);

/* A wit string over scratch / caller memory (the import only reads it). */
static inline cuo_wit_string_t cuo__wstr(const char *s)
{
    cuo_wit_string_t w;
    cuo_wit_string_set(&w, s ? s : "");
    return w;
}

static inline cuo_wit_string_t cuo__wbytes(cuo_bytes b)
{
    cuo_wit_string_t w = { (uint8_t *)b.ptr, b.len };
    return w;
}

/* One parameter of the running system / observer, fetched when the call starts. */
typedef struct cuo__pval {
    uint8_t tag; /* TINYECS_MODDING_ECS_PARAM_* */
    uint16_t type_id; /* res: its type (cuo_resource_set looks the res-mut param up by it) */
    bool mut;
    int32_t handle; /* the owned handle (dropped when the call returns) */
    tinyecs_modding_ecs_list_row_t rows; /* query */
    bool has_res;
    tinyecs_modding_ecs_json_t res; /* res */
    cuo_wit_list_json_t events; /* events */
} cuo__pval;

typedef struct cuo__params {
    cuo__pval *v;
    size_t n;
} cuo__params;

struct cuo_cmds {
    tinyecs_modding_ecs_borrow_commands_t handle;
    const cuo__params *params;
    size_t count;
};

struct cuo_input {
    uint32_t sys_id;
    uint64_t tick;
    const cuo__params *params;
};

struct cuo_obs {
    uint32_t obs_id;
    uint64_t entity;
    cuo_bytes value;
    cuo_bytes packet;
    cuo_dir dir;
    const cuo__params *params;
};

/* Called by the dispatcher once cuo_setup returned, before declaring to the host. */
void cuo__publish_hotkeys(cuo_builder *m);

#endif
