//! Hand-written conveniences on the generated types (constructors, constants) — the
//! data shapes themselves all come from `tools/mod-typegen` (types.rs).

use crate::ecs::Entity;
use crate::types::{chat_kind, interaction_state, ChatMessage, ChildOf, InputConsume, Interaction};

impl ChildOf {
    pub fn new(parent: Entity) -> ChildOf {
        ChildOf { parent }
    }
}

impl Interaction {
    /// Clickable, idle. Insert this to make a node receive `UiClick` / `UiHover`.
    pub const NONE: Interaction = Interaction { state: interaction_state::NONE };
    pub const HOVERED: Interaction = Interaction { state: interaction_state::HOVERED };
    pub const PRESSED: Interaction = Interaction { state: interaction_state::PRESSED };

    pub fn is_hovered(&self) -> bool {
        self.state == interaction_state::HOVERED
    }
    pub fn is_pressed(&self) -> bool {
        self.state == interaction_state::PRESSED
    }
}

impl InputConsume {
    /// Eat this frame's `button` (a `mouse_button` constant).
    pub fn mouse(button: u8) -> InputConsume {
        InputConsume { mouse: button, key: 0 }
    }
    /// Eat this frame's `key` (a `key_code` constant).
    pub fn key(key: u32) -> InputConsume {
        InputConsume { mouse: 0, key }
    }
}

impl ChatMessage {
    /// A system line in the journal / chat.
    pub fn system(text: impl Into<String>) -> ChatMessage {
        ChatMessage {
            text: text.into(),
            name: "System".into(),
            hue: 0x03B2,
            font: 3,
            is_unicode: true,
            // Overhead would be speech on serial 0.
            kind: chat_kind::SYSTEM,
            ..Default::default()
        }
    }
}
