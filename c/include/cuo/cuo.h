/* cuo.h — C SDK for ClassicUO mods (wasm32-wasip2 components, world cuo:modding/mod).
 *
 * A mod is a component of its OWN world (the mod's wit/world.wit): it includes
 * cuo:c-sdk/mod@0.1.0 (= cuo:modding/mod + the SDK's hotkey export) and exports one
 * function per system / observer, named like it:
 *
 *     world my-mod {
 *         use tinyecs:modding/ecs@0.1.0.{commands, query, res, trigger-data};
 *         export low-hp: func(commands: commands, hits: query, time: res);
 *         export on-click: func(trigger: trigger-data, commands: commands);
 *         include cuo:c-sdk/mod@0.1.0;
 *     }
 *
 * The mod defines ONE function, `void cuo_setup(cuo_builder *m)`, and registers each
 * system / observer there under its export's name, with its parameters in the
 * export's order. The build (mod.mk) generates the C bindings of that world
 * (wit-bindgen c) and a trampoline per export that hands the arguments to the SDK,
 * which calls the callback registered under that name. A mismatch between cuo_setup
 * and the world traps at load, naming the export line the world needs.
 *
 * - The ECS: a system DECLARES its parameters (queries, resources, event readers); each
 *   run the SDK fetches their data (one host call per param), the system reads it and
 *   records commands, which the host applies AFTER the callback returns. `commands`
 *   is a parameter of the export like the others: anywhere in its signature (it is not
 *   declared in cuo_setup), or left out - then the cuo_cmds the callback gets traps on
 *   use.
 * - Everything else (host / assets / actions / packets) is a typed import: the typed
 *   wrappers below, or the generated functions directly (cuo_modding_actions_cast_spell,
 *   cuo_modding_assets_static_tile, ... — declared in cuo_wit.h, included here).
 *
 * Memory model (read this once):
 * - Everything the SDK hands a callback (query rows, payload bytes, parsed structs,
 *   strings from cuo_fmt / cuo_storage_get / cuo_cliloc, JSON built with cuo_jw) lives
 *   until the callback returns. Copy (strdup/malloc) anything you keep across calls.
 * - Strings you pass IN are only read during the call.
 * - Calling a generated import directly: results it returns are malloc'd by the
 *   canonical ABI; free them with the matching cuo_wit_* / *_free function.
 */
#ifndef CUO_H
#define CUO_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "cJSON.h"
#include "cuo_wit.h"

