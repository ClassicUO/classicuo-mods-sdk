//! Ergonomic layer over the raw ABI: registration ([`ModBuilder`]), the PUSH-model
//! input reader ([`SystemInputView`] / [`ObserverInputView`]), the return-side
//! [`CommandBufferBuilder`], and the [`export_mod!`](crate::export_mod) macro that wires
//! the four dispatch exports.
//!
//! Lifecycle (host drives, serially):
//! 1. `mod_setup(Handshake)` — the SDK interns the type-path table, calls the mod's
//!    setup fn to collect systems/observers/packet-filter, stashes the callbacks in a
//!    per-instance registry, and returns the `SetupReply`.
//! 2. `mod_run(sys_id, SystemInput)` / `mod_observer(obs_id, entity, ObserverInput)`
//!    — dispatch to the stored callback; return its `CommandBuffer` (or 0 for none).
//! 3. `mod_filter(id, bytes)` / `mod_filter_out(id, bytes)` — run the incoming /
//!    outgoing packet filter; nonzero blocks.

use crate::abi::{self, Encoding, ObserverKind, ParamKind, QueryTermKind, Schedule};
use crate::arena;
use core::ptr::addr_of_mut;
use planus::Builder;
use std::collections::HashMap;

/// Sentinel for "no component type" (matches `CompValue.type_id` / `ObserverDecl.type_id`).
pub const NONE_TYPE: u16 = 0xFFFF;

/// The mod ABI version this SDK is compiled against. `mod_setup` panics when the host's
/// `Handshake.abi_version` differs — a silent mismatch corrupts every buffer that
/// follows, so the guest fails loud at the one point the host can still report it.
pub const ABI_VERSION: u32 = 2;

// ── entity references ──────────────────────────────────────────────────────────

/// A guest-chosen temporary id for an entity spawned earlier in the same
/// [`CommandBufferBuilder`]. Encodes to a negative wire ref (`-(temp_id) - 1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TempId(u32);

impl TempId {
    /// The raw guest-chosen temp id (positional in this buffer's spawn set).
    pub fn raw(self) -> u32 {
        self.0
    }
}

/// Anything usable as a `CommandBuffer` entity ref: a real ecs id (`u64`), a [`TempId`]
/// from an earlier spawn, or a pre-encoded `i64`.
pub trait IntoEntityRef {
    fn into_entity_ref(self) -> i64;
}

impl IntoEntityRef for TempId {
    fn into_entity_ref(self) -> i64 {
        -(self.0 as i64) - 1
    }
}
impl IntoEntityRef for u64 {
    // Real ecs id. The wire ref is i64; the host guarantees real ids stay non-negative.
    fn into_entity_ref(self) -> i64 {
        self as i64
    }
}
impl IntoEntityRef for i64 {
    fn into_entity_ref(self) -> i64 {
        self
    }
}

// ── component payloads ──────────────────────────────────────────────────────────

/// A component / resource payload to write. `type_id` is interned via the handshake
/// (see [`ModBuilder::type_id`]).
#[derive(Clone, Debug)]
pub struct Comp {
    pub type_id: u16,
    pub encoding: Encoding,
    pub data: Vec<u8>,
}

impl Comp {
    /// utf8 JSON payload (registry `SetJson` path).
    pub fn json(type_id: u16, json: impl Into<String>) -> Comp {
        Comp {
            type_id,
            encoding: Encoding::Json,
            data: json.into().into_bytes(),
        }
    }

    // No `typed` constructor: `Encoding::Typed` (the phase-2 registry `SetFlat` path)
    // is not implemented host-side — the applier skips such a payload and logs it.
    // Add the constructor back with the host support, not before.

    /// utf8 JSON payload serialized from a typed value (the ergonomic `json` twin —
    /// the wire stays JSON). Panics only if `T`'s Serialize impl itself fails, which
    /// for a plain data struct it cannot.
    pub fn value<T: serde::Serialize>(type_id: u16, value: &T) -> Comp {
        Comp {
            type_id,
            encoding: Encoding::Json,
            data: serde_json::to_vec(value).unwrap_or_else(|_| b"{}".to_vec()),
        }
    }

    /// A zero-sized tag / marker (no payload).
    pub fn marker(type_id: u16) -> Comp {
        Comp {
            type_id,
            encoding: Encoding::Json,
            data: Vec::new(),
        }
    }
}

fn comp_value(c: &Comp) -> abi::CompValue {
    abi::CompValue {
        type_id: c.type_id,
        encoding: c.encoding,
        data: if c.data.is_empty() {
            None
        } else {
            Some(c.data.clone())
        },
    }
}

fn comp_values(comps: &[Comp]) -> Option<Vec<abi::CompValue>> {
    if comps.is_empty() {
        None
    } else {
        Some(comps.iter().map(comp_value).collect())
    }
}

// ── query declarations ──────────────────────────────────────────────────────────

/// One query term in a system's query param. Ref/Mut terms carry the component into the
/// row (in declaration order); With/Without are filter-only.
#[derive(Clone, Copy, Debug)]
pub struct Term {
    pub kind: QueryTermKind,
    pub type_id: u16,
}

impl Term {
    pub fn reference(type_id: u16) -> Term {
        Term {
            kind: QueryTermKind::Ref,
            type_id,
        }
    }
    pub fn mutable(type_id: u16) -> Term {
        Term {
            kind: QueryTermKind::Mut,
            type_id,
        }
    }
    pub fn with(type_id: u16) -> Term {
        Term {
            kind: QueryTermKind::With,
            type_id,
        }
    }
    pub fn without(type_id: u16) -> Term {
        Term {
            kind: QueryTermKind::Without,
            type_id,
        }
    }
    /// Read + change-filter: the row carries the component (interleaved with Ref/Mut in
    /// declaration order) and only entities whose component changed at or after this
    /// system's last run match. Inclusive, so a write made later in the same host tick
    /// is never dropped — at the cost of at most one re-delivery.
    pub fn changed(type_id: u16) -> Term {
        Term {
            kind: QueryTermKind::Changed,
            type_id,
        }
    }
}

