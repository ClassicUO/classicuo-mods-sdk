# C mod SDK

C11 SDK for ClassicUO mods: a `wasm32-wasip1` **reactor core module** built with wasi-sdk
clang. The ECS half speaks the FlatBuffers ABI v3 in `../abi/mod-abi.fbs`; everything
else (host / assets / actions / packets in `../wit/cuo-mod.wit`) is JSON over the single
`env.mod_call` import (`../docs/p1-wire.md`). Same model as the Rust (`../rust`) and C#
(`../dotnet`) SDKs, at a lower level: you declare parameters by hand and read them by
the handle you got back.

```
c/
  include/cuo/cuo.h      the API (runtime, params, commands, observers, host calls, JSON, UI helpers)
  include/cuo/paths.h    GENERATED  CUO_PATH_* registry type paths
  include/cuo/types.h    GENERATED  payload structs + (de)serializers, CUO_KEY_*
  src/types.c            GENERATED  their implementation
  src/*.c                runtime (arena, exports, dispatch, commands, mod_call, JSON, UI)
  generated/             flatcc v0.6.1 reader/builder headers for mod-abi.fbs (regen-abi.sh)
  third_party/flatcc     flatcc runtime (builder/emitter/refmap) + headers, Apache-2.0
  third_party/cjson      cJSON v1.7.18, MIT
  mod.mk                 build recipe
```

`paths.h`, `types.h` and `src/types.c` are generated from the client's modding registry
by `make gen-mod-sdk` in the ClassicUO repo (tools/mod-typegen, `EmitC.cs`). Do not edit
them. `generated/` is regenerated with `FLATCC=<flatcc v0.6.1> ./regen-abi.sh` whenever
`abi/mod-abi.fbs` changes.

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
  `cuo_observer_*` twins for an observer's own params) return a handle; read the pushed
  data with `cuo_input_query/res/events` (`cuo_obs_*` in an observer).
- **Entities**: `cuo_spawn` returns a placeholder (`CUO_PENDING | temp`) that becomes the
  real id once the host applied the run; keep it in a static and use it later —
  `cuo_resolve` / `cuo_entity_eq` see the real id. Parent with `cuo_child_of(parent)` (or
  `cuo_spawn_child`); a parent spawned in the same run is fine.
- **Host functions**: `cuo_call("cuo:modding/<iface>#<fn>", "<json args>")` for anything
  in the WIT; typed wrappers for the common ones (`cuo_log`, `cuo_storage_get/set`,
  `cuo_measure_text`, `cuo_resolve_serial`, `cuo_gump_size`, `cuo_cliloc`, `cuo_intercept`,
  `cuo_send_to_server/client`) and `cuo_action("cast-spell", "[29]")`.
- **Packets**: `cuo_intercept(CUO_INCOMING, ids, n)` in `cuo_setup`, then
  `cuo_on_packet(m, fn, user)`; the handler returns `CUO_PASS`, `CUO_BLOCK` or
  `CUO_REPLACE` (with `*replacement` set).

`make` writes `build/mod.wasm` (`-Oz`, LTO, stripped). Ship it as
`<mods>/<name>/{mod.wasm,mod.json}`. wasi-sdk location: `WASI_SDK` or `WASI_SDK_PATH`
(default `/opt/wasi-sdk`). `make DEBUG=1` keeps names and asserts.

The reference mods are the client's `src/Mods/ecs-c` (twin of `ecs-csharp`) and
`src/Mods/ecs-cprobe` (the rest of the surface).