#ifdef __cplusplus
extern "C" {
#endif

/* ── basics ──────────────────────────────────────────────────────────────────── */

typedef struct cuo_bytes {
    const uint8_t *ptr;
    size_t len;
} cuo_bytes;

/* An entity id. cuo_spawn returns the real id right away. */
typedef uint64_t cuo_entity;

/* Kept for source compatibility: ids are always real now. */
static inline cuo_entity cuo_resolve(cuo_entity e)
{
    return e;
}

static inline bool cuo_entity_eq(cuo_entity a, cuo_entity b)
{
    return a == b;
}

/* A component / resource payload: type id (cuo_type_id / cuo_X_id()) + utf8 JSON
 * (len 0 = marker), or — `typed` != 0, built by cuo_X_typed — a typed component that
 * goes through cuo:modding/components instead (no JSON). */
typedef struct cuo_comp {
    uint16_t type_id;
    cuo_bytes data;
    uint8_t typed; /* internal: 0 = JSON */
} cuo_comp;

#define CUO_COUNT(T, ...) (sizeof((T[]){__VA_ARGS__}) / sizeof(T))
/* Expand to `array, count` for the (const X *items, size_t n) parameters. The count is
 * a sizeof, so the arguments are evaluated exactly once. */
#define CUO_COMPS(...) (const cuo_comp[]){__VA_ARGS__}, CUO_COUNT(cuo_comp, __VA_ARGS__)
#define CUO_TERMS(...) (const cuo_term[]){__VA_ARGS__}, CUO_COUNT(cuo_term, __VA_ARGS__)
#define CUO_IDS(...) (const uint16_t[]){__VA_ARGS__}, CUO_COUNT(uint16_t, __VA_ARGS__)

static inline cuo_bytes cuo_str_bytes(const char *s)
{
    size_t n = 0;
    if (s)
        while (s[n])
            n++;
    cuo_bytes b = { (const uint8_t *)s, n };
    return b;
}

cuo_comp cuo_comp_json(uint16_t type_id, const char *json);
cuo_comp cuo_comp_bytes(uint16_t type_id, cuo_bytes json);
cuo_comp cuo_comp_marker(uint16_t type_id);
/* cuo:ecs/child-of {Parent}: put an entity under `parent` (spawn or insert it). Typed. */
cuo_comp cuo_child_of(cuo_entity parent);

/* ── registration (only inside cuo_setup) ────────────────────────────────────── */

typedef struct cuo_builder cuo_builder;
typedef struct cuo_input cuo_input;
typedef struct cuo_obs cuo_obs;
typedef struct cuo_cmds cuo_cmds;
typedef uint32_t cuo_sys;
typedef uint32_t cuo_observer;
/* A declared parameter: what cuo_input_* / cuo_obs_* read its data by. */
typedef uint32_t cuo_param;

/* Values match the WIT `schedule` enum. */
typedef enum cuo_stage {
    CUO_STAGE_STARTUP = 0, /* once, after the mod loads (storage readable) */
    CUO_STAGE_FIRST = 1,
    CUO_STAGE_PRE_UPDATE = 2,
    CUO_STAGE_UPDATE = 3,
    CUO_STAGE_POST_UPDATE = 4,
    CUO_STAGE_LAST = 5,
} cuo_stage;

/* Values match the WIT `term` variant. */
typedef enum cuo_term_kind {
    CUO_TERM_REF = 0,     /* row carries the component */
    CUO_TERM_MUT = 1,     /* row carries it, and cuo_query_set may write it back */
    CUO_TERM_WITH = 2,    /* filter only */
    CUO_TERM_WITHOUT = 3, /* filter only */
    CUO_TERM_CHANGED = 4, /* row carries it AND only changed-since-last-run entities match */
    CUO_TERM_ADDED = 5,   /* row carries it AND only entities that got it since last run */
} cuo_term_kind;

typedef struct cuo_term {
    uint8_t kind;
    uint16_t type_id;
} cuo_term;

#define CUO_REF(id) ((cuo_term){ CUO_TERM_REF, (uint16_t)(id) })
#define CUO_MUT(id) ((cuo_term){ CUO_TERM_MUT, (uint16_t)(id) })
#define CUO_WITH(id) ((cuo_term){ CUO_TERM_WITH, (uint16_t)(id) })
#define CUO_WITHOUT(id) ((cuo_term){ CUO_TERM_WITHOUT, (uint16_t)(id) })
#define CUO_CHANGED(id) ((cuo_term){ CUO_TERM_CHANGED, (uint16_t)(id) })
#define CUO_ADDED(id) ((cuo_term){ CUO_TERM_ADDED, (uint16_t)(id) })

typedef void (*cuo_system_fn)(const cuo_input *in, cuo_cmds *cmds, void *user);
typedef void (*cuo_observer_fn)(const cuo_obs *ev, cuo_cmds *cmds, void *user);

/* Defined by the mod. Called once from the `setup` export. */
void cuo_setup(cuo_builder *m);

/* A guest-local id for a type path (CUO_PATH_* / cuo_X_PATH), interned on first use.
 * The host checks the paths when the mod declares or uses them: an unknown one fails
 * the load (declarations) or traps (commands). cuo_try_type_id always succeeds. */
uint16_t cuo_type_id(const char *path);
bool cuo_try_type_id(const char *path, uint16_t *out);
const char *cuo_type_path(uint16_t id); /* NULL when never interned */

/* name: the export of the mod's world that runs it ("low-hp"; the C spelling "low_hp"
 * works too), unique within the mod. Runs every frame in `stage` (once for STARTUP). */
cuo_sys cuo_add_system(cuo_builder *m, const char *name, cuo_stage stage, cuo_system_fn fn, void *user);
void cuo_system_after(cuo_builder *m, cuo_sys s, cuo_sys other);
void cuo_system_before(cuo_builder *m, cuo_sys s, cuo_sys other);
/* Skip a call when nothing it reads changed: every res param unchanged since its
 * previous call, no new event for its events params, and every query matched no rows
 * (only CHANGED / ADDED queries can be empty while idle; a plain query that matches
 * keeps it running). A system with no params always runs. Not for timers or state
 * machines driven by the mod's own state / the clock. */
void cuo_system_run_on_change(cuo_builder *m, cuo_sys s);

/* System parameters. Each returns the handle its data is read by.
 * - query: read terms (REF/MUT/CHANGED/ADDED) fill row comp slots in declaration order.
 *   A WITH on a type already read is dropped and a CHANGED/ADDED/MUT on a type already
 *   read upgrades that term in place (one payload per type).
 * - res: a resource (`mut`: you may write it back with cuo_resource_set). Reads
 *   {NULL,0} while the host has none. The bytes stay the same pointer while the host
 *   reports the value unchanged (res.unchanged: no get, no copy), so a parse keyed on
 *   that pointer can be reused.
 * - events: the events of a type sent since this system's last run. */
cuo_param cuo_system_query(cuo_builder *m, cuo_sys s, const cuo_term *terms, size_t n);
cuo_param cuo_system_res(cuo_builder *m, cuo_sys s, uint16_t type_id, bool mut);
cuo_param cuo_system_events(cuo_builder *m, cuo_sys s, uint16_t type_id);

/* Observers run the moment their trigger fires (no polling, no frame lag). name: as for
 * cuo_add_system; the export takes `trigger: trigger-data` first, then its params. */
cuo_observer cuo_on_event(cuo_builder *m, const char *name, const char *event_path, cuo_observer_fn fn,
                          void *user);
cuo_observer cuo_on_add(cuo_builder *m, const char *name, uint16_t type_id, cuo_observer_fn fn, void *user);
cuo_observer cuo_on_remove(cuo_builder *m, const char *name, uint16_t type_id, cuo_observer_fn fn, void *user);
/* An observer's own parameters, fetched with each trigger (read with cuo_obs_*). */
cuo_param cuo_observer_query(cuo_builder *m, cuo_observer o, const cuo_term *terms, size_t n);
cuo_param cuo_observer_res(cuo_builder *m, cuo_observer o, uint16_t type_id, bool mut);
cuo_param cuo_observer_events(cuo_builder *m, cuo_observer o, uint16_t type_id);

/* Hotkeys the host fires back as the cuo:input/hotkey event (observe it with
 * cuo_on_event(m, "on-hotkey", CUO_PATH_INPUT_HOTKEY, …) and parse cuo_ModHotkeyFired).
 * All bindings are published once, as the cuo:input/mod-hotkeys resource, from a
 * Startup system the SDK adds (`cuo-sdk-hotkeys`, exported by cuo:c-sdk/mod); to rebind, cuo_resource_set a new cuo_ModHotkeyBindingsDto.
 * CUO_HK_CONSUME: this mod owns the combo (the host's own binding does not fire). */
enum { CUO_HK_CONSUME = 1, CUO_HK_CTRL = 2, CUO_HK_SHIFT = 4, CUO_HK_ALT = 8 };
void cuo_hotkey(cuo_builder *m, const char *name, uint32_t key, unsigned flags);
void cuo_hotkey_mouse(cuo_builder *m, const char *name, int32_t mouse_button, unsigned flags);

/* ── packets ─────────────────────────────────────────────────────────────────── */

/* Values match the WIT packet-direction / verdict. */
typedef enum cuo_dir { CUO_INCOMING = 0, CUO_OUTGOING = 1 } cuo_dir;
typedef enum cuo_verdict { CUO_PASS = 0, CUO_BLOCK = 1, CUO_REPLACE = 2 } cuo_verdict;

#define CUO_PACKET_IDS(...) (const uint8_t[]){__VA_ARGS__}, CUO_COUNT(uint8_t, __VA_ARGS__)

/* A packet observer (the on-packet trigger): sees each `dir` packet whose id (byte 0)
 * is in `ids` (n 0 = every id) before the client handles (incoming) / sends (outgoing)
 * it; read it with cuo_obs_packet. Runs synchronously, so keep it cheap. Return
 * CUO_REPLACE after pointing *replacement at the new bytes (scratch is fine). Mods run
 * in load order, then each mod's packet observers in registration order; each sees the
 * previous replacement and CUO_BLOCK stops the chain. Packets this mod injects skip
 * its own observers. Like any observer it records commands and takes params
 * (cuo_observer_query / _res / _events). Its export:
 * `func(direction: packet-direction, packet: list<u8>, <params>) -> verdict`. */
typedef cuo_verdict (*cuo_packet_fn)(const cuo_obs *ev, cuo_cmds *cmds, cuo_bytes *replacement, void *user);
cuo_observer cuo_on_packet(cuo_builder *m, const char *name, cuo_dir dir, const uint8_t *ids, size_t n,
                           cuo_packet_fn fn, void *user);

/* Taps over cuo_on_packet: every id in one direction, `id` = data[0]; return true to
 * block. */
typedef bool (*cuo_packet_tap_fn)(uint8_t id, const uint8_t *data, size_t len, void *user);
cuo_observer cuo_on_packet_in(cuo_builder *m, const char *name, cuo_packet_tap_fn fn, void *user);
cuo_observer cuo_on_packet_out(cuo_builder *m, const char *name, cuo_packet_tap_fn fn, void *user);

/* packets.send-to-server / send-to-client: inject a packet, as if the client sent it /
 * the server sent it. */
void cuo_send_to_server(const uint8_t *data, size_t len);
void cuo_send_to_client(const uint8_t *data, size_t len);

/* ── system / observer input ─────────────────────────────────────────────────── */

typedef struct cuo_query {
    const void *impl; /* internal */
    size_t len;
    int32_t handle; /* internal */
} cuo_query;

typedef struct cuo_row {
    const void *table; /* internal */
} cuo_row;

typedef struct cuo_events {
    const void *vec; /* internal */
    size_t len;
} cuo_events;

uint32_t cuo_input_sys_id(const cuo_input *in);
/* How many times this system has run, this run included (1 on the first). */
uint64_t cuo_input_tick(const cuo_input *in);
cuo_query cuo_input_query(const cuo_input *in, cuo_param p); /* len 0 when absent */
cuo_bytes cuo_input_res(const cuo_input *in, cuo_param p);   /* {NULL,0}: host has none */
cuo_events cuo_input_events(const cuo_input *in, cuo_param p);
cuo_row cuo_query_row(cuo_query q, size_t i);
/* Linear scan for `entity` (Contains/TryGet). out may be NULL. */
bool cuo_query_find(cuo_query q, cuo_entity entity, cuo_row *out);
/* The matched entities (q.len of them), in row order — the order of every typed
 * column (cuo_X_column). */
const cuo_entity *cuo_query_entities(cuo_query q);
/* Linear scan for `entity`: its row / column index. index may be NULL. */
bool cuo_query_index(cuo_query q, cuo_entity entity, size_t *index);
/* Writes read-term slot `index` of `entity` back (query.set). Traps unless that term
 * is CUO_MUT. Lands on the host entity when the callback returns. */
void cuo_query_set(cuo_query q, cuo_entity entity, size_t index, cuo_bytes json);
uint64_t cuo_row_entity(cuo_row r);
size_t cuo_row_comp_count(cuo_row r);
/* JSON payload of read-term slot i ({NULL,0} when absent / a marker). Parse with
 * cuo_X_parse. */
cuo_bytes cuo_row_comp(cuo_row r, size_t i);
cuo_bytes cuo_events_at(cuo_events ev, size_t i);

uint32_t cuo_obs_id(const cuo_obs *ev);
uint64_t cuo_obs_entity(const cuo_obs *ev); /* 0 = global event */
/* Add/Remove: the component JSON; event: the event JSON; {NULL,0} for a marker. */
cuo_bytes cuo_obs_value(const cuo_obs *ev);
cuo_query cuo_obs_query(const cuo_obs *ev, cuo_param p);
cuo_bytes cuo_obs_res(const cuo_obs *ev, cuo_param p);
cuo_events cuo_obs_events(const cuo_obs *ev, cuo_param p);
/* Packet observers: the packet (full wire bytes, id first) and its direction. */
cuo_bytes cuo_obs_packet(const cuo_obs *ev);
cuo_dir cuo_obs_packet_dir(const cuo_obs *ev);

/* ── commands (applied by the host after the callback returns) ───────────────── */

cuo_entity cuo_spawn(cuo_cmds *c, const cuo_comp *comps, size_t n);
/* Spawn under `parent` (adds cuo:ecs/child-of). */
cuo_entity cuo_spawn_child(cuo_cmds *c, cuo_entity parent, const cuo_comp *comps, size_t n);
void cuo_insert(cuo_cmds *c, cuo_entity e, const cuo_comp *comps, size_t n);
void cuo_insert1(cuo_cmds *c, cuo_entity e, cuo_comp comp);
void cuo_remove(cuo_cmds *c, cuo_entity e, const uint16_t *type_ids, size_t n);
void cuo_despawn(cuo_cmds *c, cuo_entity e); /* the host despawns the children too */
/* Writes a resource: through the running system's / observer's res-mut param of that
 * type when it has one, else as cuo_set_resource. */
void cuo_resource_set(cuo_cmds *c, cuo_comp value);
/* commands.set-resource: overwrite any writable resource by its type path, applied with
 * the other commands; no res-mut param needed. The host traps on an unknown or
 * read-only path. */
void cuo_set_resource(cuo_cmds *c, const char *path, cuo_bytes json);
/* Send an event (utf8 JSON payload) by its type path. `entity` is ignored (events are
 * global). Typed: cuo_X_emit. */
void cuo_emit(cuo_cmds *c, const char *event_path, uint64_t entity, cuo_bytes json);
size_t cuo_cmds_len(const cuo_cmds *c);

/* Chat output through cuo:chat/message (font 3, unicode — how server speech arrives). */
void cuo_chat_system(cuo_cmds *c, const char *text, uint16_t hue); /* journal sysmessage; hue 0x5B typical */
/* Text over a live entity (serial must be known to the client; 0 shows nothing). */
void cuo_chat_overhead(cuo_cmds *c, const char *text, uint16_t hue, uint32_t serial, const char *name);

/* ── host functions (typed wrappers over the cuo:modding imports) ────────────── */
/* Anything not wrapped here: call the generated import (cuo_modding_<iface>_<fn>, e.g.
 * cuo_modding_actions_cast_spell(29), cuo_modding_actions_double_click(serial)). */

/* host */
void cuo_log(const char *msg);
void cuo_logf(const char *fmt, ...) __attribute__((format(printf, 1, 2)));
uint32_t cuo_measure_text(uint16_t font, const char *text);
bool cuo_resolve_serial(uint32_t serial, cuo_entity *out); /* false: the client doesn't know it */
typedef enum cuo_scope { CUO_SCOPE_GLOBAL = 0, CUO_SCOPE_CHARACTER = 1 } cuo_scope;
/* The mod's blob in `scope` ("" when nothing stored); write-through on set. */
const char *cuo_storage_get(cuo_scope scope);
void cuo_storage_set(cuo_scope scope, const char *json);

/* assets */
bool cuo_gump_size(uint32_t gump_id, int *w, int *h);
uint32_t cuo_hue_argb(uint16_t hue); /* the tint a white glyph gets: 0xAARRGGBB; 0 = white */
const char *cuo_cliloc(uint32_t id); /* "" when unknown */

/* GONE — kept only so the generated cuo_X_get / cuo_X_has / cuo_X_resource helpers
 * link; calling one traps. Read components through query params and resources through
 * cuo_system_res instead. */
cuo_bytes cuo_component_json(uint64_t entity, uint16_t type_id);
cuo_bytes cuo_resource_json(uint16_t type_id);

/* ── call-scoped scratch ─────────────────────────────────────────────────────── */

void *cuo_alloc(size_t n); /* 8-aligned, zeroed */
char *cuo_strdup(const char *s);
char *cuo_strndup(const char *s, size_t n);
const char *cuo_fmt(const char *fmt, ...) __attribute__((format(printf, 1, 2)));

/* ── JSON ────────────────────────────────────────────────────────────────────── */

/* Streaming writer into scratch. Commas are handled for you. */
typedef struct cuo_jw {
    char *buf;
    size_t len, cap;
    uint64_t has_item; /* bit d: container at depth d already holds an item */
    int depth;
    bool after_key;
} cuo_jw;

void cuo_jw_init(cuo_jw *w);
void cuo_jw_obj(cuo_jw *w);
void cuo_jw_obj_end(cuo_jw *w);
void cuo_jw_arr(cuo_jw *w);
void cuo_jw_arr_end(cuo_jw *w);
void cuo_jw_key(cuo_jw *w, const char *key);
void cuo_jw_str(cuo_jw *w, const char *s); /* NULL writes "" */
void cuo_jw_strn(cuo_jw *w, const char *s, size_t n);
void cuo_jw_int(cuo_jw *w, int64_t v);
void cuo_jw_uint(cuo_jw *w, uint64_t v);
void cuo_jw_num(cuo_jw *w, double v);
void cuo_jw_bool(cuo_jw *w, bool v);
void cuo_jw_null(cuo_jw *w);
void cuo_jw_raw(cuo_jw *w, const char *json);
cuo_bytes cuo_jw_bytes(cuo_jw *w); /* NUL-terminated as well */

/* Parse into a CALL-SCOPED cJSON tree (scratch-allocated): never cJSON_Delete it.
 * NULL on empty / malformed input. cJSON_Parse itself still allocates with malloc. */
cJSON *cuo_json_parse(cuo_bytes json);
int64_t cuo_json_int(const cJSON *obj, const char *key, int64_t dflt);
double cuo_json_num(const cJSON *obj, const char *key, double dflt);
bool cuo_json_bool(const cJSON *obj, const char *key, bool dflt);
const char *cuo_json_str(const cJSON *obj, const char *key, const char *dflt);

#ifdef __cplusplus
}
#endif

