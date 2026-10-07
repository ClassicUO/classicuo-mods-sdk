# p1 wire (SDK-internal)

How a `wasm32-wasip1` core module reaches the functions in `wit/cuo-mod.wit`.
Mod authors never see this: the SDKs hide it. p2 components use the WIT directly.

The ECS half (`tinyecs:modding`) stays on the FlatBuffers ABI (`abi/mod-abi.fbs`).
Everything else (`host`, `assets`, `actions`, `packets`) goes through one import.

## `mod_call`

```
import "env" "mod_call" (name_ptr: i32, name_len: i32, args_ptr: i32, args_len: i32) -> i64
```

- `name` — UTF-8 `"<package>/<interface>#<function>"` without the version,
  e.g. `"cuo:modding/assets#image-size"`.
- `args` — UTF-8 JSON array, the parameters in WIT order.
- result — `0` for a function with no result; otherwise `len << 32 | ptr` of UTF-8 JSON
  the host wrote into the guest arena (`alloc` export), valid until `arena_reset`.
- An unknown name or malformed args traps.

## JSON encoding of WIT values

| WIT | JSON |
|---|---|
| `bool`, `string`, `char` | boolean, string, one-char string |
| `u8`..`u32`, `s8`..`s32`, `f32`, `f64` | number |
| `u64`, `s64` (incl. `entity`) | decimal string |
| `list<u8>` | base64 string |
| other `list<T>`, `tuple<..>` | array |
| `record` | object, kebab-case field names as in the WIT |
| `enum` | kebab-case case name |
| `variant` | `{"<case>": payload}`; a case without payload has `null` |
| `option<T>` | `null` or the value |

## `on-packet`

Hot path, so binary:

```
export "mod_on_packet" (dir: i32, ptr: i32, len: i32) -> i64
```

`dir` 0 = incoming, 1 = outgoing; the packet bytes are in the arena. Returns `0` pass,
`1` block, otherwise `len << 32 | ptr` of the replacement bytes in the arena.
