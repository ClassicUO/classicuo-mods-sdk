/* Shared between the SDK's translation units. Not part of the mod-facing API. */
#ifndef CUO_INTERNAL_H
#define CUO_INTERNAL_H

#include "cuo/cuo.h"
#include "cuo/exports.h"

_Noreturn void cuo__trap(const char *msg);

/* call-scoped scratch */
void cuo__scratch_reset(void);

/* A wit string over scratch / caller memory (the import only reads it). */
static inline cuo_wit_string_t cuo__wstr(const char *s)
{
    cuo_wit_string_t w;
    cuo_wit_string_set(&w, s ? s : "");
    return w;
}

/* A res param's value across calls: reused while the host reports it unchanged. */
typedef struct cuo__res_cache {
    bool known;      /* the param has run before (res.unchanged is meaningful) */
    uint16_t kind;   /* which get-X filled `rec` (typed_gen.c's resource kinds) */
    bool valid, has; /* get-X asked / it returned a value */
    void *rec;       /* malloc'd record (cuo__typed_res_size(kind)) */
} cuo__res_cache;

/* One parameter of the running system / observer. A query fetches its entities as the
 * call starts; a res fetches its value on first use. */
typedef struct cuo__pval {
    uint8_t tag; /* CUO__K_QUERY / CUO__K_RES / CUO__K_EVENTS */
    bool mut;
    int32_t handle; /* the owned handle (dropped when the call returns) */
    /* query */
    cuo_wit_list_entity_t ents;
    /* entity -> row + 1 (0 = empty), open addressing in scratch memory; built by the
     * first cuo_query_index of a call with enough rows to need it */
    uint32_t *index;
    size_t index_mask;
    /* res */
    cuo__res_cache *res;
} cuo__pval;

typedef struct cuo__params {
    cuo__pval *v;
    size_t n;
} cuo__params;

struct cuo_cmds {
    tinyecs_modding_ecs_borrow_commands_t handle; /* -1: the export takes no `commands` */
    size_t count;
    const char *system;
};

struct cuo_input {
    uint32_t sys_id;
    uint64_t tick;
    const cuo__params *params;
};

struct cuo_obs {
    uint32_t obs_id;
    uint64_t entity;
    cuo_bytes packet;
    cuo_dir dir;
    const cuo__params *params;
    /* the trigger's value as the cuo_X of `value_kind` (CUO__K_TYPED + n), NULL for a tag */
    const void *value;
    uint8_t value_kind;
};

/* ── the generic half (typed.c) ── */

typedef cuo_modding_components_own_entity_builder_t cuo__own_eb;
typedef cuo_modding_components_borrow_entity_builder_t cuo__borrow_eb;

cuo_comp cuo__comp(uint16_t type_id, uint8_t kind, const void *rec);
/* Queues every comp of `comps` on builder `b` in order, and drops it. */
void cuo__eb_push(cuo__own_eb b, const cuo_comp *comps, size_t n);
cuo_modding_components_borrow_query_t cuo__typed_query(cuo_query q);
cuo_modding_components_borrow_commands_t cuo__typed_cmds(cuo_cmds *c);
cuo_wit_string_t cuo__wstr_copy(const char *s);
uint32_t cuo__char_of(const char *s);
const char *cuo__char_str(uint32_t ch);
/* The running param's resource record (kind: typed_gen.c), NULL when the host has none. */
const void *cuo__res_typed(const cuo__params *ps, cuo_param p, uint16_t kind);
/* The `mut` res param `p` to put a new value through (its cached value is dropped). */
cuo_modding_components_borrow_res_t cuo__res_put(const cuo__params *ps, cuo_param p);
/* The events param's handle (-1 when `p` is not one). */
int32_t cuo__events_handle(const cuo__params *ps, cuo_param p);

/* ── the per-type half (typed_gen.c, generated) ── */

/* Queues insert `comp` (comp->kind = its builder method) on `b`: the new builder. */
cuo__own_eb cuo__typed_push(cuo__borrow_eb b, const cuo_comp *comp);
size_t cuo__typed_res_size(uint16_t kind);
bool cuo__typed_res_get(uint16_t kind, int32_t handle, void *out);
void cuo__typed_res_free(uint16_t kind, void *rec);
/* An observer's trigger on `path`: the record kind its export takes (CUO__K_TYPED + n,
 * *record = its WIT name), 0 for a tag (no value), -1 for a path the contract lacks. */
int cuo__trigger_kind(const char *path, const char **record);
/* The trigger record (owned, freed here) as its cuo_X, in scratch. */
const void *cuo__trigger_value(uint8_t kind, void *rec);

/* Called by the dispatcher once cuo_setup returned, before declaring to the host. */
void cuo__publish_hotkeys(cuo_builder *m);

#endif
