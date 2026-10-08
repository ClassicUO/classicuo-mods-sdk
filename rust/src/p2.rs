//! wasm32-wasip2 backend: a component implementing the `cuo:modding/mod` world
//! (wit/cuo-mod.wit) with wit-bindgen. `setup` / `run` / `observe` / `observe-packet`
//! map onto the App and its systems.

#[allow(clippy::all, dead_code, unused)]
mod bindings {
    wit_bindgen::generate!({
        path: "../wit",
        world: "cuo:modding/mod",
        generate_all,
    });
}

use crate::ecs::{
    App, EntryKind, Fetched, Packet, PacketDirection, ParamDesc, RawRow, Schedule, TermKind, TriggerData, TriggerDesc,
    Verdict,
};
use core::ptr::addr_of_mut;
use std::collections::HashMap;
use bindings::tinyecs::modding::ecs as wit;

pub use bindings::cuo::modding::{actions, assets, host, packets};

#[doc(hidden)]
pub struct CmdSink(wit::Commands);
/// `None` only in unit tests (no host).
#[doc(hidden)]
pub struct QuerySink(pub(crate) Option<wit::Query>);
#[doc(hidden)]
pub struct ResSink(pub(crate) Option<wit::Res>);

fn bundle(b: Vec<(&'static str, String)>) -> Vec<(String, String)> {
    b.into_iter().map(|(p, j)| (p.to_string(), j)).collect()
}

pub(crate) fn spawn(c: &CmdSink, b: Vec<(&'static str, String)>) -> u64 {
    c.0.spawn(&bundle(b))
}
pub(crate) fn insert(c: &CmdSink, entity: u64, b: Vec<(&'static str, String)>) {
    c.0.insert(entity, &bundle(b))
}
pub(crate) fn remove(c: &CmdSink, entity: u64, paths: &[&'static str]) {
    let paths: Vec<String> = paths.iter().map(|p| p.to_string()).collect();
    c.0.remove(entity, &paths)
}
pub(crate) fn despawn(c: &CmdSink, entity: u64) {
    c.0.despawn(entity)
}
pub(crate) fn send(c: &CmdSink, path: &'static str, json: String) {
    c.0.send(path, &json)
}
pub(crate) fn row_set(q: &QuerySink, entity: u64, index: u8, json: String) {
    if let Some(q) = &q.0 {
        q.set(entity, index, &json)
    }
}
pub(crate) fn res_set(r: &ResSink, json: String) {
    if let Some(r) = &r.0 {
        r.set(&json)
    }
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

fn fetch(params: Vec<wit::Param>) -> Vec<Fetched> {
    params
        .into_iter()
        .map(|p| match p {
            wit::Param::Commands(c) => Fetched::Commands(CmdSink(c)),
            wit::Param::Query(q) => {
                // One host call per query param per run: every row at once.
                let rows = q.rows().into_iter().map(|r| RawRow { entity: r.entity, comps: r.values }).collect();
                Fetched::Query(rows, QuerySink(Some(q)))
            }
            wit::Param::Res(r) => Fetched::Res(r.get(), ResSink(Some(r))),
            wit::Param::Events(e) => Fetched::Events(e.read()),
        })
        .collect()
}

fn run(name: &str, params: Vec<wit::Param>, trigger: Option<TriggerData>) -> Verdict {
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
            by_name.insert(e.name.clone(), i);
        }
        let st = state();
        st.app = app;
        st.by_name = by_name;
    }

    fn run(system: String, params: Vec<wit::Param>) {
        run(&system, params, None);
    }

    fn observe(system: String, trigger: wit::TriggerData, params: Vec<wit::Param>) {
        run(&system, params, Some(TriggerData::Entity { entity: trigger.entity, value: trigger.value }));
    }

    fn observe_packet(
        system: String,
        direction: wit::PacketDirection,
        packet: Vec<u8>,
        params: Vec<wit::Param>,
    ) -> wit::Verdict {
        let direction = match direction {
            wit::PacketDirection::Incoming => PacketDirection::Incoming,
            wit::PacketDirection::Outgoing => PacketDirection::Outgoing,
        };
        match run(&system, params, Some(TriggerData::Packet(Packet::new(direction, packet)))) {
            Verdict::Pass => wit::Verdict::Pass,
            Verdict::Block => wit::Verdict::Block,
            Verdict::Replace(bytes) => wit::Verdict::Replace(bytes),
        }
    }
}

#[cfg(target_family = "wasm")]
bindings::export!(Mod with_types_in bindings);
