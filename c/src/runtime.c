/* Arena, scratch, registration and the ABI exports — twin of rust/src/{arena,runtime}.rs.
 *
 * Lifecycle (host drives, serially; the guest is single-threaded, so plain statics):
 * 1. mod_setup(Handshake) — intern the type-path table, call the mod's cuo_setup,
 *    return the SetupReply.
 * 2. mod_run / mod_observer — dispatch to the registered callback, return its
 *    CommandBuffer (0 = none).
 * 3. mod_filter / mod_filter_out — packet filters, nonzero blocks.
 * 4. mod_spawned — the host resolved the temp ids of the buffer it just applied.
 */
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "internal.h"

#define EXPORT(name) __attribute__((export_name(#name)))
#define ALIGN8(n) (((n) + 7u) & ~(size_t)7u)

CUO_IMPORT(log) void cuo__imp_log(const char *ptr, uint32_t len);

_Noreturn void cuo__trap(const char *msg)
{
    cuo__imp_log(msg, (uint32_t)strlen(msg));
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

void cuo_log(const char *msg)
{
    cuo__imp_log(msg, (uint32_t)strlen(msg));
}

void cuo_logf(const char *fmt, ...)
{
    char stack[256];
    va_list ap;
    va_start(ap, fmt);
    int n = vsnprintf(stack, sizeof stack, fmt, ap);
    va_end(ap);
    if (n < (int)sizeof stack) {
        cuo__imp_log(stack, (uint32_t)n);
        return;
    }
    char *d = scratch_raw((size_t)n + 1);
    va_start(ap, fmt);
    vsnprintf(d, (size_t)n + 1, fmt, ap);
    va_end(ap);
    cuo__imp_log(d, (uint32_t)n);
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

/* ── type table + named entities ───────────────────────────────────────────── */

static cuo__map types;
cuo__map cuo__named;

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
        cuo__trap(cuo_fmt("cuo: the host does not register the type path '%s' "
                          "(see paths.h / the host registry)", path));
    return id;
}

const char *cuo_type_path(uint16_t id)
{
    for (size_t i = 0; i < types.cap; i++)
        if (types.keys[i] && types.keys[i] != TOMB && types.vals[i] == id)
            return types.keys[i];
    return NULL;
}

bool cuo_entity(const char *name, uint64_t *out)
{
    return cuo__map_get(&cuo__named, name, out);
}

void cuo_forget(const char *name)
{
    cuo__map_remove(&cuo__named, name, NULL);
}

/* ── registration ────────────────────────────────────────────────────────── */

typedef struct query_decl {
    cuo_term *terms;
    size_t n;
} query_decl;

typedef struct sys_rec {
    cuo_system_fn fn;
    void *user;
    char *name;
    uint8_t schedule;
    char *custom_stage;
    query_decl *queries;
    size_t nq, capq;
    uint32_t *after, *before;
    size_t nafter, capafter, nbefore, capbefore;
    uint32_t interval_ms;
} sys_rec;

typedef struct obs_rec {
    cuo_observer_fn fn;
    void *user;
    uint8_t kind;
    uint16_t type_id;
    char *event_name;
} obs_rec;

struct cuo_builder {
    bool open;
};

static cuo_builder the_builder;
static sys_rec *systems;
static size_t nsys, capsys;
static obs_rec *observers;
static size_t nobs, capobs;
static cuo_packet_fn filter_in, filter_out;
static void *filter_in_user, *filter_out_user;
static uint64_t last_tick;

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

static cuo_sys add_system(cuo_builder *m, const char *label, uint8_t schedule, const char *custom,
                          cuo_system_fn fn, void *user)
{
    require_setup(m);
    sys_rec r = { 0 };
    r.fn = fn;
    r.user = user;
    r.schedule = schedule;
    r.name = label ? xstrdup(label) : xstrdup(cuo_fmt("sys-%zu", nsys));
    r.custom_stage = custom ? xstrdup(custom) : NULL;
    PUSH(systems, nsys, capsys, r);
    return (cuo_sys)(nsys - 1);
}

cuo_sys cuo_add_system(cuo_builder *m, const char *label, cuo_stage stage, cuo_system_fn fn, void *user)
{
    return add_system(m, label, (uint8_t)stage, NULL, fn, user);
}

cuo_sys cuo_add_system_in(cuo_builder *m, const char *label, const char *custom_stage, cuo_system_fn fn,
                          void *user)
{
    return add_system(m, label, 6 /* ModAbi Schedule.Custom */, custom_stage, fn, user);
}

uint32_t cuo_system_query(cuo_builder *m, cuo_sys s, const cuo_term *terms, size_t n)
{
    sys_rec *r = sys_at(m, s);
    query_decl q = { xrealloc(NULL, (n ? n : 1) * sizeof(cuo_term)), 0 };
    for (size_t i = 0; i < n; i++) {
        cuo_term t = terms[i];
        size_t at = q.n;
        if (t.kind != CUO_TERM_WITHOUT)
            for (size_t j = 0; j < q.n; j++)
                if (q.terms[j].type_id == t.type_id && q.terms[j].kind != CUO_TERM_WITHOUT) {
                    at = j;
                    break;
                }
        if (at < q.n) {
            /* Two terms on one type would put two payloads in the row and shift
             * every later slot: fold instead. */
            if (t.kind == CUO_TERM_CHANGED)
                q.terms[at].kind = CUO_TERM_CHANGED;
            else if (q.terms[at].kind == CUO_TERM_WITH && t.kind != CUO_TERM_WITH)
                q.terms[at].kind = t.kind;
            continue;
        }
        q.terms[q.n++] = t;
    }
    PUSH(r->queries, r->nq, r->capq, q);
    return (uint32_t)(r->nq - 1);
}

void cuo_system_every(cuo_builder *m, cuo_sys s, uint32_t ms)
{
    sys_at(m, s)->interval_ms = ms;
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

static uint32_t add_observer(cuo_builder *m, uint8_t kind, uint16_t type_id, const char *event,
                             cuo_observer_fn fn, void *user)
{
    require_setup(m);
    obs_rec r = { fn, user, kind, type_id, event ? xstrdup(event) : NULL };
    PUSH(observers, nobs, capobs, r);
    return (uint32_t)(nobs - 1);
}

uint32_t cuo_on_event(cuo_builder *m, const char *event_path, cuo_observer_fn fn, void *user)
{
    return add_observer(m, ModAbi_ObserverKind_Custom, CUO_NONE_TYPE, event_path, fn, user);
}

uint32_t cuo_on_insert(cuo_builder *m, uint16_t type_id, cuo_observer_fn fn, void *user)
{
    return add_observer(m, ModAbi_ObserverKind_Insert, type_id, NULL, fn, user);
}

uint32_t cuo_on_remove(cuo_builder *m, uint16_t type_id, cuo_observer_fn fn, void *user)
{
    return add_observer(m, ModAbi_ObserverKind_Remove, type_id, NULL, fn, user);
}

uint32_t cuo_on_spawn(cuo_builder *m, cuo_observer_fn fn, void *user)
{
    return add_observer(m, ModAbi_ObserverKind_Spawn, CUO_NONE_TYPE, NULL, fn, user);
}

uint32_t cuo_on_despawn(cuo_builder *m, cuo_observer_fn fn, void *user)
{
    return add_observer(m, ModAbi_ObserverKind_Despawn, CUO_NONE_TYPE, NULL, fn, user);
}

void cuo_on_packet_in(cuo_builder *m, cuo_packet_fn fn, void *user)
{
    require_setup(m);
    if (filter_in)
        cuo__trap("cuo: cuo_on_packet_in registered twice (one mod_filter export per mod)");
    filter_in = fn;
    filter_in_user = user;
}

void cuo_on_packet_out(cuo_builder *m, cuo_packet_fn fn, void *user)
{
    require_setup(m);
    if (filter_out)
        cuo__trap("cuo: cuo_on_packet_out registered twice (one mod_filter_out export per mod)");
    filter_out = fn;
    filter_out_user = user;
}

uint64_t cuo_tick(void)
{
    return last_tick;
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
        if (r->custom_stage)
            ModAbi_SystemDecl_custom_stage_create_str(B, r->custom_stage);

        ModAbi_SystemDecl_params_start(B);
        /* Every system gets a Commands param (like the C# SDK): the SDK always hands
         * the callback a command buffer, and an unused one costs the host nothing. */
        ModAbi_SystemDecl_params_push_start(B);
        ModAbi_ParamDecl_kind_add(B, ModAbi_ParamKind_Commands);
        ModAbi_SystemDecl_params_push_end(B);
        for (size_t q = 0; q < r->nq; q++) {
            ModAbi_SystemDecl_params_push_start(B);
            ModAbi_ParamDecl_kind_add(B, ModAbi_ParamKind_Query);
            ModAbi_ParamDecl_query_start(B);
            ModAbi_QueryDecl_terms_start(B);
            for (size_t t = 0; t < r->queries[q].n; t++) {
                ModAbi_QueryDecl_terms_push_start(B);
                ModAbi_QueryTerm_kind_add(B, r->queries[q].terms[t].kind);
                ModAbi_QueryTerm_type_id_add(B, r->queries[q].terms[t].type_id);
                ModAbi_QueryDecl_terms_push_end(B);
            }
            ModAbi_QueryDecl_terms_end(B);
            ModAbi_ParamDecl_query_end(B);
            ModAbi_SystemDecl_params_push_end(B);
        }
        ModAbi_SystemDecl_params_end(B);

        if (r->nafter)
            ModAbi_SystemDecl_after_create(B, r->after, r->nafter);
        if (r->nbefore)
            ModAbi_SystemDecl_before_create(B, r->before, r->nbefore);
        ModAbi_SystemDecl_interval_ms_add(B, r->interval_ms);
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
        ModAbi_SetupReply_observers_push_end(B);
    }
    ModAbi_SetupReply_observers_end(B);

    ModAbi_SetupReply_wants_filter_add(B, filter_in != NULL);
    ModAbi_SetupReply_wants_filter_out_add(B, filter_out != NULL);
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
        cuo__trap(cuo_fmt("cuo: mod ABI mismatch: host speaks v%u, this mod was built against v%u "
                          "- rebuild the mod against the current SDK",
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
    last_tick = in.t ? ModAbi_SystemInput_tick(in.t) : 0;
    cuo_cmds *c = cuo__cmds_instance();
    cuo__cmds_begin(c);
    systems[sys_id].fn(&in, c, systems[sys_id].user);
    return cuo__cmds_finish(c);
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
    observers[obs_id].fn(&ev, c, observers[obs_id].user);
    return cuo__cmds_finish(c);
}

EXPORT(mod_filter) uint32_t mod_filter(uint32_t id, uint32_t ptr, uint32_t len)
{
    if (!filter_in)
        return 0;
    cuo__scratch_reset();
    return filter_in((uint8_t)id, (const uint8_t *)(uintptr_t)ptr, len, filter_in_user) ? 1u : 0u;
}

EXPORT(mod_filter_out) uint32_t mod_filter_out(uint32_t id, uint32_t ptr, uint32_t len)
{
    if (!filter_out)
        return 0;
    cuo__scratch_reset();
    return filter_out((uint8_t)id, (const uint8_t *)(uintptr_t)ptr, len, filter_out_user) ? 1u : 0u;
}

/* ── input views ─────────────────────────────────────────────────────────── */

uint32_t cuo_input_sys_id(const cuo_input *in)
{
    return in->t ? ModAbi_SystemInput_sys_id(in->t) : 0;
}

uint64_t cuo_input_tick(const cuo_input *in)
{
    return in->t ? ModAbi_SystemInput_tick(in->t) : 0;
}

cuo_query cuo_input_query(const cuo_input *in, uint32_t n)
{
    cuo_query q = { NULL, 0 };
    if (!in->t)
        return q;
    /* The host emits one entry per declared query, in order: n is the ordinal. */
    ModAbi_QueryRows_vec_t qs = ModAbi_SystemInput_queries(in->t);
    if (n >= ModAbi_QueryRows_vec_len(qs))
        return q;
    ModAbi_Row_vec_t rows = ModAbi_QueryRows_rows(ModAbi_QueryRows_vec_at(qs, n));
    q.vec = rows;
    q.len = ModAbi_Row_vec_len(rows);
    return q;
}

cuo_row cuo_query_row(cuo_query q, size_t i)
{
    cuo_row r = { i < q.len ? ModAbi_Row_vec_at((ModAbi_Row_vec_t)q.vec, i) : NULL };
    return r;
}

bool cuo_query_find(cuo_query q, uint64_t entity, cuo_row *out)
{
    for (size_t i = 0; i < q.len; i++) {
        ModAbi_Row_table_t t = ModAbi_Row_vec_at((ModAbi_Row_vec_t)q.vec, i);
        if (ModAbi_Row_entity(t) == entity) {
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
