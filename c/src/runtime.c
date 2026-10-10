/* Scratch, registration, setup and the dispatcher behind the mod's exports.
 *
 * Lifecycle (host drives, serially; the guest is single-threaded, so plain statics):
 * 1. setup(app) — call the mod's cuo_setup, link every system / observer to the
 *    export of the mod's world named like it (gen-exports.awk wrote the table), check
 *    its parameters match, then declare them to the host (app.add-systems /
 *    add-observer) in the export's parameter order.
 * 2. The host calls an export with the params as typed arguments; its generated
 *    trampoline hands them to cuo__dispatch, which fetches each param's data (one host
 *    call per param), runs the registered callback and drops the handles. Commands go
 *    straight to the host's buffer as they are recorded. */
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "internal.h"

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

/* ── scratch arena (call-scoped) ───────────────────────────────────────────── */

typedef struct block {
    struct block *next;
    size_t cap, top;
    _Alignas(8) unsigned char data[];
} block;

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

/* ── string -> u64 map (open addressing, linear probe) ───────────────────────── */

typedef struct strmap {
    char **keys;
    uint64_t *vals;
    size_t cap, count;
} strmap;

static uint32_t hash(const char *s, size_t n)
{
    uint32_t h = 2166136261u;
    for (size_t i = 0; i < n; i++)
        h = (h ^ (unsigned char)s[i]) * 16777619u;
    return h;
}

static bool key_eq(const char *k, const char *s, size_t n)
{
    return strncmp(k, s, n) == 0 && k[n] == 0;
}

static bool map_getn(const strmap *m, const char *key, size_t n, uint64_t *out)
{
    if (!m->cap)
        return false;
    size_t mask = m->cap - 1;
    for (size_t i = hash(key, n) & mask; m->keys[i]; i = (i + 1) & mask)
        if (key_eq(m->keys[i], key, n)) {
            *out = m->vals[i];
            return true;
        }
    return false;
}

static void map_insert(strmap *m, char *key, uint64_t val)
{
    size_t mask = m->cap - 1, i = hash(key, strlen(key)) & mask;
    while (m->keys[i])
        i = (i + 1) & mask;
    m->keys[i] = key;
    m->vals[i] = val;
    m->count++;
}

/* Keys are copied. The key must not be present. */
static void map_put(strmap *m, const char *key, uint64_t val)
{
    if ((m->count + 1) * 4 >= m->cap * 3) {
        strmap old = *m;
        m->cap = old.cap ? old.cap * 2 : 64;
        m->keys = calloc(m->cap, sizeof(char *));
        m->vals = xrealloc(NULL, m->cap * sizeof(uint64_t));
        m->count = 0;
        for (size_t i = 0; i < old.cap; i++)
            if (old.keys[i])
                map_insert(m, old.keys[i], old.vals[i]);
        free(old.keys);
        free(old.vals);
    }
    map_insert(m, xstrdup(key), val);
}

/* ── type table: guest-local ids for type paths ──────────────────────────────── */

static strmap types;
static char **type_paths;
static size_t ntypes, captypes;

bool cuo_try_type_id(const char *path, uint16_t *out)
{
    uint64_t v;
    if (!map_getn(&types, path, strlen(path), &v)) {
        if (ntypes >= 0xFFFF)
            cuo__trap("cuo: too many type paths");
        v = ntypes;
        PUSH(type_paths, ntypes, captypes, xstrdup(path));
        map_put(&types, path, v);
    }
    if (out)
        *out = (uint16_t)v;
    return true;
}

uint16_t cuo_type_id(const char *path)
{
    uint16_t id;
    cuo_try_type_id(path, &id);
    return id;
}

const char *cuo_type_path(uint16_t id)
{
    return id < ntypes ? type_paths[id] : NULL;
}

/* ── registration ────────────────────────────────────────────────────────── */

typedef struct param_decl {
    uint8_t kind; /* CUO__K_QUERY / CUO__K_RES / CUO__K_EVENTS */
    bool mut;
    uint16_t type_id;
    cuo_term *terms;
    size_t n;
    /* query: every read term has a typed column, so the call starts with `entities`
     * instead of `rows` */
    bool typed;
    /* res: the value received on the previous call, reused while res.unchanged. */
    cuo__res_cache res;
} param_decl;

typedef struct param_list {
    param_decl *v;
    size_t n, cap;
} param_list;

enum { OBS_ADD, OBS_REMOVE, OBS_EVENT, OBS_PACKET };

