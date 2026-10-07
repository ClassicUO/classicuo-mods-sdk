/* cuo.h — C SDK for core-wasm ClassicUO mods (wasm32-wasip1, modding surface v3).
 *
 * A mod is a wasm32-wasip1 reactor module. It defines ONE function,
 *
 *     void cuo_setup(cuo_builder *m);
 *
 * and registers systems / observers / hotkeys / the packet handler from it. The SDK owns
 * every ABI export (mod_setup, mod_run, mod_observer, mod_spawned, mod_on_packet,
 * mod_alloc, mod_arena_reset).
 *
 * Two halves (docs/p1-wire.md):
 * - The ECS rides the FlatBuffers ABI (abi/mod-abi.fbs), PUSH model: a system DECLARES
 *   its parameters (queries, resources, event readers); each run the host pushes their
 *   data, the system reads it and records commands, which the host applies AFTER the
 *   callback returns.
 * - Everything else (host / assets / actions / packets in wit/cuo-mod.wit) is a plain
 *   call: cuo_call("cuo:modding/<iface>#<fn>", "<JSON args array>") through the single
 *   env.mod_call import; the common ones have typed wrappers below.
 *
 * Memory model (read this once):
 * - Everything the SDK hands a callback (query rows, payload bytes, parsed structs,
 *   strings from cuo_fmt / cuo_call / cuo_storage_get, JSON built with cuo_jw) lives in
 *   a CALL-SCOPED scratch arena that is rewound when the host makes its next call.
 *   Copy (strdup/malloc) anything you keep across calls.
 * - Strings you pass IN are only read during the call.
 */
#ifndef CUO_H
#define CUO_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "cJSON.h"