/// Builder for a system declaration. Params are declared in order; a query param's
/// Ref/Mut terms map 1:1 (in order) to a [`RowView`]'s comps.
pub struct SystemSpec {
    name: String,
    schedule: Schedule,
    custom_stage: Option<String>,
    params: Vec<abi::ParamDecl>,
    after: Vec<u32>,
    before: Vec<u32>,
    interval_ms: u32,
}

impl SystemSpec {
    pub fn new(name: impl Into<String>, schedule: Schedule) -> SystemSpec {
        SystemSpec {
            name: name.into(),
            schedule,
            custom_stage: None,
            params: Vec::new(),
            after: Vec::new(),
            before: Vec::new(),
            interval_ms: 0,
        }
    }

    /// A system on a host-named custom stage.
    pub fn custom(name: impl Into<String>, stage: impl Into<String>) -> SystemSpec {
        SystemSpec {
            name: name.into(),
            schedule: Schedule::Custom,
            custom_stage: Some(stage.into()),
            params: Vec::new(),
            after: Vec::new(),
            before: Vec::new(),
            interval_ms: 0,
        }
    }

    /// Declare a `Commands` param.
    pub fn commands(mut self) -> SystemSpec {
        self.params.push(abi::ParamDecl {
            kind: ParamKind::Commands,
            query: None,
        });
        self
    }

    /// Declare a `Query` param. Read its rows back with
    /// [`SystemInputView::query`]`(n)`, where `n` counts only `query` calls.
    pub fn query(mut self, terms: &[Term]) -> SystemSpec {
        let decl = abi::QueryDecl {
            terms: Some(
                terms
                    .iter()
                    .map(|t| abi::QueryTerm {
                        kind: t.kind,
                        type_id: t.type_id,
                    })
                    .collect(),
            ),
        };
        self.params.push(abi::ParamDecl {
            kind: ParamKind::Query,
            query: Some(Box::new(decl)),
        });
        self
    }

    pub fn after(mut self, sys_id: u32) -> SystemSpec {
        self.after.push(sys_id);
        self
    }

    pub fn before(mut self, sys_id: u32) -> SystemSpec {
        self.before.push(sys_id);
        self
    }

    /// Throttle: run at most once per `ms` host-milliseconds (0 = every tick). The host
    /// gates BEFORE evaluating the system's queries, so a throttled system is free.
    pub fn every_ms(mut self, ms: u32) -> SystemSpec {
        self.interval_ms = ms;
        self
    }

    fn into_decl(self, id: u32) -> abi::SystemDecl {
        abi::SystemDecl {
            id,
            name: Some(self.name),
            schedule: self.schedule,
            custom_stage: self.custom_stage,
            params: if self.params.is_empty() {
                None
            } else {
                Some(self.params)
            },
            after: if self.after.is_empty() {
                None
            } else {
                Some(self.after)
            },
            before: if self.before.is_empty() {
                None
            } else {
                Some(self.before)
            },
            interval_ms: self.interval_ms,
        }
    }
}

/// Builder for an observer declaration.
pub struct ObserverSpec {
    kind: ObserverKind,
    type_id: u16,
    event_name: Option<String>,
}

impl ObserverSpec {
    pub fn on_spawn() -> ObserverSpec {
        ObserverSpec {
            kind: ObserverKind::Spawn,
            type_id: NONE_TYPE,
            event_name: None,
        }
    }
    pub fn on_despawn() -> ObserverSpec {
        ObserverSpec {
            kind: ObserverKind::Despawn,
            type_id: NONE_TYPE,
            event_name: None,
        }
    }
    pub fn on_insert(type_id: u16) -> ObserverSpec {
        ObserverSpec {
            kind: ObserverKind::Insert,
            type_id,
            event_name: None,
        }
    }
    pub fn on_remove(type_id: u16) -> ObserverSpec {
        ObserverSpec {
            kind: ObserverKind::Remove,
            type_id,
            event_name: None,
        }
    }
    pub fn on_event(name: impl Into<String>) -> ObserverSpec {
        ObserverSpec {
            kind: ObserverKind::Custom,
            type_id: NONE_TYPE,
            event_name: Some(name.into()),
        }
    }

    fn into_decl(self, id: u32) -> abi::ObserverDecl {
        abi::ObserverDecl {
            id,
            kind: self.kind,
            type_id: self.type_id,
            event_name: self.event_name,
        }
    }
}

// ── the command buffer builder (guest return) ────────────────────────────────────

/// Accumulates the structural changes a system/observer wants applied. The host applies
/// them AFTER the call returns — a component written here is NOT visible to a later
/// `component_get` in the same call.
pub struct CommandBufferBuilder {
    cmds: Vec<abi::Cmd>,
    next_temp: u32,
    // temp id -> caller-chosen name, for spawn_named. Moved into the registry when the
    // buffer is returned to the host, then resolved to real ecs ids by `mod_spawned`.
    pending_names: Vec<(u32, String)>,
}