typedef struct entry {
    char *name; /* kebab-case: the export's name */
    bool is_observer;
    param_list params; /* without commands: the export's signature places that */
    uint64_t runs;
    /* the export, linked once cuo_setup returned; [first, end) = its ECS params */
    const cuo__export *x;
    size_t first, end;
    /* system */
    cuo_system_fn fn;
    uint8_t schedule;
    bool run_on_change;
    uint32_t *after, *before;
    size_t nafter, capafter, nbefore, capbefore;
    /* observer */
    cuo_observer_fn obs_fn;
    uint8_t obs_kind;
    /* a typed observer export (entity first): its record kind (CUO__K_TYPED + n), 0 = a tag */
    bool typed;
    uint8_t typed_kind;
    uint16_t type_id;
    char *event_name;
    cuo_packet_fn packet_fn;
    cuo_packet_tap_fn tap_fn;
    uint8_t packet_dir;
    uint8_t *packet_ids;
    size_t npacket_ids;
    void *user;
} entry;

struct cuo_builder {
    bool open;
};

static cuo_builder the_builder;
/* Systems and observers share one table (looked up by name); cuo_sys / cuo_observer
 * index their own lists below. */
static entry *entries;
static size_t nentries, capentries;
static uint32_t *sys_idx, *obs_idx;
static size_t nsys, capsys, nobs, capobs;
static strmap by_name;

static void require_setup(cuo_builder *m)
{
    if (!m || !m->open)
        cuo__trap("cuo: registration functions are only valid inside cuo_setup");
}

static entry *sys_at(cuo_builder *m, cuo_sys s)
{
    require_setup(m);
    if (s >= nsys)
        cuo__trap("cuo: unknown system id");
    return &entries[sys_idx[s]];
}

static entry *obs_at(cuo_builder *m, cuo_observer o)
{
    require_setup(m);
    if (o >= nobs)
        cuo__trap("cuo: unknown observer id");
    return &entries[obs_idx[o]];
}

static cuo_param add_param(param_list *l, uint8_t kind, bool mut, uint16_t type_id, cuo_term *terms, size_t n)
{
    param_decl p = { kind, mut, type_id, terms, n };
    PUSH(l->v, l->n, l->cap, p);
    return (cuo_param)(l->n - 1);
}

static bool reads(uint8_t kind)
{
    return kind != CUO_TERM_WITH && kind != CUO_TERM_WITHOUT;
}

/* Two terms on one type would put two payloads in the row and shift every later
 * slot: fold instead (a WITH on a read type is dropped, CHANGED / ADDED / MUT upgrade). */
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
            if (t.kind == CUO_TERM_CHANGED || t.kind == CUO_TERM_ADDED || t.kind == CUO_TERM_MUT)
                q[at].kind = t.kind;
            else if (q[at].kind == CUO_TERM_WITH && reads(t.kind))
                q[at].kind = t.kind;
            continue;
        }
        q[qn++] = t;
    }
    cuo_param p = add_param(l, CUO__K_QUERY, false, 0, q, qn);
    bool typed = true;
    for (size_t i = 0; i < qn; i++)
        if (reads(q[i].kind) && !cuo__typed_readable(q[i].type_id))
            typed = false;
    l->v[p].typed = typed;
    return p;
}

static entry *new_entry(const char *name, bool is_observer, void *user)
{
    if (!name || !*name)
        cuo__trap("cuo: systems and observers are named like their export in wit/world.wit");
    entry e = { 0 };
    /* The C spelling (spin_cube) of a kebab-case export name (spin-cube) works too. */
    e.name = xstrdup(name);
    for (char *p = e.name; *p; p++)
        if (*p == '_')
            *p = '-';
    e.is_observer = is_observer;
    e.user = user;
    uint64_t dup;
    if (map_getn(&by_name, e.name, strlen(e.name), &dup))
        cuo__trap(cuo_fmt("cuo: two systems / observers named '%s'", e.name));
    map_put(&by_name, e.name, nentries);
    PUSH(entries, nentries, capentries, e);
    return &entries[nentries - 1];
}

cuo_sys cuo_add_system(cuo_builder *m, const char *name, cuo_stage stage, cuo_system_fn fn, void *user)
{
    require_setup(m);
    entry *e = new_entry(name, false, user);
    e->fn = fn;
    e->schedule = (uint8_t)stage;
    uint32_t at = (uint32_t)(nentries - 1);
    PUSH(sys_idx, nsys, capsys, at);
    return (cuo_sys)(nsys - 1);
}

