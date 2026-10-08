/* Arena, scratch, registration and the ABI exports — twin of rust/src/p1/{arena,mod}.rs.
 *
 * Lifecycle (host drives, serially; the guest is single-threaded, so plain statics):
 * 1. mod_setup(Handshake) — intern the type-path table, call the mod's cuo_setup,
 *    return the SetupReply.
 * 2. mod_run / mod_observer — dispatch to the registered callback with the pushed
 *    parameter data, return its CommandBuffer (0 = none). A packet observer's verdict
 *    rides that CommandBuffer.
 * 3. mod_spawned — the host assigned real ids to the buffer it just applied (cmds.c).
 */
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "internal.h"

#define EXPORT(name) __attribute__((export_name(#name)))
#define ALIGN8(n) (((n) + 7u) & ~(size_t)7u)

_Noreturn void cuo__trap(const char *msg)
{
    cuo_log(msg);
    __builtin_trap();
}

static void *xrealloc(void *p, size_t n)
{
    void *q = realloc(p, n);
    if (!q)
        cuo__trap("cuo: out of memory");
    return q;
}

static char *xstrdup(const char *s)
{
    size_t n = strlen(s) + 1;
    return memcpy(xrealloc(NULL, n), s, n);
}

#define PUSH(arr, n, cap, item)                                          \
    do {                                                                 \
        if ((n) == (cap)) {                                              \
            (cap) = (cap) ? (cap) * 2 : 8;                               \
            (arr) = xrealloc((arr), (cap) * sizeof(*(arr)));             \
        }                                                                \
        (arr)[(n)++] = (item);                                           \
    } while (0)

/* ── ABI arena ─────────────────────────────────────────────────────────────────
 * The host writes each call's input here (mod_alloc) and reads packed returns from
 * here. Growth allocates a NEW block instead of realloc'ing so every pointer handed
 * out in this cycle stays valid; retired blocks are freed on reset. */

typedef struct block {
    struct block *next;
    size_t cap, top;
    _Alignas(8) unsigned char data[];
} block;

static block *abi_cur, *abi_retired;

static void *abi_alloc(size_t n)
{
    if (n == 0)
        n = 1;
    if (!abi_cur || ALIGN8(abi_cur->top) + n > abi_cur->cap) {
        size_t cap = abi_cur ? abi_cur->cap * 2 : 64 * 1024;
        while (cap < n)
            cap *= 2;
        block *b = xrealloc(NULL, sizeof(block) + cap);
        b->cap = cap;
        b->top = 0;
        b->next = NULL;
        if (abi_cur) {
            abi_cur->next = abi_retired;
            abi_retired = abi_cur;
        }
        abi_cur = b;
    }
    size_t start = ALIGN8(abi_cur->top);
    abi_cur->top = start + n;
    return abi_cur->data + start;
}

EXPORT(mod_alloc) uint32_t mod_alloc(uint32_t size)
{
    return (uint32_t)(uintptr_t)abi_alloc(size);
}

EXPORT(mod_arena_reset) void mod_arena_reset(void)
{
    while (abi_retired) {
        block *b = abi_retired;
        abi_retired = b->next;
        free(b);
    }
    if (abi_cur)
        abi_cur->top = 0;
}

uint64_t cuo__pack_builder(flatcc_builder_t *B)
{
    size_t n = flatcc_builder_get_buffer_size(B);
    void *dst = abi_alloc(n);
    flatcc_builder_copy_buffer(B, dst, n);
    flatcc_builder_reset(B);
    return ((uint64_t)n << 32) | (uint32_t)(uintptr_t)dst;
}

/* ── scratch arena (call-scoped) ───────────────────────────────────────────── */

static block *scr_cur;
static void *scr_last; /* most recent allocation, for in-place growth */

void cuo__scratch_reset(void)
{
    /* Keep the biggest block (the working set of a busy call), free the rest. */
    block *keep = NULL;
    for (block *b = scr_cur, *next; b; b = next) {
        next = b->next;
        if (!keep || b->cap > keep->cap) {
            if (keep)
                free(keep);
            keep = b;
        } else {
            free(b);
        }
    }
    if (keep) {
        keep->next = NULL;
        keep->top = 0;
    }
    scr_cur = keep;
    scr_last = NULL;
}

