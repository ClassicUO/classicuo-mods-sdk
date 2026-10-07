//! Common combinations of `actions`, for the queries a system already has.

use crate::actions::{self, Destination, Serial};
use crate::ecs::{Entity, Query, QueryFilter};
use crate::types::{ChildOf, Serial as SerialC};
use crate::types::{Graphic, Player};
use crate::With;

/// Bandage graphic.
pub const BANDAGE: u16 = 0x0E21;

/// Items with their serial, graphic and container — what the helpers below search.
pub type Items<F = ()> = Query<(&'static SerialC, &'static Graphic, &'static ChildOf), F>;

/// The first item of `graphic` directly inside `container`.
pub fn find_type<F: QueryFilter>(items: &Items<F>, graphic: u16, container: Entity) -> Option<Serial> {
    items
        .iter()
        .find(|(_, (_, g, c))| g.value == graphic && c.parent == container)
        .map(|(_, (s, _, _))| s.value)
}

/// Moves every item of `graphic` directly inside `from` into container `to`.
/// Returns how many moves were sent.
pub fn move_type<F: QueryFilter>(items: &Items<F>, graphic: u16, from: Entity, to: Serial) -> usize {
    let mut n = 0;
    for (_, (s, g, c)) in items {
        if g.value == graphic && c.parent == from {
            actions::move_item(s.value, 0, Destination::Container(to));
            n += 1;
        }
    }
    n
}

/// Double-clicks the first item of `graphic` inside `container`; false when none.
pub fn use_type<F: QueryFilter>(items: &Items<F>, graphic: u16, container: Entity) -> bool {
    match find_type(items, graphic, container) {
        Some(s) => {
            actions::double_click(s);
            true
        }
        None => false,
    }
}

/// Answers the open (or next) target cursor with the player.
pub fn target_self(player: &Query<&SerialC, With<Player>>) {
    if let Some((_, s)) = player.single() {
        actions::target_object(s.value);
    }
}

/// Uses a bandage from `backpack` on `me` (the player's serial); false when there is
/// none.
pub fn bandage_self<F: QueryFilter>(items: &Items<F>, backpack: Entity, me: Serial) -> bool {
    match find_type(items, BANDAGE, backpack) {
        Some(bandage) => {
            actions::use_item_on(bandage, me);
            true
        }
        None => false,
    }
}

/// Flips war mode; `war` is the current state (`PlayerData` / your own tracking).
pub fn toggle_war_mode(war: bool) {
    actions::set_war_mode(!war);
}