cuo_param cuo_system_query(cuo_builder *m, cuo_sys s, const cuo_term *terms, size_t n)
{
    return add_query(&sys_at(m, s)->params, terms, n);
}

cuo_param cuo_system_res(cuo_builder *m, cuo_sys s, uint16_t type_id, bool mut)
{
    return add_param(&sys_at(m, s)->params, CUO__K_RES, mut, type_id, NULL, 0);
}

cuo_param cuo_system_events(cuo_builder *m, cuo_sys s, uint16_t type_id)
{
    return add_param(&sys_at(m, s)->params, CUO__K_EVENTS, false, type_id, NULL, 0);
}

void cuo_system_after(cuo_builder *m, cuo_sys s, cuo_sys other)
{
    entry *r = sys_at(m, s);
    PUSH(r->after, r->nafter, r->capafter, other);
}

void cuo_system_before(cuo_builder *m, cuo_sys s, cuo_sys other)
{
    entry *r = sys_at(m, s);
    PUSH(r->before, r->nbefore, r->capbefore, other);
}

void cuo_system_run_on_change(cuo_builder *m, cuo_sys s)
{
    sys_at(m, s)->run_on_change = true;
}

static cuo_observer add_observer(cuo_builder *m, const char *name, uint8_t kind, uint16_t type_id,
                                 const char *event, cuo_observer_fn fn, void *user)
{
    require_setup(m);
    entry *e = new_entry(name, true, user);
    e->obs_fn = fn;
    e->obs_kind = kind;
    e->type_id = type_id;
    e->event_name = event ? xstrdup(event) : NULL;
    uint32_t at = (uint32_t)(nentries - 1);
    PUSH(obs_idx, nobs, capobs, at);
    return (cuo_observer)(nobs - 1);
}

cuo_observer cuo_on_event(cuo_builder *m, const char *name, const char *event_path, cuo_observer_fn fn, void *user)
{
    return add_observer(m, name, OBS_EVENT, 0, event_path, fn, user);
}

cuo_observer cuo_on_add(cuo_builder *m, const char *name, uint16_t type_id, cuo_observer_fn fn, void *user)
{
    return add_observer(m, name, OBS_ADD, type_id, NULL, fn, user);
}

cuo_observer cuo_on_remove(cuo_builder *m, const char *name, uint16_t type_id, cuo_observer_fn fn, void *user)
{
    return add_observer(m, name, OBS_REMOVE, type_id, NULL, fn, user);
}

cuo_param cuo_observer_query(cuo_builder *m, cuo_observer o, const cuo_term *terms, size_t n)
{
    return add_query(&obs_at(m, o)->params, terms, n);
}

cuo_param cuo_observer_res(cuo_builder *m, cuo_observer o, uint16_t type_id, bool mut)
{
    return add_param(&obs_at(m, o)->params, CUO__K_RES, mut, type_id, NULL, 0);
}

cuo_param cuo_observer_events(cuo_builder *m, cuo_observer o, uint16_t type_id)
{
    return add_param(&obs_at(m, o)->params, CUO__K_EVENTS, false, type_id, NULL, 0);
}

static cuo_observer add_packet_observer(cuo_builder *m, const char *name, cuo_dir dir, const uint8_t *ids,
                                        size_t n, cuo_packet_fn fn, cuo_packet_tap_fn tap, void *user)
{
    cuo_observer o = add_observer(m, name, OBS_PACKET, 0, NULL, NULL, user);
    entry *r = &entries[obs_idx[o]];
    r->packet_fn = fn;
    r->tap_fn = tap;
    r->packet_dir = (uint8_t)dir;
    if (n) {
        r->packet_ids = memcpy(xrealloc(NULL, n), ids, n);
        r->npacket_ids = n;
    }
    return o;
}

cuo_observer cuo_on_packet(cuo_builder *m, const char *name, cuo_dir dir, const uint8_t *ids, size_t n,
                           cuo_packet_fn fn, void *user)
{
    return add_packet_observer(m, name, dir, ids, n, fn, NULL, user);
}

cuo_observer cuo_on_packet_in(cuo_builder *m, const char *name, cuo_packet_tap_fn fn, void *user)
{
    return add_packet_observer(m, name, CUO_INCOMING, NULL, 0, NULL, fn, user);
}

cuo_observer cuo_on_packet_out(cuo_builder *m, const char *name, cuo_packet_tap_fn fn, void *user)
{
    return add_packet_observer(m, name, CUO_OUTGOING, NULL, 0, NULL, fn, user);
}

