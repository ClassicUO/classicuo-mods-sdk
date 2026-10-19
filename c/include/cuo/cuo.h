/* cuo.h — C SDK for ClassicUO mods (wasm32-wasip2 components, world cuo:modding/mod).
 *
 * A mod is a component of its OWN world (the mod's wit/world.wit): it includes
 * cuo:c-sdk/mod@0.1.0 (= cuo:modding/mod + the SDK's hotkey export) and exports one
 * function per system / observer, named like it:
 *
 *     world my-mod {
 *         use tinyecs:modding/ecs@0.1.0.{commands, query, res, entity};
 *         export low-hp: func(commands: commands, hits: query, time: res);
 *         export on-click: func(entity: entity, commands: commands);
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
 *   run the system reads them and records commands, which the host applies AFTER the
 *   callback returns. `commands` is a parameter of the export like the others: anywhere
 *   in its signature (it is not declared in cuo_setup), or left out - then the cuo_cmds
 *   the callback gets traps on use.
 * - Values (components, resources, events) only cross as typed WIT records, through the
 *   generated per-type functions of cuo/typed.h (cuo_X_comp / _column / _set / _res /
 *   _put / _set_resource / _send / _events / _obs_value), over the structs of cuo/types.h.
 * - Everything else (host / assets / actions / packets) is a typed import: the typed
 *   wrappers below, or the generated functions directly (cuo_modding_actions_cast_spell,
 *   cuo_modding_assets_static_tile, ... — declared in cuo_wit.h, included here).
 *
 * Memory model (read this once):
 * - Everything the SDK hands a callback (columns, events, observer values, payload bytes,
 *   strings from cuo_fmt / cuo_storage_get / cuo_cliloc) lives until the callback
 *   returns. Copy (strdup/malloc) anything you keep across calls.
 * - Strings you pass IN are only read during the call.
 * - Calling a generated import directly: results it returns are malloc'd by the
 *   canonical ABI; free them with the matching cuo_wit_* / *_free function.
 */
#ifndef CUO_H
#define CUO_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

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

static inline bool cuo_entity_eq(cuo_entity a, cuo_entity b)
{
    return a == b;
}

/* A component to spawn / insert: built by the generated cuo_X_comp(&v) (a tag:
 * cuo_X_comp()), which converts the value at once (strings copied into scratch). */
typedef struct cuo_comp {
    uint16_t type_id;
    uint8_t kind;    /* internal: which entity-builder method inserts it */
    const void *rec; /* internal: the WIT record (NULL for a tag) */
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

/* cuo:ecs/child-of {Parent}: put an entity under `parent` (spawn or insert it). */
cuo_comp cuo_child_of(cuo_entity parent);

/* ── registration (only inside cuo_setup) ────────────────────────────────────── */

typedef struct cuo_builder cuo_builder;
typedef struct cuo_input cuo_input;
typedef struct cuo_obs cuo_obs;
typedef struct cuo_cmds cuo_cmds;
typedef uint32_t cuo_sys;
typedef uint32_t cuo_observer;
/* A declared parameter: what cuo_input_* / cuo_obs_* and the typed readers read its data by. */
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
    CUO_TERM_REF = 0,     /* reads the component (a column slot) */
    CUO_TERM_MUT = 1,     /* reads it, and cuo_X_set may write it back */
    CUO_TERM_WITH = 2,    /* filter only */
    CUO_TERM_WITHOUT = 3, /* filter only */
    CUO_TERM_CHANGED = 4, /* reads it AND only changed-since-last-run entities match */
    CUO_TERM_ADDED = 5,   /* reads it AND only entities that got it since last run */
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
 * - query: the read terms (REF/MUT/CHANGED/ADDED) are the column slots cuo_X_column /
 *   cuo_X_set take, numbered in declaration order. A WITH on a type already read is
 *   dropped and a CHANGED/ADDED/MUT on a type already read upgrades that term in place
 *   (one slot per type).
 * - res: a resource (`mut`: cuo_X_put may write it). Its value (cuo_X_res) is fetched on
 *   first use and reused while the host reports it unchanged (res.unchanged).
 * - events: the events of a type sent since this system's last run (cuo_X_events). */
cuo_param cuo_system_query(cuo_builder *m, cuo_sys s, const cuo_term *terms, size_t n);
cuo_param cuo_system_res(cuo_builder *m, cuo_sys s, uint16_t type_id, bool mut);
cuo_param cuo_system_events(cuo_builder *m, cuo_sys s, uint16_t type_id);

/* Observers run the moment their trigger fires (no polling, no frame lag). name: as for
 * cuo_add_system; the export takes `entity: entity, value: <record>` first (the record of
 * the trigger's path, e.g. `value: chat-message` for cuo:chat/message), or `entity: entity`
 * alone for a path without a value (a tag), then its params. Read the value with
 * cuo_X_obs_value(ev). */
cuo_observer cuo_on_event(cuo_builder *m, const char *name, const char *event_path, cuo_observer_fn fn,
                          void *user);
cuo_observer cuo_on_add(cuo_builder *m, const char *name, uint16_t type_id, cuo_observer_fn fn, void *user);
cuo_observer cuo_on_remove(cuo_builder *m, const char *name, uint16_t type_id, cuo_observer_fn fn, void *user);
/* An observer's own parameters, fetched with each trigger (read with cuo_obs_* and the
 * cuo_X_obs_* readers). */
cuo_param cuo_observer_query(cuo_builder *m, cuo_observer o, const cuo_term *terms, size_t n);
cuo_param cuo_observer_res(cuo_builder *m, cuo_observer o, uint16_t type_id, bool mut);
cuo_param cuo_observer_events(cuo_builder *m, cuo_observer o, uint16_t type_id);

/* Hotkeys the host fires back as the cuo:input/hotkey event (observe it with
 * cuo_on_event(m, "on-hotkey", CUO_PATH_INPUT_HOTKEY, …) and read cuo_ModHotkeyFired_obs_value).
 * All bindings are published once, as the cuo:input/mod-hotkeys resource, from a
 * Startup system the SDK adds (`cuo-sdk-hotkeys`, exported by cuo:c-sdk/mod); to rebind,
 * cuo_ModHotkeyBindings_set_resource a new set.
 * `key` is a CUO_KEY_*, `mouse_button` a CUO_MOUSE_* (middle / X buttons / wheel notches).
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

uint32_t cuo_input_sys_id(const cuo_input *in);
/* How many times this system has run, this run included (1 on the first). */
uint64_t cuo_input_tick(const cuo_input *in);
cuo_query cuo_input_query(const cuo_input *in, cuo_param p); /* len 0 when absent */
/* How many events an events param received (cuo_X_events reads them). */
size_t cuo_input_events_len(const cuo_input *in, cuo_param p);
/* The matched entities (q.len of them), in row order — the order of every column
 * (cuo_X_column). */
const cuo_entity *cuo_query_entities(cuo_query q);
/* Linear scan for `entity`: its row / column index. index may be NULL. */
bool cuo_query_index(cuo_query q, cuo_entity entity, size_t *index);

uint32_t cuo_obs_id(const cuo_obs *ev);
uint64_t cuo_obs_entity(const cuo_obs *ev); /* 0 = global event */
cuo_query cuo_obs_query(const cuo_obs *ev, cuo_param p);
size_t cuo_obs_events_len(const cuo_obs *ev, cuo_param p);
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
/* The mod's blob in `scope` ("" when nothing stored); write-through on set. The format is
 * the mod's own (the host stores it as-is). */
const char *cuo_storage_get(cuo_scope scope);
void cuo_storage_set(cuo_scope scope, const char *blob);

/* assets */
bool cuo_gump_size(uint32_t gump_id, int *w, int *h);
uint32_t cuo_hue_argb(uint16_t hue); /* the tint a white glyph gets: 0xAARRGGBB; 0 = white */
const char *cuo_cliloc(uint32_t id); /* "" when unknown */

/* ── call-scoped scratch ─────────────────────────────────────────────────────── */

void *cuo_alloc(size_t n); /* 8-aligned, zeroed */
char *cuo_strdup(const char *s);
char *cuo_strndup(const char *s, size_t n);
const char *cuo_fmt(const char *fmt, ...) __attribute__((format(printf, 1, 2)));

#ifdef __cplusplus
}
#endif

