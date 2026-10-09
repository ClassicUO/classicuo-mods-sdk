//! cuo-mod-sdk — write ClassicUO mods like Bevy plugins.
//!
//! ```ignore
//! use cuo_mod_sdk::prelude::*;
//!
//! fn setup(app: &mut App) {
//!     app.add_systems(Schedule::Update, low_hp_warning);
//! }
//!
//! #[system]
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
//! Build for `wasm32-wasip2`: the mod is a component of its own world — `setup`
//! (wit/cuo-mod.wit) plus one export per system. `#[system]` makes a fn that export,
//! named like the fn in kebab-case (`low_hp_warning` -> `low-hp-warning`), taking the
//! fn's wire parameters (`Commands` -> `commands`, `Query` -> `query`, `Res` / `ResMut`
//! -> `res`, `EventReader` -> `events`; `Local` stays in the mod). Mark every fn you
//! add as a system or observer; one that isn't fails the mod's load ("system 'x' is
//! declared in setup but the mod exports no 'x' function"). The same fn added twice
//! (two schedules / triggers) is one export and shares its `Local`s; two fns with the
//! same name, or a closure, panic at setup.
//!
//! `host`, `assets`, `actions` and `packets` are the WIT interfaces (wit-bindgen's
//! modules). To see / block / rewrite packets, add a packet observer:
//! `app.add_packet_observer(dir, ids, f)`.

// The exports exist only on wasm: a host build (`cargo test`) has no mod to export.
#![cfg_attr(not(target_family = "wasm"), allow(dead_code, unused_imports))]

// `#[system]` expands to `::cuo_mod_sdk::...` paths, also inside this crate.
extern crate self as cuo_mod_sdk;

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

/// Marks a system / observer fn: it becomes the mod's export of the same name in
/// kebab-case (`snap_input` -> `snap-input`). See the crate docs.
pub use cuo_mod_sdk_macros::system;

/// What `#[system]` expands to refers to; not API.
#[doc(hidden)]
pub mod __private {
    pub use crate::ecs::{wire_is, Wire};
    pub use crate::p2::bindings::tinyecs::modding::ecs;
    pub use crate::p2::{run_observer, run_packet_observer, run_system, Param};
    pub use wit_bindgen;
}

/// `use cuo_mod_sdk::prelude::*;` — the App, the system parameters, every data type,
/// and the host function modules.
pub mod prelude {
    pub use crate::ecs::{
        Add, Added, App, Bundle, Changed, Commands, Component, Entity, Event, EventReader, Local, On, Packet,
        PacketDirection, Query, RawComponent, Remove, Res, ResMut, Schedule, SystemOrder, Verdict, With, Without,
    };
    pub use crate::system;
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
