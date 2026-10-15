//! wasm32-wasip2 backend: a component implementing the `cuo:modding/mod` world
//! (wit/cuo-mod.wit) with wit-bindgen. `setup` declares the App's systems; each system
//! is its own export of the mod's world (`#[system]` generates it), which lands in
//! `run_system` / `run_observer` / `run_packet_observer` with the typed handles. Every
//! value crosses through the typed `cuo:modding/components` functions (typed_gen.rs).

#[doc(hidden)]
#[allow(clippy::all, dead_code, unused)]
pub mod bindings {
    // Every type, used or not: verdict and the trigger records only appear in system
    // exports, which `#[system]` generates against these bindings.
    wit_bindgen::generate!({
        path: "../wit",
        world: "cuo:modding/mod",
        generate_all,
        generate_unused_types: true,
        // `cuo:modding/types` records: a `&mut` write-back compares them.
        additional_derives: [PartialEq],
    });
}

use crate::ecs::{
    App, Component, EntryKind, Fetched, Fired, GameEvent, Packet, PacketDirection, ParamDesc, ResData, Resource,
    Schedule, TermKind, TriggerDesc, Verdict, WritableComponent, WritableResource,
};
use bindings::cuo::modding::components;
use bindings::tinyecs::modding::ecs as wit;
use core::ptr::addr_of_mut;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;

pub use bindings::cuo::modding::{actions, assets, host, net, packets};

#[doc(hidden)]
pub struct CmdSink(pub(crate) wit::Commands);

/// A query parameter: the host's handle, or (unit tests) the columns it would return.
#[doc(hidden)]
pub struct QuerySink {
    pub(crate) handle: Option<wit::Query>,
    fake_entities: Vec<u64>,
    /// Per reading term: a `Vec<T>` column.
    fake_cols: RefCell<Vec<Option<Box<dyn Any>>>>,
    /// Rows (`entities().len()`): a tag's column is that many values.
    pub(crate) count: usize,
}

impl QuerySink {
    pub(crate) fn new(q: wit::Query) -> QuerySink {
        QuerySink { handle: Some(q), fake_entities: Vec::new(), fake_cols: RefCell::new(Vec::new()), count: 0 }
    }
    /// A host-less query (unit tests): these entities, and per reading term its column
    /// (a `Vec<T>`).
    #[doc(hidden)]
    pub fn fake(entities: Vec<u64>, cols: Vec<Box<dyn Any>>) -> QuerySink {
        let cols = cols.into_iter().map(Some).collect();
        QuerySink { handle: None, fake_entities: entities, fake_cols: RefCell::new(cols), count: 0 }
    }
    /// The matched entities, in column order.
    pub(crate) fn entities(&self) -> Vec<u64> {
        match &self.handle {
            Some(q) => q.entities(),
            None => self.fake_entities.clone(),
        }
    }
    /// Reading term `term`'s column.
    pub(crate) fn column<T: Component>(&self, term: u8) -> Vec<T> {
        match &self.handle {
            Some(q) => T::__column(q, term, self.count),
            None => self
                .fake_cols
                .borrow_mut()
                .get_mut(term as usize)
                .and_then(Option::take)
                .and_then(|c| c.downcast::<Vec<T>>().ok())
                .map(|c| *c)
                .unwrap_or_default(),
        }
    }
    /// Writes `v` back through `mut` reading term `term`.
    pub(crate) fn set<T: WritableComponent>(&self, entity: u64, term: u8, v: &T) {
        if let Some(q) = &self.handle {
            v.__set(q, entity, term)
        }
    }
}

/// A resource parameter. `None` only in unit tests (no host).
#[doc(hidden)]
pub struct ResSink(pub(crate) Option<wit::Res>);

impl ResSink {
    pub(crate) fn get<T: Resource>(&self) -> Option<T> {
        T::__get(self.0.as_ref()?)
    }
    pub(crate) fn put<T: WritableResource>(&self, v: &T) {
        if let Some(r) = &self.0 {
            v.__put(r)
        }
    }
}

/// An event-reader parameter. `None` only in unit tests (no host: no events).
#[doc(hidden)]
pub struct EventsSink(pub(crate) Option<wit::Events>);

