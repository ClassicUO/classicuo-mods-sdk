//! wasm32-wasip1 backend: the ECS over the FlatBuffers ABI (abi/mod-abi.fbs, push
//! model — the host pushes each run's parameter data, the guest returns a command
//! buffer), the cuo functions over `env.mod_call` (docs/p1-wire.md).

// Most of this is reached only from the wasm exports.
#![cfg_attr(not(target_family = "wasm"), allow(dead_code))]

pub mod abi;
pub mod arena;
pub mod call;
pub mod cuo;
mod wire;

use crate::ecs::{App, EntryKind, Fetched, ParamDesc, RawRow, Schedule, TermKind, TriggerData, TriggerDesc};
use abi::{Encoding, ObserverKind, ParamKind, QueryTermKind};
use core::ptr::addr_of_mut;
use planus::Builder;
use std::collections::HashMap;

/// The ABI version this SDK speaks; `mod_setup` traps on a mismatch.
pub const ABI_VERSION: u32 = 3;

const NONE_TYPE: u16 = 0xFFFF;
/// An entity spawned in this run, before the host assigned its id: `PENDING | temp_id`.
const PENDING: u64 = 1 << 63;

#[doc(hidden)]
pub struct CmdSink;
#[doc(hidden)]
pub struct RowSink;
#[doc(hidden)]
pub struct ResSink;

struct State {
    types: HashMap<String, u16>,
    app: App,
    cmds: Vec<abi::Cmd>,
    next_temp: u32,
    resolved: HashMap<u32, u64>,
}

static mut STATE: Option<State> = None;

fn state() -> &'static mut State {
    // SAFETY: single-threaded guest; the host never re-enters an export.
    unsafe {
        (*addr_of_mut!(STATE)).get_or_insert_with(|| State {
            types: HashMap::new(),
            app: App::default(),
            cmds: Vec::new(),
            next_temp: 0,
            resolved: HashMap::new(),
        })
    }
}

fn type_id(path: &str) -> u16 {
    match state().types.get(path) {
        Some(&id) => id,
        None => panic!("the client has no type path '{path}'"),
    }
}

// ── entities ─────────────────────────────────────────────────────────────────────

pub(crate) fn resolve(bits: u64) -> u64 {
    if bits & PENDING != 0 {
        if let Some(&real) = state().resolved.get(&(bits as u32)) {
            return real;
        }
    }
    bits
}

/// A command-buffer entity ref: the real id, or `-(temp_id) - 1` for an entity
/// spawned earlier in this buffer.
fn entity_ref(bits: u64) -> i64 {
    let r = resolve(bits);
    if r & PENDING != 0 {
        -((r as u32) as i64) - 1
    } else {
        r as i64
    }
}

// ── commands ─────────────────────────────────────────────────────────────────────

fn comp(path: &str, json: String) -> abi::CompValue {
    abi::CompValue { type_id: type_id(path), encoding: Encoding::Json, data: Some(json.into_bytes()) }
}

