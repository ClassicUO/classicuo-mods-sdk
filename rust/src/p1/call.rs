//! `env.mod_call` + the JSON encoding of WIT values (docs/p1-wire.md). The generated
//! `cuo.rs` builds on [`ToWit`] / [`FromWit`] and [`call`].

pub use serde_json::{Map, Value};

/// A WIT value as JSON.
pub trait ToWit {
    fn to_wit(&self) -> Value;

    /// How a `list<Self>` encodes: an array, except `list<u8>` (base64).
    fn list_to_wit(items: &[Self]) -> Value
    where
        Self: Sized,
    {
        Value::Array(items.iter().map(ToWit::to_wit).collect())
    }
}

/// A WIT value from JSON; `None` when the JSON has the wrong shape.
pub trait FromWit: Sized {
    fn from_wit(v: &Value) -> Option<Self>;

    fn list_from_wit(v: &Value) -> Option<Vec<Self>> {
        v.as_array()?.iter().map(Self::from_wit).collect()
    }
}

macro_rules! number {
    ($($t:ty => $get:ident),*) => {$(
        impl ToWit for $t {
            fn to_wit(&self) -> Value { Value::from(*self) }
        }
        impl FromWit for $t {
            fn from_wit(v: &Value) -> Option<Self> { <$t>::try_from(v.$get()?).ok() }
        }
    )*};
}
number!(u16 => as_u64, u32 => as_u64, i8 => as_i64, i16 => as_i64, i32 => as_i64);

impl ToWit for u8 {
    fn to_wit(&self) -> Value {
        Value::from(*self)
    }
    fn list_to_wit(items: &[u8]) -> Value {
        Value::String(base64_encode(items))
    }
}
impl FromWit for u8 {
    fn from_wit(v: &Value) -> Option<Self> {
        u8::try_from(v.as_u64()?).ok()
    }
    fn list_from_wit(v: &Value) -> Option<Vec<u8>> {
        base64_decode(v.as_str()?)
    }
}

// 64-bit integers cross as decimal strings (JSON numbers lose precision past 2^53).
impl ToWit for u64 {
    fn to_wit(&self) -> Value {
        Value::String(self.to_string())
    }
}
impl FromWit for u64 {
    fn from_wit(v: &Value) -> Option<Self> {
        v.as_str()?.parse().ok()
    }
}
impl ToWit for i64 {
    fn to_wit(&self) -> Value {
        Value::String(self.to_string())
    }
}
impl FromWit for i64 {
    fn from_wit(v: &Value) -> Option<Self> {
        v.as_str()?.parse().ok()
    }
}

impl ToWit for f32 {
    fn to_wit(&self) -> Value {
        Value::from(*self)
    }
}
impl FromWit for f32 {
    fn from_wit(v: &Value) -> Option<Self> {
        Some(v.as_f64()? as f32)
    }
}
impl ToWit for f64 {
    fn to_wit(&self) -> Value {
        Value::from(*self)
    }
}
impl FromWit for f64 {
    fn from_wit(v: &Value) -> Option<Self> {
        v.as_f64()
    }
}
impl ToWit for bool {
    fn to_wit(&self) -> Value {
        Value::Bool(*self)
    }
}
impl FromWit for bool {
    fn from_wit(v: &Value) -> Option<Self> {
        v.as_bool()
    }
}
impl ToWit for char {
    fn to_wit(&self) -> Value {
        Value::String(self.to_string())
    }
}
impl FromWit for char {
    fn from_wit(v: &Value) -> Option<Self> {
        let mut c = v.as_str()?.chars();
        let first = c.next()?;
        c.next().is_none().then_some(first)
    }
}
impl ToWit for str {
    fn to_wit(&self) -> Value {
        Value::String(self.into())
    }
}
impl ToWit for String {
    fn to_wit(&self) -> Value {
        Value::String(self.clone())
    }
}
impl FromWit for String {
    fn from_wit(v: &Value) -> Option<Self> {
        Some(v.as_str()?.to_owned())
    }
}
impl<T: ToWit + ?Sized> ToWit for &T {
    fn to_wit(&self) -> Value {
        (**self).to_wit()
    }
}
impl<T: ToWit> ToWit for [T] {
    fn to_wit(&self) -> Value {
        T::list_to_wit(self)
    }
}
impl<T: ToWit> ToWit for Vec<T> {
    fn to_wit(&self) -> Value {
        T::list_to_wit(self)
    }
}
impl<T: FromWit> FromWit for Vec<T> {
    fn from_wit(v: &Value) -> Option<Self> {
        T::list_from_wit(v)
    }
}
impl<T: ToWit> ToWit for Option<T> {
    fn to_wit(&self) -> Value {
        match self {
            Some(v) => v.to_wit(),
            None => Value::Null,
        }
    }
}
impl<T: FromWit> FromWit for Option<T> {
    fn from_wit(v: &Value) -> Option<Self> {
        if v.is_null() {
            Some(None)
        } else {
            T::from_wit(v).map(Some)
        }
    }
}
impl FromWit for () {
    fn from_wit(_: &Value) -> Option<Self> {
        Some(())
    }
}

