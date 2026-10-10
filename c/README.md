# C mod SDK

C11 SDK for ClassicUO mods: a `wasm32-wasip2` **component** built with wasi-sdk clang
(>= 22, which links components through `wasm-component-ld`). Same model as the Rust
(`../rust`) and C# (`../dotnet`) SDKs, at a lower level: you declare parameters by hand
and read them by the handle you got back.

Each mod is a component of **its own world** (`wit/world.wit`, wasvy-style): it exports
one function per system / observer, named like it, taking that system's parameters as
typed arguments. The build generates the C bindings of that world with `wit-bindgen c`
plus a trampoline per export, and the SDK routes each export to the callback you
registered under the same name in `cuo_setup`.

```
c/
  include/cuo/cuo.h      the API (runtime, params, commands, observers, host calls, UI helpers)
  include/cuo/paths.h    GENERATED  CUO_PATH_* registry type paths
  include/cuo/types.h    GENERATED  payload structs (cuo_X), cuo_X_PATH / cuo_X_id(), CUO_KEY_*
  include/cuo/exports.h  internal: what the generated trampolines hand the dispatcher
  src/types.c            GENERATED  the cuo_X_id() type ids
  include/cuo/typed.h    GENERATED  the value API of every path (cuo_X_comp / _column / _set / _res / _put / _set_resource / _send / _events / _obs_value)
  include/cuo/typed_kinds.h GENERATED observer value record kinds (exports.h)
  src/typed.c            the generic half of the value API (builder chain, scratch strings)
  src/typed_gen.c        GENERATED  the per-type half: cuo_X <-> record converters, per-path functions
  src/*.c                runtime (scratch, setup, dispatch, commands, host wrappers, UI)
  wit/cuo-c-sdk.wit      world cuo:c-sdk/mod = cuo:modding/mod + the SDK's own export
  gen-exports.awk        per-mod trampolines from the generated header
  mod.mk                 build recipe
```

`paths.h`, `types.h`, `typed.h`, `typed_kinds.h`, `src/types.c` and `src/typed_gen.c` are
generated from the client's modding registry by `make gen-mod-sdk` in the ClassicUO repo
(tools/mod-typegen, `EmitC.cs` + `EmitSdkC.cs`). Do not edit them.

## A mod

```
my-mod/
  Makefile
  wit/world.wit
  mod.c
```

```make
# Makefile
CUO_SDK := ../path/to/classicuo-mods-sdk
SRCS    := mod.c
include $(CUO_SDK)/c/mod.mk
```

```wit
// wit/world.wit
package my:mod;

world my-mod {
    use tinyecs:modding/ecs@0.1.0.{commands, query, res, entity};
    use cuo:modding/types@0.1.0.{chat-message};

    export low-hp: func(commands: commands, hits: query, time: res);
    export on-chat: func(entity: entity, value: chat-message);
    export on-click: func(entity: entity);

    include cuo:c-sdk/mod@0.1.0;
}
```

```c
#include "cuo/cuo.h"

static cuo_param hits_q, time_r;

static void low_hp(const cuo_input *in, cuo_cmds *c, void *user)
{
    cuo_query q = cuo_input_query(in, hits_q);
    const cuo_Hits *hits = cuo_Hits_column(q, 0);
    for (size_t i = 0; i < q.len; i++)
        if (hits[i].value < hits[i].max_value / 4)
            cuo_chat_system(c, "Low HP!", 0x21);
}

static void on_chat(const cuo_obs *ev, cuo_cmds *c, void *user)
{
    cuo_logf("chat: %s", cuo_ModChatMessage_obs_value(ev)->text);
}

static void on_click(const cuo_obs *ev, cuo_cmds *c, void *user)
{
    cuo_logf("clicked %llu", (unsigned long long)cuo_obs_entity(ev));
}

void cuo_setup(cuo_builder *m)
{
    cuo_sys s = cuo_add_system(m, "low-hp", CUO_STAGE_UPDATE, low_hp, NULL);
    hits_q = cuo_system_query(m, s, CUO_TERMS(CUO_CHANGED(cuo_Hits_id()), CUO_WITH(cuo_Player_id())));
    time_r = cuo_system_res(m, s, cuo_Time_id(), false); /* cuo_Time_res(in, time_r, &t) */
    cuo_on_event(m, "on-chat", cuo_ModChatMessage_PATH, on_chat, NULL);
    cuo_on_event(m, "on-click", cuo_ModClick_PATH, on_click, NULL);
}
```

