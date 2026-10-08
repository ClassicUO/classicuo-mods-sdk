//! Minimal absolute-offset FlatBuffer reader for the host-written input buffers
//! (Handshake / SystemInput / ObserverInput / SpawnedInput).
//!
//! WHY THIS EXISTS: the host serializes with FlatSharp, the guest reads. planus's
//! generated readers advance the buffer slice for every nested object and resolve a
//! table's vtable soffset RELATIVE to that advanced slice — so a vtable positioned
//! BEFORE a table (which FlatSharp emits when it deduplicates a nested table's vtable
//! against an ancestor's, pooling it near the front) is at a negative local offset and
//! reads as absent (`InvalidOffset`). FlatSharp is a spec-correct writer; the fault is
//! planus's forward-only slice model. This reader keeps the WHOLE buffer and resolves
//! every offset absolutely, so shared / backward-pointing vtables read correctly. It is
//! used only for the read direction; the return direction (CommandBuffer / SetupReply)
//! still builds via planus, which FlatSharp reads without issue.
//!
//! FlatBuffer layout facts this relies on:
//! - table starts with an `i32` soffset to its vtable: `vtable = table_pos - soffset`.
//! - vtable = `u16 vtable_size, u16 table_size, u16 field_slot[..]`; slot for field `i`
//!   is at `vtable + 4 + 2*i`; a slot value of 0 (or past `vtable_size`) = field absent
//!   (reader substitutes the scalar default / an empty ref).
//! - a scalar field slot points at the inline value (`table_pos + slot`).
//! - a table/vector/string field slot points at a `u32` uoffset; the object is at
//!   `slot_pos + uoffset`.
//! - a vector is `u32 len` then `len` elements; a table-element vector stores a `u32`
//!   uoffset per element (`elem_pos + uoffset` = element table).
//! - a `[ubyte]`/string is `u32 len` then the bytes.
//!
//! All reads are bounds-checked; a malformed/truncated buffer yields defaults / `None`
//! rather than panicking (the host is trusted, but the guest must never trap on input).

/// A borrowed FlatBuffer with absolute-offset accessors.
#[derive(Clone, Copy)]
pub struct Buf<'a> {
    data: &'a [u8],
}

