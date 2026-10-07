//! The Bevy-style mod API: an [`App`] you add systems and observers to, and the system
//! parameters those run with. A system's parameter list IS its declaration: the SDK
//! reads it once at setup and tells the client what to pass.
//!
//! Everything here is target-independent; `crate::backend` (p1 or p2) moves the data.

use crate::backend;
use crate::types::HasPath;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::any::Any;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

// ── data ─────────────────────────────────────────────────────────────────────────

/// A component, resource or event: any SDK type that knows its type path.
pub trait Component: HasPath + Serialize + DeserializeOwned + 'static {}
impl<T: HasPath + Serialize + DeserializeOwned + 'static> Component for T {}

/// An entity id.
///
/// On the p1 target an entity you just spawned gets its real id when your system
/// returns; until then it holds a placeholder that commands and components (e.g.
/// `ChildOf`) in the same system still accept. Compare / hash entities after that.
/// `Entity::default()` is the null entity (bits 0).
#[derive(Clone, Copy, Default)]
pub struct Entity(u64);

impl Entity {
    pub const fn from_bits(bits: u64) -> Entity {
        Entity(bits)
    }
    pub fn to_bits(self) -> u64 {
        backend::resolve(self.0)
    }
}
impl From<u64> for Entity {
    fn from(bits: u64) -> Entity {
        Entity(bits)
    }
}
impl From<Entity> for u64 {
    fn from(e: Entity) -> u64 {
        e.to_bits()
    }
}
impl PartialEq for Entity {
    fn eq(&self, other: &Entity) -> bool {
        self.to_bits() == other.to_bits()
    }
}
impl Eq for Entity {}
impl std::hash::Hash for Entity {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.to_bits().hash(h)
    }
}
impl PartialOrd for Entity {
    fn partial_cmp(&self, other: &Entity) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Entity {
    fn cmp(&self, other: &Entity) -> std::cmp::Ordering {
        self.to_bits().cmp(&other.to_bits())
    }
}
impl std::fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({})", self.to_bits())
    }
}
impl Serialize for Entity {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(self.to_bits())
    }
}
impl<'de> serde::Deserialize<'de> for Entity {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Entity, D::Error> {
        u64::deserialize(d).map(Entity)
    }
}

/// When a system runs. `Startup` runs once, after the mod loads; the rest every frame,
/// in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Schedule {
    Startup,
    First,
    PreUpdate,
    Update,
    PostUpdate,
    Last,
}

