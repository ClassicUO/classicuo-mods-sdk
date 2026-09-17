//! cuo-mod-sdk — guest-side SDK for out-of-process core-wasm ClassicUO mods.
//!
//! A mod is a `wasm32-wasip1` cdylib that depends on this crate and calls
//! [`export_mod!`] with its setup function. The macro emits the ABI exports
//! (`mod_alloc` / `mod_arena_reset` come from [`arena`]; the six dispatch
//! entrypoints — `mod_setup` / `mod_run` / `mod_observer` / `mod_filter` /
//! `mod_filter_out` / `mod_spawned` — from [`runtime`]) and wires host calls to the
//! callbacks the mod registered during setup.
//!
//! Layers:
//! - [`arena`]   bump allocator serving the host<->guest byte boundary.
//! - [`abi`]     planus flatbuffer types generated from the canonical schema.
//! - [`imports`] safe wrappers over the host `"cuo"` import module.
//! - [`runtime`] the ergonomic layer: [`runtime::ModBuilder`], the [`runtime::SystemInputView`]
//!   reader, and the [`runtime::CommandBufferBuilder`].
//! - [`ui`]      constructors for the generated UI payloads (`ui::Node` IS `types::Node`)
//!   plus the `{"Value": …}` wrapper builders gump mods need.
//! - [`types::actions`] typed game actions: `actions::cast_spell(cmds, 29)` asks the
//!   HOST to cast, so the client's own caches move with the packet. Raw
//!   [`imports::net_send`] is still there for anything not on the action list.

pub mod abi;
pub mod arena;
pub mod imports;
pub mod paths;
pub mod runtime;
pub mod storage;
pub mod types;
pub mod ui;
pub mod wire;

/// Real ecs id of an entity spawned via `CommandBufferBuilder::spawn_named`, once the
/// host has resolved it (see [`runtime::entity`]).
pub use runtime::{entity, forget_entity, ABI_VERSION};

/// Common imports for a mod crate: `use cuo_mod_sdk::prelude::*;`.
pub mod prelude {
    pub use crate::abi::{Encoding, ObserverKind, ParamKind, QueryTermKind, Schedule};
    pub use crate::export_mod;
    pub use crate::imports;
    pub use crate::{paths, storage, types};
    /// Typed game actions (`actions::cast_spell(cmds, 29)`) — the host performs the
    /// action, so client state stays consistent. `imports::net_send` remains for
    /// anything the action list doesn't cover.
    pub use crate::types::actions;
    pub use crate::runtime::{
        entity, forget_entity, Comp, CommandBufferBuilder, CompView, IntoEntityRef, ModBuilder,
        ObserverInputView, ObserverSpec, QueryRowsView, RowView, SystemInputView, SystemSpec,
        TempId, Term,
    };
    pub use crate::ui;
}
