//! Ergonomic constructors for the generated cuo UI payload types.
//!
//! There is exactly ONE `Node` / `Val` / `UiRect` / `Color` in this SDK: the generated
//! [`crate::types`] structs (enums as numbers, colors 0-255 floats). This module only
//! adds `impl` blocks; `ui::Node` and `types::Node` are the same type.
//!
//! Enum values live with the types they belong to (`types::justify_content::CENTER`,
//! `types::val_type::PX`, `types::interaction::NONE`) — the generator emits them.

pub use crate::types::{BorderRadius, Color, Node, UiRect, Val};

impl Val {
    pub fn auto() -> Val {
        Val { type_: crate::types::val_type::AUTO, value: 0.0, is_auto: true }
    }
    pub fn px(v: f32) -> Val {
        Val { type_: crate::types::val_type::PX, value: v, is_auto: false }
    }
    pub fn percent(v: f32) -> Val {
        Val { type_: crate::types::val_type::PERCENT, value: v, is_auto: false }
    }
    /// Takes the free space left on the parent's main axis (flex-grow).
    pub fn grow() -> Val {
        Val { type_: crate::types::val_type::GROW, value: 0.0, is_auto: false }
    }
}

impl UiRect {
    pub fn splat(v: Val) -> UiRect {
        UiRect { left: v.clone(), right: v.clone(), top: v.clone(), bottom: v }
    }
    pub fn zero() -> UiRect {
        UiRect::splat(Val::px(0.0))
    }
}

impl Node {
    /// The default the mods start from: Flex / Relative / Visible / Row / Start /
    /// Start, every `Val` Auto, padding + border zero (Px 0), gap Px 0.
    pub fn base() -> Node {
        Node {
            width: Val::auto(),
            height: Val::auto(),
            min_width: Val::auto(),
            min_height: Val::auto(),
            max_width: Val::auto(),
            max_height: Val::auto(),
            left: Val::auto(),
            top: Val::auto(),
            right: Val::auto(),
            bottom: Val::auto(),
            padding: UiRect::zero(),
            border: UiRect::zero(),
            gap: Val::px(0.0),
            ..Default::default()
        }
    }

    /// Absolutely positioned, fixed-size node — the shape of every UO gump element.
    pub fn abs(left: f32, top: f32, width: f32, height: f32) -> Node {
        Node {
            position_type: crate::types::position_type::ABSOLUTE,
            left: Val::px(left),
            top: Val::px(top),
            width: Val::px(width),
            height: Val::px(height),
            ..Node::base()
        }
    }
}

impl BorderRadius {
    pub const fn all(r: f32) -> Self {
        Self { top_left: r, top_right: r, bottom_left: r, bottom_right: r }
    }
}

impl Color {
    /// Channels are 0..=255 (the host's Clay convention), not 0..=1.
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: a as f32,
            // IsVisible is `A > 0` on the host (a getter-only property STJ never writes
            // back); filled only so the payload is self-consistent.
            is_visible: a > 0,
        }
    }
    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color::rgba(r, g, b, 255)
    }
}
