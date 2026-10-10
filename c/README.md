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
  include/cuo/cuo.h      the API (runtime, params, commands, observers, host calls, JSON, UI helpers)
  include/cuo/paths.h    GENERATED  CUO_PATH_* registry type paths
  include/cuo/types.h    GENERATED  payload structs + (de)serializers, CUO_KEY_*
  include/cuo/exports.h  internal: what the generated trampolines hand the dispatcher
  src/types.c            GENERATED  their implementation
  include/cuo/typed.h    GENERATED  the typed API of every curated path (cuo_X_typed / _column / _set / _res / _send / _events / _obs_typed)
  include/cuo/typed_kinds.h GENERATED typed observer record kinds (exports.h)
  src/typed.c            typed components: the generic half (builder chain, scratch strings)
  src/typed_gen.c        GENERATED  the per-type half: curated table, cuo_X <-> record converters
  src/*.c                runtime (scratch, setup, dispatch, commands, host wrappers, JSON, UI)
  wit/cuo-c-sdk.wit      world cuo:c-sdk/mod = cuo:modding/mod + the SDK's own export
  gen-exports.awk        per-mod trampolines from the generated header
  third_party/cjson      cJSON v1.7.18, MIT
  mod.mk                 build recipe
```

`paths.h`, `types.h` and `src/types.c` are generated from the client's modding registry
by `make gen-mod-sdk` in the ClassicUO repo (tools/mod-typegen, `EmitC.cs`). Do not edit
them.

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
    use tinyecs:modding/ecs@0.1.0.{commands, query, res, trigger-data};

    export low-hp: func(commands: commands, hits: query, time: res);
    export on-click: func(trigger: trigger-data);

    include cuo:c-sdk/mod@0.1.0;
}
```

```c
#include "cuo/cuo.h"

static cuo_param hits_q, time_r;

static void low_hp(const cuo_input *in, cuo_cmds *c, void *user)
{
    cuo_query q = cuo_input_query(in, hits_q);
    const cuo_Hits *hits = cuo_Hits_column(q, 0); /* typed: no JSON */
    for (size_t i = 0; i < q.len; i++)
        if (hits[i].value < hits[i].max_value / 4)
            cuo_chat_system(c, "Low HP!", 0x21);
}

static void on_click(const cuo_obs *ev, cuo_cmds *c, void *user)
{
    cuo_logf("clicked %llu", (unsigned long long)cuo_obs_entity(ev));
}

