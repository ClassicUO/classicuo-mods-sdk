//! planus-generated flatbuffer types for the canonical mod ABI schema
//! (`external/TinyEcs/src/TinyEcs.Bevy.Modding/abi/mod-abi.fbs`).
//!
//! `generated.rs` is produced by `build.rs` (planus-cli) and committed so the crate
//! builds without planus-cli on PATH. Re-export the `ModAbi` namespace flat so callers
//! use `abi::CommandBuffer`, `abi::SetupReply`, `abi::SystemInputRef`, etc.

#[allow(warnings, clippy::all)]
#[path = "generated.rs"]
mod generated;

pub use generated::mod_abi::*;
