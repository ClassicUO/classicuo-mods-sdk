//! wasm32-wasip2 backend: a component implementing the `cuo:modding/mod` world
//! (wit/cuo-mod.wit) with wit-bindgen. `setup` declares the App's systems; each system
//! is its own export of the mod's world (`#[system]` generates it), which lands in
//! `run_system` / `run_observer` / `run_packet_observer` with the typed handles.

#[doc(hidden)]
#[allow(clippy::all, dead_code, unused)]
pub mod bindings {
    // Every type, used or not: trigger-data / verdict only appear in system exports,
    // which `#[system]` generates against these bindings.
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
    App, BundleItem, Component, EntryKind, Fetched, Packet, ResData, PacketDirection, ParamDesc, RawRow, Schedule, TermKind,
    TriggerData, TriggerDesc, Verdict,
};
use crate::typed;
use core::ptr::addr_of_mut;
use std::collections::HashMap;
use bindings::tinyecs::modding::ecs as wit;
use bindings::cuo::modding::components;

pub use bindings::cuo::modding::{actions, assets, host, packets};

#[doc(hidden)]
pub struct CmdSink(wit::Commands);

/// A query parameter: the host's handle, or (unit tests) the rows it would return.
#[doc(hidden)]
pub struct QuerySink {
    pub(crate) handle: Option<wit::Query>,
    pub(crate) fake: Vec<RawRow>,
    /// Rows of the typed path (`entities().len()`): a tag's column is that many defaults.
    pub(crate) count: usize,
}

impl QuerySink {
    /// A host-less query with these rows (unit tests).
    #[doc(hidden)]
    pub fn fake(rows: Vec<RawRow>) -> QuerySink {
        let count = rows.len();
        QuerySink { handle: None, fake: rows, count }
    }
    /// Every row as JSON (one host call).
    pub(crate) fn rows(&mut self) -> Vec<RawRow> {
        match &self.handle {
            Some(q) => q.rows().into_iter().map(|r| RawRow { entity: r.entity, comps: r.values }).collect(),
            None => std::mem::take(&mut self.fake),
        }
    }
    /// The matched entities, in row / column order.
    pub(crate) fn entities(&self) -> Vec<u64> {
        match &self.handle {
            Some(q) => q.entities(),
            None => self.fake.iter().map(|r| r.entity).collect(),
        }
    }
    /// Reading term `term` as `T`: typed when `T` is curated, else `None`. Without a
    /// host, the fake rows' JSON stands in for the column.
    pub(crate) fn column<T: Component>(&self, term: u8) -> Option<Vec<T>> {
        match &self.handle {
            Some(q) => typed::column::<T>(q, term, self.count),
            None if typed::has_column::<T>() => Some(
                self.fake
                    .iter()
                    .map(|r| serde_json::from_str(&r.comps[term as usize]).expect("fake column"))
                    .collect(),
            ),
            None => None,
        }
    }
}

/// `None` only in unit tests (no host).
#[doc(hidden)]
pub struct ResSink(pub(crate) Option<wit::Res>);

/// An event-reader parameter: the host's handle, or (unit tests) the events' JSON.
#[doc(hidden)]
pub struct EventsSink {
    pub(crate) handle: Option<wit::Events>,
    pub(crate) fake: Vec<String>,
}

impl EventsSink {
    /// Host-less events (unit tests).
    #[doc(hidden)]
    pub fn fake(events: Vec<String>) -> EventsSink {
        EventsSink { handle: None, fake: events }
    }
    /// Every event as `T`: typed when `T` is curated, else parsed from JSON.
    pub(crate) fn read<T: Component>(self) -> Vec<T> {
        let json = match &self.handle {
            Some(e) => match typed::read::<T>(e) {
                Some(v) => return v,
                None => e.read(),
            },
            None => self.fake,
        };
        json.iter().filter_map(|j| serde_json::from_str(if j.is_empty() { "{}" } else { j }).ok()).collect()
    }
}

/// A bundle's components in order, consecutive JSON ones grouped into one call.
fn segments(items: Vec<BundleItem>) -> impl Iterator<Item = Segment> {
    let mut items = items.into_iter().peekable();
    std::iter::from_fn(move || match items.next()? {
        BundleItem::Typed(t) => Some(Segment::Typed(t)),
        BundleItem::Json(p, j) => {
            let mut json = vec![(p.to_string(), j)];
            while let Some(BundleItem::Json(..)) = items.peek() {
                let Some(BundleItem::Json(p, j)) = items.next() else { unreachable!() };
                json.push((p.to_string(), j));
            }
            Some(Segment::Json(json))
        }
    })
}

enum Segment {
    Json(Vec<(String, String)>),
    Typed(typed::TypedInsert),
}

/// Queues `rest` on `entity`, in order: JSON runs through `commands.insert`, typed
/// components through the entity builder (`builder`: one already open for it).
fn insert_segments(c: &CmdSink, entity: u64, mut builder: Option<components::EntityBuilder>, rest: impl Iterator<Item = Segment>) {
    for seg in rest {
        match seg {
            Segment::Json(json) => c.0.insert(entity, &json),
            Segment::Typed(t) => {
                let b = builder.get_or_insert_with(|| components::entity_of(&c.0, entity));
                *b = t.apply(b);
            }
        }
    }
}