impl CommandBufferBuilder {
    pub fn new() -> CommandBufferBuilder {
        CommandBufferBuilder {
            cmds: Vec::new(),
            next_temp: 0,
            pending_names: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cmds.is_empty()
    }

    pub fn len(&self) -> usize {
        self.cmds.len()
    }

    /// Spawn an empty entity; returns its [`TempId`] for use as a ref in later commands.
    pub fn spawn(&mut self) -> TempId {
        self.spawn_with(&[])
    }

    /// Spawn an entity carrying `comps`; returns its [`TempId`].
    pub fn spawn_with(&mut self, comps: &[Comp]) -> TempId {
        let temp_id = self.next_temp;
        self.next_temp += 1;
        self.cmds.push(abi::Cmd::SpawnCmd(Box::new(abi::SpawnCmd {
            temp_id,
            comps: comp_values(comps),
        })));
        TempId(temp_id)
    }

    /// Spawn an entity carrying `comps` and remember it under `name`. Once the host has
    /// applied this buffer it calls back through `mod_spawned`, after which
    /// [`entity(name)`](crate::entity) returns the real ecs id (across frames, until
    /// [`forget_entity`](crate::forget_entity) or a re-spawn under the same name).
    pub fn spawn_named(&mut self, name: &str, comps: &[Comp]) -> TempId {
        let temp = self.spawn_with(comps);
        self.pending_names.push((temp.raw(), name.to_string()));
        temp
    }

    /// Despawn the entity previously registered under `name` (no-op when unknown). The
    /// name is forgotten immediately — the despawn itself applies after this call.
    pub fn despawn_named(&mut self, name: &str) {
        if let Some(id) = entity(name) {
            self.despawn(id);
            forget_entity(name);
        }
    }

    /// Insert / overwrite components on an entity (covers `component.set`).
    pub fn insert(&mut self, entity: impl IntoEntityRef, comps: &[Comp]) {
        self.cmds.push(abi::Cmd::InsertCmd(Box::new(abi::InsertCmd {
            entity: entity.into_entity_ref(),
            comps: comp_values(comps),
        })));
    }

    /// Remove components (by interned type id) from an entity.
    pub fn remove(&mut self, entity: impl IntoEntityRef, type_ids: &[u16]) {
        self.cmds.push(abi::Cmd::RemoveCmd(Box::new(abi::RemoveCmd {
            entity: entity.into_entity_ref(),
            type_ids: if type_ids.is_empty() {
                None
            } else {
                Some(type_ids.to_vec())
            },
        })));
    }

    /// Despawn an entity (and, host-side, its subtree).
    pub fn despawn(&mut self, entity: impl IntoEntityRef) {
        self.cmds.push(abi::Cmd::DespawnCmd(Box::new(abi::DespawnCmd {
            entity: entity.into_entity_ref(),
        })));
    }

    /// Parent `child` under `parent` at `index` (`u32::MAX` = append).
    pub fn add_child(
        &mut self,
        parent: impl IntoEntityRef,
        child: impl IntoEntityRef,
        index: u32,
    ) {
        self.cmds.push(abi::Cmd::AddChildCmd(Box::new(abi::AddChildCmd {
            parent: parent.into_entity_ref(),
            child: child.into_entity_ref(),
            index,
        })));
    }

    /// Overwrite a singleton resource.
    pub fn resource_set(&mut self, value: Comp) {
        self.cmds
            .push(abi::Cmd::ResourceSetCmd(Box::new(abi::ResourceSetCmd {
                value: Some(Box::new(comp_value(&value))),
            })));
    }

    /// Emit a custom event (utf8 JSON payload). `entity` = 0 for a global event.
    pub fn emit_event(&mut self, event_name: &str, entity: u64, data: &[u8]) {
        self.cmds
            .push(abi::Cmd::EmitEventCmd(Box::new(abi::EmitEventCmd {
                event_name: Some(event_name.to_string()),
                entity,
                data: if data.is_empty() {
                    None
                } else {
                    Some(data.to_vec())
                },
            })));
    }

    /// Consume a mouse button for this frame (block downstream world/pickup handling).
    pub fn consume_mouse(&mut self, button: u8) {
        self.cmds
            .push(abi::Cmd::ConsumeMouseCmd(Box::new(abi::ConsumeMouseCmd {
                button,
            })));
    }

    /// Consume a key for this frame.
    pub fn consume_key(&mut self, key: u32) {
        self.cmds
            .push(abi::Cmd::ConsumeKeyCmd(Box::new(abi::ConsumeKeyCmd {
                key,
            })));
    }

    /// Hand the pending `spawn_named` bindings to the caller (the SDK dispatch), which
    /// stashes them until the host's `mod_spawned` callback resolves them.
    fn take_pending_names(&mut self) -> Vec<(u32, String)> {
        core::mem::take(&mut self.pending_names)
    }

    /// Serialize to a `CommandBuffer` flatbuffer (root). Empty → an empty-cmds buffer.
    pub fn finish(self) -> Vec<u8> {
        let cb = abi::CommandBuffer {
            cmds: if self.cmds.is_empty() {
                None
            } else {
                Some(self.cmds)
            },
        };
        Builder::new().finish(&cb, None).to_vec()
    }
}

impl Default for CommandBufferBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ── input readers (PUSH model) ────────────────────────────────────────────────────
//
// These read the FlatSharp-serialized input buffers via the absolute-offset [`wire`]
// reader, NOT planus: planus's forward-only slice reader can't follow FlatSharp's
// deduplicated (backward-pointing) vtables — see wire.rs. The return direction still
// builds via planus (which FlatSharp reads fine).

/// A single component slot on a [`RowView`], in the query's Ref/Mut declaration order.
#[derive(Clone, Copy)]
pub struct CompView<'a> {
    inner: crate::wire::CompRef<'a>,
}

