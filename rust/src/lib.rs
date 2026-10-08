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
//! Build for `wasm32-wasip2`: the mod is a component implementing wit/cuo-mod.wit.
//!
//! `host`, `assets`, `actions` and `packets` are the WIT interfaces (wit-bindgen's
//! modules). To see / block / rewrite packets, add a packet observer:
//! `app.add_packet_observer(dir, ids, f)`.

// The exports exist only on wasm: a host build (`cargo test`) has no mod to export.
#![cfg_attr(not(target_family = "wasm"), allow(dead_code, unused_imports))]

#[doc(hidden)]
pub mod p2;
pub(crate) use p2 as backend;
pub use p2::{actions, assets, host, packets};

// Every ```rust block of the user guide compiles (`cargo test`).
#[cfg(doctest)]
#[doc = include_str!("../../docs/modding.md")]
mod modding_guide {}

mod ecs;
mod extra;
pub mod helpers;
pub mod paths;
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