impl<'a> Buf<'a> {
    pub fn new(data: &'a [u8]) -> Buf<'a> {
        Buf { data }
    }

    fn u16(&self, p: usize) -> Option<u16> {
        self.data.get(p..p + 2).map(|b| u16::from_le_bytes([b[0], b[1]]))
    }
    fn u32(&self, p: usize) -> Option<u32> {
        self.data
            .get(p..p + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn i32(&self, p: usize) -> Option<i32> {
        self.u32(p).map(|v| v as i32)
    }
    fn u64(&self, p: usize) -> Option<u64> {
        self.data.get(p..p + 8).map(|b| {
            u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
        })
    }

    /// The root table position (`root_uoffset` at buffer start).
    pub fn root(&self) -> Option<usize> {
        self.u32(0).map(|v| v as usize)
    }

    /// Byte position of field `index`'s data within `table`, or `None` if the field is
    /// absent (vtable slot 0 or past the vtable). Resolves the vtable absolutely, so a
    /// vtable before the table (shared/pooled) still works.
    fn field(&self, table: usize, index: usize) -> Option<usize> {
        let soffset = self.i32(table)? as i64;
        let vtable = (table as i64).checked_sub(soffset)?;
        if vtable < 0 {
            return None;
        }
        let vtable = vtable as usize;
        let vtable_size = self.u16(vtable)? as usize;
        let slot = 4 + 2 * index;
        if slot + 2 > vtable_size {
            return None;
        }
        let field_off = self.u16(vtable + slot)? as usize;
        if field_off == 0 {
            return None;
        }
        Some(table + field_off)
    }

    fn field_u16(&self, table: usize, index: usize) -> u16 {
        self.field(table, index).and_then(|p| self.u16(p)).unwrap_or(0)
    }
    fn field_u32(&self, table: usize, index: usize) -> u32 {
        self.field(table, index).and_then(|p| self.u32(p)).unwrap_or(0)
    }
    fn field_u64(&self, table: usize, index: usize) -> u64 {
        self.field(table, index).and_then(|p| self.u64(p)).unwrap_or(0)
    }

    /// Absolute position of the object referenced by a table/vector/string field.
    fn field_ref(&self, table: usize, index: usize) -> Option<usize> {
        let slot = self.field(table, index)?;
        let rel = self.u32(slot)? as usize;
        Some(slot + rel)
    }

    /// Number of elements in the vector at `vec_pos`.
    fn vector_len(&self, vec_pos: usize) -> usize {
        self.u32(vec_pos).unwrap_or(0) as usize
    }

    /// Table position of element `i` in a table-element (offset) vector at `vec_pos`.
    fn vector_table(&self, vec_pos: usize, i: usize) -> Option<usize> {
        let elem = vec_pos + 4 + 4 * i;
        let rel = self.u32(elem)? as usize;
        Some(elem + rel)
    }

    /// Byte slice of a `[ubyte]`/string object at `obj_pos` (`u32 len` + bytes).
    fn bytes_at(&self, obj_pos: usize) -> &'a [u8] {
        let Some(len) = self.u32(obj_pos) else {
            return &[];
        };
        self.data
            .get(obj_pos + 4..obj_pos + 4 + len as usize)
            .unwrap_or(&[])
    }
}

// ── run inputs ──────────────────────────────────────────────────────────────────
// Field indices mirror abi/mod-abi.fbs declaration order.

/// Where a root table keeps its parameter vectors.
#[derive(Clone, Copy)]
pub struct ParamFields {
    queries: usize,
    resources: usize,
    events: usize,
}

/// SystemInput: `sys_id 0, queries 1, tick 2, resources 3, events 4`.
pub const SYSTEM_INPUT: ParamFields = ParamFields { queries: 1, resources: 3, events: 4 };
/// ObserverInput: `obs_id 0, entity 1, value 2, queries 3, resources 4, events 5,
/// packet_direction 6, packet 7`.
pub const OBSERVER_INPUT: ParamFields = ParamFields { queries: 3, resources: 4, events: 5 };

/// One Res param as delivered.
#[derive(Debug, PartialEq)]
pub enum ResInput {
    /// The host has no such resource.
    Absent,
    /// The value, as JSON.
    Value(String),
    /// Byte-identical to what this param received on its previous run.
    Unchanged,
}

/// The parameter data of a SystemInput / ObserverInput, by param index.
pub struct Params<'a> {
    buf: Buf<'a>,
    root: Option<usize>,
    fields: ParamFields,
}

impl<'a> Params<'a> {
    pub fn new(bytes: &'a [u8], fields: ParamFields) -> Params<'a> {
        let buf = Buf::new(bytes);
        Params { buf, root: buf.root(), fields }
    }

    /// Tables of the vector at `field` (each `{ param_index: u32 (0), .. }`) whose
    /// param_index is `param`.
    fn entry(&self, field: usize, param: u32) -> Option<usize> {
        let vec = self.buf.field_ref(self.root?, field)?;
        (0..self.buf.vector_len(vec))
            .filter_map(|i| self.buf.vector_table(vec, i))
            .find(|&t| self.buf.field_u32(t, 0) == param)
    }

    /// The `data` of a CompValue table, as text.
    fn comp_json(&self, comp: usize) -> String {
        let data = self.buf.field_ref(comp, 2).map(|p| self.buf.bytes_at(p)).unwrap_or(&[]);
        String::from_utf8_lossy(data).into_owned()
    }

    fn comps(&self, vec: Option<usize>) -> Vec<String> {
        let Some(vec) = vec else { return Vec::new() };
        (0..self.buf.vector_len(vec))
            .filter_map(|i| self.buf.vector_table(vec, i))
            .map(|c| self.comp_json(c))
            .collect()
    }

    /// QueryRows `{ param_index 0, rows 1 }`, Row `{ entity 0, comps 1 }`.
    pub fn rows(&self, param: u32) -> Vec<(u64, Vec<String>)> {
        let Some(q) = self.entry(self.fields.queries, param) else { return Vec::new() };
        let Some(vec) = self.buf.field_ref(q, 1) else { return Vec::new() };
        (0..self.buf.vector_len(vec))
            .filter_map(|i| self.buf.vector_table(vec, i))
            .map(|row| (self.buf.field_u64(row, 0), self.comps(self.buf.field_ref(row, 1))))
            .collect()
    }

    /// ResValue `{ param_index 0, value 1, unchanged 2 }`.
    pub fn resource(&self, param: u32) -> ResInput {
        let Some(r) = self.entry(self.fields.resources, param) else { return ResInput::Absent };
        match self.buf.field_ref(r, 1) {
            Some(comp) => ResInput::Value(self.comp_json(comp)),
            None if self.buf.field(r, 2).and_then(|p| self.buf.data.get(p).copied()).unwrap_or(0) != 0 => {
                ResInput::Unchanged
            }
            None => ResInput::Absent,
        }
    }

    /// EventValues `{ param_index 0, values 1 }`.
    pub fn events(&self, param: u32) -> Vec<String> {
        match self.entry(self.fields.events, param) {
            Some(e) => self.comps(self.buf.field_ref(e, 1)),
            None => Vec::new(),
        }
    }
}

/// An ObserverInput's `value` (field 2) as text; empty when absent.
pub fn observer_value(bytes: &[u8]) -> String {
    let p = Params::new(bytes, OBSERVER_INPUT);
    match p.root.and_then(|r| p.buf.field_ref(r, 2)) {
        Some(c) => p.comp_json(c),
        None => String::new(),
    }
}

/// A packet observer's ObserverInput `(packet_direction (6), packet (7))`; `None`
/// when there is no packet (any other observer).
pub fn observer_packet(bytes: &[u8]) -> Option<(u8, Vec<u8>)> {
    let buf = Buf::new(bytes);
    let root = buf.root()?;
    let packet = buf.bytes_at(buf.field_ref(root, 7)?).to_vec();
    let dir = buf.field(root, 6).and_then(|p| buf.data.get(p).copied()).unwrap_or(0);
    Some((dir, packet))
}

/// A Handshake's `abi_version` (field 0). 0 when absent/malformed.
pub fn read_abi_version(bytes: &[u8]) -> u32 {
    let buf = Buf::new(bytes);
    match buf.root() {
        Some(root) => buf.field_u32(root, 0),
        None => 0,
    }
}

/// A SpawnedInput: `{ spawned: [SpawnResolved] (0) }`; each SpawnResolved is
/// `{ temp_id: u32 (0), entity: u64 (1) }`. Returns `(temp_id, entity)` pairs in
/// SpawnCmd order.
pub fn read_spawned(bytes: &[u8]) -> Vec<(u32, u64)> {
    let buf = Buf::new(bytes);
    let mut out = Vec::new();
    let Some(root) = buf.root() else {
        return out;
    };
    let Some(vec) = buf.field_ref(root, 0) else {
        return out;
    };
    let len = buf.vector_len(vec);
    for i in 0..len {
        if let Some(sr) = buf.vector_table(vec, i) {
            out.push((buf.field_u32(sr, 0), buf.field_u64(sr, 1)));
        }
    }
    out
}

/// A Handshake: `{ abi_version: u32 (0), type_paths: [TypePath] (1) }`; each TypePath
/// is `{ id: u16 (0), path: string (1) }`. Returns `(id, path)` pairs.
pub fn read_type_paths(bytes: &[u8]) -> Vec<(u16, String)> {
    let buf = Buf::new(bytes);
    let mut out = Vec::new();
    let Some(root) = buf.root() else {
        return out;
    };
    let Some(vec) = buf.field_ref(root, 1) else {
        return out;
    };
    let len = buf.vector_len(vec);
    for i in 0..len {
        if let Some(tp) = buf.vector_table(vec, i) {
            let id = buf.field_u16(tp, 0);
            let path = match buf.field_ref(tp, 1) {
                Some(p) => String::from_utf8_lossy(buf.bytes_at(p)).into_owned(),
                None => String::new(),
            };
            out.push((id, path));
        }
    }
    out
}
