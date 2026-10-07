//! planus-generated flatbuffer types for the mod ABI schema (`abi/mod-abi.fbs`, a
//! snapshot of `external/TinyEcs/src/TinyEcs.Bevy.Modding/abi/mod-abi.fbs`).
//!
//! `generated.rs` is checked in; regenerate with `make gen-abi` (needs planus-cli 1.3.0).

#[allow(warnings, clippy::all)]
#[path = "generated.rs"]
mod generated;

pub use generated::mod_abi::*;