static void *scratch_raw(size_t n)
{
    if (!scr_cur || ALIGN8(scr_cur->top) + n > scr_cur->cap) {
        size_t cap = 64 * 1024;
        while (cap < n)
            cap *= 2;
        block *b = xrealloc(NULL, sizeof(block) + cap);
        b->cap = cap;
        b->top = 0;
        b->next = scr_cur;
        scr_cur = b;
    }
    size_t start = ALIGN8(scr_cur->top);
    scr_cur->top = start + n;
    return scr_last = scr_cur->data + start;
}

void *cuo_alloc(size_t n)
{
    return memset(scratch_raw(n ? n : 1), 0, n ? n : 1);
}

void *cuo__scratch_grow(void *p, size_t old_n, size_t new_n)
{
    if (p && p == scr_last && (unsigned char *)p + new_n <= scr_cur->data + scr_cur->cap) {
        scr_cur->top = (size_t)((unsigned char *)p - scr_cur->data) + new_n;
        return p;
    }
    void *q = scratch_raw(new_n);
    if (p && old_n)
        memcpy(q, p, old_n);
    return q;
}

char *cuo_strndup(const char *s, size_t n)
{
    char *d = scratch_raw(n + 1);
    memcpy(d, s, n);
    d[n] = 0;
    return d;
}

char *cuo_strdup(const char *s)
{
    return cuo_strndup(s ? s : "", s ? strlen(s) : 0);
}

const char *cuo_fmt(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    int n = vsnprintf(NULL, 0, fmt, ap);
    va_end(ap);
    char *d = scratch_raw((size_t)n + 1);
    va_start(ap, fmt);
    vsnprintf(d, (size_t)n + 1, fmt, ap);
    va_end(ap);
    return d;
}

/* ── string -> u64 map (open addressing, linear probe, tombstones) ───────────── */

static char tomb;
#define TOMB (&tomb)

static uint32_t hash(const char *s)
{
    uint32_t h = 2166136261u;
    while (*s)
        h = (h ^ (unsigned char)*s++) * 16777619u;
    return h;
}

static size_t map_slot(const cuo__map *m, const char *key)
{
    size_t mask = m->cap - 1, i = hash(key) & mask, first_tomb = (size_t)-1;
    for (;;) {
        char *k = m->keys[i];
        if (!k)
            return first_tomb != (size_t)-1 ? first_tomb : i;
        if (k == TOMB) {
            if (first_tomb == (size_t)-1)
                first_tomb = i;
        } else if (strcmp(k, key) == 0) {
            return i;
        }
        i = (i + 1) & mask;
    }
}

bool cuo__map_get(const cuo__map *m, const char *key, uint64_t *out)
{
    if (!m->cap)
        return false;
    size_t mask = m->cap - 1, i = hash(key) & mask;
    for (;;) {
        char *k = m->keys[i];
        if (!k)
            return false;
        if (k != TOMB && strcmp(k, key) == 0) {
            if (out)
                *out = m->vals[i];
            return true;
        }
        i = (i + 1) & mask;
    }
}

static void map_grow(cuo__map *m)
{
    cuo__map old = *m;
    m->cap = old.cap ? old.cap * 2 : 64;
    m->keys = xrealloc(NULL, m->cap * sizeof(char *));
    m->vals = xrealloc(NULL, m->cap * sizeof(uint64_t));
    memset(m->keys, 0, m->cap * sizeof(char *));
    m->count = m->used = 0;
    for (size_t i = 0; i < old.cap; i++)
        if (old.keys[i] && old.keys[i] != TOMB) {
            size_t s = map_slot(m, old.keys[i]);
            m->keys[s] = old.keys[i];
            m->vals[s] = old.vals[i];
            m->count++;
            m->used++;
        }
    free(old.keys);
    free(old.vals);
}

