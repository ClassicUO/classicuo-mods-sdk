//! Ergonomic constructors for the generated cuo UI payload types.
//!
//! There is exactly ONE `Node` / `Val` / `UiRect` / `Color` in this SDK: the generated
//! [`crate::types`] structs, which mirror what the host writes through
//! System.Text.Json (PascalCase fields, enums as numbers, `Val` as `{Type,Value}`,
//! colors as `{R,G,B,A}` 0-255 floats). This module only adds `impl` blocks + the
//! `{"Value": …}` wrapper helpers; `ui::Node` and `types::Node` are the same type.
//!
//! Enum values live with the types they belong to (`types::justify_content::CENTER`,
//! `types::val_type::PX`, `types::interaction::NONE`) — the generator emits them.

use serde::Serialize;

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

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Node serialize")
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

#[derive(Serialize)]
struct Wrapped<T: Serialize> {
    #[serde(rename = "Value")]
    value: T,
}

/// `{"Value": <color>}` — for `cuo:ui/bg-color` and `cuo:ui/text-color`.
pub fn color_value_json(c: Color) -> String {
    serde_json::to_string(&Wrapped { value: c }).expect("color value serialize")
}

/// `{"Value": <string>}` — for `cuo:ui/text` and `cuo:ui/name`.
pub fn value_str_json(s: &str) -> String {
    serde_json::to_string(&Wrapped { value: s }).expect("value str serialize")
}

/// `{"Value": <int>}` — for `cuo:ui/global-z`.
pub fn value_i32_json(v: i32) -> String {
    serde_json::to_string(&Wrapped { value: v }).expect("value i32 serialize")
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct TextFontRec {
    font_id: u16,
    size: u16,
}

/// `{"FontId":..,"Size":..}` — for `cuo:ui/text-font`.
pub fn text_font_json(font_id: u16, size: u16) -> String {
    serde_json::to_string(&TextFontRec { font_id, size }).expect("text font serialize")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_json_has_pascal_case_fields() {
        let json = Node::abs(30.0, 120.0, 440.0, 300.0).to_json();
        assert!(json.contains("\"PositionType\":1"));
        assert!(json.contains("\"Left\":{\"Type\":1,\"Value\":30"));
        assert!(json.contains("\"Width\":{\"Type\":1,\"Value\":440"));
        // base defaults still present.
        assert!(json.contains("\"Display\":0"));
        assert!(json.contains("\"Gap\":{\"Type\":1,\"Value\":0"));
    }

    /// The generated `Node` must stay wire-compatible with the hand-written `ui::Node`
    /// it replaced. The only additions are the host-COMPUTED members STJ emits and
    /// ignores on the way back in (`Val.IsAuto`, `Color.IsVisible`); strip those and
    /// the bytes must be identical.
    #[test]
    fn node_json_matches_the_pre_merge_hand_written_shape() {
        const LEGACY: &str = concat!(
            r#"{"Display":0,"PositionType":1,"Overflow":0,"FlexDirection":0,"#,
            r#""JustifyContent":0,"AlignItems":0,"#,
            r#""Width":{"Type":1,"Value":3.0},"Height":{"Type":1,"Value":4.0},"#,
            r#""MinWidth":{"Type":0,"Value":0.0},"MinHeight":{"Type":0,"Value":0.0},"#,
            r#""MaxWidth":{"Type":0,"Value":0.0},"MaxHeight":{"Type":0,"Value":0.0},"#,
            r#""Left":{"Type":1,"Value":1.0},"Top":{"Type":1,"Value":2.0},"#,
            r#""Right":{"Type":0,"Value":0.0},"Bottom":{"Type":0,"Value":0.0},"#,
            r#""Padding":{"Left":{"Type":1,"Value":0.0},"Right":{"Type":1,"Value":0.0},"#,
            r#""Top":{"Type":1,"Value":0.0},"Bottom":{"Type":1,"Value":0.0}},"#,
            r#""Border":{"Left":{"Type":1,"Value":0.0},"Right":{"Type":1,"Value":0.0},"#,
            r#""Top":{"Type":1,"Value":0.0},"Bottom":{"Type":1,"Value":0.0}},"#,
            r#""Gap":{"Type":1,"Value":0.0},"AspectRatio":0.0}"#,
        );
        let json = Node::abs(1.0, 2.0, 3.0, 4.0).to_json();
        let stripped = json
            .replace(",\"IsAuto\":true", "")
            .replace(",\"IsAuto\":false", "");
        assert_eq!(stripped, LEGACY);
    }

    #[test]
    fn color_value_wraps() {
        assert_eq!(
            color_value_json(Color::rgba(40, 90, 160, 255)),
            "{\"Value\":{\"R\":40.0,\"G\":90.0,\"B\":160.0,\"A\":255.0,\"IsVisible\":true}}"
        );
    }

    #[test]
    fn text_font_shape() {
        assert_eq!(text_font_json(1, 13), "{\"FontId\":1,\"Size\":13}");
    }
}
