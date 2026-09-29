/* cuo.h — C SDK for core-wasm ClassicUO mods (FlatBuffers ABI, abi/mod-abi.fbs).
 *
 * A mod is a wasm32-wasip1 reactor module. It defines ONE function,
 *
 *     void cuo_setup(cuo_builder *m);
 *
 * and registers systems / observers / packet filters / hotkeys from it. The SDK owns
 * every ABI export (mod_setup, mod_run, mod_observer, mod_filter, mod_filter_out,
 * mod_spawned, mod_alloc, mod_arena_reset).
 *
 * Memory model (read this once):
 * - Everything the SDK hands a callback (query rows, payload bytes, parsed structs,
 *   strings from cuo_fmt / cuo_cliloc / cuo_storage_get, JSON built with cuo_jw) lives
 *   in a CALL-SCOPED scratch arena that is rewound when the host makes its next call.
 *   Copy (strdup/malloc) anything you keep across calls.
 * - Strings you pass IN are only read during the call.
 *
 * Wire model: PUSH. Each tick the host serializes the rows of a system's queries into
 * its input; the system reads them and records commands, which the host applies AFTER
 * the callback returns (a component inserted here is not visible to a
 * cuo_component_json read later in the same call).
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

#define CUO_ABI_VERSION 2u
#define CUO_NONE_TYPE 0xFFFFu
#define CUO_APPEND 0xFFFFFFFFu

/* ── basics ──────────────────────────────────────────────────────────────────── */

typedef struct cuo_bytes {
    const uint8_t *ptr;
    size_t len;
} cuo_bytes;

/* Command-buffer entity ref: >= 0 a real ecs id, < 0 an entity spawned earlier in the
 * same buffer (the value cuo_spawn returned). Real ids convert with CUO_E(id). */
typedef int64_t cuo_eref;
#define CUO_E(id) ((cuo_eref)(uint64_t)(id))

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

/* ── registration (only inside cuo_setup) ────────────────────────────────────── */

typedef struct cuo_builder cuo_builder;
typedef struct cuo_input cuo_input;
typedef struct cuo_obs cuo_obs;
typedef struct cuo_cmds cuo_cmds;
typedef uint32_t cuo_sys;

/* Values match ModAbi.Schedule. */
typedef enum cuo_stage {
    CUO_STAGE_STARTUP = 0, /* once, after the host registered the mod (storage readable) */
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
} cuo_term_kind;

typedef struct cuo_term {
    uint8_t kind;
    uint16_t type_id;
} cuo_term;

#define CUO_REF(id) ((cuo_term){ CUO_TERM_REF, (uint16_t)(id) })
#define CUO_WITH(id) ((cuo_term){ CUO_TERM_WITH, (uint16_t)(id) })
#define CUO_WITHOUT(id) ((cuo_term){ CUO_TERM_WITHOUT, (uint16_t)(id) })
#define CUO_CHANGED(id) ((cuo_term){ CUO_TERM_CHANGED, (uint16_t)(id) })

typedef void (*cuo_system_fn)(const cuo_input *in, cuo_cmds *cmds, void *user);
typedef void (*cuo_observer_fn)(const cuo_obs *ev, cuo_cmds *cmds, void *user);
/* Return true to BLOCK the packet. Runs outside the ECS: no command buffer. */
typedef bool (*cuo_packet_fn)(uint8_t id, const uint8_t *data, size_t len, void *user);

/* Defined by the mod. Called once from mod_setup. */
void cuo_setup(cuo_builder *m);

/* Interned id of a registered type path (CUO_PATH_* / cuo_X_PATH). TRAPS when the host
 * does not register it — a silent sentinel would make every later command a no-op. */
uint16_t cuo_type_id(const char *path);
bool cuo_try_type_id(const char *path, uint16_t *out);
const char *cuo_type_path(uint16_t id); /* NULL when unknown (diagnostics) */

/* label: unique within the mod, used by the host for diagnostics and ordering
 * (NULL = "sys-<id>"). Default: every tick in `stage`. */