void cuo__map_put(cuo__map *m, const char *key, uint64_t val)
{
    if ((m->used + 1) * 4 >= m->cap * 3)
        map_grow(m);
    size_t s = map_slot(m, key);
    char *k = m->keys[s];
    if (k && k != TOMB) {
        m->vals[s] = val;
        return;
    }
    if (!k)
        m->used++;
    m->keys[s] = xstrdup(key);
    m->vals[s] = val;
    m->count++;
}

bool cuo__map_remove(cuo__map *m, const char *key, uint64_t *out)
{
    if (!m->cap)
        return false;
    size_t mask = m->cap - 1, i = hash(key) & mask;
    for (;;) {
        char *k = m->keys[i];
        if (!k)
            return false;
        if (k != TOMB && strcmp(k, key) == 0) {
            if (out)
                *out = m->vals[i];
            free(k);
            m->keys[i] = TOMB;
            m->count--;
            return true;
        }
        i = (i + 1) & mask;
    }
}

/* ── type table ───────────────────────────────────────────────────────────── */

static cuo__map types;

bool cuo_try_type_id(const char *path, uint16_t *out)
{
    uint64_t v;
    if (!cuo__map_get(&types, path, &v))
        return false;
    if (out)
        *out = (uint16_t)v;
    return true;
}

uint16_t cuo_type_id(const char *path)
{
    uint16_t id;
    if (!cuo_try_type_id(path, &id))
        cuo__trap(cuo_fmt("cuo: the client has no type path '%s'", path));
    return id;
}

const char *cuo_type_path(uint16_t id)
{
    for (size_t i = 0; i < types.cap; i++)
        if (types.keys[i] && types.keys[i] != TOMB && types.vals[i] == id)
            return types.keys[i];
    return NULL;
}

/* ── registration ────────────────────────────────────────────────────────── */

typedef struct param_decl {
    uint8_t kind; /* ModAbi_ParamKind */
    uint16_t type_id;
    cuo_term *terms;
    size_t n;
} param_decl;

typedef struct param_list {
    param_decl *v;
    size_t n, cap;
} param_list;

typedef struct sys_rec {
    cuo_system_fn fn;
    void *user;
    char *name;
    uint8_t schedule;
    param_list params;
    uint32_t *after, *before;
    size_t nafter, capafter, nbefore, capbefore;
} sys_rec;

typedef struct obs_rec {
    cuo_observer_fn fn;
    void *user;
    uint8_t kind;
    uint16_t type_id;
    char *event_name;
    param_list params;
    /* kind == Packet: fn is unused */
    cuo_packet_fn packet_fn;
    cuo_packet_tap_fn tap_fn;
    uint8_t packet_dir;
    uint8_t *packet_ids;
    size_t npacket_ids;
} obs_rec;

struct cuo_builder {
    bool open;
};

static cuo_builder the_builder;
static sys_rec *systems;
static size_t nsys, capsys;
static obs_rec *observers;
static size_t nobs, capobs;

static void require_setup(cuo_builder *m)
{
    if (!m || !m->open)
        cuo__trap("cuo: registration functions are only valid inside cuo_setup");
}

static sys_rec *sys_at(cuo_builder *m, cuo_sys s)
{
    require_setup(m);
    if (s >= nsys)
        cuo__trap("cuo: unknown system id");
    return &systems[s];
}

static obs_rec *obs_at(cuo_builder *m, cuo_observer o)
{
    require_setup(m);
    if (o >= nobs)
        cuo__trap("cuo: unknown observer id");
    return &observers[o];
}

static cuo_param add_param(param_list *l, uint8_t kind, uint16_t type_id, cuo_term *terms, size_t n)
{
    param_decl p = { kind, type_id, terms, n };
    PUSH(l->v, l->n, l->cap, p);
    return (cuo_param)(l->n - 1);
}

/* Two terms on one type would put two payloads in the row and shift every later
 * slot: fold instead (a WITH on a read type is dropped, a CHANGED / ADDED upgrades). */