pub(crate) fn spawn(c: &CmdSink, b: Vec<BundleItem>) -> u64 {
    let mut segs = segments(b).peekable();
    match segs.next() {
        Some(Segment::Typed(t)) => {
            let b = components::spawn(&c.0);
            let id = b.id();
            let b = t.apply(&b);
            insert_segments(c, id, Some(b), segs);
            id
        }
        Some(Segment::Json(json)) => {
            let id = c.0.spawn(&json);
            insert_segments(c, id, None, segs);
            id
        }
        None => c.0.spawn(&[]),
    }
}
pub(crate) fn insert(c: &CmdSink, entity: u64, b: Vec<BundleItem>) {
    insert_segments(c, entity, None, segments(b))
}
pub(crate) fn remove(c: &CmdSink, entity: u64, paths: &[&'static str]) {
    let paths: Vec<String> = paths.iter().map(|p| p.to_string()).collect();
    c.0.remove(entity, &paths)
}
pub(crate) fn despawn(c: &CmdSink, entity: u64) {
    c.0.despawn(entity)
}
/// Sends `event`: typed when `E` is curated, else as JSON.
pub(crate) fn send<E: Component>(c: &CmdSink, event: &E) {
    if !typed::send(&c.0, event) {
        c.0.send(E::PATH, &crate::ecs::to_json(event))
    }
}
/// Writes `v` back through `mut` reading term `index`: typed when `T` is curated and
/// writable, else as JSON.
pub(crate) fn row_set<T: Component>(q: &QuerySink, entity: u64, index: u8, v: &T) {
    if let Some(q) = &q.handle {
        if !typed::set(q, entity, index, v) {
            q.set(entity, index, &crate::ecs::to_json(v))
        }
    }
}
pub(crate) fn row_set_json(q: &QuerySink, entity: u64, index: u8, json: String) {
    if let Some(q) = &q.handle {
        q.set(entity, index, &json)
    }
}
pub(crate) fn res_set(r: &ResSink, json: String) {
    if let Some(r) = &r.0 {
        r.set(&json)
    }
}
pub(crate) fn res_get(r: &ResSink) -> Option<String> {
    r.0.as_ref().and_then(|r| r.get())
}
/// The resource's current value: typed when `T` is curated, else parsed from JSON.
pub(crate) fn res_value<T: Component>(r: &ResSink) -> Option<T> {
    let h = r.0.as_ref()?;
    match typed::res_get::<T>(h) {
        Some(v) => v,
        None => serde_json::from_str(&h.get()?).ok(),
    }
}
pub(crate) fn set_resource(c: &CmdSink, path: &'static str, json: String) {
    c.0.set_resource(path, &json)
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
            // The query reads its rows / columns itself: which depends on its types.
            Param::Query(q) => Fetched::Query(QuerySink { handle: Some(q), fake: Vec::new(), count: 0 }),
            Param::Res(r) => {
                let data = if r.unchanged() { ResData::Unchanged } else { ResData::Changed };
                Fetched::Res(data, ResSink(Some(r)))
            }
            Param::Events(e) => Fetched::Events(EventsSink { handle: Some(e), fake: Vec::new() }),
        })
        .collect()
}

fn run(name: &str, params: Vec<Param>, trigger: Option<TriggerData>) -> Verdict {
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

/// An observer's export (`#[system]` glue).
#[doc(hidden)]
pub fn run_observer(name: &str, trigger: wit::TriggerData, params: Vec<Param>) {
    run(name, params, Some(TriggerData::Entity { entity: trigger.entity, value: trigger.value }));
}

/// A typed observer's export (`#[system]` glue): the trigger value already lifted into
/// the SDK type (`typed::trigger_value` / `trigger_tag`).
#[doc(hidden)]
pub fn run_observer_typed(name: &str, entity: u64, value: Box<dyn std::any::Any>, params: Vec<Param>) {
    run(name, params, Some(TriggerData::Typed { entity, value }));
}

/// A packet observer's export (`#[system]` glue).
#[doc(hidden)]
pub fn run_packet_observer(name: &str, direction: wit::PacketDirection, packet: Vec<u8>, params: Vec<Param>) -> wit::Verdict {
    let direction = match direction {
        wit::PacketDirection::Incoming => PacketDirection::Incoming,
        wit::PacketDirection::Outgoing => PacketDirection::Outgoing,
    };
    match run(name, params, Some(TriggerData::Packet(Packet::new(direction, packet)))) {
        Verdict::Pass => wit::Verdict::Pass,
        Verdict::Block => wit::Verdict::Block,
        Verdict::Replace(bytes) => wit::Verdict::Replace(bytes),
    }
}

#[cfg(target_family = "wasm")]
bindings::export!(Mod with_types_in bindings);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consecutive_json_components_share_one_call() {
        let json = |p: &'static str| BundleItem::Json(p, "{}".into());
        let typed = || BundleItem::Typed(typed::TypedInsert::Movable);
        let segs: Vec<String> = segments(vec![json("a"), json("b"), typed(), typed(), json("c")])
            .map(|s| match s {
                Segment::Json(v) => v.iter().map(|(p, _)| p.as_str()).collect::<Vec<_>>().join("+"),
                Segment::Typed(_) => "typed".into(),
            })
            .collect();
        assert_eq!(segs, ["a+b", "typed", "typed", "c"]);
    }
}
