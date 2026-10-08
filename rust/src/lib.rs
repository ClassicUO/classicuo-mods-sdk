//! cuo-mod-sdk — write ClassicUO mods like Bevy plugins.
//!
//! ```ignore
//! use cuo_mod_sdk::prelude::*;
//!
//! fn setup(app: &mut App) {
//!     app.add_systems(Schedule::Update, low_hp_warning);
//! }
//!
//! fn low_hp_warning(q: Query<&Hits, (With<Player>, Changed<Hits>)>, mut cmds: Commands) {
//!     for (_, hits) in &q {
//!         if hits.value < hits.max_value / 4 {
//!             cmds.send(ChatMessage::system("Low HP!"));
//!         }
//!     }
//! }
//!
//! export_mod!(setup);
//! ```
//!
//! One source, two targets (pick one feature):
//! - `p1` (default) — `wasm32-wasip1` core module: ECS over the FlatBuffers ABI
//!   (abi/mod-abi.fbs), host functions over `env.mod_call` (docs/p1-wire.md).
//! - `p2` — `wasm32-wasip2` component implementing wit/cuo-mod.wit.
//!
//! `host`, `assets`, `actions` and `packets` are the WIT interfaces: wit-bindgen's
//! modules on p2, a generated twin with the same API on p1. To see / block / rewrite
//! packets, add a packet observer: `app.add_packet_observer(dir, ids, f)`.

#[cfg(all(feature = "p1", feature = "p2"))]
compile_error!("enable exactly one of the `p1` / `p2` features");
#[cfg(not(any(feature = "p1", feature = "p2")))]
compile_error!("enable one of the `p1` (wasm32-wasip1) / `p2` (wasm32-wasip2) features");

#[cfg(feature = "p1")]
#[doc(hidden)]
pub mod p1;
#[cfg(feature = "p1")]
pub(crate) use p1 as backend;
#[cfg(feature = "p1")]
pub use p1::cuo::{actions, assets, host, packets};

#[cfg(feature = "p2")]
#[doc(hidden)]
pub mod p2;
#[cfg(feature = "p2")]
pub(crate) use p2 as backend;
#[cfg(feature = "p2")]
pub use p2::{actions, assets, host, packets};

// Every ```rust block of the user guide compiles (`cargo test`).
#[cfg(doctest)]
#[doc = include_str!("../../docs/modding.md")]
mod modding_guide {}

mod ecs;
mod extra;
pub mod helpers;
pub mod paths;
mod sigcheck;
pub mod storage;
pub mod types;
pub mod ui;

pub use ecs::*;

/// `use cuo_mod_sdk::prelude::*;` — the App, the system parameters, every data type,
/// and the host function modules.
pub mod prelude {
    pub use crate::ecs::{
        Add, Added, App, Bundle, Changed, Commands, Component, Entity, Event, EventReader, Local, On, Packet,
        PacketDirection, Query, RawComponent, Remove, Res, ResMut, Schedule, SystemOrder, Verdict, With, Without,
    };
    pub use crate::types::*;
    pub use crate::ui::{Color, Val, UiRect};
    pub use crate::{actions, assets, export_mod, helpers, host, packets, storage};
}

/// Registers the mod: `export_mod!(setup)`, where `setup: fn(&mut App)`.
#[macro_export]
macro_rules! export_mod {
    ($setup:path) => {
        #[no_mangle]
        pub fn __cuo_mod_setup(app: &mut $crate::App) {
            $setup(app)
        }
    };
}