static cuo_param add_query(param_list *l, const cuo_term *terms, size_t n)
{
    cuo_term *q = xrealloc(NULL, (n ? n : 1) * sizeof(cuo_term));
    size_t qn = 0;
    for (size_t i = 0; i < n; i++) {
        cuo_term t = terms[i];
        size_t at = qn;
        if (t.kind != CUO_TERM_WITHOUT)
            for (size_t j = 0; j < qn; j++)
                if (q[j].type_id == t.type_id && q[j].kind != CUO_TERM_WITHOUT) {
                    at = j;
                    break;
                }
        if (at < qn) {
            if (t.kind == CUO_TERM_CHANGED || t.kind == CUO_TERM_ADDED)
                q[at].kind = t.kind;
            else if (q[at].kind == CUO_TERM_WITH && t.kind != CUO_TERM_WITH)
                q[at].kind = t.kind;
            continue;
        }
        q[qn++] = t;
    }
    return add_param(l, ModAbi_ParamKind_Query, CUO_NONE_TYPE, q, qn);
}

cuo_sys cuo_add_system(cuo_builder *m, const char *label, cuo_stage stage, cuo_system_fn fn, void *user)
{
    require_setup(m);
    sys_rec r = { 0 };
    r.fn = fn;
    r.user = user;
    r.schedule = (uint8_t)stage;
    r.name = label ? xstrdup(label) : xstrdup(cuo_fmt("sys-%zu", nsys));
    /* Param 0 is always Commands: the SDK hands every callback a command buffer. */
    add_param(&r.params, ModAbi_ParamKind_Commands, CUO_NONE_TYPE, NULL, 0);
    PUSH(systems, nsys, capsys, r);
    return (cuo_sys)(nsys - 1);
}

cuo_param cuo_system_query(cuo_builder *m, cuo_sys s, const cuo_term *terms, size_t n)
{
    return add_query(&sys_at(m, s)->params, terms, n);
}

cuo_param cuo_system_res(cuo_builder *m, cuo_sys s, uint16_t type_id, bool mut)
{
    return add_param(&sys_at(m, s)->params, mut ? ModAbi_ParamKind_ResMut : ModAbi_ParamKind_Res, type_id, NULL, 0);
}

cuo_param cuo_system_events(cuo_builder *m, cuo_sys s, uint16_t type_id)
{
    return add_param(&sys_at(m, s)->params, ModAbi_ParamKind_Events, type_id, NULL, 0);
}

void cuo_system_after(cuo_builder *m, cuo_sys s, cuo_sys other)
{
    sys_rec *r = sys_at(m, s);
    PUSH(r->after, r->nafter, r->capafter, other);
}

void cuo_system_before(cuo_builder *m, cuo_sys s, cuo_sys other)
{
    sys_rec *r = sys_at(m, s);
    PUSH(r->before, r->nbefore, r->capbefore, other);
}

static cuo_observer add_observer(cuo_builder *m, uint8_t kind, uint16_t type_id, const char *event,
                                 cuo_observer_fn fn, void *user)
{
    require_setup(m);
    obs_rec r = { fn, user, kind, type_id, event ? xstrdup(event) : NULL };
    add_param(&r.params, ModAbi_ParamKind_Commands, CUO_NONE_TYPE, NULL, 0);
    PUSH(observers, nobs, capobs, r);
    return (cuo_observer)(nobs - 1);
}

cuo_observer cuo_on_event(cuo_builder *m, const char *event_path, cuo_observer_fn fn, void *user)
{
    return add_observer(m, ModAbi_ObserverKind_Custom, CUO_NONE_TYPE, event_path, fn, user);
}

cuo_observer cuo_on_add(cuo_builder *m, uint16_t type_id, cuo_observer_fn fn, void *user)
{
    return add_observer(m, ModAbi_ObserverKind_Insert, type_id, NULL, fn, user);
}

cuo_observer cuo_on_remove(cuo_builder *m, uint16_t type_id, cuo_observer_fn fn, void *user)
{
    return add_observer(m, ModAbi_ObserverKind_Remove, type_id, NULL, fn, user);
}