/* ── linking a declaration to its export ─────────────────────────────────── */

/* The SDK's own system (cuo_hotkey's Startup publisher): an export of cuo:c-sdk/mod,
 * which every C mod's world includes. */
static const uint8_t hotkeys_kinds[] = { CUO__K_COMMANDS };
static const cuo__export hotkeys_export = { "cuo-sdk-hotkeys", hotkeys_kinds, 1 };

void exports_cuo_wit_cuo_sdk_hotkeys(tinyecs_modding_ecs_own_commands_t commands)
{
    cuo__arg a[] = { CUO__ARG(commands) };
    cuo__dispatch(&hotkeys_export, a);
}

static const cuo__export *find_export(const char *name)
{
    for (size_t i = 0; i < cuo__nexports; i++)
        if (strcmp(cuo__exports[i].name, name) == 0)
            return &cuo__exports[i];
    return strcmp(name, hotkeys_export.name) == 0 ? &hotkeys_export : NULL;
}

static const char *wit_type(uint8_t kind)
{
    switch (kind) {
    case CUO__K_QUERY: return "query";
    case CUO__K_RES: return "res";
    default: return "events";
    }
}

/* The export line wit/world.wit needs for this declaration. */
static const char *expected_export(const entry *e)
{
    bool packet = e->is_observer && e->obs_kind == OBS_PACKET;
    const char *s = cuo_fmt("export %s: func(%scommands: commands", e->name,
                            packet ? "direction: packet-direction, packet: list<u8>, "
                                   : e->is_observer ? "trigger: trigger-data, " : "");
    for (size_t i = 0; i < e->params.n; i++)
        s = cuo_fmt("%s, p%zu: %s", s, i, wit_type(e->params.v[i].kind));
    return cuo_fmt("%s)%s;", s, packet ? " -> verdict" : "");
}

/* The export's ECS params must be the declared ones, in order, with at most one
 * `commands` anywhere among them (none: the callback's cuo_cmds traps on use). */
static void link_export(entry *e)
{
    const cuo__export *x = find_export(e->name);
    if (!x)
        cuo__trap(cuo_fmt("cuo: '%s' is declared in cuo_setup but the mod's world has no export for it: add `%s` "
                          "to wit/world.wit",
                          e->name, expected_export(e)));
    size_t first = 0, end = x->n;
    bool ok = true;
    if (e->is_observer && e->obs_kind == OBS_PACKET) {
        ok = x->n >= 3 && x->kinds[0] == CUO__K_DIR && x->kinds[1] == CUO__K_PACKET &&
             x->kinds[x->n - 1] == CUO__K_VERDICT;
        first = 2;
        end = ok ? x->n - 1 : 0;
    } else if (e->is_observer && x->n >= 1 && x->kinds[0] == CUO__K_ENTITY) {
        /* Typed: `entity: entity, value: <record>` (a tag: entity alone). */
        int kind = cuo__typed_trigger_kind(e->obs_kind == OBS_EVENT ? e->event_name : cuo_type_path(e->type_id));
        if (kind < 0)
            cuo__trap(cuo_fmt("cuo: the export '%s' takes its trigger typed (`entity: entity, ...`), but that type has "
                              "no typed trigger: declare it `trigger: trigger-data`",
                              e->name));
        ok = kind == 0 || (x->n >= 2 && x->kinds[1] == kind);
        first = kind == 0 ? 1 : 2;
        e->typed = true;
        e->typed_kind = (uint8_t)kind;
    } else if (e->is_observer) {
        ok = x->n >= 1 && x->kinds[0] == CUO__K_TRIGGER;
        first = 1;
    }
    size_t p = 0;
    bool cmds = false;
    for (size_t i = first; ok && i < end; i++) {
        uint8_t k = x->kinds[i];
        if (k == CUO__K_COMMANDS) {
            ok = !cmds;
            cmds = true;
        } else {
            ok = p < e->params.n && e->params.v[p++].kind == k;
        }
    }
    if (!ok || p != e->params.n)
        cuo__trap(cuo_fmt("cuo: the export '%s' in wit/world.wit does not match its declaration in cuo_setup: "
                          "expected `%s` (`commands` may sit anywhere among the params, or be left out)",
                          e->name, expected_export(e)));
    e->x = x;
    e->first = first;
    e->end = end;
}

/* ── setup: declare to the host ──────────────────────────────────────────── */

static cuo_wit_string_t path_of(uint16_t type_id)
{
    return cuo__wstr(cuo_type_path(type_id));
}