cuo_sys cuo_add_system(cuo_builder *m, const char *label, cuo_stage stage, cuo_system_fn fn, void *user);
cuo_sys cuo_add_system_in(cuo_builder *m, const char *label, const char *custom_stage, cuo_system_fn fn, void *user);
/* Declare a query; returns its ordinal (0, 1, … per system) for cuo_input_query.
 * Read terms (REF/MUT/CHANGED) fill row comp slots in declaration order. Like the C#
 * SDK, a WITH on a type already read is dropped and a CHANGED on a type already read
 * upgrades that term in place (one payload per type). The host drives the scan from
 * the FIRST required term: put the narrowest (a CHANGED, a rare marker) first. */
uint32_t cuo_system_query(cuo_builder *m, cuo_sys s, const cuo_term *terms, size_t n);
/* Run at most once per `ms` host-ms (gated before the queries are evaluated). */
void cuo_system_every(cuo_builder *m, cuo_sys s, uint32_t ms);
void cuo_system_after(cuo_builder *m, cuo_sys s, cuo_sys other);
void cuo_system_before(cuo_builder *m, cuo_sys s, cuo_sys other);

/* Observers run synchronously when the host fires them (no polling, no frame lag). */
uint32_t cuo_on_event(cuo_builder *m, const char *event_path, cuo_observer_fn fn, void *user);
uint32_t cuo_on_insert(cuo_builder *m, uint16_t type_id, cuo_observer_fn fn, void *user);
uint32_t cuo_on_remove(cuo_builder *m, uint16_t type_id, cuo_observer_fn fn, void *user);
uint32_t cuo_on_spawn(cuo_builder *m, cuo_observer_fn fn, void *user);
uint32_t cuo_on_despawn(cuo_builder *m, cuo_observer_fn fn, void *user);

/* One filter per direction (the ABI has one export each). A mod's own cuo_net_send
 * bypasses the outgoing filter. */
void cuo_on_packet_in(cuo_builder *m, cuo_packet_fn fn, void *user);
void cuo_on_packet_out(cuo_builder *m, cuo_packet_fn fn, void *user);

/* Hotkeys the host fires back as the cuo:input/hotkey event (observe it with
 * cuo_on_event(m, CUO_PATH_INPUT_HOTKEY, …) and parse cuo_ModHotkeyFired). All bindings
 * are published once, as the cuo:input/mod-hotkeys resource, from a Startup system
 * the SDK adds; re-publish a cuo_ModHotkeyBindingsDto with cuo_resource_set to rebind.
 * CUO_HK_CONSUME: this mod owns the combo (the host's own binding does not fire). */
enum { CUO_HK_CONSUME = 1, CUO_HK_CTRL = 2, CUO_HK_SHIFT = 4, CUO_HK_ALT = 8 };
void cuo_hotkey(cuo_builder *m, const char *name, uint32_t key, unsigned flags);
void cuo_hotkey_mouse(cuo_builder *m, const char *name, int32_t mouse_button, unsigned flags);

/* ── system input (PUSH snapshot) ────────────────────────────────────────────── */

typedef struct cuo_query {
    const void *vec; /* internal */
    size_t len;
} cuo_query;

typedef struct cuo_row {
    const void *table; /* internal */
} cuo_row;

uint32_t cuo_input_sys_id(const cuo_input *in);
uint64_t cuo_input_tick(const cuo_input *in);
/* The rows of the n-th cuo_system_query of this system (len 0 when absent). */
cuo_query cuo_input_query(const cuo_input *in, uint32_t n);
cuo_row cuo_query_row(cuo_query q, size_t i);
/* Linear scan for `entity` (Contains/TryGet). out may be NULL. */
bool cuo_query_find(cuo_query q, uint64_t entity, cuo_row *out);
uint64_t cuo_row_entity(cuo_row r);
size_t cuo_row_comp_count(cuo_row r);
/* JSON payload of read-term slot i ({NULL,0} when absent). Parse with cuo_X_parse. */
cuo_bytes cuo_row_comp(cuo_row r, size_t i);

/* ── observer input ──────────────────────────────────────────────────────────── */

uint32_t cuo_obs_id(const cuo_obs *ev);
uint64_t cuo_obs_entity(const cuo_obs *ev); /* 0 = global event */
/* Insert/Remove: the component JSON; Custom event: the event JSON; else {NULL,0}. */
cuo_bytes cuo_obs_value(const cuo_obs *ev);