cuo_param cuo_observer_query(cuo_builder *m, cuo_observer o, const cuo_term *terms, size_t n)
{
    return add_query(&obs_at(m, o)->params, terms, n);
}

cuo_param cuo_observer_res(cuo_builder *m, cuo_observer o, uint16_t type_id, bool mut)
{
    return add_param(&obs_at(m, o)->params, mut ? ModAbi_ParamKind_ResMut : ModAbi_ParamKind_Res, type_id, NULL, 0);
}

cuo_param cuo_observer_events(cuo_builder *m, cuo_observer o, uint16_t type_id)
{
    return add_param(&obs_at(m, o)->params, ModAbi_ParamKind_Events, type_id, NULL, 0);
}

static cuo_observer add_packet_observer(cuo_builder *m, cuo_dir dir, const uint8_t *ids, size_t n,
                                        cuo_packet_fn fn, cuo_packet_tap_fn tap, void *user)
{
    cuo_observer o = add_observer(m, ModAbi_ObserverKind_Packet, CUO_NONE_TYPE, NULL, NULL, user);
    obs_rec *r = &observers[o];
    r->packet_fn = fn;
    r->tap_fn = tap;
    r->packet_dir = (uint8_t)dir;
    if (n) {
        r->packet_ids = memcpy(xrealloc(NULL, n), ids, n);
        r->npacket_ids = n;
    }
    return o;
}

cuo_observer cuo_on_packet(cuo_builder *m, cuo_dir dir, const uint8_t *ids, size_t n, cuo_packet_fn fn,
                           void *user)
{
    return add_packet_observer(m, dir, ids, n, fn, NULL, user);
}

cuo_observer cuo_on_packet_in(cuo_builder *m, cuo_packet_tap_fn fn, void *user)
{
    return add_packet_observer(m, CUO_INCOMING, NULL, 0, NULL, fn, user);
}

cuo_observer cuo_on_packet_out(cuo_builder *m, cuo_packet_tap_fn fn, void *user)
{
    return add_packet_observer(m, CUO_OUTGOING, NULL, 0, NULL, fn, user);
}

/* ── setup reply ─────────────────────────────────────────────────────────── */

static flatcc_builder_t fb;
static bool fb_ready;

flatcc_builder_t *cuo__builder(void)
{
    if (!fb_ready) {
        flatcc_builder_init(&fb);
        fb_ready = true;
    }
    return &fb;
}

static void build_param(flatcc_builder_t *B, const param_decl *p)
{
    ModAbi_ParamDecl_kind_add(B, p->kind);
    ModAbi_ParamDecl_type_id_add(B, p->type_id);
    if (p->kind != ModAbi_ParamKind_Query)
        return;
    ModAbi_ParamDecl_query_start(B);
    ModAbi_QueryDecl_terms_start(B);
    for (size_t t = 0; t < p->n; t++) {
        ModAbi_QueryDecl_terms_push_start(B);
        ModAbi_QueryTerm_kind_add(B, p->terms[t].kind);
        ModAbi_QueryTerm_type_id_add(B, p->terms[t].type_id);
        ModAbi_QueryDecl_terms_push_end(B);
    }
    ModAbi_QueryDecl_terms_end(B);
    ModAbi_ParamDecl_query_end(B);
}