/* In the export's order: the order the host passes them back in. */
static void declare_params(tinyecs_modding_ecs_borrow_system_t sys, const entry *e)
{
    size_t p = 0;
    for (size_t i = e->first; i < e->end; i++) {
        if (e->x->kinds[i] == CUO__K_COMMANDS) {
            tinyecs_modding_ecs_method_system_add_commands(sys);
            continue;
        }
        const param_decl *d = &e->params.v[p++];
        cuo_wit_string_t path = path_of(d->type_id);
        switch (d->kind) {
        case CUO__K_QUERY: {
            tinyecs_modding_ecs_term_t *t = cuo_alloc((d->n ? d->n : 1) * sizeof *t);
            for (size_t j = 0; j < d->n; j++) {
                /* cuo_term_kind values are the WIT variant's cases; every case is a path. */
                t[j].tag = d->terms[j].kind;
                t[j].val.ref = path_of(d->terms[j].type_id);
            }
            tinyecs_modding_ecs_list_term_t terms = { t, d->n };
            tinyecs_modding_ecs_method_system_add_query(sys, &terms);
            break;
        }
        case CUO__K_RES:
            if (d->mut)
                tinyecs_modding_ecs_method_system_add_res_mut(sys, &path);
            else
                tinyecs_modding_ecs_method_system_add_res(sys, &path);
            break;
        case CUO__K_EVENTS:
            tinyecs_modding_ecs_method_system_add_events(sys, &path);
            break;
        }
    }
}

void exports_cuo_wit_setup(tinyecs_modding_ecs_own_app_t own_app)
{
    cuo__scratch_reset();
    the_builder.open = true;
    cuo_setup(&the_builder);
    cuo__publish_hotkeys(&the_builder);
    the_builder.open = false;
    for (size_t i = 0; i < nentries; i++)
        link_export(&entries[i]);

    tinyecs_modding_ecs_borrow_app_t app = tinyecs_modding_ecs_borrow_app(own_app);
    /* Every handle first: `after` / `before` borrow the other system. */
    tinyecs_modding_ecs_own_system_t *own = cuo_alloc((nentries ? nentries : 1) * sizeof *own);
    for (size_t i = 0; i < nentries; i++) {
        cuo_wit_string_t name = cuo__wstr(entries[i].name);
        own[i] = tinyecs_modding_ecs_constructor_system(&name);
    }
#define BORROW(i) tinyecs_modding_ecs_borrow_system(own[(i)])
    for (size_t i = 0; i < nentries; i++) {
        entry *e = &entries[i];
        tinyecs_modding_ecs_borrow_system_t sys = BORROW(i);
        for (size_t j = 0; j < e->nafter; j++)
            tinyecs_modding_ecs_method_system_after(sys, BORROW(sys_idx[e->after[j]]));
        for (size_t j = 0; j < e->nbefore; j++)
            tinyecs_modding_ecs_method_system_before(sys, BORROW(sys_idx[e->before[j]]));
        if (e->run_on_change)
            tinyecs_modding_ecs_method_system_run_on_change(sys);
        declare_params(sys, e);

        if (!e->is_observer) {
            tinyecs_modding_ecs_list_borrow_system_t one = { &sys, 1 };
            tinyecs_modding_ecs_method_app_add_systems(app, e->schedule, &one);
            continue;
        }
        tinyecs_modding_ecs_trigger_t trig;
        switch (e->obs_kind) {
        case OBS_ADD:
            trig.tag = TINYECS_MODDING_ECS_TRIGGER_ON_ADD;
            trig.val.on_add = path_of(e->type_id);
            break;
        case OBS_REMOVE:
            trig.tag = TINYECS_MODDING_ECS_TRIGGER_ON_REMOVE;
            trig.val.on_remove = path_of(e->type_id);
            break;
        case OBS_EVENT:
            trig.tag = TINYECS_MODDING_ECS_TRIGGER_ON_EVENT;
            trig.val.on_event = cuo__wstr(e->event_name);
            break;
        default:
            trig.tag = TINYECS_MODDING_ECS_TRIGGER_ON_PACKET;
            trig.val.on_packet.direction = e->packet_dir;
            trig.val.on_packet.ids.ptr = e->packet_ids;
            trig.val.on_packet.ids.len = e->npacket_ids;
            break;
        }
        tinyecs_modding_ecs_method_app_add_observer(app, &trig, sys);
    }
#undef BORROW
    for (size_t i = 0; i < nentries; i++)
        tinyecs_modding_ecs_system_drop_own(own[i]);
    tinyecs_modding_ecs_app_drop_own(own_app);
}

