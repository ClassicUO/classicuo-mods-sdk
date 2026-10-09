# classicuo-mods-sdk

Guest-side SDKs for out-of-process ClassicUO mods: `wasm32-wasip2` WebAssembly components
implementing `wit/cuo-mod.wit`. Consumed as a git submodule — by the client
(`external/classicuo-mods-sdk`) and by standalone mod repos.

| Folder | What |
|---|---|
| `rust/` | `cuo-mod-sdk` crate (rlib) — `cuo-mod-sdk = { path = ".../rust" }` from a `cdylib` targeting `wasm32-wasip2` |
| `dotnet/` | `CuoModSdk` classlib + `ModSdk.props` / `ModSdk.targets` — import them top/bottom of a mod csproj, publish `-r wasi-wasm` (NativeAOT-LLVM) |
| `c/` | C11 SDK (cJSON vendored) — a mod has its own `wit/world.wit` and its Makefile includes `c/mod.mk`, which runs `wit-bindgen c` on it + wasi-sdk clang → `wasm32-wasip2` component; see `c/README.md` |

`rust/src/{paths,types}.rs`, `dotnet/{Paths,Types,Actions,KeyCode}.cs`, `c/include/cuo/{paths,types}.h`
and `c/src/types.c`
are GENERATED from the client's modding registry by `make gen-mod-sdk` in the ClassicUO repo.
Do not edit them here; change the registry and regenerate.

A Rust mod: Bevy-style `fn setup(app: &mut App)` + `export_mod!(setup)`; mark every system /
observer fn `#[system]` (from the prelude). Each becomes an export of the mod's own world, named
like the fn in kebab-case (`snap_input` -> `snap-input`), its WIT params derived from the fn's
param types (`Local` stays guest-side); wit-component merges them with `setup` when the
component links. See `rust/src/lib.rs` and `rust/examples/low_hp`.

A C# mod: `<Import Project="<sdk>\dotnet\ModSdk.props" />` … `<CuoModType>My.Mod</CuoModType>` …
`<Import Project="<sdk>\dotnet\ModSdk.targets" />`. Each system / observer is an export of the mod's
own world, named like the system (`.Label("x.y")` -> `x-y`, default `systemN` / `observerN`): the
build runs the mod's `Setup` on the build machine to generate the exports and the world WIT
(`dotnet/ModDescribe.cs`), so `Setup` must only register, deterministically. Restore needs the `dotnet-experimental`
NuGet feed for `Microsoft.DotNet.ILCompiler.LLVM` (see the client's `src/Mods/nuget.config`).
