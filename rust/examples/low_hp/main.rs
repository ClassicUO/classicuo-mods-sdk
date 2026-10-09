//! The docs/modding.md example:
//!   cargo build --example low_hp --target wasm32-wasip2
#![allow(dead_code)]

use cuo_mod_sdk::prelude::*;

fn setup(app: &mut App) {
    app.add_systems(Schedule::Update, low_hp_warning);
    app.add_observer(bandage_hotkey);
    app.add_systems(Schedule::Startup, spawn_window);
    app.add_observer(on_click);
    app.add_packet_observer(PacketDirection::Incoming, &[0x1C], block_spam); // ASCII speech
}

// The parameters ARE the declaration: the client sees the query + `Commands` and passes
// exactly those (`Local` stays in the mod). `#[system]` makes the fn the mod's
// `low-hp-warning` export.
#[system]
fn low_hp_warning(
    player: Query<&Hits, (With<Player>, Changed<Hits>)>,
    mut warned: Local<bool>,
    mut cmds: Commands,
) {
    for (_, hits) in &player {
        let low = hits.value < hits.max_value / 4;
        if low && !*warned {
            cmds.send(ChatMessage::system("Low HP!"));
        }
        *warned = low;
    }
}

// A key bound to "bandage" in the mod's hotkeys: bandage yourself.
#[system]
fn bandage_hotkey(
    on: On<Event, ModHotkeyFired>,
    me: Query<(&Serial, &EquipmentSlotsDto), With<Player>>,
    items: helpers::Items,
) {
    if on.name != "bandage" {
        return;
    }
    let Some((_, (serial, equipment))) = me.single() else { return };
    let backpack = equipment.serials.get(0x15).copied().unwrap_or(0);
    let Some(backpack) = host::resolve_serial(backpack) else { return };
    if !helpers::bandage_self(&items, Entity::from_bits(backpack), serial.value) {
        host::log("no bandages");
    }
}

// A movable window with two lines of text; clicking a line logs it.
#[system]
fn spawn_window(mut cmds: Commands) {
    let root = cmds.spawn((Node::abs(30.0, 120.0, 200.0, 100.0), UiMovable {})).id();
    cmds.spawn((Text { value: "Hello".into() }, ChildOf::new(root)));
    cmds.entity(root).with_child(Text { value: "World".into() });
}

#[system]
fn on_click(click: On<Event, UiClick>, texts: Query<&Text>) {
    if let Some(text) = texts.get(click.entity()) {
        host::log(&format!("clicked {}", text.value));
    }
}

#[system]
fn block_spam(packet: Packet) -> Verdict {
    if packet.windows(4).any(|w| w == b"spam") {
        Verdict::Block
    } else {
        Verdict::Pass
    }
}

export_mod!(setup);