/* ── commands (applied by the host after the callback returns) ───────────────── */

cuo_eref cuo_spawn(cuo_cmds *c, const cuo_comp *comps, size_t n);
/* Spawn and remember the real ecs id under `name`: cuo_entity(name) resolves it from
 * the NEXT call on, across frames, until cuo_forget / cuo_despawn_named. */
cuo_eref cuo_spawn_named(cuo_cmds *c, const char *name, const cuo_comp *comps, size_t n);
/* Spawn (named when name != NULL) and parent under `parent` (append). */
cuo_eref cuo_spawn_child(cuo_cmds *c, cuo_eref parent, const char *name, const cuo_comp *comps, size_t n);
void cuo_insert(cuo_cmds *c, cuo_eref e, const cuo_comp *comps, size_t n);
void cuo_insert1(cuo_cmds *c, cuo_eref e, cuo_comp comp);
void cuo_remove(cuo_cmds *c, cuo_eref e, const uint16_t *type_ids, size_t n);
void cuo_despawn(cuo_cmds *c, cuo_eref e); /* host despawns the subtree too */
/* Despawn the entity spawned under `name` and forget the name; false when unknown. */
bool cuo_despawn_named(cuo_cmds *c, const char *name);
void cuo_add_child(cuo_cmds *c, cuo_eref parent, cuo_eref child, uint32_t index);
void cuo_resource_set(cuo_cmds *c, cuo_comp value);
/* Custom event, utf8 JSON payload; entity 0 = global. Typed: cuo_X_emit. */
void cuo_emit(cuo_cmds *c, const char *event_path, uint64_t entity, cuo_bytes json);
void cuo_consume_mouse(cuo_cmds *c, uint8_t button);
void cuo_consume_key(cuo_cmds *c, uint32_t key);
size_t cuo_cmds_len(const cuo_cmds *c);

/* Chat output through cuo:chat/message (font 3, unicode — how server speech arrives). */
void cuo_chat_system(cuo_cmds *c, const char *text, uint16_t hue); /* journal sysmessage; hue 0x5B typical */
/* Text over a live entity (serial must be known to the client; 0 shows nothing). */
void cuo_chat_overhead(cuo_cmds *c, const char *text, uint16_t hue, uint32_t serial, const char *name);

/* ── pull side: synchronous host RPCs ────────────────────────────────────────── */

uint64_t cuo_tick(void); /* last host tick pushed to a system (ms) */
void cuo_log(const char *msg);
void cuo_logf(const char *fmt, ...) __attribute__((format(printf, 1, 2)));

bool cuo_entity(const char *name, uint64_t *out); /* see cuo_spawn_named */
void cuo_forget(const char *name);

/* JSON of a component on any entity / of a resource; {NULL,0} when absent. Typed:
 * cuo_X_get(entity, &out) / cuo_X_resource(&out). */
cuo_bytes cuo_component_json(uint64_t entity, uint16_t type_id);
cuo_bytes cuo_resource_json(uint16_t type_id);

/* Raw framed packet to the server. Prefer cuo_action_* (host updates client state). */
void cuo_net_send(const uint8_t *data, size_t len);
uint64_t cuo_resolve_serial(uint32_t serial); /* 0 = not mapped */

void cuo_gump_size(uint32_t gump_id, int *w, int *h); /* 0,0 when unknown */
int cuo_measure_text(uint32_t font, const char *text);
uint32_t cuo_hue_argb(uint32_t hue); /* 0xAARRGGBB; hue 0 = white */
const char *cuo_cliloc(uint32_t id); /* "" when unknown */
uint64_t cuo_entity_parent(uint64_t entity); /* 0 = root */
size_t cuo_entity_children(uint64_t entity, const uint64_t **out);

/* Per-mod persistent blob (<client>/Data/Mods/<mod>/storage.json), write-through.
 * Readable from CUO_STAGE_STARTUP on ("" before, and when never written). */
const char *cuo_storage_get(void);
void cuo_storage_set(const char *json);
void cuo_storage_set_bytes(cuo_bytes json);

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