#ifdef __cplusplus
extern "C" {
#endif

#define CUO_ABI_VERSION 3u
#define CUO_NONE_TYPE 0xFFFFu

/* ── basics ──────────────────────────────────────────────────────────────────── */

typedef struct cuo_bytes {
    const uint8_t *ptr;
    size_t len;
} cuo_bytes;

/* An entity id. One you just spawned holds a placeholder (CUO_PENDING | temp id) until
 * the host assigns the real id when the run returns; commands and components naming it
 * (cuo_child_of) in the same run still accept it, and the SDK swaps in the real id from
 * then on. Compare entities with cuo_entity_eq (or compare cuo_resolve()d ids). */
typedef uint64_t cuo_entity;
#define CUO_PENDING (1ull << 63)

/* The real id of a spawned entity once the host assigned it; anything else unchanged. */
cuo_entity cuo_resolve(cuo_entity e);

static inline bool cuo_entity_eq(cuo_entity a, cuo_entity b)
{
    return cuo_resolve(a) == cuo_resolve(b);
}

/* A component / resource payload: interned type id + utf8 JSON (len 0 = marker). */
typedef struct cuo_comp {
    uint16_t type_id;
    cuo_bytes data;
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
/* cuo:ecs/child-of {Parent}: put an entity under `parent` (spawn or insert it). */
cuo_comp cuo_child_of(cuo_entity parent);

/* ── registration (only inside cuo_setup) ────────────────────────────────────── */

typedef struct cuo_builder cuo_builder;
typedef struct cuo_input cuo_input;
typedef struct cuo_obs cuo_obs;
typedef struct cuo_cmds cuo_cmds;
typedef uint32_t cuo_sys;
typedef uint32_t cuo_observer;
/* A declared parameter: what cuo_input_* / cuo_obs_* read its pushed data by. */
typedef uint32_t cuo_param;

/* Values match ModAbi.Schedule. */
typedef enum cuo_stage {
    CUO_STAGE_STARTUP = 0, /* once, after the mod loads (storage readable) */
    CUO_STAGE_FIRST = 1,
    CUO_STAGE_PRE_UPDATE = 2,
    CUO_STAGE_UPDATE = 3,
    CUO_STAGE_POST_UPDATE = 4,
    CUO_STAGE_LAST = 5,
} cuo_stage;

typedef enum cuo_term_kind {
    CUO_TERM_REF = 0,     /* row carries the component */
    CUO_TERM_MUT = 1,     /* same as REF on the wire (writes go through cuo_insert) */
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
#define CUO_WITH(id) ((cuo_term){ CUO_TERM_WITH, (uint16_t)(id) })
#define CUO_WITHOUT(id) ((cuo_term){ CUO_TERM_WITHOUT, (uint16_t)(id) })
#define CUO_CHANGED(id) ((cuo_term){ CUO_TERM_CHANGED, (uint16_t)(id) })
#define CUO_ADDED(id) ((cuo_term){ CUO_TERM_ADDED, (uint16_t)(id) })

typedef void (*cuo_system_fn)(const cuo_input *in, cuo_cmds *cmds, void *user);
typedef void (*cuo_observer_fn)(const cuo_obs *ev, cuo_cmds *cmds, void *user);

/* Defined by the mod. Called once from mod_setup. */
void cuo_setup(cuo_builder *m);

/* Interned id of a registered type path (CUO_PATH_* / cuo_X_PATH). TRAPS when the
 * client has no such path — a silent sentinel would make every later command a no-op. */
uint16_t cuo_type_id(const char *path);
bool cuo_try_type_id(const char *path, uint16_t *out);
const char *cuo_type_path(uint16_t id); /* NULL when unknown (diagnostics) */

/* label: unique within the mod, for host diagnostics (NULL = "sys-<id>"). Runs every
 * frame in `stage` (once for STARTUP). Every system gets a command buffer. */
cuo_sys cuo_add_system(cuo_builder *m, const char *label, cuo_stage stage, cuo_system_fn fn, void *user);
void cuo_system_after(cuo_builder *m, cuo_sys s, cuo_sys other);
void cuo_system_before(cuo_builder *m, cuo_sys s, cuo_sys other);

/* System parameters. Each returns the handle its pushed data is read by.
 * - query: read terms (REF/MUT/CHANGED/ADDED) fill row comp slots in declaration order.
 *   A WITH on a type already read is dropped and a CHANGED/ADDED on a type already read
 *   upgrades that term in place (one payload per type). The host drives the scan from
 *   the FIRST required term: put the narrowest (a CHANGED, a rare marker) first.
 * - res: a resource (`mut`: you may write it back with cuo_resource_set). Reads
 *   {NULL,0} while the host has none.
 * - events: the events of a type sent since this system's last run. */
cuo_param cuo_system_query(cuo_builder *m, cuo_sys s, const cuo_term *terms, size_t n);
cuo_param cuo_system_res(cuo_builder *m, cuo_sys s, uint16_t type_id, bool mut);
cuo_param cuo_system_events(cuo_builder *m, cuo_sys s, uint16_t type_id);

/* Observers run the moment their trigger fires (no polling, no frame lag). */
cuo_observer cuo_on_event(cuo_builder *m, const char *event_path, cuo_observer_fn fn, void *user);
cuo_observer cuo_on_add(cuo_builder *m, uint16_t type_id, cuo_observer_fn fn, void *user);
cuo_observer cuo_on_remove(cuo_builder *m, uint16_t type_id, cuo_observer_fn fn, void *user);
/* An observer's own parameters, pushed with each trigger (read with cuo_obs_*). */
cuo_param cuo_observer_query(cuo_builder *m, cuo_observer o, const cuo_term *terms, size_t n);
cuo_param cuo_observer_res(cuo_builder *m, cuo_observer o, uint16_t type_id, bool mut);
cuo_param cuo_observer_events(cuo_builder *m, cuo_observer o, uint16_t type_id);

/* Hotkeys the host fires back as the cuo:input/hotkey event (observe it with
 * cuo_on_event(m, CUO_PATH_INPUT_HOTKEY, …) and parse cuo_ModHotkeyFired). All bindings
 * are published once, as the cuo:input/mod-hotkeys resource, from a Startup system
 * the SDK adds; re-publish a cuo_ModHotkeyBindingsDto with cuo_resource_set to rebind.
 * CUO_HK_CONSUME: this mod owns the combo (the host's own binding does not fire). */
enum { CUO_HK_CONSUME = 1, CUO_HK_CTRL = 2, CUO_HK_SHIFT = 4, CUO_HK_ALT = 8 };
void cuo_hotkey(cuo_builder *m, const char *name, uint32_t key, unsigned flags);
void cuo_hotkey_mouse(cuo_builder *m, const char *name, int32_t mouse_button, unsigned flags);

/* ── packets ─────────────────────────────────────────────────────────────────── */

typedef enum cuo_dir { CUO_INCOMING = 0, CUO_OUTGOING = 1 } cuo_dir;
typedef enum cuo_verdict { CUO_PASS = 0, CUO_BLOCK = 1, CUO_REPLACE = 2 } cuo_verdict;

/* The packet handler: sees each packet (full wire bytes, id first) whose id was passed to
 * cuo_intercept, before the client handles (incoming) / sends (outgoing) it. Return
 * CUO_REPLACE after pointing *replacement at the new bytes (scratch is fine). Packets
 * this mod injects skip it. Runs outside the ECS: no command buffer. */
typedef cuo_verdict (*cuo_packet_fn)(cuo_dir dir, const uint8_t *data, size_t len, cuo_bytes *replacement,
                                     void *user);
void cuo_on_packet(cuo_builder *m, cuo_packet_fn fn, void *user);

/* packets.intercept: ask for cuo_on_packet calls for these ids (call it in cuo_setup). */
void cuo_intercept(cuo_dir dir, const uint8_t *ids, size_t n);
/* packets.send-to-server / send-to-client: inject a packet, as if the client sent it /
 * the server sent it. */
void cuo_send_to_server(const uint8_t *data, size_t len);
void cuo_send_to_client(const uint8_t *data, size_t len);

/* ── system / observer input (PUSH snapshot) ─────────────────────────────────── */

typedef struct cuo_query {
    const void *vec; /* internal */
    size_t len;
} cuo_query;

typedef struct cuo_row {
    const void *table; /* internal */
} cuo_row;

typedef struct cuo_events {
    const void *vec; /* internal */
    size_t len;
} cuo_events;

uint32_t cuo_input_sys_id(const cuo_input *in);
uint64_t cuo_input_tick(const cuo_input *in);
cuo_query cuo_input_query(const cuo_input *in, cuo_param p); /* len 0 when absent */
cuo_bytes cuo_input_res(const cuo_input *in, cuo_param p);   /* {NULL,0}: host has none */
cuo_events cuo_input_events(const cuo_input *in, cuo_param p);
cuo_row cuo_query_row(cuo_query q, size_t i);
/* Linear scan for `entity` (Contains/TryGet). out may be NULL. */
bool cuo_query_find(cuo_query q, cuo_entity entity, cuo_row *out);
uint64_t cuo_row_entity(cuo_row r);
size_t cuo_row_comp_count(cuo_row r);
/* JSON payload of read-term slot i ({NULL,0} when absent). Parse with cuo_X_parse. */
cuo_bytes cuo_row_comp(cuo_row r, size_t i);
cuo_bytes cuo_events_at(cuo_events ev, size_t i);

uint32_t cuo_obs_id(const cuo_obs *ev);
uint64_t cuo_obs_entity(const cuo_obs *ev); /* 0 = global event */
/* Add/Remove: the component JSON; event: the event JSON; {NULL,0} for a marker. */
cuo_bytes cuo_obs_value(const cuo_obs *ev);
cuo_query cuo_obs_query(const cuo_obs *ev, cuo_param p);
cuo_bytes cuo_obs_res(const cuo_obs *ev, cuo_param p);
cuo_events cuo_obs_events(const cuo_obs *ev, cuo_param p);

/* ── commands (applied by the host after the callback returns) ───────────────── */

cuo_entity cuo_spawn(cuo_cmds *c, const cuo_comp *comps, size_t n);
/* Spawn under `parent` (adds cuo:ecs/child-of). */
cuo_entity cuo_spawn_child(cuo_cmds *c, cuo_entity parent, const cuo_comp *comps, size_t n);
void cuo_insert(cuo_cmds *c, cuo_entity e, const cuo_comp *comps, size_t n);
void cuo_insert1(cuo_cmds *c, cuo_entity e, cuo_comp comp);
void cuo_remove(cuo_cmds *c, cuo_entity e, const uint16_t *type_ids, size_t n);
void cuo_despawn(cuo_cmds *c, cuo_entity e); /* the host despawns the children too */
void cuo_resource_set(cuo_cmds *c, cuo_comp value);
/* Send an event (utf8 JSON payload) by its type path; entity 0 = global. Typed:
 * cuo_X_emit. */
void cuo_emit(cuo_cmds *c, const char *event_path, uint64_t entity, cuo_bytes json);
size_t cuo_cmds_len(const cuo_cmds *c);

/* Chat output through cuo:chat/message (font 3, unicode — how server speech arrives). */
void cuo_chat_system(cuo_cmds *c, const char *text, uint16_t hue); /* journal sysmessage; hue 0x5B typical */
/* Text over a live entity (serial must be known to the client; 0 shows nothing). */
void cuo_chat_overhead(cuo_cmds *c, const char *text, uint16_t hue, uint32_t serial, const char *name);

/* ── host functions (wit/cuo-mod.wit over env.mod_call) ──────────────────────── */

/* Any function: name "cuo:modding/<iface>#<fn>", args a JSON array in WIT order
 * (docs/p1-wire.md: u64 as decimal strings, list<u8> base64, enums kebab-case, ...).
 * Returns the result JSON (NUL-terminated scratch), {NULL,0} for a function without
 * one. The host traps on an unknown name / malformed args. */
cuo_bytes cuo_call(const char *name, const char *args_json);
/* cuo_call("cuo:modding/actions#<fn>", args): fire-and-forget player actions, e.g.
 * cuo_action("cast-spell", "[29]"), cuo_action("double-click", cuo_fmt("[%u]", serial)). */
void cuo_action(const char *fn, const char *args_json);

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

/* GONE in v3 — kept only so the generated cuo_X_get / cuo_X_has / cuo_X_resource
 * helpers link; calling one traps. Read components through query params and resources
 * through cuo_system_res instead. */
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