/* ── dispatch: the host called an export ─────────────────────────────────── */

static void drop_res_typed(cuo__res_cache *r)
{
    if (r->typed_valid && r->typed_has)
        cuo__typed_res_free(r->typed_kind, r->typed);
    r->typed_valid = false;
}

static void drop_res_json(cuo__res_cache *r)
{
    if (r->json_valid && r->json_has)
        tinyecs_modding_ecs_json_free(&r->json);
    r->json_valid = false;
}

static void fetch_rows(cuo__pval *v)
{
    tinyecs_modding_ecs_method_query_rows((tinyecs_modding_ecs_borrow_query_t){ v->handle }, &v->rows);
    v->has_rows = true;
    v->len = v->rows.len;
}

static void fetch_ents(cuo__pval *v)
{
    tinyecs_modding_ecs_method_query_entities((tinyecs_modding_ecs_borrow_query_t){ v->handle }, &v->ents);
    v->has_ents = true;
    v->ent_view = v->ents.ptr;
    v->len = v->ents.len;
}

/* One host call per query / events as the call starts; a res only asks whether its
 * value changed (its value is fetched on first use, and reused while unchanged). */
static void fetch(param_decl *d, cuo__pval *v, int32_t handle)
{
    v->tag = d->kind;
    v->type_id = d->type_id;
    v->mut = d->mut;
    v->handle = handle;
    switch (d->kind) {
    case CUO__K_QUERY:
        if (d->typed)
            fetch_ents(v);
        else
            fetch_rows(v);
        break;
    case CUO__K_RES: {
        cuo__res_cache *r = &d->res;
        if (!r->known || !tinyecs_modding_ecs_method_res_unchanged((tinyecs_modding_ecs_borrow_res_t){ handle })) {
            drop_res_json(r);
            drop_res_typed(r);
        }
        r->known = true;
        v->res = r;
        break;
    }
    case CUO__K_EVENTS:
        break;
    }
}

static void release(cuo__params *ps, cuo_cmds *c)
{
    if (c->handle.__handle >= 0)
        tinyecs_modding_ecs_commands_drop_own((tinyecs_modding_ecs_own_commands_t){ c->handle.__handle });
    for (size_t i = 0; i < ps->n; i++) {
        cuo__pval *v = &ps->v[i];
        switch (v->tag) {
        case CUO__K_QUERY:
            if (v->has_rows)
                tinyecs_modding_ecs_list_row_free(&v->rows);
            if (v->has_ents)
                cuo_wit_list_entity_free(&v->ents);
            tinyecs_modding_ecs_query_drop_own((tinyecs_modding_ecs_own_query_t){ v->handle });
            break;
        case CUO__K_RES:
            tinyecs_modding_ecs_res_drop_own((tinyecs_modding_ecs_own_res_t){ v->handle });
            break;
        case CUO__K_EVENTS:
            if (v->has_events)
                cuo_wit_list_json_free(&v->events);
            tinyecs_modding_ecs_events_drop_own((tinyecs_modding_ecs_own_events_t){ v->handle });
            break;
        }
    }
}

static cuo_bytes json_bytes(const cuo_wit_string_t *s)
{
    cuo_bytes b = { s->len ? s->ptr : NULL, s->len };
    return b;
}