impl<'a> CompView<'a> {
    pub fn type_id(&self) -> u16 {
        self.inner.type_id()
    }
    pub fn encoding(&self) -> Encoding {
        match self.inner.encoding_raw() {
            1 => Encoding::Typed,
            _ => Encoding::Json,
        }
    }
    /// Raw payload bytes (typed sub-buffer or JSON utf8).
    pub fn bytes(&self) -> &'a [u8] {
        self.inner.data()
    }
    /// Payload as a JSON string view (empty if not valid utf8).
    pub fn json(&self) -> &'a str {
        core::str::from_utf8(self.bytes()).unwrap_or("")
    }
    /// Deserialize the JSON payload into `T`; `None` on empty or malformed data.
    /// The wire stays JSON — this is ergonomics over [`json`](Self::json).
    pub fn parse<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        let b = self.bytes();
        if b.is_empty() {
            return None;
        }
        serde_json::from_slice(b).ok()
    }
}

/// One query result row: an entity plus its Ref/Mut components in declaration order.
#[derive(Clone, Copy)]
pub struct RowView<'a> {
    inner: crate::wire::RowRef<'a>,
}

impl<'a> RowView<'a> {
    pub fn entity(&self) -> u64 {
        self.inner.entity()
    }
    /// Component at declared Ref/Mut index `i`.
    pub fn comp(&self, i: usize) -> Option<CompView<'a>> {
        self.inner.comp(i).map(|inner| CompView { inner })
    }
    pub fn comp_count(&self) -> usize {
        self.inner.comp_count()
    }
}

/// Iterator over the rows of a query param.
pub struct RowIter<'a> {
    query: crate::wire::QueryRowsRef<'a>,
    idx: usize,
    len: usize,
}

impl<'a> Iterator for RowIter<'a> {
    type Item = RowView<'a>;
    fn next(&mut self) -> Option<RowView<'a>> {
        while self.idx < self.len {
            let i = self.idx;
            self.idx += 1;
            if let Some(inner) = self.query.row(i) {
                return Some(RowView { inner });
            }
        }
        None
    }
}

/// The rows of one query param.
#[derive(Clone, Copy)]
pub struct QueryRowsView<'a> {
    inner: crate::wire::QueryRowsRef<'a>,
}

impl<'a> QueryRowsView<'a> {
    pub fn param_index(&self) -> u32 {
        self.inner.param_index()
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    pub fn row(&self, i: usize) -> Option<RowView<'a>> {
        self.inner.row(i).map(|inner| RowView { inner })
    }
    pub fn rows(&self) -> RowIter<'a> {
        RowIter {
            query: self.inner,
            idx: 0,
            len: self.inner.len(),
        }
    }
}

/// Reader over a `mod_run` input buffer.
pub struct SystemInputView<'a> {
    inner: crate::wire::SystemInputRef<'a>,
}

impl<'a> SystemInputView<'a> {
    pub fn new(bytes: &'a [u8]) -> SystemInputView<'a> {
        SystemInputView {
            inner: crate::wire::SystemInputRef::new(bytes),
        }
    }
    pub fn sys_id(&self) -> u32 {
        self.inner.sys_id()
    }
    /// Host frame tick (for guest-side timers).
    pub fn tick(&self) -> u64 {
        self.inner.tick()
    }
    /// The rows of the `n`th `.query(..)` declared on the [`SystemSpec`] (0-based;
    /// `commands()` and any other param kind do NOT consume an index).
    pub fn query(&self, n: u32) -> Option<QueryRowsView<'a>> {
        self.inner.query_at(n as usize).map(|inner| QueryRowsView { inner })
    }
}

/// Reader over a `mod_observer` input buffer.
pub struct ObserverInputView<'a> {
    inner: crate::wire::ObserverInputRef<'a>,
    entity: u64,
}

impl<'a> ObserverInputView<'a> {
    pub fn new(bytes: &'a [u8], entity: u64) -> ObserverInputView<'a> {
        ObserverInputView {
            inner: crate::wire::ObserverInputRef::new(bytes),
            entity,
        }
    }
    pub fn obs_id(&self) -> u32 {
        self.inner.obs_id()
    }
    /// The triggering entity (authoritative — from the export arg).
    pub fn entity(&self) -> u64 {
        self.entity
    }
    /// The event/component payload (Insert/Remove component, or Custom event JSON).
    pub fn value(&self) -> Option<CompView<'a>> {
        self.inner.value().map(|inner| CompView { inner })
    }
}

// ── registration ────────────────────────────────────────────────────────────────

/// Interned component/resource/event type-path table from the handshake.
pub struct TypeTable {
    map: HashMap<String, u16>,
}

impl TypeTable {
    pub fn id(&self, path: &str) -> Option<u16> {
        self.map.get(path).copied()
    }
    pub fn path(&self, id: u16) -> Option<&str> {
        // Reverse lookup is rare (diagnostics); the forward path is the hot one.
        self.map
            .iter()
            .find(|(_, i)| **i == id)
            .map(|(p, _)| p.as_str())
    }
}

type SystemFn = Box<dyn FnMut(&SystemInputView, &mut CommandBufferBuilder)>;
type ObserverFn = Box<dyn FnMut(&ObserverInputView, &mut CommandBufferBuilder)>;
type PacketFn = Box<dyn FnMut(u8, &[u8]) -> bool>;

/// What a mod's setup fn populates: systems, observers, and the optional packet filters.
pub struct ModBuilder {
    types: TypeTable,
    systems: Vec<(abi::SystemDecl, SystemFn)>,
    observers: Vec<(abi::ObserverDecl, ObserverFn)>,
    packet_filter: Option<PacketFn>,
    packet_filter_out: Option<PacketFn>,
    wants_filter: bool,
    wants_filter_out: bool,
    next_sys_id: u32,
    next_obs_id: u32,
}

