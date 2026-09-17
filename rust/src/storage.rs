//! Per-mod persistent storage over the host `"cuo"` imports `storage_get` /
//! `storage_set`.
//!
//! One opaque UTF-8 blob per mod, persisted by the host at
//! `<client>/Data/Mods/<modname>/storage.json`. Writes are write-through (no
//! debounce) — call [`set`] on a user action, not every frame.
//!
//! The blob is only reachable once the host has registered this mod, which happens
//! AFTER `mod_setup` returns: read your settings from a `mod-startup` (or `Update`)
//! system, not from the setup callback — [`get`] returns `""` if called too early.
//!
//! ```ignore
//! #[derive(serde::Serialize, serde::Deserialize, Default)]
//! struct Settings { x: i32, y: i32 }
//!
//! let mut s: Settings = storage::get_as().unwrap_or_default();
//! s.x += 1;
//! storage::set_value(&s);
//! ```

use serde::de::DeserializeOwned;
use serde::Serialize;

#[cfg(target_family = "wasm")]
mod ffi {
    #[link(wasm_import_module = "cuo")]
    extern "C" {
        pub fn storage_get(arg: u32, out_ptr: u32, cap: u32) -> u32;
        pub fn storage_set(ptr: u32, len: u32);
    }
}

// Host-target stubs so the crate still links off wasm (`cargo test`).
#[cfg(not(target_family = "wasm"))]
mod ffi {
    pub unsafe fn storage_get(_arg: u32, _out_ptr: u32, _cap: u32) -> u32 {
        unimplemented!("cuo host import only available under wasm")
    }
    pub unsafe fn storage_set(_ptr: u32, _len: u32) {
        unimplemented!("cuo host import only available under wasm")
    }
}

/// The stored blob, or `""` when the mod has never written one (or storage isn't
/// available yet — see the module docs).
pub fn get() -> String {
    // Same needed-length retry protocol as `imports::resolve_cliloc`: the host
    // returns the UTF-8 byte length and only fills the buffer when it fits.
    let mut cap = 512usize;
    loop {
        let mut buf = vec![0u8; cap];
        let needed = unsafe { ffi::storage_get(0, buf.as_mut_ptr() as u32, cap as u32) } as usize;
        if needed == 0 {
            return String::new();
        }
        if needed <= cap {
            buf.truncate(needed);
            return String::from_utf8_lossy(&buf).into_owned();
        }
        cap = needed;
    }
}

/// Replace the stored blob. Persisted immediately.
pub fn set(json: &str) {
    let b = json.as_bytes();
    unsafe { ffi::storage_set(b.as_ptr() as u32, b.len() as u32) }
}

/// [`get`] parsed as `T`. `None` when nothing is stored or the blob doesn't
/// deserialize (a schema change in the mod, a hand-edited file).
pub fn get_as<T: DeserializeOwned>() -> Option<T> {
    let raw = get();
    if raw.is_empty() {
        return None;
    }
    serde_json::from_str(&raw).ok()
}

/// Serialize `value` as JSON and [`set`] it. Silently does nothing if `value`
/// can't be serialized.
pub fn set_value<T: Serialize>(value: &T) {
    if let Ok(json) = serde_json::to_string(value) {
        set(&json);
    }
}
