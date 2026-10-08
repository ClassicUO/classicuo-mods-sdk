# classicuo-mods-sdk

Guest-side SDKs for out-of-process ClassicUO mods: `wasm32-wasip2` WebAssembly components
implementing `wit/cuo-mod.wit`. Consumed as a git submodule — by the client
(`external/classicuo-mods-sdk`) and by standalone mod repos.

| Folder | What |
|---|---|
| `rust/` | `cuo-mod-sdk` crate (rlib) — `cuo-mod-sdk = { path = ".../rust" }` from a `cdylib` targeting `wasm32-wasip2` |
| `dotnet/` | `CuoModSdk` classlib + `ModSdk.props` / `ModSdk.targets` — import them top/bottom of a mod csproj, publish `-r wasi-wasm` (NativeAOT-LLVM) |
| `c/` | C11 SDK (wit-bindgen C bindings + cJSON vendored) — a mod Makefile includes `c/mod.mk`, wasi-sdk clang → `wasm32-wasip2` component; see `c/README.md` |

`rust/src/{paths,types}.rs`, `dotnet/{Paths,Types,Actions,KeyCode}.cs`, `c/include/cuo/{paths,types}.h`
and `c/src/types.c`
are GENERATED from the client's modding registry by `make gen-mod-sdk` in the ClassicUO repo.
Do not edit them here; change the registry and regenerate.

A C# mod: `<Import Project="<sdk>\dotnet\ModSdk.props" />` … `<CuoModType>My.Mod</CuoModType>` …
`<Import Project="<sdk>\dotnet\ModSdk.targets" />`. Restore needs the `dotnet-experimental`
NuGet feed for `Microsoft.DotNet.ILCompiler.LLVM` (see the client's `src/Mods/nuget.config`).
