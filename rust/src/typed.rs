//! The typed path for the curated types (`cuo:modding/components`): query columns,
//! `&mut` write-back, inserts, resource reads, event send / read and observer triggers
//! as WIT records instead of JSON.
//!
//! Mods keep writing the SDK types (`types::Node`, `types::Hits`, ...): the record a
//! value crosses as is a field-for-field copy ([`Conv`]), so author code is the same on
//! both paths, and the types keep serde for what stays JSON (`ResMut`, storage, every
//! non-curated type). A type that is not in the generated tables (typed_gen.rs) — or a
//! mod's own type under a curated path — keeps the JSON path.
//!
//! Dispatch is by `TypeId`: the generic `Query` / `Bundle` / `Res` / `EventReader` /
//! `Commands::send` code asks the generated [`column`] / [`set`] / [`insert`] /
//! [`res_get`] / [`read`] / [`send`] and gets `None` / `false` for a JSON type.

use crate::ecs::Entity;
use std::any::{Any, TypeId};

#[path = "typed_gen.rs"]
mod gen;
pub(crate) use gen::*;

/// An SDK type <-> the WIT record (or field type) it crosses as.
pub(crate) trait Conv<W> {
    fn to_wit(&self) -> W;
    fn from_wit(w: W) -> Self;
}

macro_rules! same_type {
    ($($t:ty),*) => {$(
        impl Conv<$t> for $t {
            fn to_wit(&self) -> $t {
                self.clone()
            }
            fn from_wit(w: $t) -> $t {
                w
            }
        }
    )*};
}
same_type!(bool, u8, i8, u16, i16, u32, i32, u64, i64, f32, f64, String);

impl Conv<u64> for Entity {
    fn to_wit(&self) -> u64 {
        self.to_bits()
    }
    fn from_wit(w: u64) -> Entity {
        Entity::from_bits(w)
    }
}

/// A C# `char`: one-character string in the SDK, a WIT `char`.
impl Conv<char> for String {
    fn to_wit(&self) -> char {
        self.chars().next().unwrap_or('\0')
    }
    fn from_wit(w: char) -> String {
        if w == '\0' {
            String::new()
        } else {
            w.to_string()
        }
    }
}

impl<A: Conv<W>, W> Conv<Vec<W>> for Vec<A> {
    fn to_wit(&self) -> Vec<W> {
        self.iter().map(Conv::to_wit).collect()
    }
    fn from_wit(w: Vec<W>) -> Vec<A> {
        w.into_iter().map(A::from_wit).collect()
    }
}

impl<A: Conv<W>, W> Conv<Option<W>> for Option<A> {
    fn to_wit(&self) -> Option<W> {
        self.as_ref().map(Conv::to_wit)
    }
    fn from_wit(w: Option<W>) -> Option<A> {
        w.map(A::from_wit)
    }
}

/// A curated type an observer can take typed: the record it arrives as.
#[doc(hidden)]
pub trait TypedTrigger: Sized + 'static {
    type Wit;
    fn lift(v: Self::Wit) -> Self;
}

/// A curated tag (zero-size component / event): a typed observer gets no value for it.
#[doc(hidden)]
pub trait TagTrigger: Sized + 'static {
    fn tag() -> Self;
}

/// A typed trigger's value, as the observer's `On<_, T>` takes it (`#[system]` glue).
#[doc(hidden)]
pub fn trigger_value<T: TypedTrigger>(v: T::Wit) -> Box<dyn Any> {
    Box::new(T::lift(v))
}

/// A tag trigger's value (`#[system]` glue).
#[doc(hidden)]
pub fn trigger_tag<T: TagTrigger>() -> Box<dyn Any> {
    Box::new(T::tag())
}

// ── dispatch helpers (used by typed_gen.rs) ──────────────────────────────────────

fn is<T: 'static, U: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<U>()
}