impl ModBuilder {
    fn new(types: TypeTable) -> ModBuilder {
        ModBuilder {
            types,
            systems: Vec::new(),
            observers: Vec::new(),
            packet_filter: None,
            packet_filter_out: None,
            wants_filter: false,
            wants_filter_out: false,
            next_sys_id: 0,
            next_obs_id: 0,
        }
    }

    /// Interned id for a registered type path. PANICS when the host didn't register it —
    /// a `NONE_TYPE` sentinel silently made every later command on that type a no-op
    /// (spawned UI that never appears, reads that never resolve), which is far more
    /// expensive to diagnose than a trap at setup. Use
    /// [`try_type_id`](Self::try_type_id) for genuinely optional host types.
    pub fn type_id(&self, path: &str) -> u16 {
        match self.types.id(path) {
            Some(id) => id,
            None => panic!("unknown type path '{path}' (see paths.rs / the host registry)"),
        }
    }

    /// Like [`type_id`](Self::type_id) but `None` (no log) when absent.
    pub fn try_type_id(&self, path: &str) -> Option<u16> {
        self.types.id(path)
    }

    /// Interned id for the path a generated payload type is registered under:
    /// `m.type_id_of::<types::Node>()` instead of `m.type_id(paths::ui::NODE)`. Works
    /// for markers too — they are empty structs with the same [`HasPath`] impl.
    pub fn type_id_of<T: crate::types::HasPath>(&self) -> u16 {
        self.type_id(T::PATH)
    }

    /// Like [`type_id_of`](Self::type_id_of) but `None` when absent.
    pub fn try_type_id_of<T: crate::types::HasPath>(&self) -> Option<u16> {
        self.try_type_id(T::PATH)
    }

    /// Direct access to the interned table.
    pub fn types(&self) -> &TypeTable {
        &self.types
    }

    /// Register a system; returns its assigned id (usable in `after`/`before`).
    pub fn add_system<F>(&mut self, spec: SystemSpec, run: F) -> u32
    where
        F: FnMut(&SystemInputView, &mut CommandBufferBuilder) + 'static,
    {
        let id = self.next_sys_id;
        self.next_sys_id += 1;
        self.systems.push((spec.into_decl(id), Box::new(run)));
        id
    }

    /// Register an observer; returns its assigned id.
    pub fn add_observer<F>(&mut self, spec: ObserverSpec, run: F) -> u32
    where
        F: FnMut(&ObserverInputView, &mut CommandBufferBuilder) + 'static,
    {
        let id = self.next_obs_id;
        self.next_obs_id += 1;
        self.observers.push((spec.into_decl(id), Box::new(run)));
        id
    }

    /// Register the incoming-packet filter (also sets `wants_filter`). Return
    /// `true` from `f` to BLOCK the packet.
    pub fn on_packet<F>(&mut self, f: F)
    where
        F: FnMut(u8, &[u8]) -> bool + 'static,
    {
        assert!(
            self.packet_filter.is_none(),
            "on_packet registered twice: the ABI has ONE mod_filter export, so the              second registration silently replaced the first"
        );
        self.wants_filter = true;
        self.packet_filter = Some(Box::new(f));
    }

    /// Register the OUTGOING-packet filter (also sets `wants_filter_out`). Called on
    /// the host's send path with the framed bytes the client is about to encrypt;
    /// return `true` to BLOCK the packet so it never reaches the socket.
    ///
    /// A mod's own `imports::net_send` bypasses this filter — it cannot see, or
    /// block, its own sends.
    pub fn on_packet_out<F>(&mut self, f: F)
    where
        F: FnMut(u8, &[u8]) -> bool + 'static,
    {
        assert!(
            self.packet_filter_out.is_none(),
            "on_packet_out registered twice: the ABI has ONE mod_filter_out export, so              the second registration silently replaced the first"
        );
        self.wants_filter_out = true;
        self.packet_filter_out = Some(Box::new(f));
    }

    fn build(self) -> (Vec<u8>, Registry) {
        let mut sys_decls = Vec::with_capacity(self.systems.len());
        let mut sys_fns = Vec::with_capacity(self.systems.len());
        for (decl, f) in self.systems {
            sys_fns.push((decl.id, f));
            sys_decls.push(decl);
        }
        let mut obs_decls = Vec::with_capacity(self.observers.len());
        let mut obs_fns = Vec::with_capacity(self.observers.len());
        for (decl, f) in self.observers {
            obs_fns.push((decl.id, f));
            obs_decls.push(decl);
        }

        let reply = abi::SetupReply {
            systems: if sys_decls.is_empty() {
                None
            } else {
                Some(sys_decls)
            },
            observers: if obs_decls.is_empty() {
                None
            } else {
                Some(obs_decls)
            },
            wants_filter: self.wants_filter,
            wants_filter_out: self.wants_filter_out,
        };
        let bytes = Builder::new().finish(&reply, None).to_vec();

        let reg = Registry {
            systems: sys_fns,
            observers: obs_fns,
            packet_filter: self.packet_filter,
            packet_filter_out: self.packet_filter_out,
            _types: self.types,
            pending_names: Vec::new(),
            named: HashMap::new(),
        };
        (bytes, reg)
    }
}

// ── per-instance registry + dispatch ──────────────────────────────────────────────