fn comps(bundle: Vec<(&'static str, String)>) -> Option<Vec<abi::CompValue>> {
    Some(bundle.into_iter().map(|(p, j)| comp(p, j)).collect())
}

fn push(cmd: abi::Cmd) {
    state().cmds.push(cmd);
}

pub(crate) fn spawn(_: &CmdSink, bundle: Vec<(&'static str, String)>) -> u64 {
    let st = state();
    let temp_id = st.next_temp;
    st.next_temp = st.next_temp.wrapping_add(1);
    push(abi::Cmd::SpawnCmd(Box::new(abi::SpawnCmd { temp_id, comps: comps(bundle) })));
    PENDING | temp_id as u64
}

pub(crate) fn insert(_: &CmdSink, entity: u64, bundle: Vec<(&'static str, String)>) {
    push(abi::Cmd::InsertCmd(Box::new(abi::InsertCmd { entity: entity_ref(entity), comps: comps(bundle) })));
}

pub(crate) fn remove(_: &CmdSink, entity: u64, paths: &[&'static str]) {
    let type_ids = Some(paths.iter().map(|p| type_id(p)).collect());
    push(abi::Cmd::RemoveCmd(Box::new(abi::RemoveCmd { entity: entity_ref(entity), type_ids })));
}

pub(crate) fn despawn(_: &CmdSink, entity: u64) {
    push(abi::Cmd::DespawnCmd(Box::new(abi::DespawnCmd { entity: entity_ref(entity) })));
}

pub(crate) fn send(_: &CmdSink, path: &'static str, json: String) {
    push(abi::Cmd::EmitEventCmd(Box::new(abi::EmitEventCmd {
        event_name: Some(path.to_string()),
        entity: 0,
        data: Some(json.into_bytes()),
    })));
}

pub(crate) fn row_set(_: &RowSink, entity: u64, _index: u8, path: &'static str, json: String) {
    push(abi::Cmd::InsertCmd(Box::new(abi::InsertCmd {
        entity: entity_ref(entity),
        comps: Some(vec![comp(path, json)]),
    })));
}

pub(crate) fn res_set(_: &ResSink, path: &'static str, json: String) {
    push(abi::Cmd::ResourceSetCmd(Box::new(abi::ResourceSetCmd { value: Some(Box::new(comp(path, json))) })));
}

/// The command buffer the last run produced, packed for the host (0 = none).
fn finish_cmds() -> u64 {
    let cmds = std::mem::take(&mut state().cmds);
    if cmds.is_empty() {
        return 0;
    }
    let cb = abi::CommandBuffer { cmds: Some(cmds) };
    arena::pack_ret(Builder::new().finish(&cb, None))
}

// ── setup ────────────────────────────────────────────────────────────────────────

fn param_decl(p: &ParamDesc) -> abi::ParamDecl {
    let simple = |kind, path: &str| abi::ParamDecl { kind, query: None, type_id: type_id(path) };
    match p {
        ParamDesc::Commands => abi::ParamDecl { kind: ParamKind::Commands, query: None, type_id: NONE_TYPE },
        ParamDesc::Query(terms) => abi::ParamDecl {
            kind: ParamKind::Query,
            query: Some(Box::new(abi::QueryDecl {
                terms: Some(
                    terms
                        .iter()
                        .map(|t| abi::QueryTerm {
                            kind: match t.kind {
                                TermKind::Ref => QueryTermKind::Ref,
                                TermKind::Mut => QueryTermKind::Mut,
                                TermKind::With => QueryTermKind::With,
                                TermKind::Without => QueryTermKind::Without,
                                TermKind::Changed => QueryTermKind::Changed,
                                TermKind::Added => QueryTermKind::Added,
                            },
                            type_id: type_id(t.path),
                        })
                        .collect(),
                ),
            })),
            type_id: NONE_TYPE,
        },
        ParamDesc::Res(p) => simple(ParamKind::Res, p),
        ParamDesc::ResMut(p) => simple(ParamKind::ResMut, p),
        ParamDesc::Events(p) => simple(ParamKind::Events, p),
    }
}

fn schedule(s: Schedule) -> abi::Schedule {
    match s {
        Schedule::Startup => abi::Schedule::ModStartup,
        Schedule::First => abi::Schedule::First,
        Schedule::PreUpdate => abi::Schedule::PreUpdate,
        Schedule::Update => abi::Schedule::Update,
        Schedule::PostUpdate => abi::Schedule::PostUpdate,
        Schedule::Last => abi::Schedule::Last,
    }
}

/// The SetupReply for an App. Entry index = system id = observer id.
pub(crate) fn setup_reply(app: &App) -> abi::SetupReply {
    let mut systems = Vec::new();
    let mut observers = Vec::new();
    for (i, e) in app.entries.iter().enumerate() {
        let params: Vec<_> = e.params.iter().map(param_decl).collect();
        match e.kind {
            EntryKind::System(s) => systems.push(abi::SystemDecl {
                id: i as u32,
                name: Some(e.name.clone()),
                schedule: schedule(s),
                custom_stage: None,
                params: Some(params),
                after: Some(app.resolve_order(&e.after).into_iter().map(|i| i as u32).collect()),
                before: Some(app.resolve_order(&e.before).into_iter().map(|i| i as u32).collect()),
                interval_ms: 0,
            }),
            EntryKind::Observer(t) => {
                let (kind, type_id, event_name) = match t {
                    TriggerDesc::Add(p) => (ObserverKind::Insert, type_id(p), None),
                    TriggerDesc::Remove(p) => (ObserverKind::Remove, type_id(p), None),
                    TriggerDesc::Event(p) => (ObserverKind::Custom, NONE_TYPE, Some(p.to_string())),
                };
                observers.push(abi::ObserverDecl { id: i as u32, kind, type_id, event_name, params: Some(params) });
            }
        }
    }
    abi::SetupReply {
        systems: Some(systems),
        observers: Some(observers),
        wants_filter: false,
        wants_filter_out: false,
        res_unchanged: true,
    }
}

// ── run ──────────────────────────────────────────────────────────────────────────

/// Copies a run's parameter data out of the input buffer (before any user code: a
/// `mod_call` can grow and move the arena).
fn fetch(params: &[ParamDesc], res_cache: &mut Vec<Option<String>>, input: &wire::Params) -> Vec<Fetched> {
    if res_cache.len() < params.len() {
        res_cache.resize(params.len(), None);
    }
    params
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let idx = i;
            let i = i as u32;
            match p {
                ParamDesc::Commands => Fetched::Commands(CmdSink),
                ParamDesc::Query(_) => Fetched::Query(
                    input
                        .rows(i)
                        .into_iter()
                        .map(|(entity, comps)| RawRow { entity, comps, sink: RowSink })
                        .collect(),
                ),
                ParamDesc::Res(_) | ParamDesc::ResMut(_) => {
                    let value = match input.resource(i) {
                        wire::ResInput::Value(json) => {
                            res_cache[idx] = Some(json.clone());
                            Some(json)
                        }
                        wire::ResInput::Unchanged => res_cache[idx].clone(),
                        wire::ResInput::Absent => {
                            res_cache[idx] = None;
                            None
                        }
                    };
                    Fetched::Res(value, ResSink)
                }
                ParamDesc::Events(_) => Fetched::Events(input.events(i)),
            }
        })
        .collect()
}

fn run(id: u32, input: &[u8], fields: wire::ParamFields, trigger: Option<TriggerData>) -> u64 {
    run_entry(id, input, fields, trigger);
    finish_cmds()
}

/// Runs an entry; its commands stay in the state until `finish_cmds`.
fn run_entry(id: u32, input: &[u8], fields: wire::ParamFields, trigger: Option<TriggerData>) {
    // Out of the state while it runs: user code reaches back into the state
    // (commands), never into the app.
    let mut app = std::mem::take(&mut state().app);
    if let Some(entry) = app.entries.get_mut(id as usize) {
        let fetched = fetch(&entry.params, &mut entry.res_cache, &wire::Params::new(input, fields));
        entry.run(fetched, trigger.as_ref());
    }
    state().app = app;
}

// ── exports ──────────────────────────────────────────────────────────────────────

#[cfg(target_family = "wasm")]
mod exports {
    use super::*;

    extern "Rust" {
        fn __cuo_mod_setup(app: &mut App);
        fn __cuo_mod_on_packet(dir: cuo::packets::Direction, packet: &[u8]) -> cuo::packets::Verdict;
    }

    #[no_mangle]
    pub extern "C" fn mod_setup(ptr: u32, len: u32) -> u64 {
        let bytes = unsafe { arena::input_slice(ptr, len) };
        let host_abi = wire::read_abi_version(bytes);
        assert!(
            host_abi == ABI_VERSION,
            "mod ABI mismatch: the client speaks v{host_abi}, this mod was built for v{ABI_VERSION}: rebuild it against the current SDK"
        );
        state().types = wire::read_type_paths(bytes).into_iter().map(|(id, p)| (p, id)).collect();
        let mut app = App::default();
        unsafe { __cuo_mod_setup(&mut app) };
        let reply = setup_reply(&app);
        state().app = app;
        arena::pack_ret(Builder::new().finish(&reply, None))
    }

    #[no_mangle]
    pub extern "C" fn mod_run(sys_id: u32, ptr: u32, len: u32) -> u64 {
        let input = unsafe { arena::input_slice(ptr, len) }.to_vec();
        run(sys_id, &input, wire::SYSTEM_INPUT, None)
    }

    #[no_mangle]
    pub extern "C" fn mod_observer(obs_id: u32, entity: u64, ptr: u32, len: u32) -> u64 {
        let input = unsafe { arena::input_slice(ptr, len) }.to_vec();
        let value = wire::observer_value(&input);
        run(obs_id, &input, wire::OBSERVER_INPUT, Some(TriggerData { entity, value }))
    }

    #[no_mangle]
    pub extern "C" fn mod_spawned(ptr: u32, len: u32) {
        let bytes = unsafe { arena::input_slice(ptr, len) };
        let st = state();
        for (temp, real) in wire::read_spawned(bytes) {
            st.resolved.insert(temp, real);
        }
    }

    #[no_mangle]
    pub extern "C" fn mod_on_packet(dir: u32, ptr: u32, len: u32) -> u64 {
        let packet = unsafe { arena::input_slice(ptr, len) }.to_vec();
        let dir = if dir == 0 { cuo::packets::Direction::Incoming } else { cuo::packets::Direction::Outgoing };
        match unsafe { __cuo_mod_on_packet(dir, &packet) } {
            cuo::packets::Verdict::Pass => 0,
            cuo::packets::Verdict::Block => 1,
            cuo::packets::Verdict::Replace(bytes) => arena::pack_ret(&bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // One test owns the global state (tests run on parallel threads).
    #[test]
    fn setup_run_and_commands_round_trip_through_the_state() {
        use crate::ecs::{Commands, Query};
        use crate::types::{ChatMessage, ChildOf};
        use crate::types::{Graphic, Hue};

        fn tint(mut q: Query<(&Graphic, &mut Hue)>, mut cmds: Commands) {
            for (_, (g, h)) in &mut q {
                if g.value == 1 {
                    h.value = 9;
                }
            }
            let root = cmds.spawn(Graphic { value: 5 }).id();
            cmds.spawn((Hue { value: 2 }, ChildOf::new(root)));
            cmds.entity(root).despawn();
            cmds.send(ChatMessage::system("hi"));
        }

        let st = state();
        st.types = [("cuo:ent/graphic", 1u16), ("cuo:ent/hue", 2), ("cuo:ecs/child-of", 3), ("cuo:chat/message", 4)]
            .into_iter()
            .map(|(p, i)| (p.to_string(), i))
            .collect();
        st.resolved.clear();
        st.next_temp = 7;

        let mut app = App::default();
        app.add_systems(Schedule::Update, tint);
        let reply = setup_reply(&app);
        let decl = &reply.systems.as_ref().unwrap()[0];
        assert_eq!(decl.schedule, abi::Schedule::Update);
        let params = decl.params.as_ref().unwrap();
        assert_eq!(params[0].kind, ParamKind::Query);
        let terms = params[0].query.as_ref().unwrap().terms.clone().unwrap();
        assert_eq!((terms[0].kind, terms[0].type_id), (QueryTermKind::Ref, 1));
        assert_eq!((terms[1].kind, terms[1].type_id), (QueryTermKind::Mut, 2));
        assert_eq!(params[1].kind, ParamKind::Commands);
        st.app = app;

        let cv = |t: u16, j: &str| abi::CompValue { type_id: t, encoding: Encoding::Json, data: Some(j.as_bytes().to_vec()) };
        let row = |e: u64, g: &str| abi::Row { entity: e, comps: Some(vec![cv(1, g), cv(2, r#"{"Value":0}"#)]) };
        let input = abi::SystemInput {
            sys_id: 0,
            tick: 0,
            queries: Some(vec![abi::QueryRows {
                param_index: 0,
                rows: Some(vec![row(100, r#"{"Value":1}"#), row(101, r#"{"Value":2}"#)]),
            }]),
            resources: None,
            events: None,
        };
        let bytes = Builder::new().finish(&input, None).to_vec();
        run_entry(0, &bytes, wire::SYSTEM_INPUT, None);

        let cmds = std::mem::take(&mut state().cmds);
        let kinds: Vec<_> = cmds
            .iter()
            .map(|c| match c {
                abi::Cmd::SpawnCmd(s) => format!("spawn{}", s.temp_id),
                abi::Cmd::DespawnCmd(d) => format!("despawn{}", d.entity),
                abi::Cmd::EmitEventCmd(e) => format!("emit {}", e.event_name.as_deref().unwrap()),
                abi::Cmd::InsertCmd(i) => format!("insert{}", i.entity),
                _ => "other".into(),
            })
            .collect();
        // Only the row whose Hue changed is written back, after the system's commands.
        assert_eq!(kinds, ["spawn7", "spawn8", "despawn-8", "emit cuo:chat/message", "insert100"]);
        // The child's ChildOf names its pending parent: top bit + temp id.
        let abi::Cmd::SpawnCmd(child) = &cmds[1] else { unreachable!() };
        let child_of = &child.comps.as_ref().unwrap()[1];
        assert_eq!(child_of.type_id, 3);
        assert_eq!(child_of.data.as_deref(), Some(format!(r#"{{"Parent":{}}}"#, PENDING | 7).as_bytes()));

        // mod_spawned resolves the placeholder; later uses get the real id.
        state().resolved.insert(7, 1234);
        assert_eq!(resolve(PENDING | 7), 1234);
        assert_eq!(entity_ref(PENDING | 7), 1234);
        assert_eq!(entity_ref(PENDING | 8), -9);
        assert_eq!(entity_ref(55), 55);
    }

    #[test]
    fn system_input_params_read_back_by_param_index() {
        let cv = |t: u16, j: &str| abi::CompValue { type_id: t, encoding: Encoding::Json, data: Some(j.as_bytes().to_vec()) };
        let input = abi::SystemInput {
            sys_id: 0,
            tick: 1,
            queries: Some(vec![abi::QueryRows {
                param_index: 1,
                rows: Some(vec![abi::Row { entity: 42, comps: Some(vec![cv(3, r#"{"Value":5}"#)]) }]),
            }]),
            resources: Some(vec![
                abi::ResValue { param_index: 0, value: Some(Box::new(cv(9, r#"{"Total":1.0}"#))), unchanged: false },
                abi::ResValue { param_index: 3, value: None, unchanged: false },
                abi::ResValue { param_index: 4, value: None, unchanged: true },
            ]),
            events: Some(vec![abi::EventValues { param_index: 2, values: Some(vec![cv(4, "{}"), cv(4, "{\"A\":1}")]) }]),
        };
        let bytes = Builder::new().finish(&input, None).to_vec();
        let p = wire::Params::new(&bytes, wire::SYSTEM_INPUT);
        assert_eq!(p.rows(1), vec![(42, vec![r#"{"Value":5}"#.to_string()])]);
        assert!(p.rows(0).is_empty());
        assert_eq!(p.resource(0), wire::ResInput::Value(r#"{"Total":1.0}"#.to_string()));
        assert_eq!(p.resource(3), wire::ResInput::Absent);
        assert_eq!(p.resource(4), wire::ResInput::Unchanged);
        assert_eq!(p.resource(5), wire::ResInput::Absent);

        // fetch: an `unchanged` param serves the value its previous run delivered.
        let descs = vec![ParamDesc::Res("a"), ParamDesc::Res("b")];
        let mut cache = Vec::new();
        let first = abi::SystemInput {
            sys_id: 0,
            tick: 1,
            queries: None,
            resources: Some(vec![abi::ResValue { param_index: 0, value: Some(Box::new(cv(9, "{\"V\":1}"))), unchanged: false }]),
            events: None,
        };
        let bytes = Builder::new().finish(&first, None).to_vec();
        let got = fetch(&descs, &mut cache, &wire::Params::new(&bytes, wire::SYSTEM_INPUT));
        assert!(matches!(&got[0], Fetched::Res(Some(j), _) if j == "{\"V\":1}"));
        assert!(matches!(&got[1], Fetched::Res(None, _)));
        let second = abi::SystemInput {
            sys_id: 0,
            tick: 2,
            queries: None,
            resources: Some(vec![abi::ResValue { param_index: 0, value: None, unchanged: true }]),
            events: None,
        };
        let bytes = Builder::new().finish(&second, None).to_vec();
        let got = fetch(&descs, &mut cache, &wire::Params::new(&bytes, wire::SYSTEM_INPUT));
        assert!(matches!(&got[0], Fetched::Res(Some(j), _) if j == "{\"V\":1}"));
        assert_eq!(p.events(2).len(), 2);
    }
}