static uint64_t build_reply(void)
{
    flatcc_builder_t *B = cuo__builder();
    ModAbi_SetupReply_start_as_root(B);

    ModAbi_SetupReply_systems_start(B);
    for (size_t i = 0; i < nsys; i++) {
        sys_rec *r = &systems[i];
        ModAbi_SetupReply_systems_push_start(B);
        ModAbi_SystemDecl_id_add(B, (uint32_t)i);
        ModAbi_SystemDecl_name_create_str(B, r->name);
        ModAbi_SystemDecl_schedule_add(B, r->schedule);
        ModAbi_SystemDecl_params_start(B);
        for (size_t p = 0; p < r->params.n; p++) {
            ModAbi_SystemDecl_params_push_start(B);
            build_param(B, &r->params.v[p]);
            ModAbi_SystemDecl_params_push_end(B);
        }
        ModAbi_SystemDecl_params_end(B);
        if (r->nafter)
            ModAbi_SystemDecl_after_create(B, r->after, r->nafter);
        if (r->nbefore)
            ModAbi_SystemDecl_before_create(B, r->before, r->nbefore);
        ModAbi_SetupReply_systems_push_end(B);
    }
    ModAbi_SetupReply_systems_end(B);

    ModAbi_SetupReply_observers_start(B);
    for (size_t i = 0; i < nobs; i++) {
        obs_rec *r = &observers[i];
        ModAbi_SetupReply_observers_push_start(B);
        ModAbi_ObserverDecl_id_add(B, (uint32_t)i);
        ModAbi_ObserverDecl_kind_add(B, r->kind);
        ModAbi_ObserverDecl_type_id_add(B, r->type_id);
        if (r->event_name)
            ModAbi_ObserverDecl_event_name_create_str(B, r->event_name);
        if (r->kind == ModAbi_ObserverKind_Packet) {
            ModAbi_ObserverDecl_packet_direction_add(B, r->packet_dir);
            if (r->npacket_ids)
                ModAbi_ObserverDecl_packet_ids_create(B, r->packet_ids, r->npacket_ids);
        }
        ModAbi_ObserverDecl_params_start(B);
        for (size_t p = 0; p < r->params.n; p++) {
            ModAbi_ObserverDecl_params_push_start(B);
            build_param(B, &r->params.v[p]);
            ModAbi_ObserverDecl_params_push_end(B);
        }
        ModAbi_ObserverDecl_params_end(B);
        ModAbi_SetupReply_observers_push_end(B);
    }
    ModAbi_SetupReply_observers_end(B);

    ModAbi_SetupReply_end_as_root(B);
    return cuo__pack_builder(B);
}

/* ── exports ─────────────────────────────────────────────────────────────── */

EXPORT(mod_setup) uint64_t mod_setup(uint32_t ptr, uint32_t len)
{
    (void)len;
    cuo__scratch_reset();
    ModAbi_Handshake_table_t hs = ModAbi_Handshake_as_root((const void *)(uintptr_t)ptr);
    uint32_t host_abi = hs ? ModAbi_Handshake_abi_version(hs) : 0;
    if (host_abi != CUO_ABI_VERSION)
        cuo__trap(cuo_fmt("cuo: mod ABI mismatch: the client speaks v%u, this mod was built for v%u: "
                          "rebuild it against the current SDK",
                          host_abi, CUO_ABI_VERSION));

    ModAbi_TypePath_vec_t paths = ModAbi_Handshake_type_paths(hs);
    for (size_t i = 0, n = ModAbi_TypePath_vec_len(paths); i < n; i++) {
        ModAbi_TypePath_table_t tp = ModAbi_TypePath_vec_at(paths, i);
        flatbuffers_string_t p = ModAbi_TypePath_path(tp);
        if (p)
            cuo__map_put(&types, p, ModAbi_TypePath_id(tp));
    }

    the_builder.open = true;
    cuo_setup(&the_builder);
    cuo__publish_hotkeys(&the_builder);
    the_builder.open = false;
    return build_reply();
}

EXPORT(mod_run) uint64_t mod_run(uint32_t sys_id, uint32_t ptr, uint32_t len)
{
    (void)len;
    if (sys_id >= nsys)
        return 0;
    cuo__scratch_reset();
    cuo_input in = { ModAbi_SystemInput_as_root((const void *)(uintptr_t)ptr) };
    cuo_cmds *c = cuo__cmds_instance();
    cuo__cmds_begin(c);
    systems[sys_id].fn(&in, c, systems[sys_id].user);
    return cuo__cmds_finish(c, CUO_PASS, (cuo_bytes){ NULL, 0 });
}