/* Generated: CUO_PATH_* constants, payload structs + (de)serializers, typed actions,
 * key codes. */
#include "cuo/paths.h"
#include "cuo/types.h"

#ifdef __cplusplus
extern "C" {
#endif

/* ── typed components (cuo:modding/components: no JSON) ────────────────────────
 * Every curated type path (the generated cuo/typed.h lists them: most components,
 * resources and events the shipped mods use) also crosses the boundary as a typed WIT
 * record. Same structs as the JSON path (cuo_Node, cuo_Hits, ...); the SDK converts
 * struct to record. Every other type keeps the JSON path.
 *
 * - Insert: cuo_X_typed(&v) is the typed twin of cuo_X_comp(&v); mix both freely in
 *   cuo_spawn / cuo_spawn_child / cuo_insert / cuo_insert1 (a tag: cuo_X_typed()). A
 *   spawn / insert with typed components is one entity builder (components.spawn /
 *   entity-of) plus one chained call per typed component, after one commands.insert of
 *   its JSON ones. The value is converted (strings copied) at once.
 * - Read: cuo_X_column(q, term) = read-term slot `term` of every matched entity, aligned
 *   with cuo_query_entities(q) (q.len items, call-scoped; one host call — keep the
 *   pointer, do not call it per row). Traps unless that slot is a term on X. A query
 *   whose read terms are all typed (tags included) skips query.rows (fetched only if
 *   you read a row).
 * - Write: cuo_X_set(q, entity, term, &v) — the typed cuo_query_set (CUO_MUT term).
 * - Resources: cuo_X_res(in, p, &out) / cuo_X_obs_res(ev, p, &out) read a res param;
 *   false while the host has none.
 * - Events: cuo_X_send(cmds, &v) sends one (the typed cuo_X_emit; a zero-size event:
 *   cuo_X_send(cmds)); cuo_X_events(in, p, &n) / cuo_X_obs_events(ev, p, &n) read an
 *   events param (call-scoped array; a zero-size event: the count).
 * - Observers: an observer of a typed component / event may take its trigger typed —
 *   declare its export `func(entity: entity, value: <record>, ...)` (a tag:
 *   `func(entity: entity, ...)`) in wit/world.wit instead of `trigger: trigger-data`
 *   (`use tinyecs:modding/ecs@0.1.0.{entity}` and `use cuo:modding/types@0.1.0.{<record>}`)
 *   and read the value with cuo_X_obs_typed(ev) (cuo_obs_value is empty then):
 *
 *       export on-chat: func(entity: entity, value: chat-message, commands: commands);
 *
 *       static void on_chat(const cuo_obs *ev, cuo_cmds *c, void *u)
 *       {
 *           const cuo_ModChatMessage *msg = cuo_ModChatMessage_obs_typed(ev);
 *           ...
 *       }
 */
#include "cuo/typed.h"

/* ── UI payload constructors (twin of ui.rs / Types.Ui.cs) ───────────────────── */

cuo_Val cuo_val_auto(void);
cuo_Val cuo_val_px(float v);
cuo_Val cuo_val_percent(float v);
cuo_Val cuo_val_grow(void); /* fill what the parent has left; MinWidth/MinHeight is the floor */
cuo_UiRect cuo_rect_splat(cuo_Val v);
cuo_UiRect cuo_rect_zero(void);
cuo_BorderRadius cuo_radius_all(float r);
/* Flex / Relative / Row / Start, every Val Auto, padding + border + gap Px 0. */
cuo_Node cuo_node_base(void);
/* Absolutely positioned fixed-size node — the shape of every UO gump element. */
cuo_Node cuo_node_abs(float left, float top, float width, float height);
/* Channels 0..255 (Clay convention). */
cuo_Color cuo_rgba(uint8_t r, uint8_t g, uint8_t b, uint8_t a);
cuo_Color cuo_rgb(uint8_t r, uint8_t g, uint8_t b);
cuo_Color cuo_hue_color(uint32_t hue); /* a UO hue as the tint the host would apply */

#ifdef __cplusplus
}
#endif

#endif