macro_rules! tuple {
    ($($n:tt $t:ident),*) => {
        impl<$($t: ToWit),*> ToWit for ($($t,)*) {
            fn to_wit(&self) -> Value { Value::Array(vec![$(self.$n.to_wit()),*]) }
        }
        impl<$($t: FromWit),*> FromWit for ($($t,)*) {
            fn from_wit(v: &Value) -> Option<Self> {
                let a = v.as_array()?;
                if a.len() != [$($n),*].len() { return None; }
                Some(($($t::from_wit(&a[$n])?,)*))
            }
        }
    };
}
tuple!(0 A);
tuple!(0 A, 1 B);
tuple!(0 A, 1 B, 2 C);
tuple!(0 A, 1 B, 2 C, 3 D);

/// A record field; a missing key reads as `null` (an absent `option`).
pub fn field<'a>(v: &'a Value, name: &str) -> &'a Value {
    v.get(name).unwrap_or(&Value::Null)
}

/// `{"<case>": payload}`.
pub fn variant(case: &str, payload: Value) -> Value {
    let mut o = Map::new();
    o.insert(case.into(), payload);
    Value::Object(o)
}

/// The single `(case, payload)` of a variant object.
pub fn case(v: &Value) -> Option<(&str, &Value)> {
    let o = v.as_object()?;
    if o.len() != 1 {
        return None;
    }
    o.iter().next().map(|(k, v)| (k.as_str(), v))
}

#[cfg(target_family = "wasm")]
mod ffi {
    #[link(wasm_import_module = "env")]
    extern "C" {
        pub fn mod_call(name_ptr: i32, name_len: i32, args_ptr: i32, args_len: i32) -> i64;
    }
}

#[cfg(not(target_family = "wasm"))]
mod ffi {
    pub unsafe fn mod_call(_: i32, _: i32, _: i32, _: i32) -> i64 {
        unimplemented!("env.mod_call only exists inside the client")
    }
}