EXPORT(mod_observer) uint64_t mod_observer(uint32_t obs_id, uint64_t entity, uint32_t ptr, uint32_t len)
{
    (void)len;
    if (obs_id >= nobs)
        return 0;
    cuo__scratch_reset();
    cuo_obs ev = { ModAbi_ObserverInput_as_root((const void *)(uintptr_t)ptr), entity };
    cuo_cmds *c = cuo__cmds_instance();
    cuo__cmds_begin(c);
    obs_rec *r = &observers[obs_id];
    cuo_verdict verdict = CUO_PASS;
    cuo_bytes replacement = { NULL, 0 };
    if (r->kind != ModAbi_ObserverKind_Packet) {
        r->fn(&ev, c, r->user);
    } else if (r->packet_fn) {
        verdict = r->packet_fn(&ev, c, &replacement, r->user);
    } else {
        cuo_bytes p = cuo_obs_packet(&ev);
        if (p.len && r->tap_fn(p.ptr[0], p.ptr, p.len, r->user))
            verdict = CUO_BLOCK;
    }
    return cuo__cmds_finish(c, verdict, replacement);
}

/* ── input views ─────────────────────────────────────────────────────────── */

/* The host emits one entry per param of each kind, keyed by its param index. */
static cuo_query query_of(ModAbi_QueryRows_vec_t qs, cuo_param p)
{
    cuo_query q = { NULL, 0 };
    for (size_t i = 0, n = ModAbi_QueryRows_vec_len(qs); i < n; i++) {
        ModAbi_QueryRows_table_t t = ModAbi_QueryRows_vec_at(qs, i);
        if (ModAbi_QueryRows_param_index(t) == p) {
            ModAbi_Row_vec_t rows = ModAbi_QueryRows_rows(t);
            q.vec = rows;
            q.len = ModAbi_Row_vec_len(rows);
            break;
        }
    }
    return q;
}

static cuo_bytes comp_bytes(ModAbi_CompValue_table_t cv)
{
    cuo_bytes b = { NULL, 0 };
    if (!cv)
        return b;
    flatbuffers_uint8_vec_t d = ModAbi_CompValue_data(cv);
    b.len = flatbuffers_uint8_vec_len(d);
    b.ptr = b.len ? d : NULL;
    return b;
}

static cuo_bytes res_of(ModAbi_ResValue_vec_t rs, cuo_param p)
{
    cuo_bytes none = { NULL, 0 };
    for (size_t i = 0, n = ModAbi_ResValue_vec_len(rs); i < n; i++) {
        ModAbi_ResValue_table_t t = ModAbi_ResValue_vec_at(rs, i);
        if (ModAbi_ResValue_param_index(t) == p) {
            ModAbi_CompValue_table_t v = ModAbi_ResValue_value(t);
            if (!v)
                return none;
            cuo_bytes b = comp_bytes(v);
            /* A present marker resource: report "{}", not absence. */
            return b.ptr ? b : cuo_str_bytes("{}");
        }
    }
    return none;
}

static cuo_events events_of(ModAbi_EventValues_vec_t es, cuo_param p)
{
    cuo_events e = { NULL, 0 };
    for (size_t i = 0, n = ModAbi_EventValues_vec_len(es); i < n; i++) {
        ModAbi_EventValues_table_t t = ModAbi_EventValues_vec_at(es, i);
        if (ModAbi_EventValues_param_index(t) == p) {
            ModAbi_CompValue_vec_t v = ModAbi_EventValues_values(t);
            e.vec = v;
            e.len = ModAbi_CompValue_vec_len(v);
            break;
        }
    }
    return e;
}

uint32_t cuo_input_sys_id(const cuo_input *in)
{
    return in->t ? ModAbi_SystemInput_sys_id(in->t) : 0;
}

uint64_t cuo_input_tick(const cuo_input *in)
{
    return in->t ? ModAbi_SystemInput_tick(in->t) : 0;
}

cuo_query cuo_input_query(const cuo_input *in, cuo_param p)
{
    cuo_query none = { NULL, 0 };
    return in->t ? query_of(ModAbi_SystemInput_queries(in->t), p) : none;
}

cuo_bytes cuo_input_res(const cuo_input *in, cuo_param p)
{
    cuo_bytes none = { NULL, 0 };
    return in->t ? res_of(ModAbi_SystemInput_resources(in->t), p) : none;
}

