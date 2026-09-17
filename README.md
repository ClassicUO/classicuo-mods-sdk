# classicuo-mods-sdk

Guest-side SDKs for out-of-process, core-wasm ClassicUO mods (FlatBuffers ABI, NOT the
WebAssembly Component Model). Consumed as a git submodule — by the client
(`external/classicuo-mods-sdk`) and by standalone mod repos.

| Folder | What |
|---|---|
| `abi/mod-abi.fbs` | snapshot of the wire schema; canonical copy lives in `TinyEcs.Bevy.Modding/abi/` |
| `rust/` | `cuo-mod-sdk` crate (rlib) — `cuo-mod-sdk = { path = ".../rust" }` from a `cdylib` targeting `wasm32-wasip1` |
| `dotnet/` | `CuoModSdk` classlib + `ModSdk.props` / `ModSdk.targets` — import them top/bottom of a mod csproj, publish `-r wasi-wasm` (NativeAOT-LLVM) |

`rust/src/{paths,types}.rs`, `dotnet/{Paths,Types,Actions,KeyCode}.cs` and `abi/mod-abi.fbs`
are GENERATED from the client's modding registry by `make gen-mod-sdk` in the ClassicUO repo.
Do not edit them here; change the registry and regenerate.

A C# mod: `<Import Project="<sdk>\dotnet\ModSdk.props" />` … `<CuoModType>My.Mod</CuoModType>` …
`<Import Project="<sdk>\dotnet\ModSdk.targets" />`. Restore needs the `dotnet-experimental`
NuGet feed for `Microsoft.DotNet.ILCompiler.LLVM` (see the client's `src/Mods/nuget.config`).
