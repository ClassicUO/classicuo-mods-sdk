//! Bump arena for ABI I/O.
//!
//! The host writes call inputs here (via the `mod_alloc` export) and the guest
//! copies finished flatbuffers here for its packed returns (via `pack_ret`). The
//! guest's own heap (planus `Builder`, `Vec`s) is separate — this region only serves
//! the host<->guest byte boundary.
//!
//! Single-threaded: a wasm guest instance has one thread and the host drives the
//! exports serially, so a plain `static mut` needs no locking.

use core::ptr::addr_of_mut;

// flatbuffer roots want 8-byte alignment (covers the u64/i64 fields on the wire).
const ALIGN: usize = 8;

struct Arena {
    buf: Vec<u8>,
    top: usize,
}

static mut ARENA: Arena = Arena {
    buf: Vec::new(),
    top: 0,
};

#[inline]
fn arena() -> &'static mut Arena {
    // SAFETY: single-threaded guest; the host never re-enters an export, so no aliasing
    // `&mut ARENA` is ever live across a call boundary.
    unsafe { &mut *addr_of_mut!(ARENA) }
}

impl Arena {
    fn reset(&mut self) {
        self.top = 0;
    }

    /// Reserve `size` bytes (>=1, 8-aligned start) and return the byte offset into `buf`.
    fn alloc_offset(&mut self, size: usize) -> usize {
        let start = (self.top + ALIGN - 1) & !(ALIGN - 1);
        let end = start + size.max(1);
        if end > self.buf.len() {
            // Grow with headroom. Any pointer previously handed out is dead by the time
            // we grow: the host reads/writes a region only between the mod_alloc that
            // handed it out and the export that consumes it, and the guest builds its
            // return in a separate heap Vec, copying in (pack_ret) only after the input
            // borrow has ended.
            let new_len = end.next_power_of_two().max(64 * 1024);
            self.buf.resize(new_len, 0);
        }
        self.top = end;
        start
    }

    /// Copy `bytes` into a fresh reservation, returning its offset.
    fn write(&mut self, bytes: &[u8]) -> usize {
        let start = self.alloc_offset(bytes.len());
        self.buf[start..start + bytes.len()].copy_from_slice(bytes);
        start
    }

    /// Absolute address in wasm linear memory of a byte offset.
    fn ptr_of(&self, offset: usize) -> u32 {
        (self.buf.as_ptr() as usize + offset) as u32
    }
}

// ── host-facing exports (become the module's `mod_alloc` / `mod_arena_reset`) ──

/// Reserve `size` bytes and return a pointer into linear memory for the host to fill.
#[no_mangle]
pub extern "C" fn mod_alloc(size: u32) -> u32 {
    let a = arena();
    let off = a.alloc_offset(size as usize);
    a.ptr_of(off)
}

/// Rewind the arena, freeing every prior allocation.
#[no_mangle]
pub extern "C" fn mod_arena_reset() {
    arena().reset();
}

// ── guest-side helpers ────────────────────────────────────────────────────────────

/// Copy `bytes` into the arena and return the pointer to the copy.
pub fn alloc_bytes(bytes: &[u8]) -> u32 {
    let a = arena();
    let off = a.write(bytes);
    a.ptr_of(off)
}

/// Copy a finished flatbuffer into the arena, returning the ABI's `len << 32 | ptr`.
pub fn pack_ret(bytes: &[u8]) -> u64 {
    let ptr = alloc_bytes(bytes) as u64;
    ((bytes.len() as u64) << 32) | ptr
}

/// Reconstruct the input slice the host wrote at `(ptr, len)`.
///
/// # Safety
/// `(ptr, len)` must be a region previously handed out by `mod_alloc` and filled by
/// the host for the current call.
pub unsafe fn input_slice<'a>(ptr: u32, len: u32) -> &'a [u8] {
    core::slice::from_raw_parts(ptr as *const u8, len as usize)
}

#[cfg(test)]
mod tests {
    use super::Arena;

    fn fresh() -> Arena {
        Arena {
            buf: Vec::new(),
            top: 0,
        }
    }

    #[test]
    fn allocations_are_8_aligned_and_non_overlapping() {
        let mut a = fresh();
        let o1 = a.alloc_offset(3);
        let o2 = a.alloc_offset(5);
        let o3 = a.alloc_offset(1);
        assert_eq!(o1 % 8, 0);
        assert_eq!(o2 % 8, 0);
        assert_eq!(o3 % 8, 0);
        assert!(o2 >= o1 + 3);
        assert!(o3 >= o2 + 5);
    }

    #[test]
    fn write_then_read_round_trips() {
        let mut a = fresh();
        let bytes = [0u8, 1, 0xFF, 0x7F, 0x80, 0xFE];
        let off = a.write(&bytes);
        assert_eq!(&a.buf[off..off + bytes.len()], &bytes);
    }

    #[test]
    fn reset_rewinds_and_reuses_space() {
        let mut a = fresh();
        let first = a.write(&[9u8; 40]);
        a.reset();
        let second = a.write(&[7u8; 40]);
        assert_eq!(first, second, "reset should rewind the bump pointer");
    }

    #[test]
    fn grows_past_initial_capacity() {
        let mut a = fresh();
        // Force several growths; every returned region must stay writable.
        let mut last_end = 0;
        for i in 0..8 {
            let n = 50_000 * (i + 1);
            let off = a.alloc_offset(n);
            assert!(off >= last_end);
            assert!(a.buf.len() >= off + n);
            last_end = off + n;
        }
    }
}
