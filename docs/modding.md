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
// and passes exactly those.
fn low_hp_warning(player: Query<&Hits, (With<Player>, Changed<Hits>)>, mut cmds: Commands) {
    for (_, hits) in &player {
        if hits.value < hits.max_value / 4 {
            cmds.send(ChatMessage::system("Low HP!"));
        }
    }
}

export_mod!(setup);
```

Build it (`cargo build --release --target wasm32-wasip1`), drop the `.wasm` (as
`mod.wasm`) + `mod.json` in the client's `Data/Mods/<name>/` folder (next to the
exe; settings.json `mods_path`), start the client.
`rust/examples/low_hp` is this mod, ready to build.

## The seven ideas

| Idea | What it is | How you use it |
|---|---|---|
| **Type path** | The name of a piece of game data: `cuo:ent/graphic`, `cuo:player/hits`. | Every component, resource and event has one. See the [reference](modding-reference.md). |
| **System** | A function the client runs every frame (or once, in `Startup`). | `app.add_systems(Schedule::Update, my_fn)`; its parameters say what it gets. |
| **Query** | The entities that have some components. | `Query<(&Graphic, &mut Hue), (With<Item>, Changed<Amount>)>`, then `for (e, (g, h)) in &mut q` (`&q` to only read). Also `q.get(e)`, `q.contains(e)`, `q.single()`. Filters: `With`, `Without`, `Changed`, `Added`. `&mut` changes are written back when the system returns. |
| **Resource** | One global value (the mouse, the time, the game state). | `Res<Time>` to read, `ResMut<T>` to write. A system whose resource is missing is skipped; take `Option<Res<T>>` to run anyway. |
| **Event** | Something that happened (a message, a container opening). | `EventReader<T>` to read the ones since last run; `cmds.send(value)` to send your own. |
| **Commands** | Changes to the world: spawn, insert, remove, despawn. | `Commands`; applied after your system returns. |
| **Observer** | A system that runs the moment something happens, instead of every frame. | `app.add_observer(my_fn)` where the first parameter is `On<Add, T>`, `On<Remove, T>` or `On<Event, T>`. |

Every type (`Hits`, `Graphic`, `ChatMessage`, ...) comes from the SDK and knows its own
type path, so you never write path strings in a typical mod. `Local<T>` is a system's
own state between runs (a counter, a timer, the window it spawned).

Schedules, in frame order: `Startup` (once), `First`, `PreUpdate`, `Update`,
`PostUpdate`, `Last`.

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
- **`packets`** — raw packets both ways. `intercept(dir, ids)` in `setup`, then
  your `on_packet(dir, bytes)` sees each one first and returns `Pass`, `Block` or
  `Replace(bytes)`. `send_to_server` / `send_to_client` inject your own.
- **`host`** — `log`, `measure_text`, `resolve_serial`, `storage_get` /
  `storage_set` (global or per character; `storage::load::<T>(scope)` /
  `storage::save(scope, &value)` do the JSON for you).

```rust,ignore
fn setup(app: &mut App) {
    packets::intercept(Direction::Incoming, &[0x1C]); // ASCII speech
}

fn on_packet(dir: Direction, p: &[u8]) -> Verdict {
    if is_spam(p) { Verdict::Block } else { Verdict::Pass }
}

export_mod!(setup, on_packet);
```

### Settings

Declare your options once (`cuo:options/schema`) and they appear in the client's
Options window under **Mods**, saved per character. Read them back from
`cuo:options/values`.

## Targets

Same source, two builds:

- `wasm32-wasip1` — a core module, via the SDK (feature `p1`, the default):
  `cargo build --release --target wasm32-wasip1`.
- `wasm32-wasip2` — a WebAssembly component. Any language with WIT bindings works
  against `wit/cuo-mod.wit`; with the Rust SDK:
  `cargo build --release --target wasm32-wasip2 --no-default-features --features p2`.
