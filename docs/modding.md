# Writing a ClassicUO mod

A mod is a WebAssembly module the client loads at startup. It works like a small Bevy
plugin: in `setup` you add **systems**; the client runs them every frame with the
**parameters** they asked for. That is the whole model.

The contract is two files: [`wit/deps/tinyecs-mod/tinyecs-mod.wit`](../wit/deps/tinyecs-mod/tinyecs-mod.wit)
(the ECS) and [`wit/cuo-mod.wit`](../wit/cuo-mod.wit) (ClassicUO host functions). The
SDKs wrap them; you never have to read them.

## Your first mod

```rust
use cuo_mod_sdk::prelude::*;

fn setup(app: &mut App) {
    app.add_systems(Schedule::Update, low_hp_warning);
}

// The parameters ARE the declaration: the client sees the query + `Commands`
// and passes exactly those. `#[system]` makes the fn an export of the mod
// (`low-hp-warning`), which is what the client calls.
#[system]
fn low_hp_warning(player: Query<&Hits, (With<Player>, Changed<Hits>)>, mut cmds: Commands) {
    for (_, hits) in &player {
        if hits.value < hits.max_value / 4 {
            cmds.send(ChatMessage::system("Low HP!"));
        }
    }
}

export_mod!(setup);
```

Build it (`cargo build --release --target wasm32-wasip2`), drop the `.wasm` (as
`mod.wasm`) + `mod.json` in the client's `Data/Mods/<name>/` folder (next to the
exe; settings.json `mods_path`), start the client.
`rust/examples/low_hp` is this mod, ready to build.

## The seven ideas

| Idea | What it is | How you use it |
|---|---|---|
| **Type path** | The name of a piece of game data: `cuo:ent/graphic`, `cuo:player/hits`. | Every component, resource and event has one. See the [reference](modding-reference.md). |
| **System** | A function the client runs every frame (or once, in `Startup`). | Mark it `#[system]`, then `app.add_systems(Schedule::Update, my_fn)`; its parameters say what it gets. |
| **Query** | The entities that have some components. | `Query<(&Graphic, &mut Hue), (With<Item>, Changed<Amount>)>`, then `for (e, (g, h)) in &mut q` (`&q` to only read). Also `q.get(e)`, `q.contains(e)`, `q.single()`. Filters: `With`, `Without`, `Changed`, `Added`. `&mut` changes are written back when the system returns. |
| **Resource** | One global value (the mouse, the time, the game state). | `Res<Time>` to read, `ResMut<T>` to write. A system whose resource is missing is skipped; take `Option<Res<T>>` to run anyway. |
| **Event** | Something that happened (a message, a container opening). | `EventReader<T>` to read the ones since last run; `cmds.send(value)` to send your own. |
| **Commands** | Changes to the world: spawn, insert, remove, despawn. | `Commands`; applied after your system returns. |
| **Observer** | A system that runs the moment something happens, instead of every frame. | `#[system]` too, then `app.add_observer(my_fn)` where the first parameter is `On<Add, T>`, `On<Remove, T>` or `On<Event, T>`. |

Every type (`Hits`, `Graphic`, `ChatMessage`, ...) comes from the SDK and knows its own
type path, so you never write path strings in a typical mod. `Local<T>` is a system's
own state between runs (a counter, a timer, the window it spawned).

Schedules, in frame order: `Startup` (once), `First`, `PreUpdate`, `Update`,
`PostUpdate`, `Last`.

Every system and observer is its own export of the mod, named after the function in
kebab-case (`low_hp_warning` -> `low-hp-warning`), so names must be unique. Rust: mark
each one `#[system]` and write its parameter types out (no `type` alias in the
signature — the macro reads the type names); closures can't be systems. C#: a system's
name is its `.Label(...)` (kebab-cased), else `systemN` / `observerN` in registration
order; `Setup` must only register systems, the build runs it to generate the exports.
C: list the exports in the mod's `wit/world.wit` (which includes `cuo:c-sdk/mod@0.1.0`); the
build generates the bindings from it (needs `wit-bindgen` and `awk`).

### Writing through a query

