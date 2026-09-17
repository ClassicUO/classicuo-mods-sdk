//! Minimal absolute-offset FlatBuffer reader for the host-written input buffers
//! (Handshake / SystemInput / ObserverInput).
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

    fn u8(&self, p: usize) -> Option<u8> {
        self.data.get(p).copied()
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
    fn field_u8(&self, table: usize, index: usize) -> u8 {
        self.field(table, index).and_then(|p| self.u8(p)).unwrap_or(0)
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

// ── typed views over the raw reader ─────────────────────────────────────────────
// Field indices mirror abi/mod-abi.fbs declaration order.

/// A CompValue: `{ type_id: u16 (0), encoding: u8 (1), data: [ubyte] (2) }`.
#[derive(Clone, Copy)]
pub struct CompRef<'a> {
    buf: Buf<'a>,
    table: usize,
}

impl<'a> CompRef<'a> {
    pub fn type_id(&self) -> u16 {
        self.buf.field_u16(self.table, 0)
    }
    /// Raw encoding byte (0 = Json, 1 = Typed).
    pub fn encoding_raw(&self) -> u8 {
        self.buf.field_u8(self.table, 1)
    }
    pub fn data(&self) -> &'a [u8] {
        match self.buf.field_ref(self.table, 2) {
            Some(p) => self.buf.bytes_at(p),
            None => &[],
        }
    }
}

/// A Row: `{ entity: u64 (0), comps: [CompValue] (1) }`.
#[derive(Clone, Copy)]
pub struct RowRef<'a> {
    buf: Buf<'a>,
    table: usize,
}

impl<'a> RowRef<'a> {
    pub fn entity(&self) -> u64 {
        self.buf.field_u64(self.table, 0)
    }
    pub fn comp_count(&self) -> usize {
        self.buf.field_ref(self.table, 1).map(|v| self.buf.vector_len(v)).unwrap_or(0)
    }
    pub fn comp(&self, i: usize) -> Option<CompRef<'a>> {
        let vec = self.buf.field_ref(self.table, 1)?;
        if i >= self.buf.vector_len(vec) {
            return None;
        }
        let table = self.buf.vector_table(vec, i)?;
        Some(CompRef { buf: self.buf, table })
    }
}

/// A QueryRows: `{ param_index: u32 (0), rows: [Row] (1) }`.
#[derive(Clone, Copy)]
pub struct QueryRowsRef<'a> {
    buf: Buf<'a>,
    table: usize,
}

impl<'a> QueryRowsRef<'a> {
    pub fn param_index(&self) -> u32 {
        self.buf.field_u32(self.table, 0)
    }
    pub fn len(&self) -> usize {
        self.buf.field_ref(self.table, 1).map(|v| self.buf.vector_len(v)).unwrap_or(0)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn row(&self, i: usize) -> Option<RowRef<'a>> {
        let vec = self.buf.field_ref(self.table, 1)?;
        if i >= self.buf.vector_len(vec) {
            return None;
        }
        let table = self.buf.vector_table(vec, i)?;
        Some(RowRef { buf: self.buf, table })
    }
}

/// A SystemInput: `{ sys_id: u32 (0), queries: [QueryRows] (1), tick: u64 (2) }`.
#[derive(Clone, Copy)]
pub struct SystemInputRef<'a> {
    buf: Buf<'a>,
    root: Option<usize>,
}

impl<'a> SystemInputRef<'a> {
    pub fn new(bytes: &'a [u8]) -> SystemInputRef<'a> {
        let buf = Buf::new(bytes);
        let root = buf.root();
        SystemInputRef { buf, root }
    }
    pub fn sys_id(&self) -> u32 {
        self.root.map(|r| self.buf.field_u32(r, 0)).unwrap_or(0)
    }
    pub fn tick(&self) -> u64 {
        self.root.map(|r| self.buf.field_u64(r, 2)).unwrap_or(0)
    }
    /// The `n`th QueryRows in the vector, or `None`. The host emits exactly one entry
    /// per declared query param, in declaration order, so `n` is the query ORDINAL —
    /// non-query params (`commands()`) never occupy a slot. The wire `param_index` is
    /// still readable off the entry.
    pub fn query_at(&self, n: usize) -> Option<QueryRowsRef<'a>> {
        let root = self.root?;
        let vec = self.buf.field_ref(root, 1)?;
        if n >= self.buf.vector_len(vec) {
            return None;
        }
        let table = self.buf.vector_table(vec, n)?;
        Some(QueryRowsRef { buf: self.buf, table })
    }
}

/// An ObserverInput: `{ obs_id: u32 (0), entity: u64 (1), value: CompValue (2) }`.
#[derive(Clone, Copy)]
pub struct ObserverInputRef<'a> {
    buf: Buf<'a>,
    root: Option<usize>,
}

impl<'a> ObserverInputRef<'a> {
    pub fn new(bytes: &'a [u8]) -> ObserverInputRef<'a> {
        let buf = Buf::new(bytes);
        let root = buf.root();
        ObserverInputRef { buf, root }
    }
    pub fn obs_id(&self) -> u32 {
        self.root.map(|r| self.buf.field_u32(r, 0)).unwrap_or(0)
    }
    pub fn value(&self) -> Option<CompRef<'a>> {
        let root = self.root?;
        let table = self.buf.field_ref(root, 2)?;
        Some(CompRef { buf: self.buf, table })
    }
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
