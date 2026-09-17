//! Host imports (wasm import module `"cuo"`) + safe wrappers.
//!
//! These are the mid-run RPCs a guest can call synchronously during an export
//! (`mod_run` / `mod_observer` / `mod_filter` / `mod_filter_out`). Everything else rides the
//! ABI buffers. Out-parameter calls (`resolve_cliloc`, `entity_children`,
//! `component_get`, `resource_get`) use a scratch buffer + retry loop: the host returns
//! the needed length (or count); if it exceeds our capacity we recall with a bigger one.
//!
//! Scratch buffers are guest-heap `Vec`s (in linear memory the host can write to) — not
//! the ABI arena, which is reserved for the call in flight.
//!
//! The raw `extern` block is wasm-only; on other targets (host `cargo test`) the wrappers
//! are stubbed so the crate links without the "cuo" module present.

#[cfg(target_family = "wasm")]
mod ffi {
    #[link(wasm_import_module = "cuo")]
    extern "C" {
        pub fn resolve_serial(serial: u32) -> u64;
        pub fn gump_size(id: u32) -> u32;
        pub fn measure_text(font: u32, ptr: u32, len: u32) -> u32;
        pub fn resolve_cliloc(id: u32, out_ptr: u32, cap: u32) -> u32;
        pub fn net_send(ptr: u32, len: u32);
        pub fn log(ptr: u32, len: u32);
        pub fn entity_parent(entity: u64) -> u64;
        pub fn entity_children(entity: u64, out_ptr: u32, cap: u32) -> u32;
        pub fn component_get(entity: u64, type_id: u32, out_ptr: u32, cap: u32) -> u32;
        pub fn resource_get(type_id: u32, out_ptr: u32, cap: u32) -> u32;
    }
}

// Host-target stubs: the wrappers below stay callable so the crate compiles/links off
// wasm, but the host never provides these — invoking one off-wasm is a bug.
#[cfg(not(target_family = "wasm"))]
mod ffi {
    pub unsafe fn resolve_serial(_serial: u32) -> u64 {
        unimplemented!("cuo host import only available under wasm")
    }
    pub unsafe fn gump_size(_id: u32) -> u32 {
        unimplemented!("cuo host import only available under wasm")
    }
    pub unsafe fn measure_text(_font: u32, _ptr: u32, _len: u32) -> u32 {
        unimplemented!("cuo host import only available under wasm")
    }
    pub unsafe fn resolve_cliloc(_id: u32, _out_ptr: u32, _cap: u32) -> u32 {
        unimplemented!("cuo host import only available under wasm")
    }
    pub unsafe fn net_send(_ptr: u32, _len: u32) {
        unimplemented!("cuo host import only available under wasm")
    }
    pub unsafe fn log(_ptr: u32, _len: u32) {
        unimplemented!("cuo host import only available under wasm")
    }
    pub unsafe fn entity_parent(_entity: u64) -> u64 {
        unimplemented!("cuo host import only available under wasm")
    }
    pub unsafe fn entity_children(_entity: u64, _out_ptr: u32, _cap: u32) -> u32 {
        unimplemented!("cuo host import only available under wasm")
    }
    pub unsafe fn component_get(_entity: u64, _type_id: u32, _out_ptr: u32, _cap: u32) -> u32 {
        unimplemented!("cuo host import only available under wasm")
    }
    pub unsafe fn resource_get(_type_id: u32, _out_ptr: u32, _cap: u32) -> u32 {
        unimplemented!("cuo host import only available under wasm")
    }
}

/// Append a diagnostic line to the host log.
pub fn log(msg: &str) {
    let b = msg.as_bytes();
    unsafe { ffi::log(b.as_ptr() as u32, b.len() as u32) }
}

/// Resolve a UO serial to its entity id (0 = not mapped).
pub fn resolve_serial(serial: u32) -> u64 {
    unsafe { ffi::resolve_serial(serial) }
}

/// Gump dimensions, packed `width << 16 | height`.
pub fn gump_size(id: u32) -> u32 {
    unsafe { ffi::gump_size(id) }
}

/// Rendered pixel width of `text` in the given UO font.
pub fn measure_text(font: u32, text: &str) -> u32 {
    let b = text.as_bytes();
    unsafe { ffi::measure_text(font, b.as_ptr() as u32, b.len() as u32) }
}

/// Send raw framed packet bytes to the server (usable from `on_packet`). Bypasses
/// the outgoing filter (`on_packet_out`) — a mod never sees its own sends.
pub fn net_send(bytes: &[u8]) {
    unsafe { ffi::net_send(bytes.as_ptr() as u32, bytes.len() as u32) }
}

/// Parent entity id (0 = none / root).
pub fn entity_parent(entity: u64) -> u64 {
    unsafe { ffi::entity_parent(entity) }
}

/// Child entity ids of `entity`, in order.
pub fn entity_children(entity: u64) -> Vec<u64> {
    let mut cap = 16usize; // count of u64s
    loop {
        let mut buf = vec![0u64; cap];
        let count =
            unsafe { ffi::entity_children(entity, buf.as_mut_ptr() as u32, cap as u32) } as usize;
        if count <= cap {
            buf.truncate(count);
            return buf;
        }
        cap = count;
    }
}

/// Resolve a cliloc id to its string (empty if unknown).
pub fn resolve_cliloc(id: u32) -> String {
    read_string(|ptr, cap| unsafe { ffi::resolve_cliloc(id, ptr, cap) }).unwrap_or_default()
}

/// JSON of a component on `entity`, or `None` when the component is absent.
pub fn component_get_json(entity: u64, type_id: u16) -> Option<String> {
    read_string(|ptr, cap| unsafe { ffi::component_get(entity, type_id as u32, ptr, cap) })
}

/// JSON of a resource, or `None` when absent.
pub fn resource_get_json(type_id: u16) -> Option<String> {
    read_string(|ptr, cap| unsafe { ffi::resource_get(type_id as u32, ptr, cap) })
}

/// A component on `entity` deserialized into `T` (JSON wire), or `None` when absent /
/// malformed. Typed sugar over [`component_get_json`].
pub fn component_get<T: serde::de::DeserializeOwned>(entity: u64, type_id: u16) -> Option<T> {
    serde_json::from_str(&component_get_json(entity, type_id)?).ok()
}

/// A singleton resource deserialized into `T` (JSON wire), or `None` when absent /
/// malformed. Typed sugar over [`resource_get_json`].
pub fn resource_get<T: serde::de::DeserializeOwned>(type_id: u16) -> Option<T> {
    serde_json::from_str(&resource_get_json(type_id)?).ok()
}

/// Shared out-buffer retry loop for the string-returning imports. `call(ptr, cap)`
/// returns the needed byte length; 0 means absent (`None`). Recalls with a bigger buffer
/// when the needed length exceeds `cap`.
fn read_string(mut call: impl FnMut(u32, u32) -> u32) -> Option<String> {
    let mut cap = 256usize;
    loop {
        let mut buf = vec![0u8; cap];
        let needed = call(buf.as_mut_ptr() as u32, cap as u32) as usize;
        if needed == 0 {
            return None;
        }
        if needed <= cap {
            buf.truncate(needed);
            return Some(String::from_utf8_lossy(&buf).into_owned());
        }
        cap = needed;
    }
}