/// Calls `name` ("cuo:modding/assets#image-size") with `args` in WIT order and decodes the
/// result. Traps on a malformed result: that is a host bug, not a condition to handle.
pub fn call<R: FromWit>(name: &str, args: Vec<Value>) -> R {
    let args = Value::Array(args).to_string();
    let packed = unsafe {
        ffi::mod_call(
            name.as_ptr() as i32,
            name.len() as i32,
            args.as_ptr() as i32,
            args.len() as i32,
        )
    } as u64;
    let result = if packed == 0 {
        Value::Null
    } else {
        // The host wrote the result into the arena; parse it before anything else can
        // grow (and move) the arena.
        let bytes = unsafe { super::arena::input_slice(packed as u32, (packed >> 32) as u32) };
        serde_json::from_slice(bytes).unwrap_or_else(|e| panic!("{name}: bad result JSON: {e}"))
    };
    R::from_wit(&result).unwrap_or_else(|| panic!("{name}: unexpected result {result}"))
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16
            | (*chunk.get(1).unwrap_or(&0) as u32) << 8
            | *chunk.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(B64[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

pub fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.trim_end_matches('=').as_bytes();
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for &c in s {
        let v = B64.iter().position(|&b| b == c)? as u32;
        acc = acc << 6 | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{actions, assets, host, packets};
    use serde_json::json;

    #[test]
    fn record_round_trips_with_kebab_fields_and_u64_strings() {
        let info = assets::StaticInfo {
            name: "Bandage".into(),
            flags: 0x8000_0000_0000_0001,
            weight: 1,
            layer: 0,
            count: -1,
            anim_id: 7,
            hue: 0,
            light_index: 0,
            height: 1,
        };
        let v = info.to_wit();
        assert_eq!(v["flags"], json!("9223372036854775809"));
        assert_eq!(v["anim-id"], json!(7));
        assert_eq!(v["light-index"], json!(0));
        let back = assets::StaticInfo::from_wit(&v).unwrap();
        assert_eq!(back.flags, info.flags);
        assert_eq!(back.name, "Bandage");
        assert_eq!(back.count, -1);
    }

    #[test]
    fn enum_is_kebab_case() {
        assert_eq!(actions::Window::WorldMap.to_wit(), json!("world-map"));
        assert_eq!(actions::Direction::from_wit(&json!("north-east")), Some(actions::Direction::NorthEast));
        assert_eq!(host::Scope::from_wit(&json!("nope")), None);
    }

    #[test]
    fn variant_cases_carry_payload_or_null() {
        let d = actions::Destination::ContainerAt((0x4000_0001, 10, 20));
        assert_eq!(d.to_wit(), json!({"container-at": [1073741825, 10, 20]}));
        let g = actions::Destination::Ground(actions::Point { x: 1, y: 2, z: -3 });
        assert_eq!(g.to_wit(), json!({"ground": {"x": 1, "y": 2, "z": -3}}));
        assert!(matches!(packets::Verdict::from_wit(&json!({"pass": null})), Some(packets::Verdict::Pass)));
        match packets::Verdict::from_wit(&json!({"replace": "AQID"})) {
            Some(packets::Verdict::Replace(b)) => assert_eq!(b, vec![1, 2, 3]),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn option_is_null_or_value() {
        assert_eq!(Option::<u16>::None.to_wit(), Value::Null);
        assert_eq!(Some(5u16).to_wit(), json!(5));
        assert_eq!(Option::<u64>::from_wit(&json!("42")), Some(Some(42)));
        assert_eq!(Option::<u64>::from_wit(&Value::Null), Some(None));
        let size = Option::<assets::Size>::from_wit(&json!({"width": 44, "height": 44})).unwrap();
        assert_eq!(size.map(|s| s.width), Some(44));
    }

    #[test]
    fn list_u8_is_base64_other_lists_are_arrays() {
        assert_eq!([0x1Cu8, 0xAE].to_wit(), json!("HK4="));
        assert_eq!(vec![1u32, 2].to_wit(), json!([1, 2]));
        assert_eq!(Vec::<u8>::from_wit(&json!("HK4=")), Some(vec![0x1C, 0xAE]));
        let texts = [(1u16, String::from("a"))];
        assert_eq!(texts[..].to_wit(), json!([[1, "a"]]));
        for n in 0..8 {
            let bytes: Vec<u8> = (0..n).map(|i| (i * 37) as u8).collect();
            assert_eq!(base64_decode(&base64_encode(&bytes)), Some(bytes));
        }
    }

    #[test]
    fn s64_and_u64_are_decimal_strings() {
        assert_eq!(u64::MAX.to_wit(), json!("18446744073709551615"));
        assert_eq!(i64::MIN.to_wit(), json!("-9223372036854775808"));
        assert_eq!(u64::from_wit(&json!(5)), None);
    }
}