struct Registry {
    systems: Vec<(u32, SystemFn)>,
    observers: Vec<(u32, ObserverFn)>,
    packet_filter: Option<PacketFn>,
    packet_filter_out: Option<PacketFn>,
    // Kept alive for runtime type lookups a mod may want; unused by the SDK itself.
    _types: TypeTable,
    // temp id -> name, from the LAST command buffer handed to the host. Consumed by
    // `mod_spawned` (which the host calls synchronously right after applying it).
    pending_names: Vec<(u32, String)>,
    // Resolved real ecs ids by name — the whole point of `spawn_named`.
    named: HashMap<String, u64>,
}

static mut REGISTRY: Option<Registry> = None;

fn set_registry(reg: Registry) {
    // SAFETY: single-threaded guest; setup runs once before any dispatch.
    unsafe { *addr_of_mut!(REGISTRY) = Some(reg) }
}

fn registry() -> &'static mut Registry {
    // SAFETY: single-threaded guest; the host never re-enters an export.
    unsafe {
        (*addr_of_mut!(REGISTRY))
            .as_mut()
            .expect("mod_setup was not called before dispatch")
    }
}

// ── named entities (spawn_named / mod_spawned) ────────────────────────────────────

/// The real ecs id of an entity spawned earlier via
/// [`CommandBufferBuilder::spawn_named`], once the host has applied that buffer and
/// called back through `mod_spawned`. `None` before that (same tick) or after
/// [`forget_entity`].
pub fn entity(name: &str) -> Option<u64> {
    // SAFETY: single-threaded guest; may be called before setup (returns None then).
    let reg = unsafe { (*addr_of_mut!(REGISTRY)).as_ref() }?;
    reg.named.get(name).copied()
}

/// Drop a name binding (the entity itself is untouched — despawn it separately, or use
/// [`CommandBufferBuilder::despawn_named`], which does both).
pub fn forget_entity(name: &str) {
    if let Some(reg) = unsafe { (*addr_of_mut!(REGISTRY)).as_mut() } {
        reg.named.remove(name);
    }
}

/// Stash the names a just-returned command buffer minted, so the host's `mod_spawned`
/// callback can pair them with the real ids. Replaces (not appends to) any prior batch:
/// the host resolves each buffer before the next call.
fn stash_pending_names(names: Vec<(u32, String)>) {
    if names.is_empty() {
        return;
    }
    registry().pending_names = names;
}

/// Wired by [`export_mod!`](crate::export_mod) into `mod_setup`.
pub fn __dispatch_setup(ptr: u32, len: u32, setup: fn(&mut ModBuilder)) -> u64 {
    let bytes = unsafe { arena::input_slice(ptr, len) };
    let host_abi = crate::wire::read_abi_version(bytes);
    assert!(
        host_abi == ABI_VERSION,
        "mod ABI mismatch: host speaks v{host_abi}, this mod was built against v{ABI_VERSION}          — rebuild the mod against the current cuo-mod-sdk"
    );
    let types = read_handshake(bytes);
    let mut builder = ModBuilder::new(types);
    setup(&mut builder);
    let (reply, reg) = builder.build();
    set_registry(reg);
    arena::pack_ret(&reply)
}

/// Wired by [`export_mod!`](crate::export_mod) into `mod_run`.
pub fn __dispatch_run(sys_id: u32, ptr: u32, len: u32) -> u64 {
    let bytes = unsafe { arena::input_slice(ptr, len) };
    let reg = registry();
    let Some((_, run)) = reg.systems.iter_mut().find(|(id, _)| *id == sys_id) else {
        return 0;
    };
    let input = SystemInputView::new(bytes);
    let mut cmds = CommandBufferBuilder::new();
    run(&input, &mut cmds);
    if cmds.is_empty() {
        0
    } else {
        let names = cmds.take_pending_names();
        let out = arena::pack_ret(&cmds.finish());
        stash_pending_names(names);
        out
    }
}

/// Wired by [`export_mod!`](crate::export_mod) into `mod_observer`.
pub fn __dispatch_observer(obs_id: u32, entity: u64, ptr: u32, len: u32) -> u64 {
    let bytes = unsafe { arena::input_slice(ptr, len) };
    let reg = registry();
    let Some((_, run)) = reg.observers.iter_mut().find(|(id, _)| *id == obs_id) else {
        return 0;
    };
    let input = ObserverInputView::new(bytes, entity);
    let mut cmds = CommandBufferBuilder::new();
    run(&input, &mut cmds);
    if cmds.is_empty() {
        0
    } else {
        let names = cmds.take_pending_names();
        let out = arena::pack_ret(&cmds.finish());
        stash_pending_names(names);
        out
    }
}

/// Wired by [`export_mod!`](crate::export_mod) into `mod_spawned`. Pairs the temp ids
/// the host just resolved with the names `spawn_named` recorded.
pub fn __dispatch_spawned(ptr: u32, len: u32) {
    let bytes = unsafe { arena::input_slice(ptr, len) };
    let resolved = crate::wire::read_spawned(bytes);
    let reg = registry();
    if reg.pending_names.is_empty() {
        return;
    }
    let pending = core::mem::take(&mut reg.pending_names);
    for (temp_id, real_id) in resolved {
        if let Some((_, name)) = pending.iter().find(|(t, _)| *t == temp_id) {
            reg.named.insert(name.clone(), real_id);
        }
    }
}

/// Wired by [`export_mod!`](crate::export_mod) into `mod_filter`.
pub fn __dispatch_on_packet(id: u32, ptr: u32, len: u32) -> u32 {
    let data = unsafe { arena::input_slice(ptr, len) };
    let reg = registry();
    match reg.packet_filter.as_mut() {
        Some(f) => u32::from(f(id as u8, data)),
        None => 0,
    }
}

