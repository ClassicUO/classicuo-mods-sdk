/* Shared between the SDK's translation units. Not part of the mod-facing API. */
#ifndef CUO_INTERNAL_H
#define CUO_INTERNAL_H

#include "cuo/cuo.h"
#include "cuo/exports.h"

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

/* A res param's value across calls: reused while the host reports it unchanged. */
typedef struct cuo__res_cache {
    bool known;                    /* the param has run before (res.unchanged is meaningful) */
    bool json_valid, json_has;     /* res.get */
    tinyecs_modding_ecs_json_t json;
    uint16_t typed_kind;           /* which get-X filled `typed` (typed_gen.c's resource kinds) */
    bool typed_valid, typed_has;   /* cuo:modding/components get-X */
    void *typed;                   /* malloc'd record (cuo__typed_res_size(typed_kind)) */
} cuo__res_cache;

/* One parameter of the running system / observer. A query fetches rows (or only its
 * entities, when every read term has a typed column) as the call starts and the other
 * on first use; a res fetches its value on first use. */
typedef struct cuo__pval {
    uint8_t tag; /* CUO__K_QUERY / CUO__K_RES / CUO__K_EVENTS */
    uint16_t type_id; /* res: its type (cuo_resource_set looks the res-mut param up by it) */
    bool mut;
    int32_t handle; /* the owned handle (dropped when the call returns) */
    /* query */
    size_t len;
    bool has_rows, has_ents;
    tinyecs_modding_ecs_list_row_t rows;
    cuo_wit_list_entity_t ents;   /* fetched from the host (owned) */
    const cuo_entity *ent_view;   /* = ents.ptr, or built from rows (scratch) */
    /* res */
    cuo__res_cache *res;
    /* events: the JSON list, fetched on first use (a typed read asks the host itself) */
    bool has_events;
    cuo_wit_list_json_t events;
} cuo__pval;

typedef struct cuo__params {
    cuo__pval *v;
    size_t n;
} cuo__params;

struct cuo_cmds {
    tinyecs_modding_ecs_borrow_commands_t handle; /* -1: the export takes no `commands` */
    const cuo__params *params;
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
    cuo_bytes value;
    cuo_bytes packet;
    cuo_dir dir;
    const cuo__params *params;
    /* a typed observer: its value as the cuo_X of `typed_kind` (CUO__K_TYPED + n), NULL for a tag */
    const void *typed;
    uint8_t typed_kind;
};

/* ── typed components: the generic half (typed.c) ── */

typedef cuo_modding_components_own_entity_builder_t cuo__own_eb;
typedef cuo_modding_components_borrow_entity_builder_t cuo__borrow_eb;

/* Typed inserts: an entity builder (components.spawn / entity-of) by handle;
 * cuo__eb_push queues every typed comp of `comps` on it, in order, and drops it. */
int32_t cuo__eb_spawn(tinyecs_modding_ecs_borrow_commands_t c, cuo_entity *e);
int32_t cuo__eb_of(tinyecs_modding_ecs_borrow_commands_t c, cuo_entity e);
void cuo__eb_push(int32_t builder, const cuo_comp *comps, size_t n);
cuo_comp cuo__typed_comp(uint16_t type_id, uint8_t kind, const void *w, size_t size);
cuo_modding_components_borrow_query_t cuo__typed_query(cuo_query q);
cuo_modding_components_borrow_commands_t cuo__typed_cmds(cuo_cmds *c);
cuo_wit_string_t cuo__wstr_copy(const char *s);
uint32_t cuo__char_of(const char *s);
const char *cuo__char_str(uint32_t ch);
/* The running param's typed resource record (kind: typed_gen.c), NULL when the host has none. */
const void *cuo__res_typed(const cuo__params *ps, cuo_param p, uint16_t kind);
/* The events param's handle (-1 when `p` is not one). */
int32_t cuo__events_handle(const cuo__params *ps, cuo_param p);

/* ── the per-type half (typed_gen.c, generated) ── */

/* True when a read term on this type has a typed column (or is a tag). */
bool cuo__typed_readable(uint16_t type_id);
/* Queues typed insert `comp` (comp->typed = its insert kind) on `b`: the new builder. */
cuo__own_eb cuo__typed_push(cuo__borrow_eb b, const cuo_comp *comp);
size_t cuo__typed_res_size(uint16_t kind);
bool cuo__typed_res_get(uint16_t kind, int32_t handle, void *out);
void cuo__typed_res_free(uint16_t kind, void *rec);
/* A typed observer's trigger: the record kind its export takes for `path` (CUO__K_TYPED + n),
 * 0 for a tag (no value), -1 when the path has no typed trigger. */
int cuo__typed_trigger_kind(const char *path);
/* The trigger record (owned, freed here) as its cuo_X, in scratch. */
const void *cuo__typed_trigger_value(uint8_t kind, void *rec);

/* Called by the dispatcher once cuo_setup returned, before declaring to the host. */
void cuo__publish_hotkeys(cuo_builder *m);

#endif