impl EventsSink {
    pub(crate) fn read<T: GameEvent>(&self) -> Vec<T> {
        self.0.as_ref().map(T::__read).unwrap_or_default()
    }
}

/// Collects one entity's inserts into the entity builder, opened on the first one. A
/// host-less one (unit tests) records the paths instead.
#[doc(hidden)]
pub struct Inserter<'a> {
    cmds: Option<&'a wit::Commands>,
    /// The entity to insert on; `None` = spawn one.
    target: Option<u64>,
    builder: Option<components::EntityBuilder>,
    pub(crate) paths: Vec<&'static str>,
}

impl<'a> Inserter<'a> {
    pub(crate) fn new(cmds: Option<&'a wit::Commands>, target: Option<u64>) -> Inserter<'a> {
        Inserter { cmds, target, builder: None, paths: Vec::new() }
    }

    pub(crate) fn push<T: WritableComponent>(&mut self, v: &T) {
        self.paths.push(T::PATH);
        let Some(c) = self.cmds else { return };
        let b = match self.builder.take() {
            Some(b) => b,
            None => match self.target {
                Some(e) => components::entity_of(c, e),
                None => components::spawn(c),
            },
        };
        self.builder = Some(v.__insert(&b));
    }

    /// The entity: a spawn with nothing inserted is a bare `commands.spawn`.
    pub(crate) fn finish(self) -> u64 {
        if let Some(b) = &self.builder {
            return b.id();
        }
        match (self.target, self.cmds) {
            (Some(e), _) => e,
            (None, Some(c)) => c.spawn(),
            (None, None) => 0,
        }
    }
}

pub(crate) fn remove(c: &CmdSink, entity: u64, paths: &[&'static str]) {
    let paths: Vec<String> = paths.iter().map(|p| p.to_string()).collect();
    c.0.remove(entity, &paths)
}
pub(crate) fn despawn(c: &CmdSink, entity: u64) {
    c.0.despawn(entity)
}
pub(crate) fn send<E: GameEvent>(c: &CmdSink, event: &E) {
    event.__send(&c.0)
}
pub(crate) fn set_resource<R: WritableResource>(c: &CmdSink, value: &R) {
    value.__set_resource(&c.0)
}

struct State {
    app: App,
    by_name: HashMap<String, usize>,
}

static mut STATE: Option<State> = None;

fn state() -> &'static mut State {
    // SAFETY: single-threaded component; exports are never re-entered.
    unsafe {
        (*addr_of_mut!(STATE)).get_or_insert_with(|| State { app: App::default(), by_name: HashMap::new() })
    }
}

#[cfg(target_family = "wasm")]
extern "Rust" {
    fn __cuo_mod_setup(app: &mut App);
}

fn term(t: &crate::ecs::Term) -> wit::Term {
    let p = t.path.to_string();
    match t.kind {
        TermKind::Ref => wit::Term::Ref(p),
        TermKind::Mut => wit::Term::Mut(p),
        TermKind::With => wit::Term::With(p),
        TermKind::Without => wit::Term::Without(p),
        TermKind::Changed => wit::Term::Changed(p),
        TermKind::Added => wit::Term::Added(p),
    }
}

fn schedule(s: Schedule) -> wit::Schedule {
    match s {
        Schedule::Startup => wit::Schedule::Startup,
        Schedule::First => wit::Schedule::First,
        Schedule::PreUpdate => wit::Schedule::PreUpdate,
        Schedule::Update => wit::Schedule::Update,
        Schedule::PostUpdate => wit::Schedule::PostUpdate,
        Schedule::Last => wit::Schedule::Last,
    }
}

/// One wire parameter of a system's export, as the host passed it.
#[doc(hidden)]
pub enum Param {
    Commands(wit::Commands),
    Query(wit::Query),
    Res(wit::Res),
    Events(wit::Events),
}

fn fetch(params: Vec<Param>) -> Vec<Fetched> {
    params
        .into_iter()
        .map(|p| match p {
            Param::Commands(c) => Fetched::Commands(CmdSink(c)),
            // The query reads its columns itself: which depends on its types.
            Param::Query(q) => Fetched::Query(QuerySink::new(q)),
            Param::Res(r) => {
                let data = if r.unchanged() { ResData::Unchanged } else { ResData::Changed };
                Fetched::Res(data, ResSink(Some(r)))
            }
            Param::Events(e) => Fetched::Events(EventsSink(Some(e))),
        })
        .collect()
}

fn run(name: &str, params: Vec<Param>, trigger: Option<Fired>) -> Verdict {
    let st = state();
    let Some(&i) = st.by_name.get(name) else { return Verdict::Pass };
    let mut app = std::mem::take(&mut st.app);
    let entry = &mut app.entries[i];
    let fetched = fetch(params);
    let verdict = entry.run(fetched, trigger);
    state().app = app;
    verdict
}

fn wit_direction(d: PacketDirection) -> wit::PacketDirection {
    match d {
        PacketDirection::Incoming => wit::PacketDirection::Incoming,
        PacketDirection::Outgoing => wit::PacketDirection::Outgoing,
    }
}

struct Mod;

#[cfg(target_family = "wasm")]
impl bindings::Guest for Mod {
    fn setup(wit_app: wit::App) {
        let mut app = App::default();
        unsafe { __cuo_mod_setup(&mut app) };
        let mut by_name = HashMap::new();
        // Every handle first: `after` / `before` borrow the other system.
        let systems: Vec<wit::System> = app.entries.iter().map(|e| wit::System::new(&e.name)).collect();
        for (i, e) in app.entries.iter().enumerate() {
            let sys = &systems[i];
            for j in app.resolve_order(&e.after) {
                sys.after(&systems[j]);
            }
            for j in app.resolve_order(&e.before) {
                sys.before(&systems[j]);
            }
            if e.run_on_change {
                sys.run_on_change();
            }
            for p in &e.params {
                match p {
                    ParamDesc::Commands => sys.add_commands(),
                    ParamDesc::Query(terms) => {
                        let terms: Vec<_> = terms.iter().map(term).collect();
                        sys.add_query(&terms)
                    }
                    ParamDesc::Res(path) => sys.add_res(path),
                    ParamDesc::ResMut(path) => sys.add_res_mut(path),
                    ParamDesc::Events(path) => sys.add_events(path),
                }
            }
            match &e.kind {
                EntryKind::System(s) => wit_app.add_systems(schedule(*s), &[sys]),
                EntryKind::Observer(t) => {
                    let trigger = match t {
                        TriggerDesc::Add(p) => wit::Trigger::OnAdd(p.to_string()),
                        TriggerDesc::Remove(p) => wit::Trigger::OnRemove(p.to_string()),
                        TriggerDesc::Event(p) => wit::Trigger::OnEvent(p.to_string()),
                        TriggerDesc::Packet(dir, ids) => {
                            wit::Trigger::OnPacket(wit::PacketFilter { direction: wit_direction(*dir), ids: ids.clone() })
                        }
                    };
                    wit_app.add_observer(&trigger, sys)
                }
            }
            by_name.entry(e.name.clone()).or_insert(i);
        }
        let st = state();
        st.app = app;
        st.by_name = by_name;
    }
}

/// A scheduled system's export (`#[system]` glue).
#[doc(hidden)]
pub fn run_system(name: &str, params: Vec<Param>) {
    run(name, params, None);
}

/// An observer's export (`#[system]` glue): the trigger value already lifted into the
/// SDK type (`typed::trigger_value` / `trigger_tag`).
#[doc(hidden)]
pub fn run_observer(name: &str, entity: u64, value: Box<dyn Any>, params: Vec<Param>) {
    run(name, params, Some(Fired::Value { entity, value }));
}

/// A packet observer's export (`#[system]` glue).
#[doc(hidden)]
pub fn run_packet_observer(name: &str, direction: wit::PacketDirection, packet: Vec<u8>, params: Vec<Param>) -> wit::Verdict {
    let direction = match direction {
        wit::PacketDirection::Incoming => PacketDirection::Incoming,
        wit::PacketDirection::Outgoing => PacketDirection::Outgoing,
    };
    match run(name, params, Some(Fired::Packet(Packet::new(direction, packet)))) {
        Verdict::Pass => wit::Verdict::Pass,
        Verdict::Block => wit::Verdict::Block,
        Verdict::Replace(bytes) => wit::Verdict::Replace(bytes),
    }
}

#[cfg(target_family = "wasm")]
bindings::export!(Mod with_types_in bindings);