void cuo_setup(cuo_builder *m)
{
    cuo_sys s = cuo_add_system(m, "low-hp", CUO_STAGE_UPDATE, low_hp, NULL);
    hits_q = cuo_system_query(m, s, CUO_TERMS(CUO_CHANGED(cuo_Hits_id()), CUO_WITH(cuo_Player_id())));
    time_r = cuo_system_res(m, s, cuo_Time_id(), false); /* cuo_input_res(in, time_r) */
    cuo_on_event(m, "on-click", cuo_ModClick_PATH, on_click, NULL);
}
```

- **The world**: `include cuo:c-sdk/mod@0.1.0` (that is `cuo:modding/mod` plus the
  SDK's `cuo-sdk-hotkeys` export), and one export per system / observer you declare:
  - system: `func(<params>)`
  - observer (`cuo_on_event` / `_add` / `_remove`): `func(trigger: trigger-data, <params>)`
  - packet observer (`cuo_on_packet`, `cuo_on_packet_in/out`):
    `func(direction: packet-direction, packet: list<u8>, <params>) -> verdict`

  where the params are, in declaration order, `query` (`cuo_*_query`), `res`
  (`cuo_*_res`, read-only or mut) and `events` (`cuo_*_events`), plus `commands` —
  which is not declared in `cuo_setup`: put it anywhere in the export, or leave it out
  for a system that records nothing (its `cuo_cmds` then traps on use). Names are the
  export's kebab-case name (`"low-hp"`; `"low_hp"` works too). A declaration whose
  export is missing or does not match traps at load, quoting the export line to add.
  Exports nothing declares are ignored.
- **Params**: `cuo_system_query` / `cuo_system_res` / `cuo_system_events` (and the
  `cuo_observer_*` twins for an observer's own params) return a handle; read the data
  with `cuo_input_query/res/events` (`cuo_obs_*` in an observer). Each query's rows
  arrive in one host call (`query.rows`) when the callback starts.
- **Writing components**: `cuo_insert` (a command), or declare the term `CUO_MUT(id)`
  and write the row back with `cuo_query_set(q, entity, slot, json)` (`query.set`).
- **Typed components** (below): the curated types skip JSON.
  **Resources**: declare `cuo_system_res(m, s, id, true)` and `cuo_resource_set`, or
  `cuo_set_resource` (a command, no declaration).
- **Entities**: `cuo_spawn` returns the real id at once; keep it in a static and use it
  in later runs. Parent with `cuo_child_of(parent)` (or `cuo_spawn_child`).
- **Host functions**: typed wrappers for the common ones (`cuo_log`,
  `cuo_storage_get/set`, `cuo_measure_text`, `cuo_resolve_serial`, `cuo_gump_size`,
  `cuo_hue_argb`, `cuo_cliloc`, `cuo_send_to_server/client`); anything else is the
  generated import, e.g. `cuo_modding_actions_cast_spell(29)`,
  `cuo_modding_actions_double_click(serial)` (results a generated import returns are
  malloc'd: free them with the matching `*_free`).
- **Packets**: `cuo_on_packet(m, "name", CUO_INCOMING, CUO_PACKET_IDS(0x1C), fn, user)`
  (n 0 = every id); the observer reads `cuo_obs_packet`, may record commands and declare
  `cuo_observer_*` params, and returns `CUO_PASS`, `CUO_BLOCK` or `CUO_REPLACE` (with
  `*replacement` set). `cuo_on_packet_in/out(m, "name", fn, user)` are block-or-pass
  taps on every id.
- **Hotkeys**: `cuo_hotkey` / `cuo_hotkey_mouse` in `cuo_setup`; the SDK publishes them
  from its own Startup system (`cuo-sdk-hotkeys`, exported through `cuo:c-sdk/mod`).
- **Type ids** (`cuo_X_id()`, `cuo_type_id(path)`) are guest-local handles for type
  paths; the host checks the paths themselves when the mod declares / uses them.

## Typed components

The curated types — every component / resource / event the shipped mods use whose
payload has a WIT shape (`include/cuo/typed.h` lists them) — also cross as typed
WIT records through `cuo:modding/components` (generated in `../wit/cuo-mod.wit`; the
world gets it through `cuo:c-sdk/mod`). You keep the same structs (`cuo_Node`,
`cuo_Hits`, ...) and UI helpers; the SDK converts them to the records. Everything else
stays on the JSON path, and both mix freely.

| | JSON | typed |
|---|---|---|
| insert / spawn | `cuo_X_comp(&v)` | `cuo_X_typed(&v)` (tags: `cuo_UiMovable_typed()`, `cuo_UiNoWindowDrag_typed()`) |
| read a query column | `cuo_X_parse(cuo_row_comp(row, slot), &v)` | `const cuo_X *col = cuo_X_column(q, slot)`, aligned with `cuo_query_entities(q)` |
| write back (`CUO_MUT`) | `cuo_query_set(q, e, slot, json)` | `cuo_X_set(q, e, slot, &v)` (not the read-only player / serial types) |
| resource param | `cuo_X_parse(cuo_input_res(in, p), &v)` | `cuo_X_res(in, p, &v)` (`_obs_res` in observers) |
| send an event | `cuo_X_emit(c, 0, &v)` | `cuo_X_send(c, &v)` (zero-size event: `cuo_X_send(c)`) |
| events param | `cuo_X_parse(cuo_events_at(cuo_input_events(in, p), i), &v)` | `const cuo_X *ev = cuo_X_events(in, p, &n)` (zero-size: the count) |
| observer trigger | export `func(trigger: trigger-data, ...)`, `cuo_X_parse(cuo_obs_value(ev), &v)` | export `func(entity: entity, value: <record>, ...)` (a tag: `entity` alone), `const cuo_X *v = cuo_X_obs_typed(ev)` |

A typed observer export names its record from `cuo:modding/types` in the mod's world:

```wit
use tinyecs:modding/ecs@0.1.0.{commands, entity};
use cuo:modding/types@0.1.0.{item-move-result};
export on-move: func(entity: entity, value: item-move-result, commands: commands);
```

```c
/* spawn: node / colours / movable / z typed, the name (not curated) as JSON */
cuo_Node n = cuo_node_abs(40, 140, 240, 120);
cuo_entity root = cuo_spawn(c, CUO_COMPS(
    cuo_Node_typed(&n),
    cuo_BackgroundColor_typed(&(cuo_BackgroundColor){ .value = cuo_rgba(18, 18, 26, 235) }),
    cuo_UiMovable_typed(),
    cuo_GlobalZIndex_typed(&(cuo_GlobalZIndex){ .value = 100 }),
    cuo_UiName_comp(&(cuo_UiName){ .value = "my.root" })));
cuo_insert1(c, label, cuo_Text_typed(&(cuo_Text){ .value = cuo_fmt("clicks: %d", n) }));

/* read: a column per read term, one host call each */
cuo_query q = cuo_input_query(in, texts_q);            /* CUO_MUT(cuo_Text_id()) */
const cuo_entity *ents = cuo_query_entities(q);
const cuo_Text *texts = cuo_Text_column(q, 0);
for (size_t i = 0; i < q.len; i++)
    if (strcmp(texts[i].value, "old") == 0)
        cuo_Text_set(q, ents[i], 0, &(cuo_Text){ .value = "new" });
```

- A spawn / insert holding typed components is the WIT entity builder
  (`components.spawn` / `entity-of`) with one chained call per typed component, after
  one `commands.insert` of its JSON ones — deferred and in order like every command.
  `cuo_child_of` / `cuo_spawn_child` are typed. The value is copied (strings too).
- A query whose read terms all have a column starts with `query.entities` instead of
  `query.rows`; rows are fetched only if you read one (`cuo_query_row` /
  `cuo_query_find`). Columns and `cuo_query_entities` are call-scoped scratch.
- A res param's value is fetched on first use (JSON or typed) and reused while the host
  reports it unchanged.
- The generated imports are callable directly too
  (`cuo_modding_components_column_hits`, `cuo_modding_components_spawn`, ...).
- The curated list lives in `src/typed.c` (`CURATED` + one converter per type): a type
  added to the generated `components` interface needs its row there.

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