A `&mut T` term (C#: `Mut<T>` in `Data<..>`, then `row.Value.Field = x`) is written back
when the system or observer **returns**, after its own commands — so it wins over a
`cmds.insert` of the same component on the same entity in that run. It is written only
when the value actually changed: assigning the same value sends nothing, so it never
fires `Changed<T>` (or a UI relayout) on its own. Read-only types — marked *(ro)* in the
[reference](modding-reference.md), like `cuo:player/hits` or `cuo:ui/computed` — can't
be `&mut` / `Mut`: the client refuses to load the mod and names the system and the path.

### Parents and children

Hierarchy is a component, `cuo:ecs/child-of { parent }` (`ChildOf`). To put an entity
under another, insert it; to find an entity's children, query it. Items in a bag are the
bag's children, UI nodes are their panel's children — same rule.

```rust,ignore
let root = cmds.spawn((Node::abs(30.0, 120.0, 200.0, 100.0), UiMovable {})).id();
cmds.spawn((Text { value: "Hello".into() }, ChildOf::new(root)));
cmds.entity(root).with_child(Text { value: "World".into() }); // same thing
```

### Building UI

UI is entities too: spawn a `cuo:ui/node` (layout), add `cuo:ui/text`,
`cuo:ui/bg-color`, ... and parent them with `child-of`. Add `cuo:ui/movable` to a window
root and it drags and right-click-closes like every client window. Clicks arrive as the
`cuo:ui/click` event: `app.add_observer(on_click)` with
`fn on_click(click: On<Event, UiClick>, ...)`; `click.entity()` is what was clicked.

### Functions: assets, actions, host

Things that are not world data are plain functions, in four groups:

- **`assets`** — the UO data files: image size + pixels (land, art, gumps, texmaps,
  lights), animation frames, tiledata, hue ramps, clilocs, map cells, multis, skills.
- **`actions`** — what the player can do: `double_click`, `move_item`, `walk`,
  `cast_spell`, `target_object`, `say`, `invoke_virtue`, `gump_reply`, `vendor_buy`,
  ... They go through the client's own code, so last target / held item / war mode
  stay right. Combinations (`move_type`, `target_self`, `bandage_self`, ...) are SDK
  helpers.
- **`packets`** — raw packets both ways. `send_to_server` / `send_to_client` inject
  your own. To see them, add a packet observer for a direction and a set of ids (none
  = every id): it gets each one first and returns `Pass`, `Block` or
  `Replace(bytes)`. Like any observer it can take `Commands`, queries, resources.
- **`host`** — `log`, `measure_text`, `resolve_serial`, `storage_get` /
  `storage_set` (global or per character; `storage::load::<T>(scope)` /
  `storage::save(scope, &value)` do the JSON for you).

```rust,ignore
fn setup(app: &mut App) {
    app.add_packet_observer(PacketDirection::Incoming, &[0x1C], block_spam); // ASCII speech
}

#[system]
fn block_spam(p: Packet) -> Verdict {
    if is_spam(&p) { Verdict::Block } else { Verdict::Pass }
}

export_mod!(setup);
```

### Idle cost

By default every system runs every frame, and each run ships its resources across the
wasm boundary. A system that only reacts to input can opt out of idle frames:

```rust,ignore
app.add_systems(Schedule::Update, refresh_window.run_on_change());
```

(C#: `m.AddSystem(...).RunOnChange()`; C: `cuo_system_run_on_change(m, sys)`.) The
system is skipped on a frame when every `Res` / `ResMut` it takes is unchanged since its
previous run, no new event arrived for its `EventReader`s, and every query matched no
rows. Only queries with `Changed` / `Added` filters can be empty while idle — a plain
query that matches keeps the system running — and a system with no parameters always
runs. `Commands` doesn't count as an input.

Don't use it for anything driven by its own state or the clock: timers, cooldowns,
macro / state machines stepping through `Local`s, retries. With nothing new to read
they would never wake up. There is no periodic safety-net run.

To write a resource without reading it, use `cmds.set_resource(value)` (C#:
`cmds.SetResource(value)`; C: `cuo_set_resource(cmds, path, json)`): no `ResMut`
parameter, applied with the run's other commands. The client refuses an unknown or
read-only path.

Resource reads are cached for you: while the client reports a resource unchanged since
the system's previous run, the SDK reuses the value it parsed then instead of fetching
and parsing it again (C# also parses `Res<T>.Value` only on first access). Treat a
`Res` value as read-only — the same object comes back next run.

### Typed components

A curated set of components crosses the wasm boundary as typed WIT records instead of
JSON: player `hits` / `mana` / `stamina` / `data`; entity `serial` / `graphic` / `hue` /
`world-position` / `notoriety` / `name` / `amount`; UI `node` / `text` / `text-font` /
`text-color` / `bg-color` / `border-radius` / `child-of` / `movable` / `global-z` /
`interaction` / `no-window-drag`; resources `mouse` / `keyboard`. With an SDK nothing
changes in your code — `Query<&Hits>`, `&mut Node`, `insert(Text{..})`, `Res<..>` use the
typed path by themselves (a query goes typed when every component it reads is curated;
anything else stays JSON). Without an SDK, `cuo:modding/components` gives typed columns
aligned with `query.entities()` (`column-hits(q, term)`), typed writes (`set-node`), typed
resource reads (`get-mouse`) and a chainable builder:
`spawn(&cmds).node(&n).text(&t).child-of(&c).id()`. A value the game sends that an enum
has no case for reads as case 0.

### Settings

Declare your options once (`cuo:options/schema`) and they appear in the client's
Options window under **Mods**, saved per character. Read them back from
`cuo:options/values`.

## Targets

A mod is a `wasm32-wasip2` WebAssembly component. Any language with WIT bindings works;
with the Rust SDK: `cargo build --release --target wasm32-wasip2`. The client rejects
core-wasm (`wasm32-wasip1`) modules.

Without an SDK, a mod is a component of its **own world**, which includes
`cuo:modding/mod` (the `setup` export plus the host imports) and exports one function per
system it declares in `setup`, named like the system (wasvy-style):

```wit
package example:hello;

world hello {
    use tinyecs:modding/ecs@0.1.0.{commands, query, trigger-data, packet-direction, verdict};

    export show-hits: func(hits: query, commands: commands);       // a scheduled system
    export on-click: func(trigger: trigger-data);                  // an observer
    export filter: func(direction: packet-direction, packet: list<u8>) -> verdict; // on-packet

    include cuo:modding/mod@0.1.0;
}
```

`setup` declares `System::new("show-hits")`, adds its params (`add-query`, then
`add-commands`) and schedules it; the client then calls `show-hits` with those params as
typed arguments, in the order they were added. Observers get the `trigger-data` first,
on-packet observers the direction and the packet. A declared system without a matching
export fails the mod's load. A complete example: `src/Mods/ecs-hello-raw` in the client
repo.
