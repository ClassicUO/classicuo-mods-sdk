# C mod SDK

C11 SDK for ClassicUO mods: a `wasm32-wasip2` **component** of the `cuo:modding/mod`
world (`../wit/cuo-mod.wit` + `../wit/deps/tinyecs-mod`), built with wasi-sdk clang
(>= 22, which links components through `wasm-component-ld`). The SDK implements the
component's exports (`setup` / `run` / `observe` / `observe-packet`) over the
wit-bindgen C bindings in `generated/`. Same model as the Rust (`../rust`) and C#
(`../dotnet`) SDKs, at a lower level: you declare parameters by hand and read them by
the handle you got back.

```
c/
  include/cuo/cuo.h      the API (runtime, params, commands, observers, host calls, JSON, UI helpers)
  include/cuo/paths.h    GENERATED  CUO_PATH_* registry type paths
  include/cuo/types.h    GENERATED  payload structs + (de)serializers, CUO_KEY_*
  src/types.c            GENERATED  their implementation
  src/*.c                runtime (scratch, exports, dispatch, commands, host wrappers, JSON, UI)
  generated/             GENERATED  wit-bindgen C bindings: cuo_wit.{c,h} + the component-type object
  third_party/cjson      cJSON v1.7.18, MIT
  mod.mk                 build recipe
  regen-wit.sh           regenerates generated/ (wit-bindgen-cli 0.57)
```

`paths.h`, `types.h` and `src/types.c` are generated from the client's modding registry
by `make gen-mod-sdk` in the ClassicUO repo (tools/mod-typegen, `EmitC.cs`). Do not edit
them. `generated/` is regenerated with `./regen-wit.sh` whenever `../wit` changes; mod
authors do not need wit-bindgen.

## A mod

```make
# Makefile
CUO_SDK := ../path/to/classicuo-mods-sdk
SRCS    := mod.c
include $(CUO_SDK)/c/mod.mk
```

```c
#include "cuo/cuo.h"

static cuo_param hits_q, time_r;

static void low_hp(const cuo_input *in, cuo_cmds *c, void *user)
{
    cuo_query q = cuo_input_query(in, hits_q);
    for (size_t i = 0; i < q.len; i++) {
        cuo_Hits h;
        if (cuo_Hits_parse(cuo_row_comp(cuo_query_row(q, i), 0), &h) && h.value < h.max_value / 4)
            cuo_chat_system(c, "Low HP!", 0x21);
    }
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
    cuo_on_event(m, cuo_ModClick_PATH, on_click, NULL);
}
```

- **Params**: `cuo_system_query` / `cuo_system_res` / `cuo_system_events` (and the
  `cuo_observer_*` twins for an observer's own params) return a handle; read the data
  with `cuo_input_query/res/events` (`cuo_obs_*` in an observer). Each query's rows
  arrive in one host call (`query.rows`) when the callback starts.
- **Writing components**: `cuo_insert` (a command), or declare the term `CUO_MUT(id)`
  and write the row back with `cuo_query_set(q, entity, slot, json)` (`query.set`).
  **Resources**: declare `cuo_system_res(m, s, id, true)` and `cuo_resource_set`.
- **Entities**: `cuo_spawn` returns the real id at once; keep it in a static and use it
  in later runs. Parent with `cuo_child_of(parent)` (or `cuo_spawn_child`).
- **Host functions**: typed wrappers for the common ones (`cuo_log`,
  `cuo_storage_get/set`, `cuo_measure_text`, `cuo_resolve_serial`, `cuo_gump_size`,
  `cuo_hue_argb`, `cuo_cliloc`, `cuo_send_to_server/client`); anything else is the
  generated import, e.g. `cuo_modding_actions_cast_spell(29)`,
  `cuo_modding_actions_double_click(serial)` (results a generated import returns are
  malloc'd: free them with the matching `*_free`).
- **Packets**: packet observers. `cuo_on_packet(m, CUO_INCOMING, CUO_PACKET_IDS(0x1C), fn, user)`
  (n 0 = every id); the observer reads `cuo_obs_packet`, may record commands and declare
  `cuo_observer_*` params, and returns `CUO_PASS`, `CUO_BLOCK` or `CUO_REPLACE` (with
  `*replacement` set). `cuo_on_packet_in/out(m, fn, user)` are block-or-pass taps on every id.
- **Type ids** (`cuo_X_id()`, `cuo_type_id(path)`) are guest-local handles for type
  paths; the host checks the paths themselves when the mod declares / uses them.

`make` writes `build/mod.wasm` (`-Oz`, LTO, stripped). Ship it as
`<mods>/<name>/{mod.wasm,mod.json}`. wasi-sdk location: `WASI_SDK` or `WASI_SDK_PATH`
(default `/opt/wasi-sdk`). `make DEBUG=1` keeps names and asserts.

The reference mods are the client's `src/Mods/ecs-c` (twin of `ecs-csharp`) and
`src/Mods/ecs-cprobe` (the rest of the surface).
