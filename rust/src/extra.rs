//! Hand-written conveniences on the generated types (constructors, constants) — the
//! data shapes themselves all come from `tools/mod-typegen` (types.rs).

use crate::ecs::Entity;
use crate::types::{ChatMessage, ChildOf, InputConsume, Interaction};

impl ChildOf {
    pub fn new(parent: Entity) -> ChildOf {
        ChildOf { parent }
    }
}

impl Interaction {
    /// Clickable, idle. Insert this to make a node receive `UiClick` / `UiHover`.
    pub const NONE: Interaction = Interaction { state: 0 };
    pub const HOVERED: Interaction = Interaction { state: 1 };
    pub const PRESSED: Interaction = Interaction { state: 2 };

    pub fn is_hovered(&self) -> bool {
        self.state == 1
    }
    pub fn is_pressed(&self) -> bool {
        self.state == 2
    }
}

impl InputConsume {
    /// Eat this frame's `button` (`MouseButtonType`; 1 left, 2 middle, 3 right, ...).
    pub fn mouse(button: u8) -> InputConsume {
        InputConsume { mouse: button, key: 0 }
    }
    /// Eat this frame's `key` (`KeyCode`).
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
            ..Default::default()
        }
    }
}