cuo_events cuo_input_events(const cuo_input *in, cuo_param p)
{
    cuo_events none = { NULL, 0 };
    return in->t ? events_of(ModAbi_SystemInput_events(in->t), p) : none;
}

cuo_row cuo_query_row(cuo_query q, size_t i)
{
    cuo_row r = { i < q.len ? ModAbi_Row_vec_at((ModAbi_Row_vec_t)q.vec, i) : NULL };
    return r;
}

bool cuo_query_find(cuo_query q, cuo_entity entity, cuo_row *out)
{
    uint64_t id = cuo_resolve(entity);
    for (size_t i = 0; i < q.len; i++) {
        ModAbi_Row_table_t t = ModAbi_Row_vec_at((ModAbi_Row_vec_t)q.vec, i);
        if (ModAbi_Row_entity(t) == id) {
            if (out)
                out->table = t;
            return true;
        }
    }
    return false;
}

uint64_t cuo_row_entity(cuo_row r)
{
    return r.table ? ModAbi_Row_entity((ModAbi_Row_table_t)r.table) : 0;
}

size_t cuo_row_comp_count(cuo_row r)
{
    return r.table ? ModAbi_CompValue_vec_len(ModAbi_Row_comps((ModAbi_Row_table_t)r.table)) : 0;
}

cuo_bytes cuo_row_comp(cuo_row r, size_t i)
{
    cuo_bytes none = { NULL, 0 };
    if (!r.table)
        return none;
    ModAbi_CompValue_vec_t comps = ModAbi_Row_comps((ModAbi_Row_table_t)r.table);
    if (i >= ModAbi_CompValue_vec_len(comps))
        return none;
    return comp_bytes(ModAbi_CompValue_vec_at(comps, i));
}

cuo_bytes cuo_events_at(cuo_events ev, size_t i)
{
    cuo_bytes none = { NULL, 0 };
    if (i >= ev.len)
        return none;
    return comp_bytes(ModAbi_CompValue_vec_at((ModAbi_CompValue_vec_t)ev.vec, i));
}

uint32_t cuo_obs_id(const cuo_obs *ev)
{
    return ev->t ? ModAbi_ObserverInput_obs_id(ev->t) : 0;
}

uint64_t cuo_obs_entity(const cuo_obs *ev)
{
    return ev->entity;
}

cuo_bytes cuo_obs_value(const cuo_obs *ev)
{
    cuo_bytes none = { NULL, 0 };
    return ev->t ? comp_bytes(ModAbi_ObserverInput_value(ev->t)) : none;
}

cuo_query cuo_obs_query(const cuo_obs *ev, cuo_param p)
{
    cuo_query none = { NULL, 0 };
    return ev->t ? query_of(ModAbi_ObserverInput_queries(ev->t), p) : none;
}

cuo_bytes cuo_obs_res(const cuo_obs *ev, cuo_param p)
{
    cuo_bytes none = { NULL, 0 };
    return ev->t ? res_of(ModAbi_ObserverInput_resources(ev->t), p) : none;
}

cuo_events cuo_obs_events(const cuo_obs *ev, cuo_param p)
{
    cuo_events none = { NULL, 0 };
    return ev->t ? events_of(ModAbi_ObserverInput_events(ev->t), p) : none;
}

cuo_bytes cuo_obs_packet(const cuo_obs *ev)
{
    cuo_bytes b = { NULL, 0 };
    if (!ev->t)
        return b;
    flatbuffers_uint8_vec_t d = ModAbi_ObserverInput_packet(ev->t);
    b.len = flatbuffers_uint8_vec_len(d);
    b.ptr = b.len ? d : NULL;
    return b;
}

cuo_dir cuo_obs_packet_dir(const cuo_obs *ev)
{
    return ev->t && ModAbi_ObserverInput_packet_direction(ev->t) == ModAbi_PacketDirection_Outgoing ? CUO_OUTGOING
                                                                                                     : CUO_INCOMING;
}