/// `A` as `B` when they are the same type (checked by the caller with [`is`]).
fn cast<A: 'static, B: 'static>(a: A) -> B {
    let mut slot = Some(a);
    (&mut slot as &mut dyn Any).downcast_mut::<Option<B>>().and_then(Option::take).expect("typed cast")
}

fn cast_ref<A: 'static, B: 'static>(a: &A) -> &B {
    (a as &dyn Any).downcast_ref::<B>().expect("typed cast")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p2::bindings::cuo::modding::types as w;
    use crate::types::{position_type, val_type, ChildOf, Color, Node, UiCustomDto, UiMovable, Val};

    fn json<T: serde::Serialize>(v: &T) -> String {
        serde_json::to_string(v).unwrap()
    }

    #[test]
    fn records_round_trip_and_refill_host_computed_members() {
        let mut n = Node::abs(1.0, 2.0, 30.0, 40.0);
        n.display = crate::types::display::NONE;
        n.gap = Val { type_: val_type::GROW, value: 0.0, is_auto: false };
        let rec: w::Node = n.to_wit();
        assert!(matches!(rec.position_type, w::PositionType::Absolute));
        assert!(matches!(rec.display, w::Display::None));
        assert!(matches!(rec.min_width.type_, w::ValType::Auto));
        let back = Node::from_wit(rec);
        assert_eq!(json(&back), json(&n));
        assert_eq!(back.position_type, position_type::ABSOLUTE);
        assert!(back.min_width.is_auto && !back.width.is_auto);

        let c = Color::from_wit(Color::rgba(1, 2, 3, 0).to_wit());
        assert!(!c.is_visible);
        assert!(Color::from_wit(Color::rgba(1, 2, 3, 4).to_wit()).is_visible);

        let parent = Entity::from_bits(0x4000_0000_0000_0007);
        let rec: w::ChildOf = ChildOf::new(parent).to_wit();
        assert_eq!(rec.parent, parent.to_bits());
        assert_eq!(ChildOf::from_wit(rec).parent, parent);
    }

    #[test]
    fn an_out_of_range_enum_number_becomes_the_first_case() {
        let n = Node { position_type: 9, ..Node::base() };
        assert!(matches!(n.to_wit().position_type, w::PositionType::Relative));
    }

    #[test]
    fn the_curated_types_are_typed_and_tags_read_as_defaults() {
        assert!(has_column::<crate::types::Hits>() && has_column::<Node>() && has_column::<ChildOf>());
        assert!(has_column::<UiCustomDto>() && has_column::<UiMovable>());
        assert!(!has_column::<crate::types::MouseInputDto>());
        assert!(matches!(insert(&Node::base()), Some(TypedInsert::Node(_))));
        assert!(matches!(insert(&UiMovable {}), Some(TypedInsert::Movable)));
        // Read-only on the host: no typed insert, JSON as before.
        assert!(insert(&crate::types::Hits::default()).is_none());
        assert_eq!(same(&UiMovable {}, &UiMovable {}), Some(true));
    }

    #[test]
    fn same_compares_the_crossing_record() {
        let a = Node::abs(1.0, 2.0, 3.0, 4.0);
        let mut b = dup(&a).unwrap();
        assert_eq!(same(&a, &b), Some(true));
        // A host-computed member alone is no change: it doesn't cross.
        b.width.is_auto = true;
        assert_eq!(same(&a, &b), Some(true));
        b.width = Val::px(5.0);
        assert_eq!(same(&a, &b), Some(false));
        assert_eq!(same(&crate::types::MouseInputDto::default(), &crate::types::MouseInputDto::default()), None);
    }

    #[test]
    fn typed_triggers_lift_the_record() {
        let v = trigger_value::<ChildOf>(w::ChildOf { parent: 7 });
        assert_eq!(v.downcast_ref::<ChildOf>().unwrap().parent, Entity::from_bits(7));
        assert!(trigger_tag::<crate::types::UiClick>().downcast_ref::<crate::types::UiClick>().is_some());
    }
}