/// Wired by [`export_mod!`](crate::export_mod) into `mod_filter_out`.
pub fn __dispatch_on_packet_out(id: u32, ptr: u32, len: u32) -> u32 {
    let data = unsafe { arena::input_slice(ptr, len) };
    let reg = registry();
    match reg.packet_filter_out.as_mut() {
        Some(f) => u32::from(f(id as u8, data)),
        None => 0,
    }
}

fn read_handshake(bytes: &[u8]) -> TypeTable {
    // wire reader (absolute offsets) — see the input-readers note / wire.rs for why
    // planus's reader can't be used on the FlatSharp-written handshake.
    TypeTable {
        map: crate::wire::read_type_paths(bytes)
            .into_iter()
            .map(|(id, path)| (path, id))
            .collect(),
    }
}

/// Emit the ABI exports for a mod, dispatching to the callbacks registered by
/// `$setup` (a `fn(&mut ModBuilder)`). `mod_alloc` / `mod_arena_reset` come from
/// [`arena`](crate::arena); this macro adds the five dispatch entrypoints
/// (`mod_setup` / `mod_run` / `mod_observer` / `mod_filter` / `mod_filter_out` /
/// `mod_spawned`).
///
/// ```ignore
/// use cuo_mod_sdk::prelude::*;
/// fn setup(m: &mut ModBuilder) {
///     m.on_packet(|id, _bytes| id == 0x99);
///     m.on_packet_out(|id, _bytes| id == 0x02);
/// }
/// export_mod!(setup);
/// ```
#[macro_export]
macro_rules! export_mod {
    ($setup:path) => {
        #[no_mangle]
        pub extern "C" fn mod_setup(ptr: u32, len: u32) -> u64 {
            $crate::runtime::__dispatch_setup(ptr, len, $setup)
        }

        #[no_mangle]
        pub extern "C" fn mod_run(sys_id: u32, ptr: u32, len: u32) -> u64 {
            $crate::runtime::__dispatch_run(sys_id, ptr, len)
        }

        #[no_mangle]
        pub extern "C" fn mod_observer(obs_id: u32, entity: u64, ptr: u32, len: u32) -> u64 {
            $crate::runtime::__dispatch_observer(obs_id, entity, ptr, len)
        }

        #[no_mangle]
        pub extern "C" fn mod_filter(id: u32, ptr: u32, len: u32) -> u32 {
            $crate::runtime::__dispatch_on_packet(id, ptr, len)
        }

        #[no_mangle]
        pub extern "C" fn mod_filter_out(id: u32, ptr: u32, len: u32) -> u32 {
            $crate::runtime::__dispatch_on_packet_out(id, ptr, len)
        }

        #[no_mangle]
        pub extern "C" fn mod_spawned(ptr: u32, len: u32) {
            $crate::runtime::__dispatch_spawned(ptr, len)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::{Cmd, CmdRef, CommandBufferRef};
    use planus::ReadAsRoot;

    #[test]
    fn command_buffer_round_trips_all_cmd_kinds() {
        let mut b = CommandBufferBuilder::new();
        let temp = b.spawn_with(&[Comp::json(0x10, "{\"x\":1}")]);
        b.insert(temp, &[Comp::json(0x11, "{\"y\":2}")]);
        b.add_child(42u64, temp, u32::MAX);
        b.remove(42u64, &[0x11, 0x12]);
        b.despawn(temp);
        b.resource_set(Comp::json(0x20, "{\"v\":2}"));
        b.emit_event("cuo:ui/opened", 7, b"{}");
        b.consume_mouse(1);
        b.consume_key(0x1B);
        assert_eq!(b.len(), 9);

        let bytes = b.finish();
        let cb = CommandBufferRef::read_as_root(&bytes).unwrap();
        let cmds = cb.cmds().unwrap().unwrap();
        assert_eq!(cmds.len(), 9);

        // spawn -> temp_id 0; add_child/despawn reference it as -1.
        match cmds.get(0).unwrap().unwrap() {
            CmdRef::SpawnCmd(s) => assert_eq!(s.temp_id().unwrap(), 0),
            other => panic!("cmd0 not spawn: {other:?}"),
        }
        match cmds.get(2).unwrap().unwrap() {
            CmdRef::AddChildCmd(a) => {
                assert_eq!(a.parent().unwrap(), 42);
                assert_eq!(a.child().unwrap(), -1); // TempId(0) -> -1
                assert_eq!(a.index().unwrap(), u32::MAX);
            }
            other => panic!("cmd2 not add_child: {other:?}"),
        }
        match cmds.get(7).unwrap().unwrap() {
            CmdRef::ConsumeMouseCmd(c) => assert_eq!(c.button().unwrap(), 1),
            other => panic!("cmd7 not consume_mouse: {other:?}"),
        }

        // sanity: the owned round-trip reads back too.
        let _owned: Cmd = cmds.get(0).unwrap().unwrap().try_into().unwrap();
    }

    #[test]
    fn temp_id_encodes_negative_ref() {
        assert_eq!(TempId(0).into_entity_ref(), -1);
        assert_eq!(TempId(1).into_entity_ref(), -2);
        assert_eq!(5u64.into_entity_ref(), 5);
    }

    #[test]
    fn empty_builder_finishes_to_empty_cmds() {
        let bytes = CommandBufferBuilder::new().finish();
        let cb = CommandBufferRef::read_as_root(&bytes).unwrap();
        // No cmds vector (None) reads back as absent/empty.
        assert!(cb.cmds().unwrap().map(|v| v.len()).unwrap_or(0) == 0);
    }

    #[test]
    fn system_input_view_reads_pushed_rows() {
        // Build a SystemInput as root (mirrors what the host pushes).
        let input = abi::SystemInput {
            sys_id: 3,
            tick: 99,
            queries: Some(vec![abi::QueryRows {
                param_index: 0,
                rows: Some(vec![abi::Row {
                    entity: 1234,
                    comps: Some(vec![abi::CompValue {
                        type_id: 0x30,
                        encoding: Encoding::Json,
                        data: Some(b"{\"hp\":42}".to_vec()),
                    }]),
                }]),
            }]),
        };
        let bytes = Builder::new().finish(&input, None).to_vec();

        let view = SystemInputView::new(&bytes);
        assert_eq!(view.sys_id(), 3);
        assert_eq!(view.tick(), 99);
        let q = view.query(0).expect("param 0 present");
        assert_eq!(q.len(), 1);
        let row = q.rows().next().expect("one row");
        assert_eq!(row.entity(), 1234);
        let c = row.comp(0).expect("comp 0");
        assert_eq!(c.type_id(), 0x30);
        assert_eq!(c.json(), "{\"hp\":42}");
        assert!(view.query(1).is_none());
    }

    #[test]
    fn changed_term_and_interval_land_in_the_decl() {
        let spec = SystemSpec::new("s", Schedule::Update)
            .query(&[Term::changed(0x10), Term::reference(0x11)])
            .every_ms(250);
        let decl = spec.into_decl(7);
        assert_eq!(decl.interval_ms, 250);
        let terms = decl.params.unwrap()[0].query.as_ref().unwrap().terms.clone().unwrap();
        assert_eq!(terms[0].kind, QueryTermKind::Changed);
        assert_eq!(terms[0].type_id, 0x10);
        assert_eq!(terms[1].kind, QueryTermKind::Ref);
    }

    #[test]
    fn spawn_named_records_pending_names_in_temp_order() {
        let mut b = CommandBufferBuilder::new();
        let a = b.spawn_named("root", &[]);
        let c = b.spawn_named("child", &[]);
        assert_eq!(a.raw(), 0);
        assert_eq!(c.raw(), 1);
        let names = b.take_pending_names();
        assert_eq!(
            names,
            vec![(0u32, "root".to_string()), (1u32, "child".to_string())]
        );
        // Taken once: the second take is empty (the host resolves one batch per call).
        assert!(b.take_pending_names().is_empty());
    }

    #[test]
    fn spawned_input_round_trips_through_the_wire_reader() {
        // Build a SpawnedInput the way the host does, then read it back.
        let si = abi::SpawnedInput {
            spawned: Some(vec![
                abi::SpawnResolved { temp_id: 0, entity: 4242 },
                abi::SpawnResolved { temp_id: 1, entity: 4243 },
            ]),
        };
        let bytes = Builder::new().finish(&si, None).to_vec();
        assert_eq!(crate::wire::read_spawned(&bytes), vec![(0u32, 4242u64), (1u32, 4243u64)]);
    }

    #[test]
    fn handshake_abi_version_reads_back() {
        let hs = abi::Handshake {
            abi_version: ABI_VERSION,
            type_paths: Some(vec![abi::TypePath { id: 3, path: Some("cuo:ui/node".into()) }]),
        };
        let bytes = Builder::new().finish(&hs, None).to_vec();
        assert_eq!(crate::wire::read_abi_version(&bytes), ABI_VERSION);
        assert_eq!(read_handshake(&bytes).id("cuo:ui/node"), Some(3));
    }

    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Pos {
        x: i32,
        y: i32,
    }

    #[test]
    fn typed_comp_value_round_trips_through_comp_view() {
        let comp = Comp::value(0x40, &Pos { x: 3, y: -4 });
        assert_eq!(comp.type_id, 0x40);

        // Push it through a SystemInput so CompView::parse reads it off the wire.
        let input = abi::SystemInput {
            sys_id: 0,
            tick: 0,
            queries: Some(vec![abi::QueryRows {
                param_index: 0,
                rows: Some(vec![abi::Row {
                    entity: 1,
                    comps: Some(vec![comp_value(&comp)]),
                }]),
            }]),
        };
        let bytes = Builder::new().finish(&input, None).to_vec();
        let view = SystemInputView::new(&bytes);
        let c = view.query(0).unwrap().rows().next().unwrap().comp(0).unwrap();
        assert_eq!(c.parse::<Pos>(), Some(Pos { x: 3, y: -4 }));
    }

    // Regression: read a SystemInput serialized by the C# FlatSharp HOST (not planus).
    // FlatSharp deduplicated the two QueryRows vtables against SystemInput's own vtable
    // and pooled it at the front (byte 16) — before the vector-reached QueryRows tables
    // (bytes 36/52). planus's forward-only slice reader returned InvalidOffset for each
    // element; the wire reader resolves vtables absolutely, so this must read cleanly.
    #[test]
    fn reads_flatsharp_written_systeminput() {
        let bytes: Vec<u8> = vec![
            0x04, 0x00, 0x00, 0x00, 0xf4, 0xff, 0xff, 0xff, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x0c, 0x00, 0x08, 0x00, 0x04, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x24, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let v = SystemInputView::new(&bytes);
        assert_eq!(v.sys_id(), 1);
        // Wire param_index 1 and 2 (the system declared commands() first) are query
        // ORDINALS 0 and 1 to the mod.
        assert_eq!(v.query(0).map(|q| (q.param_index(), q.len())), Some((1, 0)));
        assert_eq!(v.query(1).map(|q| (q.param_index(), q.len())), Some((2, 0)));
        assert!(v.query(2).is_none());
    }
}