// ── declarations (SDK-internal, shared by both backends) ─────────────────────────

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermKind {
    Ref,
    Mut,
    With,
    Without,
    Changed,
    Added,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Term {
    pub kind: TermKind,
    pub path: &'static str,
}

impl Term {
    /// Whether each row carries this term's value.
    pub fn reads(&self) -> bool {
        !matches!(self.kind, TermKind::With | TermKind::Without)
    }
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParamDesc {
    Commands,
    Query(Vec<Term>),
    Res(&'static str),
    ResMut(&'static str),
    Events(&'static str),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerDesc {
    Add(&'static str),
    Remove(&'static str),
    Event(&'static str),
}

/// One parameter's data as the backend delivered it.
#[doc(hidden)]
pub enum Fetched {
    Commands(backend::CmdSink),
    Query(Vec<RawRow>),
    Res(Option<String>, backend::ResSink),
    Events(Vec<String>),
    Taken,
}

#[doc(hidden)]
pub struct RawRow {
    pub entity: u64,
    /// JSON of the reading terms, in term order.
    pub comps: Vec<String>,
    pub sink: backend::RowSink,
}

/// What a system run gets: its parameters' data, in declaration order, plus its
/// `Local` storage.
#[doc(hidden)]
pub struct ParamCtx<'a> {
    fetched: Vec<Fetched>,
    next: usize,
    locals: &'a mut Vec<Box<dyn Any>>,
    next_local: usize,
}

impl ParamCtx<'_> {
    fn take(&mut self) -> Fetched {
        let f = self
            .fetched
            .get_mut(self.next)
            .map(|f| std::mem::replace(f, Fetched::Taken))
            .unwrap_or(Fetched::Taken);
        self.next += 1;
        f
    }
}

#[doc(hidden)]
pub struct TriggerData {
    pub entity: u64,
    pub value: String,
}

pub(crate) type RunFn = Box<dyn FnMut(&mut ParamCtx, Option<&TriggerData>)>;

pub(crate) enum EntryKind {
    System(Schedule),
    Observer(TriggerDesc),
}

/// A registered system or observer.
pub(crate) struct Entry {
    pub name: String,
    /// The fn's type name, what `.after(f)` / `.before(f)` refer to.
    pub base: &'static str,
    pub after: Vec<&'static str>,
    pub before: Vec<&'static str>,
    pub kind: EntryKind,
    pub params: Vec<ParamDesc>,
    run: RunFn,
    locals: Vec<Box<dyn Any>>,
    /// Each Res param's last delivered JSON, by param index: what a host
    /// `unchanged` resource value stands for (p1 backend).
    pub(crate) res_cache: Vec<Option<String>>,
}

impl Entry {
    /// Runs the entry with the backend-delivered parameter data.
    pub(crate) fn run(&mut self, fetched: Vec<Fetched>, trigger: Option<&TriggerData>) {
        let mut ctx = ParamCtx { fetched, next: 0, locals: &mut self.locals, next_local: 0 };
        (self.run)(&mut ctx, trigger);
    }
}

// ── App ──────────────────────────────────────────────────────────────────────────

/// Passed to your `setup`: add systems and observers here.
#[derive(Default)]
pub struct App {
    pub(crate) entries: Vec<Entry>,
}

impl App {
    /// Runs `systems` (one system or a tuple of them) every frame in `schedule`, in
    /// the order given.
    pub fn add_systems<M>(&mut self, schedule: Schedule, systems: impl IntoSystems<M>) -> &mut App {
        systems.add_to(self, schedule);
        self
    }

    /// Runs `observer` each time its trigger (its first parameter, `On<Add, T>`,
    /// `On<Remove, T>` or `On<Event, T>`) fires.
    pub fn add_observer<M>(&mut self, observer: impl IntoObserver<M>) -> &mut App {
        let (trigger, built) = observer.into_observer();
        self.push(built, EntryKind::Observer(trigger));
        self
    }

    fn push(&mut self, built: Built, kind: EntryKind) {
        // `run` / `observe` find the entry by name: keep names unique.
        let mut name = built.name.to_string();
        if self.entries.iter().any(|e| e.name == name) {
            name = format!("{name}#{}", self.entries.len());
        }
        self.entries.push(Entry {
            name,
            base: built.name,
            after: built.after,
            before: built.before,
            kind,
            params: built.params,
            run: built.run,
            locals: Vec::new(),
            res_cache: Vec::new(),
        });
    }

    /// Entry indices of the systems `names` refer to (`.after(f)` / `.before(f)`).
    /// Panics on a system that was never added: the ordering would silently vanish.
    pub(crate) fn resolve_order(&self, names: &[&'static str]) -> Vec<usize> {
        names
            .iter()
            .flat_map(|n| {
                let found: Vec<usize> = (0..self.entries.len()).filter(|&i| self.entries[i].base == *n).collect();
                assert!(!found.is_empty(), "ordered against `{n}`, which is not an added system");
                found
            })
            .collect()
    }
}

#[doc(hidden)]
pub struct Built {
    name: &'static str,
    params: Vec<ParamDesc>,
    run: RunFn,
    after: Vec<&'static str>,
    before: Vec<&'static str>,
}

// ── system params ────────────────────────────────────────────────────────────────

/// Something a system can take as a parameter.
pub trait SystemParam: Sized {
    #[doc(hidden)]
    fn declare(params: &mut Vec<ParamDesc>);
    /// `None` skips this run (e.g. a `Res` the client doesn't have right now).
    #[doc(hidden)]
    fn fetch(ctx: &mut ParamCtx) -> Option<Self>;
}

/// `Option<Res<T>>` and friends: `None` instead of skipping the system.
impl<P: SystemParam> SystemParam for Option<P> {
    fn declare(params: &mut Vec<ParamDesc>) {
        P::declare(params)
    }
    fn fetch(ctx: &mut ParamCtx) -> Option<Self> {
        Some(P::fetch(ctx))
    }
}

/// Read-only access to a resource. The system is skipped while the client has none
/// (take `Option<Res<T>>` to run anyway).
pub struct Res<T>(T);

impl<T> Deref for Res<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Component> SystemParam for Res<T> {
    fn declare(params: &mut Vec<ParamDesc>) {
        params.push(ParamDesc::Res(T::PATH));
    }
    fn fetch(ctx: &mut ParamCtx) -> Option<Self> {
        match ctx.take() {
            Fetched::Res(Some(json), _) => serde_json::from_str(&json).ok().map(Res),
            _ => None,
        }
    }
}

/// Writable access to a resource: changes are written back when the system returns.
pub struct ResMut<T: Component> {
    value: T,
    original: String,
    sink: backend::ResSink,
}

impl<T: Component> Deref for ResMut<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl<T: Component> DerefMut for ResMut<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
impl<T: Component> Drop for ResMut<T> {
    fn drop(&mut self) {
        let json = to_json(&self.value);
        if json != self.original {
            backend::res_set(&self.sink, T::PATH, json);
        }
    }
}

impl<T: Component> SystemParam for ResMut<T> {
    fn declare(params: &mut Vec<ParamDesc>) {
        params.push(ParamDesc::ResMut(T::PATH));
    }
    fn fetch(ctx: &mut ParamCtx) -> Option<Self> {
        match ctx.take() {
            Fetched::Res(Some(json), sink) => {
                let value: T = serde_json::from_str(&json).ok()?;
                // Compare against our own serialization, not the host's text: the
                // two format differently, and a spurious write marks it changed.
                let original = to_json(&value);
                Some(ResMut { value, original, sink })
            }
            _ => None,
        }
    }
}

/// The events of type `T` sent since this system last ran.
pub struct EventReader<T>(Vec<T>);

impl<T> EventReader<T> {
    pub fn read(&mut self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: Component> SystemParam for EventReader<T> {
    fn declare(params: &mut Vec<ParamDesc>) {
        params.push(ParamDesc::Events(T::PATH));
    }
    fn fetch(ctx: &mut ParamCtx) -> Option<Self> {
        let events = match ctx.take() {
            Fetched::Events(v) => v.iter().filter_map(|j| serde_json::from_str(j).ok()).collect(),
            _ => Vec::new(),
        };
        Some(EventReader(events))
    }
}

/// State that belongs to one system and survives between its runs (counters,
/// timers, the entity of a window you spawned).
pub struct Local<T: 'static>(*mut T);

impl<T> Deref for Local<T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: points into a Box owned by the system's entry, which outlives the
        // run; one Local per slot per run, and the guest is single-threaded.
        unsafe { &*self.0 }
    }
}
impl<T> DerefMut for Local<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0 }
    }
}

impl<T: Default + 'static> SystemParam for Local<T> {
    fn declare(_: &mut Vec<ParamDesc>) {}
    fn fetch(ctx: &mut ParamCtx) -> Option<Self> {
        let i = ctx.next_local;
        ctx.next_local += 1;
        if ctx.locals.len() <= i {
            ctx.locals.push(Box::new(T::default()));
        }
        let slot = ctx.locals[i].downcast_mut::<T>().expect("Local slot type");
        Some(Local(slot as *mut T))
    }
}

// ── commands ─────────────────────────────────────────────────────────────────────

/// Components to spawn or insert: one component, or a tuple of them.
pub trait Bundle {
    #[doc(hidden)]
    fn write(self, out: &mut Vec<(&'static str, String)>);
    #[doc(hidden)]
    fn paths(out: &mut Vec<&'static str>);
}

impl<T: Component> Bundle for T {
    fn write(self, out: &mut Vec<(&'static str, String)>) {
        out.push((T::PATH, to_json(&self)));
    }
    fn paths(out: &mut Vec<&'static str>) {
        out.push(T::PATH);
    }
}

/// One component as its type path + JSON, for a bridge that receives data typed
/// elsewhere (a script engine): `cmds.spawn(RawComponent::new("cuo:ui/text", json))`.
/// The path must be one the client registers. A typed mod never needs this.
pub struct RawComponent {
    path: &'static str,
    json: String,
}

impl RawComponent {
    pub fn new(path: &str, json: impl Into<String>) -> RawComponent {
        thread_local!(static PATHS: std::cell::RefCell<std::collections::HashSet<&'static str>> = Default::default());
        let path = PATHS.with(|p| {
            let mut p = p.borrow_mut();
            match p.get(path) {
                Some(&interned) => interned,
                None => {
                    let leaked: &'static str = Box::leak(path.to_owned().into_boxed_str());
                    p.insert(leaked);
                    leaked
                }
            }
        });
        RawComponent { path, json: json.into() }
    }
}

/// `paths()` is empty: a raw bundle names its paths at runtime, so it can't drive
/// `remove::<B>()`.
impl Bundle for RawComponent {
    fn write(self, out: &mut Vec<(&'static str, String)>) {
        out.push((self.path, self.json));
    }
    fn paths(_: &mut Vec<&'static str>) {}
}

impl Bundle for Vec<RawComponent> {
    fn write(self, out: &mut Vec<(&'static str, String)>) {
        out.extend(self.into_iter().map(|c| (c.path, c.json)));
    }
    fn paths(_: &mut Vec<&'static str>) {}
}

impl Bundle for () {
    fn write(self, _: &mut Vec<(&'static str, String)>) {}
    fn paths(_: &mut Vec<&'static str>) {}
}

macro_rules! bundle_tuple {
    ($($B:ident),*) => {
        impl<$($B: Bundle),*> Bundle for ($($B,)*) {
            #[allow(non_snake_case)]
            fn write(self, out: &mut Vec<(&'static str, String)>) {
                let ($($B,)*) = self;
                $($B.write(out);)*
            }
            fn paths(out: &mut Vec<&'static str>) {
                $($B::paths(out);)*
            }
        }
    };
}
bundle_tuple!(A);
bundle_tuple!(A, B);
bundle_tuple!(A, B, C);
bundle_tuple!(A, B, C, D);
bundle_tuple!(A, B, C, D, E);
bundle_tuple!(A, B, C, D, E, F);
bundle_tuple!(A, B, C, D, E, F, G);
bundle_tuple!(A, B, C, D, E, F, G, H);
bundle_tuple!(A, B, C, D, E, F, G, H, I);
bundle_tuple!(A, B, C, D, E, F, G, H, I, J);
bundle_tuple!(A, B, C, D, E, F, G, H, I, J, K);
bundle_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

/// Changes to the world: spawn, insert, remove, despawn, send events. Applied in
/// order after the system returns.
pub struct Commands {
    sink: backend::CmdSink,
}

impl Commands {
    /// Spawns an entity with `bundle` (one component or a tuple).
    pub fn spawn(&mut self, bundle: impl Bundle) -> EntityCommands<'_> {
        let mut comps = Vec::new();
        bundle.write(&mut comps);
        let id = Entity(backend::spawn(&self.sink, comps));
        EntityCommands { id, cmds: self }
    }

    /// Commands for an existing entity.
    pub fn entity(&mut self, entity: Entity) -> EntityCommands<'_> {
        EntityCommands { id: entity, cmds: self }
    }

    /// Sends an event: observers of it fire, `EventReader`s see it.
    pub fn send<E: Component>(&mut self, event: E) {
        backend::send(&self.sink, E::PATH, to_json(&event));
    }
}

impl SystemParam for Commands {
    fn declare(params: &mut Vec<ParamDesc>) {
        params.push(ParamDesc::Commands);
    }
    fn fetch(ctx: &mut ParamCtx) -> Option<Self> {
        match ctx.take() {
            Fetched::Commands(sink) => Some(Commands { sink }),
            _ => None,
        }
    }
}

pub struct EntityCommands<'c> {
    id: Entity,
    cmds: &'c mut Commands,
}

impl EntityCommands<'_> {
    pub fn id(&self) -> Entity {
        self.id
    }

    /// Adds or overwrites components.
    pub fn insert(&mut self, bundle: impl Bundle) -> &mut Self {
        let mut comps = Vec::new();
        bundle.write(&mut comps);
        backend::insert(&self.cmds.sink, self.id.0, comps);
        self
    }

    /// Removes the components of bundle type `B`: `.remove::<(Hue, Amount)>()`.
    pub fn remove<B: Bundle>(&mut self) -> &mut Self {
        let mut paths = Vec::new();
        B::paths(&mut paths);
        backend::remove(&self.cmds.sink, self.id.0, &paths);
        self
    }

    /// Spawns a child of this entity (adds `ChildOf`) with `bundle`.
    pub fn with_child(&mut self, bundle: impl Bundle) -> &mut Self {
        let parent = crate::types::ChildOf { parent: self.id };
        self.cmds.spawn((bundle, parent));
        self
    }

    /// Despawns the entity and its children.
    pub fn despawn(&mut self) {
        backend::despawn(&self.cmds.sink, self.id.0);
    }
}

// ── queries ──────────────────────────────────────────────────────────────────────

/// What a query reads: `&T`, `&mut T`, or a tuple of them.
pub trait QueryData {
    #[doc(hidden)]
    type Owned: 'static;
    type Item<'a>;
    type ReadItem<'a>;
    #[doc(hidden)]
    fn terms(out: &mut Vec<Term>);
    #[doc(hidden)]
    fn decode(comps: &mut std::slice::Iter<'_, String>) -> Option<Self::Owned>;
    #[doc(hidden)]
    fn item(o: &mut Self::Owned) -> Self::Item<'_>;
    #[doc(hidden)]
    fn read(o: &Self::Owned) -> Self::ReadItem<'_>;
    /// Hands each changed `&mut` component to `out(reading index, path, json)`.
    #[doc(hidden)]
    fn write_back(o: &Self::Owned, idx: &mut u8, out: &mut dyn FnMut(u8, &'static str, String));
}

impl<T: Component> QueryData for &T {
    type Owned = T;
    type Item<'a> = &'a T;
    type ReadItem<'a> = &'a T;
    fn terms(out: &mut Vec<Term>) {
        out.push(Term { kind: TermKind::Ref, path: T::PATH });
    }
    fn decode(comps: &mut std::slice::Iter<'_, String>) -> Option<T> {
        serde_json::from_str(comps.next()?).ok()
    }
    fn item(o: &mut T) -> &T {
        o
    }
    fn read(o: &T) -> &T {
        o
    }
    fn write_back(_: &T, idx: &mut u8, _: &mut dyn FnMut(u8, &'static str, String)) {
        *idx += 1;
    }
}

impl<T: Component> QueryData for &mut T {
    type Owned = (T, String);
    type Item<'a> = &'a mut T;
    type ReadItem<'a> = &'a T;
    fn terms(out: &mut Vec<Term>) {
        out.push(Term { kind: TermKind::Mut, path: T::PATH });
    }
    fn decode(comps: &mut std::slice::Iter<'_, String>) -> Option<(T, String)> {
        let v: T = serde_json::from_str(comps.next()?).ok()?;
        let original = to_json(&v);
        Some((v, original))
    }
    fn item(o: &mut (T, String)) -> &mut T {
        &mut o.0
    }
    fn read(o: &(T, String)) -> &T {
        &o.0
    }
    fn write_back(o: &(T, String), idx: &mut u8, out: &mut dyn FnMut(u8, &'static str, String)) {
        let json = to_json(&o.0);
        if json != o.1 {
            out(*idx, T::PATH, json);
        }
        *idx += 1;
    }
}

macro_rules! data_tuple {
    ($($D:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($D: QueryData),*> QueryData for ($($D,)*) {
            type Owned = ($($D::Owned,)*);
            type Item<'a> = ($($D::Item<'a>,)*);
            type ReadItem<'a> = ($($D::ReadItem<'a>,)*);
            fn terms(out: &mut Vec<Term>) {
                $($D::terms(out);)*
            }
            fn decode(comps: &mut std::slice::Iter<'_, String>) -> Option<Self::Owned> {
                Some(($($D::decode(comps)?,)*))
            }
            fn item(o: &mut Self::Owned) -> Self::Item<'_> {
                let ($($D,)*) = o;
                ($($D::item($D),)*)
            }
            fn read(o: &Self::Owned) -> Self::ReadItem<'_> {
                let ($($D,)*) = o;
                ($($D::read($D),)*)
            }
            fn write_back(o: &Self::Owned, idx: &mut u8, out: &mut dyn FnMut(u8, &'static str, String)) {
                let ($($D,)*) = o;
                $($D::write_back($D, idx, out);)*
            }
        }
    };
}
data_tuple!(A);
data_tuple!(A, B);
data_tuple!(A, B, C);
data_tuple!(A, B, C, D);
data_tuple!(A, B, C, D, E);
data_tuple!(A, B, C, D, E, F);
data_tuple!(A, B, C, D, E, F, G);
data_tuple!(A, B, C, D, E, F, G, H);

/// Which entities a query matches, beyond having its data: `With<T>`, `Without<T>`,
/// `Changed<T>`, `Added<T>`, or a tuple of them.
pub trait QueryFilter {
    #[doc(hidden)]
    fn terms(out: &mut Vec<Term>);
}

/// The entity has `T`.
pub struct With<T>(PhantomData<T>);
/// The entity does not have `T`.
pub struct Without<T>(PhantomData<T>);
/// `T` changed since this system last ran.
pub struct Changed<T>(PhantomData<T>);
/// The entity got `T` since this system last ran.
pub struct Added<T>(PhantomData<T>);

macro_rules! filter {
    ($($F:ident => $kind:ident),*) => {$(
        impl<T: Component> QueryFilter for $F<T> {
            fn terms(out: &mut Vec<Term>) {
                out.push(Term { kind: TermKind::$kind, path: T::PATH });
            }
        }
    )*};
}
filter!(With => With, Without => Without, Changed => Changed, Added => Added);

impl QueryFilter for () {
    fn terms(_: &mut Vec<Term>) {}
}

macro_rules! filter_tuple {
    ($($F:ident),*) => {
        impl<$($F: QueryFilter),*> QueryFilter for ($($F,)*) {
            fn terms(out: &mut Vec<Term>) {
                $($F::terms(out);)*
            }
        }
    };
}
filter_tuple!(A);
filter_tuple!(A, B);
filter_tuple!(A, B, C);
filter_tuple!(A, B, C, D);
filter_tuple!(A, B, C, D, E);
filter_tuple!(A, B, C, D, E, F);

/// The term list of `Query<D, F>`. A `Changed<T>` / `Added<T>` filter on a `&T` the
/// query already reads becomes that term (one value per row, not two).
///
/// Order: `With` filters, then the other filters that still read (an unfolded
/// `Changed` / `Added`), then the data terms, then `Without`. The client builds a
/// query's rows by scanning its FIRST term's entities, so the narrowing filter (the
/// `With<Player>` of `Query<&Node, With<Player>>`) must lead, not the data (every
/// `Node` in the world). The rows carry one value per reading term in this order; the
/// leading filter values are skipped ([`filter_values`]).
pub(crate) fn query_terms<D: QueryData, F: QueryFilter>() -> Vec<Term> {
    query_layout::<D, F>().0
}

/// The terms, and how many values at the start of each row belong to filters, not `D`.
fn query_layout<D: QueryData, F: QueryFilter>() -> (Vec<Term>, usize) {
    let mut data = Vec::new();
    D::terms(&mut data);
    let mut filters = Vec::new();
    F::terms(&mut filters);
    let mut with = Vec::new();
    let mut reading = Vec::new();
    let mut without = Vec::new();
    for f in filters {
        match f.kind {
            TermKind::Changed | TermKind::Added => {
                if let Some(t) = data.iter_mut().find(|t| t.kind == TermKind::Ref && t.path == f.path) {
                    t.kind = f.kind;
                } else {
                    reading.push(f);
                }
            }
            TermKind::Without => without.push(f),
            _ => with.push(f),
        }
    }
    let skip = reading.len();
    (with.into_iter().chain(reading).chain(data).chain(without).collect(), skip)
}

struct QRow<D: QueryData> {
    entity: Entity,
    data: D::Owned,
    sink: backend::RowSink,
}

/// The entities that have `D` (and pass `F`), with their components. Iterate with
/// `for (entity, data) in &query` (read) or `&mut query` (when `D` has `&mut T`;
/// changes are written back when the system returns).
pub struct Query<D: QueryData, F: QueryFilter = ()> {
    rows: Vec<QRow<D>>,
    /// Leading filter values per row (see `query_layout`): `D`'s reading index base.
    skip: u8,
    _f: PhantomData<F>,
}

impl<D: QueryData, F: QueryFilter> Query<D, F> {
    pub fn iter(&self) -> Iter<'_, D> {
        Iter(self.rows.iter())
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, D> {
        IterMut(self.rows.iter_mut())
    }
    pub fn get(&self, entity: Entity) -> Option<D::ReadItem<'_>> {
        self.rows.iter().find(|r| r.entity == entity).map(|r| D::read(&r.data))
    }
    pub fn get_mut(&mut self, entity: Entity) -> Option<D::Item<'_>> {
        self.rows.iter_mut().find(|r| r.entity == entity).map(|r| D::item(&mut r.data))
    }
    pub fn contains(&self, entity: Entity) -> bool {
        self.rows.iter().any(|r| r.entity == entity)
    }
    /// The only match; `None` when there are none or several.
    pub fn single(&self) -> Option<(Entity, D::ReadItem<'_>)> {
        match self.rows.as_slice() {
            [r] => Some((r.entity, D::read(&r.data))),
            _ => None,
        }
    }
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

impl<D: QueryData, F: QueryFilter> Drop for Query<D, F> {
    fn drop(&mut self) {
        for r in &self.rows {
            let mut idx = self.skip;
            D::write_back(&r.data, &mut idx, &mut |i, path, json| {
                backend::row_set(&r.sink, r.entity.0, i, path, json)
            });
        }
    }
}

pub struct Iter<'q, D: QueryData>(std::slice::Iter<'q, QRow<D>>);
impl<'q, D: QueryData> Iterator for Iter<'q, D> {
    type Item = (Entity, D::ReadItem<'q>);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|r| (r.entity, D::read(&r.data)))
    }
}

pub struct IterMut<'q, D: QueryData>(std::slice::IterMut<'q, QRow<D>>);
impl<'q, D: QueryData> Iterator for IterMut<'q, D> {
    type Item = (Entity, D::Item<'q>);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|r| (r.entity, D::item(&mut r.data)))
    }
}

impl<'q, D: QueryData, F: QueryFilter> IntoIterator for &'q Query<D, F> {
    type Item = (Entity, D::ReadItem<'q>);
    type IntoIter = Iter<'q, D>;
    fn into_iter(self) -> Iter<'q, D> {
        self.iter()
    }
}
impl<'q, D: QueryData, F: QueryFilter> IntoIterator for &'q mut Query<D, F> {
    type Item = (Entity, D::Item<'q>);
    type IntoIter = IterMut<'q, D>;
    fn into_iter(self) -> IterMut<'q, D> {
        self.iter_mut()
    }
}

impl<D: QueryData + 'static, F: QueryFilter + 'static> SystemParam for Query<D, F> {
    fn declare(params: &mut Vec<ParamDesc>) {
        params.push(ParamDesc::Query(query_terms::<D, F>()));
    }
    fn fetch(ctx: &mut ParamCtx) -> Option<Self> {
        let raw = match ctx.take() {
            Fetched::Query(rows) => rows,
            _ => Vec::new(),
        };
        let skip = query_layout::<D, F>().1;
        let rows = raw
            .into_iter()
            .filter_map(|r| {
                let mut comps = r.comps.iter();
                comps.by_ref().take(skip).for_each(drop);
                let data = D::decode(&mut comps)?;
                Some(QRow { entity: Entity(r.entity), data, sink: r.sink })
            })
            .collect();
        Some(Query { rows, skip: skip as u8, _f: PhantomData })
    }
}

// ── observers ────────────────────────────────────────────────────────────────────

/// `On<Add, T>`: `T` was added to an entity.
pub struct Add;
/// `On<Remove, T>`: `T` was removed from an entity (or it despawned).
pub struct Remove;
/// `On<Event, T>`: the event `T` was sent.
pub struct Event;

#[doc(hidden)]
pub trait TriggerKind {
    fn desc(path: &'static str) -> TriggerDesc;
}
impl TriggerKind for Add {
    fn desc(path: &'static str) -> TriggerDesc {
        TriggerDesc::Add(path)
    }
}
impl TriggerKind for Remove {
    fn desc(path: &'static str) -> TriggerDesc {
        TriggerDesc::Remove(path)
    }
}
impl TriggerKind for Event {
    fn desc(path: &'static str) -> TriggerDesc {
        TriggerDesc::Event(path)
    }
}

/// An observer's first parameter: what fired it.
pub struct On<K, T> {
    entity: Entity,
    event: T,
    _k: PhantomData<K>,
}

impl<K, T> On<K, T> {
    /// The entity it happened to (`Add` / `Remove`, and events aimed at an entity,
    /// like `UiClick`).
    pub fn entity(&self) -> Entity {
        self.entity
    }
    /// The component (`Add` / `Remove`) or the event.
    pub fn event(&self) -> &T {
        &self.event
    }
}

impl<K, T> Deref for On<K, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.event
    }
}

impl<K: TriggerKind, T: Component> On<K, T> {
    fn from_trigger(t: &TriggerData) -> Option<Self> {
        // Marker components cross as an empty payload.
        let json = if t.value.is_empty() { "{}" } else { t.value.as_str() };
        let event = serde_json::from_str(json).ok()?;
        Some(On { entity: Entity(t.entity), event, _k: PhantomData })
    }
}

// ── fn → system / observer ───────────────────────────────────────────────────────

/// A function whose parameters are all [`SystemParam`]s.
pub trait IntoSystem<M> {
    #[doc(hidden)]
    fn into_system(self) -> Built;
}

/// A function whose first parameter is `On<K, T>` and the rest [`SystemParam`]s.
pub trait IntoObserver<M> {
    #[doc(hidden)]
    fn into_observer(self) -> (TriggerDesc, Built);
}

macro_rules! fn_params {
    ($($P:ident),*) => {
        impl<Func, $($P: SystemParam + 'static),*> IntoSystem<fn($($P,)*)> for Func
        where
            Func: FnMut($($P),*) + 'static,
        {
            #[allow(non_snake_case, unused_mut, unused_variables)]
            fn into_system(mut self) -> Built {
                let mut params = Vec::new();
                $($P::declare(&mut params);)*
                Built {
                    name: std::any::type_name::<Func>(),
                    params,
                    after: Vec::new(),
                    before: Vec::new(),
                    run: Box::new(move |ctx: &mut ParamCtx, _: Option<&TriggerData>| {
                        $(let Some($P) = $P::fetch(ctx) else { return };)*
                        self($($P),*);
                    }),
                }
            }
        }

        impl<Func, K: TriggerKind + 'static, T: Component, $($P: SystemParam + 'static),*>
            IntoObserver<(K, T, fn($($P,)*))> for Func
        where
            Func: FnMut(On<K, T>, $($P),*) + 'static,
        {
            #[allow(non_snake_case, unused_mut, unused_variables)]
            fn into_observer(mut self) -> (TriggerDesc, Built) {
                let mut params = Vec::new();
                $($P::declare(&mut params);)*
                let built = Built {
                    name: std::any::type_name::<Func>(),
                    params,
                    after: Vec::new(),
                    before: Vec::new(),
                    run: Box::new(move |ctx: &mut ParamCtx, trigger: Option<&TriggerData>| {
                        let Some(on) = trigger.and_then(On::<K, T>::from_trigger) else { return };
                        $(let Some($P) = $P::fetch(ctx) else { return };)*
                        self(on, $($P),*);
                    }),
                };
                (K::desc(T::PATH), built)
            }
        }
    };
}
fn_params!();
fn_params!(P1);
fn_params!(P1, P2);
fn_params!(P1, P2, P3);
fn_params!(P1, P2, P3, P4);
fn_params!(P1, P2, P3, P4, P5);
fn_params!(P1, P2, P3, P4, P5, P6);
fn_params!(P1, P2, P3, P4, P5, P6, P7);
fn_params!(P1, P2, P3, P4, P5, P6, P7, P8);

/// `.after(other)` / `.before(other)` on a system fn: run it after / before `other`
/// (a system added to the same schedule, by its fn).
pub trait SystemOrder<M>: IntoSystem<M> + Sized {
    fn after<M2>(self, other: impl IntoSystem<M2>) -> Ordered<Self, M> {
        Ordered::new(self).after(other)
    }
    fn before<M2>(self, other: impl IntoSystem<M2>) -> Ordered<Self, M> {
        Ordered::new(self).before(other)
    }
}
impl<M, S: IntoSystem<M>> SystemOrder<M> for S {}

/// A system with ordering constraints; chain more `.after` / `.before`.
pub struct Ordered<S, M> {
    system: S,
    after: Vec<&'static str>,
    before: Vec<&'static str>,
    _m: PhantomData<fn() -> M>,
}

fn type_name_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

impl<S: IntoSystem<M>, M> Ordered<S, M> {
    fn new(system: S) -> Self {
        Ordered { system, after: Vec::new(), before: Vec::new(), _m: PhantomData }
    }
    pub fn after<M2>(mut self, other: impl IntoSystem<M2>) -> Self {
        self.after.push(type_name_of(&other));
        self
    }
    pub fn before<M2>(mut self, other: impl IntoSystem<M2>) -> Self {
        self.before.push(type_name_of(&other));
        self
    }
}

#[doc(hidden)]
pub struct OrderedMarker;

impl<S: IntoSystem<M>, M> IntoSystem<(OrderedMarker, M)> for Ordered<S, M> {
    fn into_system(self) -> Built {
        let mut built = self.system.into_system();
        built.after = self.after;
        built.before = self.before;
        built
    }
}

/// One system, or a tuple of systems.
pub trait IntoSystems<M> {
    #[doc(hidden)]
    fn add_to(self, app: &mut App, schedule: Schedule);
}

#[doc(hidden)]
pub struct One;
#[doc(hidden)]
pub struct Many;

impl<M, S: IntoSystem<M>> IntoSystems<(One, M)> for S {
    fn add_to(self, app: &mut App, schedule: Schedule) {
        app.push(self.into_system(), EntryKind::System(schedule));
    }
}

macro_rules! systems_tuple {
    ($($S:ident $M:ident),*) => {
        impl<$($M, $S: IntoSystem<$M>),*> IntoSystems<(Many, $($M,)*)> for ($($S,)*) {
            #[allow(non_snake_case)]
            fn add_to(self, app: &mut App, schedule: Schedule) {
                let ($($S,)*) = self;
                $(app.push($S.into_system(), EntryKind::System(schedule));)*
            }
        }
    };
}
systems_tuple!(S1 M1, S2 M2);
systems_tuple!(S1 M1, S2 M2, S3 M3);
systems_tuple!(S1 M1, S2 M2, S3 M3, S4 M4);
systems_tuple!(S1 M1, S2 M2, S3 M3, S4 M4, S5 M5);
systems_tuple!(S1 M1, S2 M2, S3 M3, S4 M4, S5 M5, S6 M6);
systems_tuple!(S1 M1, S2 M2, S3 M3, S4 M4, S5 M5, S6 M6, S7 M7);
systems_tuple!(S1 M1, S2 M2, S3 M3, S4 M4, S5 M5, S6 M6, S7 M7, S8 M8);

fn to_json<T: Serialize>(v: &T) -> String {
    serde_json::to_string(v).expect("component serialize")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChatMessage, ChildOf, Item, UiClick};
    use crate::types::{GameStateDto, Graphic, Hue, Time};

    fn sys(
        _q: Query<(&Graphic, &mut Hue), (With<Item>, Changed<Graphic>)>,
        _r: Res<Time>,
        _rm: Option<ResMut<GameStateDto>>,
        _e: EventReader<ChatMessage>,
        _l: Local<u32>,
        _c: Commands,
    ) {
    }

    fn t(kind: TermKind, path: &'static str) -> Term {
        Term { kind, path }
    }

    #[test]
    fn system_params_are_the_declaration() {
        let built = sys.into_system();
        assert_eq!(
            built.params,
            vec![
                // Changed<Graphic> folds into the &Graphic it filters; With stays a
                // filter and leads (the client scans the first term's entities).
                ParamDesc::Query(vec![
                    t(TermKind::With, "cuo:ent/is-item"),
                    t(TermKind::Changed, "cuo:ent/graphic"),
                    t(TermKind::Mut, "cuo:ent/hue"),
                ]),
                ParamDesc::Res("cuo:engine/time"),
                ParamDesc::ResMut("cuo:game/state"),
                ParamDesc::Events("cuo:chat/message"),
                // Local is guest-side only: no wire param.
                ParamDesc::Commands,
            ]
        );
        assert!(built.name.ends_with("sys"));
    }

    #[test]
    fn filters_that_read_nothing_else_stay_terms() {
        assert_eq!(
            query_terms::<&Hue, (Changed<Graphic>, Without<ChildOf>, Added<Hue>)>(),
            vec![
                t(TermKind::Changed, "cuo:ent/graphic"),
                t(TermKind::Added, "cuo:ent/hue"),
                t(TermKind::Without, "cuo:ecs/child-of"),
            ]
        );
        assert_eq!(query_layout::<&Hue, (Changed<Graphic>, Without<ChildOf>, Added<Hue>)>().1, 1);
    }

    #[test]
    fn observer_trigger_is_the_first_param() {
        fn on_click(_on: On<Event, UiClick>, _q: Query<&Graphic>, _c: Commands) {}
        fn on_added(_on: On<Add, Graphic>) {}
        let (trigger, built) = on_click.into_observer();
        assert_eq!(trigger, TriggerDesc::Event("cuo:ui/click"));
        assert_eq!(
            built.params,
            vec![ParamDesc::Query(vec![t(TermKind::Ref, "cuo:ent/graphic")]), ParamDesc::Commands]
        );
        assert_eq!(on_added.into_observer().0, TriggerDesc::Add("cuo:ent/graphic"));
    }

    #[test]
    fn app_registers_tuples_in_order_with_unique_names() {
        fn a(_: Commands) {}
        fn b(_: Res<Time>) {}
        let mut app = App::default();
        app.add_systems(Schedule::Update, (a, b)).add_systems(Schedule::Last, a);
        let names: Vec<_> = app.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names[0].ends_with("::a") && names[1].ends_with("::b"));
        assert_ne!(names[0], names[2]);
        assert!(matches!(app.entries[2].kind, EntryKind::System(Schedule::Last)));
    }

    #[test]
    fn after_and_before_name_the_other_fn() {
        fn a(_: Commands) {}
        fn b(_: Res<Time>) {}
        fn c() {}
        let mut app = App::default();
        app.add_systems(Schedule::Update, (b.after(a).before(c), a, c));
        assert!(app.entries[0].after[0].ends_with("::a"));
        assert_eq!(app.resolve_order(&app.entries[0].after), vec![1]);
        assert_eq!(app.resolve_order(&app.entries[0].before), vec![2]);
    }

    #[test]
    fn run_decodes_params_skips_on_missing_res_and_keeps_locals() {
        use std::cell::Cell;
        thread_local!(static SEEN: Cell<(u32, u16, usize)> = const { Cell::new((0, 0, 0)) });
        fn counted(q: Query<&Graphic>, time: Res<Time>, mut runs: Local<u32>) {
            *runs += 1;
            let g = q.iter().map(|(_, g)| g.value).sum();
            SEEN.with(|s| s.set((*runs, g, time.total as usize)));
        }
        let mut app = App::default();
        app.add_systems(Schedule::Update, counted);
        let fetched = |res: Option<&str>| {
            vec![
                Fetched::Query(vec![RawRow {
                    entity: 9,
                    comps: vec![r#"{"Value":3}"#.into()],
                    sink: test_sink(),
                }]),
                Fetched::Res(res.map(String::from), test_res_sink()),
            ]
        };
        let e = &mut app.entries[0];
        e.run(fetched(Some(r#"{"Total":7.0,"Frame":0.016}"#)), None);
        e.run(fetched(None), None); // no Time: skipped, Local untouched
        e.run(fetched(Some(r#"{"Total":8.0,"Frame":0.016}"#)), None);
        assert_eq!(SEEN.with(|s| s.get()), (2, 3, 8));
    }

    #[test]
    fn leading_filter_values_are_skipped_when_decoding() {
        use std::cell::Cell;
        thread_local!(static HUE: Cell<u16> = const { Cell::new(0) });
        fn read(q: Query<&Hue, Changed<Graphic>>) {
            HUE.with(|h| h.set(q.iter().map(|(_, h)| h.value).sum()));
        }
        let mut app = App::default();
        app.add_systems(Schedule::Update, read);
        let row = RawRow { entity: 1, comps: vec![r#"{"Value":7}"#.into(), r#"{"Value":42}"#.into()], sink: test_sink() };
        app.entries[0].run(vec![Fetched::Query(vec![row])], None);
        assert_eq!(HUE.with(|h| h.get()), 42);
    }

    #[cfg(feature = "p1")]
    fn test_sink() -> backend::RowSink {
        backend::RowSink
    }
    #[cfg(feature = "p1")]
    fn test_res_sink() -> backend::ResSink {
        backend::ResSink
    }
}
