# C mod SDK

C11 SDK for core-wasm ClassicUO mods: a `wasm32-wasip1` **reactor core module** built
with wasi-sdk clang, speaking the FlatBuffers ABI in `../abi/mod-abi.fbs` (NOT the
component model). Same shape as the Rust (`../rust`) and C# (`../dotnet`) SDKs.

```
c/
  include/cuo/cuo.h      the API (runtime, commands, queries, observers, JSON, UI helpers)
  include/cuo/paths.h    GENERATED  CUO_PATH_* registry type paths
  include/cuo/types.h    GENERATED  payload structs + (de)serializers, typed actions, CUO_KEY_*
  src/types.c            GENERATED  their implementation
  src/*.c                runtime (arena, exports, dispatch, commands, imports, JSON, UI)
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

static void hello(const cuo_input *in, cuo_cmds *c, void *user)
{
    cuo_chat_system(c, "hello from C", 0x5B);
}

void cuo_setup(cuo_builder *m)
{
    cuo_add_system(m, "hello", CUO_STAGE_STARTUP, hello, NULL);
}
```

`make` writes `build/mod.wasm` (`-Oz`, LTO, stripped). Ship it as
`<mods>/<name>/{mod.wasm,mod.json}`. wasi-sdk location: `WASI_SDK` or `WASI_SDK_PATH`
(default `/opt/wasi-sdk`). `make DEBUG=1` keeps names and asserts.

The reference mod is the client's `src/Mods/ecs-c` (twin of `ecs-csharp`).