- **The world**: `include cuo:c-sdk/mod@0.1.0` (that is `cuo:modding/mod` plus the
  SDK's `cuo-sdk-hotkeys` export), and one export per system / observer you declare:
  - system: `func(<params>)`
  - observer (`cuo_on_event` / `_add` / `_remove`): `func(entity: entity, value: <record>,
    <params>)`, where `<record>` is the `cuo:modding/types` record of the observed path
    (`use cuo:modding/types@0.1.0.{<record>}`), or `func(entity: entity, <params>)` for a
    path without a value (a tag component, a zero-size event like `cuo:ui/click`)
  - packet observer (`cuo_on_packet`, `cuo_on_packet_in/out`):
    `func(direction: packet-direction, packet: list<u8>, <params>) -> verdict`

  where the params are, in declaration order, `query` (`cuo_*_query`), `res`
  (`cuo_*_res`, read-only or mut) and `events` (`cuo_*_events`), plus `commands` —
  which is not declared in `cuo_setup`: put it anywhere in the export, or leave it out
  for a system that records nothing (its `cuo_cmds` then traps on use). Names are the
  export's kebab-case name (`"low-hp"`; `"low_hp"` works too). A declaration whose
  export is missing or does not match traps at load, quoting the export line to add
  (records included). Exports nothing declares are ignored.
- **Params**: `cuo_system_query` / `cuo_system_res` / `cuo_system_events` (and the
  `cuo_observer_*` twins for an observer's own params) return a handle; read the data
  with `cuo_input_query` + the generated `cuo_X_column` / `cuo_X_res` / `cuo_X_events`
  (`cuo_obs_query` / `cuo_X_obs_res` / `cuo_X_obs_events` in an observer). Each query's
  entities arrive in one host call (`query.entities`) when the callback starts.
- **Writing**: `cuo_insert` (a command), `cuo_X_set` through a `CUO_MUT` term,
  `cuo_X_put` through a `mut` res param (immediate) or `cuo_X_set_resource` (a command, no
  declaration), `cuo_X_send` for events.
- **Entities**: `cuo_spawn` returns the real id at once; keep it in a static and use it
  in later runs. Parent with `cuo_child_of(parent)` (or `cuo_spawn_child`).
- **Host functions**: typed wrappers for the common ones (`cuo_log`,
  `cuo_storage_get/set`, `cuo_measure_text`, `cuo_resolve_serial`, `cuo_gump_size`,
  `cuo_hue_argb`, `cuo_cliloc`, `cuo_send_to_server/client`); anything else is the
  generated import, e.g. `cuo_modding_actions_cast_spell(29)`,
  `cuo_modding_actions_double_click(serial)` (results a generated import returns are
  malloc'd: free them with the matching `*_free`). The storage blob's format is the
  mod's own; the SDK ships no JSON library.
- **Packets**: `cuo_on_packet(m, "name", CUO_INCOMING, CUO_PACKET_IDS(0x1C), fn, user)`
  (n 0 = every id); the observer reads `cuo_obs_packet`, may record commands and declare
  `cuo_observer_*` params, and returns `CUO_PASS`, `CUO_BLOCK` or `CUO_REPLACE` (with
  `*replacement` set). `cuo_on_packet_in/out(m, "name", fn, user)` are block-or-pass
  taps on every id.
- **Hotkeys**: `cuo_hotkey` / `cuo_hotkey_mouse` in `cuo_setup`; the SDK publishes them
  from its own Startup system (`cuo-sdk-hotkeys`, exported through `cuo:c-sdk/mod`).
- **Type ids** (`cuo_X_id()`, `cuo_type_id(path)`) are guest-local handles for type
  paths; the host checks the paths themselves when the mod declares / uses them.

## Components, resources, events

Every registered type path crosses as a typed WIT record through
`cuo:modding/components` (generated in `../wit/cuo-mod.wit`; the world gets it through
`cuo:c-sdk/mod`). There is no JSON anywhere: the SDK converts the generated structs
(`cuo_Node`, `cuo_Hits`, ...) to and from the records. Read-only paths (player stats,
serials, host-computed UI state, presence markers) have no insert / set / put function.

| | |
|---|---|
| insert / spawn | `cuo_X_comp(&v)` (tags: `cuo_UiMovable_comp()`, `cuo_UiNoWindowDrag_comp()`) |
| read a query column | `const cuo_X *col = cuo_X_column(q, slot)`, aligned with `cuo_query_entities(q)` |
| write back (`CUO_MUT`) | `cuo_X_set(q, e, slot, &v)` |
| resource param | `cuo_X_res(in, p, &v)` (`_obs_res` in observers; false while the host has none) |
| write a resource | `cuo_X_put(in, p, &v)` through a `mut` res param (`_obs_put`), or `cuo_X_set_resource(c, &v)` (deferred) |
| send an event | `cuo_X_send(c, &v)` (zero-size event: `cuo_X_send(c)`) |
| events param | `const cuo_X *ev = cuo_X_events(in, p, &n)` (zero-size: the count); `cuo_input_events_len(in, p)` |
| observer trigger | `const cuo_X *v = cuo_X_obs_value(ev)` (NULL when X is not the trigger's type) |

```c
cuo_Node n = cuo_node_abs(40, 140, 240, 120);
cuo_entity root = cuo_spawn(c, CUO_COMPS(
    cuo_Node_comp(&n),
    cuo_BackgroundColor_comp(&(cuo_BackgroundColor){ .value = cuo_rgba(18, 18, 26, 235) }),
    cuo_UiMovable_comp(),
    cuo_GlobalZIndex_comp(&(cuo_GlobalZIndex){ .value = 100 }),
    cuo_UiName_comp(&(cuo_UiName){ .value = "my.root" })));
cuo_insert1(c, label, cuo_Text_comp(&(cuo_Text){ .value = cuo_fmt("clicks: %d", n) }));

/* read: a column per read term, one host call each */
cuo_query q = cuo_input_query(in, texts_q);            /* CUO_MUT(cuo_Text_id()) */
const cuo_entity *ents = cuo_query_entities(q);
const cuo_Text *texts = cuo_Text_column(q, 0);
for (size_t i = 0; i < q.len; i++)
    if (strcmp(texts[i].value, "old") == 0)
        cuo_Text_set(q, ents[i], 0, &(cuo_Text){ .value = "new" });
```

- A spawn / insert is the WIT entity builder (`components.spawn` / `entity-of`) with one
  chained call per component — deferred and in order like every command. A spawn of no
  components is `commands.spawn`. The value is copied (strings too).
- Columns and `cuo_query_entities` are call-scoped scratch.
- A res param's value is fetched on first use and reused while the host reports it
  unchanged; a `cuo_X_put` through it drops the cached value.
- The generated imports are callable directly too
  (`cuo_modding_components_column_hits`, `cuo_modding_components_spawn`, ...).

## Build

`make` writes `build/mod.wasm` (`-Oz`, LTO, stripped). Ship it as
`<mods>/<name>/{mod.wasm,mod.json}`. On the way it stages `build/wit/` (your
`wit/world.wit` + the contract from `../wit` and `c/wit` under `deps/`), runs
`wit-bindgen c` into `build/gen/` (`cuo_wit.{c,h}`, the component-type object; C names
are always `cuo_wit_*` / `exports_cuo_wit_*` via `--rename-world`) and
`gen-exports.awk` into `build/gen/cuo_exports.c`. Needs:

- wasi-sdk: `WASI_SDK` or `WASI_SDK_PATH` (default `/opt/wasi-sdk`);
- wit-bindgen-cli 0.57 (`cargo install wit-bindgen-cli --version ^0.57`, the family the
  Rust SDK uses) on `PATH`, or `WIT_BINDGEN=...`;
- awk.

`make DEBUG=1` keeps names and asserts. `WORLD_WIT=...` points at a world elsewhere.

The reference mods are the client's `src/Mods/ecs-c` (twin of `ecs-csharp`) and
`src/Mods/ecs-cprobe` (the rest of the surface).