void cuo__dispatch(const cuo__export *x, const cuo__arg *a)
{
    cuo__scratch_reset();
    uint64_t at;
    if (!map_getn(&by_name, x->name, strlen(x->name), &at) || entries[at].x != x)
        cuo__trap(cuo_fmt("cuo: export '%s' was called but cuo_setup declared nothing by that name", x->name));
    entry *e = &entries[at];
    cuo__params ps = { cuo_alloc((e->params.n ? e->params.n : 1) * sizeof(cuo__pval)), e->params.n };
    cuo_cmds c = { { -1 }, &ps, 0, e->name };
    for (size_t i = e->first, p = 0; i < e->end; i++) {
        if (x->kinds[i] == CUO__K_COMMANDS) {
            c.handle.__handle = a[i].handle;
        } else {
            fetch(&e->params.v[p], &ps.v[p], a[i].handle);
            p++;
        }
    }
    e->runs++;

    if (!e->is_observer) {
        cuo_input in = { (uint32_t)at, e->runs, &ps };
        e->fn(&in, &c, e->user);
        release(&ps, &c);
        return;
    }

    cuo_obs ev = { 0 };
    ev.obs_id = (uint32_t)at;
    ev.params = &ps;
    if (e->typed) {
        ev.entity = a[0].entity;
        if (e->typed_kind) {
            ev.typed = cuo__typed_trigger_value(e->typed_kind, a[1].ptr);
            ev.typed_kind = e->typed_kind;
        }
        e->obs_fn(&ev, &c, e->user);
        release(&ps, &c);
        return;
    }
    if (e->obs_kind != OBS_PACKET) {
        tinyecs_modding_ecs_trigger_data_t *trigger = a[0].ptr;
        ev.entity = trigger->entity;
        ev.value = json_bytes(&trigger->value);
        e->obs_fn(&ev, &c, e->user);
        release(&ps, &c);
        tinyecs_modding_ecs_trigger_data_free(trigger);
        return;
    }

    cuo_wit_list_u8_t *packet = a[1].ptr;
    tinyecs_modding_ecs_verdict_t *ret = a[x->n - 1].ptr;
    ev.packet.ptr = packet->ptr;
    ev.packet.len = packet->len;
    ev.dir = a[0].dir == TINYECS_MODDING_ECS_PACKET_DIRECTION_OUTGOING ? CUO_OUTGOING : CUO_INCOMING;
    cuo_bytes replacement = { NULL, 0 };
    cuo_verdict v;
    if (e->packet_fn)
        v = e->packet_fn(&ev, &c, &replacement, e->user);
    else
        v = packet->len && e->tap_fn(packet->ptr[0], packet->ptr, packet->len, e->user) ? CUO_BLOCK : CUO_PASS;
    ret->tag = (uint8_t)v;
    if (v == CUO_REPLACE) {
        /* Freed by the post-return hook: it must be malloc'd, and may alias `packet`. */
        uint8_t *copy = xrealloc(NULL, replacement.len ? replacement.len : 1);
        if (replacement.len)
            memcpy(copy, replacement.ptr, replacement.len);
        ret->val.replace.ptr = copy;
        ret->val.replace.len = replacement.len;
    }
    release(&ps, &c);
    cuo_wit_list_u8_free(packet);
}

/* ── input views ─────────────────────────────────────────────────────────── */

static const cuo__pval *pval(const cuo__params *ps, cuo_param p, uint8_t tag)
{
    return p < ps->n && ps->v[p].tag == tag ? &ps->v[p] : NULL;
}

static cuo_query query_of(const cuo__params *ps, cuo_param p)
{
    cuo_query q = { NULL, 0, -1 };
    const cuo__pval *v = pval(ps, p, CUO__K_QUERY);
    if (v) {
        q.impl = v;
        q.len = v->len;
        q.handle = v->handle;
    }
    return q;
}

static cuo_bytes res_of(const cuo__params *ps, cuo_param p)
{
    cuo_bytes none = { NULL, 0 };
    const cuo__pval *v = pval(ps, p, CUO__K_RES);
    if (!v)
        return none;
    cuo__res_cache *r = v->res;
    if (!r->json_valid) {
        r->json_has = tinyecs_modding_ecs_method_res_get((tinyecs_modding_ecs_borrow_res_t){ v->handle }, &r->json);
        r->json_valid = true;
    }
    if (!r->json_has)
        return none;
    /* A present marker resource: report "{}", not absence. */
    return r->json.len ? json_bytes(&r->json) : cuo_str_bytes("{}");
}

const void *cuo__res_typed(const cuo__params *ps, cuo_param p, uint16_t kind)
{
    const cuo__pval *v = pval(ps, p, CUO__K_RES);
    if (!v)
        return NULL;
    cuo__res_cache *r = v->res;
    if (r->typed_valid && r->typed_kind != kind)
        drop_res_typed(r);
    if (!r->typed_valid) {
        if (r->typed_kind != kind) {
            free(r->typed);
            r->typed = xrealloc(NULL, cuo__typed_res_size(kind));
        }
        r->typed_kind = kind;
        r->typed_has = cuo__typed_res_get(kind, v->handle, r->typed);
        r->typed_valid = true;
    }
    return r->typed_has ? r->typed : NULL;
}

int32_t cuo__events_handle(const cuo__params *ps, cuo_param p)
{
    const cuo__pval *v = pval(ps, p, CUO__K_EVENTS);
    return v ? v->handle : -1;
}