/* Generated: CUO_PATH_* constants, payload structs, key codes. */
#include "cuo/paths.h"
#include "cuo/types.h"

#ifdef __cplusplus
extern "C" {
#endif

/* ── components, resources, events (cuo:modding/components) ────────────────────
 * Every registered type path crosses the boundary as a typed WIT record; the generated
 * cuo/typed.h has its functions, over the cuo_X structs of cuo/types.h (the SDK converts
 * struct <-> record). Read-only paths (player stats, serials, host-computed UI state, ...)
 * have no insert / set / put / set_resource function.
 *
 * - Insert: cuo_X_comp(&v) (a tag: cuo_X_comp()) in cuo_spawn / cuo_spawn_child /
 *   cuo_insert / cuo_insert1. A spawn / insert is one entity builder (components.spawn /
 *   entity-of) plus one chained call per component.
 * - Read: cuo_X_column(q, term) = read-term slot `term` of every matched entity, aligned
 *   with cuo_query_entities(q) (q.len items, call-scoped; one host call — keep the
 *   pointer, do not call it per row). Traps unless that slot is a term on X.
 * - Write: cuo_X_set(q, entity, term, &v) through a CUO_MUT term.
 * - Resources: cuo_X_res(in, p, &out) / cuo_X_obs_res(ev, p, &out) read a res param
 *   (false while the host has none); cuo_X_put(in, p, &v) / cuo_X_obs_put writes it at
 *   once through a `mut` res param; cuo_X_set_resource(cmds, &v) sets it as a command
 *   (no param needed).
 * - Events: cuo_X_send(cmds, &v) sends one (a zero-size event: cuo_X_send(cmds));
 *   cuo_X_events(in, p, &n) / cuo_X_obs_events(ev, p, &n) read an events param
 *   (call-scoped array; a zero-size event: the count).
 * - Observers: the trigger's value is cuo_X_obs_value(ev) (NULL when X is not the
 *   trigger's type). The export declares it `func(entity: entity, value: <record>, ...)`
 *   (`use tinyecs:modding/ecs@0.1.0.{entity}` and `use cuo:modding/types@0.1.0.{<record>}`),
 *   or `func(entity: entity, ...)` for a tag:
 *
 *       export on-chat: func(entity: entity, value: chat-message, commands: commands);
 *
 *       static void on_chat(const cuo_obs *ev, cuo_cmds *c, void *u)
 *       {
 *           const cuo_ModChatMessage *msg = cuo_ModChatMessage_obs_value(ev);
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