static cuo_events events_of(const cuo__params *ps, cuo_param p)
{
    cuo_events e = { NULL, 0 };
    cuo__pval *v = (cuo__pval *)pval(ps, p, CUO__K_EVENTS);
    if (v) {
        if (!v->has_events) {
            tinyecs_modding_ecs_method_events_read((tinyecs_modding_ecs_borrow_events_t){ v->handle }, &v->events);
            v->has_events = true;
        }
        e.vec = v->events.ptr;
        e.len = v->events.len;
    }
    return e;
}

uint32_t cuo_input_sys_id(const cuo_input *in) { return in->sys_id; }
uint64_t cuo_input_tick(const cuo_input *in) { return in->tick; }
cuo_query cuo_input_query(const cuo_input *in, cuo_param p) { return query_of(in->params, p); }
cuo_bytes cuo_input_res(const cuo_input *in, cuo_param p) { return res_of(in->params, p); }
cuo_events cuo_input_events(const cuo_input *in, cuo_param p) { return events_of(in->params, p); }

typedef tinyecs_modding_ecs_row_t row_t;

static const row_t *rows_of(cuo_query q)
{
    cuo__pval *v = (cuo__pval *)q.impl;
    if (!v)
        return NULL;
    if (!v->has_rows)
        fetch_rows(v);
    return v->rows.ptr;
}

static const cuo_entity *entities_of(cuo_query q)
{
    cuo__pval *v = (cuo__pval *)q.impl;
    if (!v)
        return NULL;
    if (!v->ent_view) {
        cuo_entity *e = cuo_alloc((v->len ? v->len : 1) * sizeof *e);
        for (size_t i = 0; i < v->len; i++)
            e[i] = v->rows.ptr[i].entity;
        v->ent_view = e;
    }
    return v->ent_view;
}

const cuo_entity *cuo_query_entities(cuo_query q)
{
    return entities_of(q);
}

bool cuo_query_index(cuo_query q, cuo_entity entity, size_t *index)
{
    const cuo_entity *e = entities_of(q);
    for (size_t i = 0; i < q.len; i++)
        if (e[i] == entity) {
            if (index)
                *index = i;
            return true;
        }
    return false;
}

cuo_row cuo_query_row(cuo_query q, size_t i)
{
    cuo_row r = { i < q.len ? &rows_of(q)[i] : NULL };
    return r;
}

bool cuo_query_find(cuo_query q, cuo_entity entity, cuo_row *out)
{
    size_t i;
    if (!cuo_query_index(q, entity, &i))
        return false;
    if (out)
        *out = cuo_query_row(q, i);
    return true;
}

void cuo_query_set(cuo_query q, cuo_entity entity, size_t index, cuo_bytes json)
{
    if (q.handle < 0)
        cuo__trap("cuo: cuo_query_set on an absent query");
    cuo_wit_string_t v = cuo__wbytes(json);
    tinyecs_modding_ecs_method_query_set((tinyecs_modding_ecs_borrow_query_t){ q.handle }, entity, (uint8_t)index,
                                         &v);
}

uint64_t cuo_row_entity(cuo_row r)
{
    return r.table ? ((const row_t *)r.table)->entity : 0;
}

size_t cuo_row_comp_count(cuo_row r)
{
    return r.table ? ((const row_t *)r.table)->values.len : 0;
}

cuo_bytes cuo_row_comp(cuo_row r, size_t i)
{
    cuo_bytes none = { NULL, 0 };
    if (!r.table || i >= ((const row_t *)r.table)->values.len)
        return none;
    return json_bytes(&((const row_t *)r.table)->values.ptr[i]);
}

cuo_bytes cuo_events_at(cuo_events ev, size_t i)
{
    cuo_bytes none = { NULL, 0 };
    if (i >= ev.len)
        return none;
    return json_bytes(&((const tinyecs_modding_ecs_json_t *)ev.vec)[i]);
}

uint32_t cuo_obs_id(const cuo_obs *ev) { return ev->obs_id; }
uint64_t cuo_obs_entity(const cuo_obs *ev) { return ev->entity; }
cuo_bytes cuo_obs_value(const cuo_obs *ev) { return ev->value; }
cuo_query cuo_obs_query(const cuo_obs *ev, cuo_param p) { return query_of(ev->params, p); }
cuo_bytes cuo_obs_res(const cuo_obs *ev, cuo_param p) { return res_of(ev->params, p); }
cuo_events cuo_obs_events(const cuo_obs *ev, cuo_param p) { return events_of(ev->params, p); }
cuo_bytes cuo_obs_packet(const cuo_obs *ev) { return ev->packet; }
cuo_dir cuo_obs_packet_dir(const cuo_obs *ev) { return ev->dir; }
