# Mod data reference

Every component, resource and event a mod can use, by type path. In the SDKs each
one is a type that knows its path (`cuo:player/hits` → `Hits`), so this is the list of
types you can put in a system's parameters.

> **Review draft.** The **Change** column is the plan against today's surface:
> *keep*, *rename* (new name in **bold**), *new*, or *cut* (with what replaces it).
> It disappears once the plan is applied.

Kinds: **C** component (query it) · **R** resource (`Res` / `ResMut`) · **E** event
(read it / observe it / send it) · **A** a request the client acts on (send it).
Read-only data is marked *(ro)*; writing it is ignored, and a `&mut` / `Mut<T>` query term on it fails the mod's load.

## ECS

| Path | Kind | Change |
|---|---|---|
| **`cuo:ecs/child-of`** `{parent}` | C | *new* — the hierarchy. Insert to re-parent; query to find children. Replaces `cuo:ent/parent`, `cuo:ent/container`, the `add-child` command, the `child_of` query term and `set_child_of`. |
| `cuo:engine/time` | R (ro) | keep |

## World entities (items and mobiles)

| Path | Kind | Change |
|---|---|---|
| `cuo:ent/serial` | C (ro) | keep |
| `cuo:ent/graphic` · `hue` · `amount` · `name` · `facing` · `notoriety` | C | keep |
| `cuo:ent/world-position` · `slot-position` | C | keep |
| `cuo:ent/is-item` · `is-mobile` · `is-multi` · `is-container` · `contained-into` | C (ro, markers) | keep |
| `cuo:ent/server-flags` · `animation` · `equipment` · `mob-steps` | C (ro) | keep |
| `cuo:ent/properties` | C (ro) | keep — the server tooltip data; ask with `actions.request-properties` |
| `cuo:ent/auto-opened-corpse` · `manual-opened-corpse` | C (ro, markers) | keep |
| `cuo:ent/parent` · `cuo:ent/container` | — | *cut* → `cuo:ecs/child-of` |
| `cuo:ent/tile` | — | *cut* → `assets.static-tile(graphic)` |

## Player

| Path | Kind | Change |
|---|---|---|
| `cuo:player/player` | C (marker) | keep |
| `cuo:player/hits` · `mana` · `stamina` · `data` · `stat-locks` | C (ro) | keep |
| `cuo:player/skills` · `buffs` · `spellbook` · `party` · `steps` · `grabbed-item` | R (ro) | keep |
| `cuo:player/move-request` | E | keep |
| `cuo:player/profile` | — | *cut* (raw dump of the whole profile) → **`cuo:game/settings`** |

## Game

| Path | Kind | Change |
|---|---|---|
| `cuo:game/context` | R | keep |
| `cuo:game/state` | R | keep |
| **`cuo:game/settings`** | R (ro) | *new* — the client options a mod may need (grid loot, skip empty corpse, …), named fields only |
| `cuo:options/schema` · `cuo:options/values` | R | keep — your mod's Options section |
| `cuo:target/state` | R (ro) | keep |
| `cuo:target/local-result` | E | keep |
| `cuo:chat/message` | E | keep |
| `cuo:chat/prompt` | E | keep |

## Items moving

| Path | Kind | Change |
|---|---|---|
| `cuo:item/drop-sent` · `cuo:item/move-result` | E | keep |

## Actions

| Path | Kind | Change |
|---|---|---|
| `cuo:action/*` (27 events) | — | *cut* → the `actions` WIT interface (functions). Same 27, plus: `wear`, `use-item-on`, `equip-last-weapon`, `cancel-walk-to`, `resync`, `set-war-mode` (replaces `toggle-war-mode`), `attack`, `use-ability`, `clear-ability`, `toggle-flying`, `clear-target-queue`, `open-spellbook`, `set-skill-lock`, `set-stat-lock`, `invoke-virtue`, `emote-action`, `say` with a kind, `party-*`, `menu-reply`, `text-entry-reply`, `dye-reply`, `help-request`, `trade-*`, `vendor-buy` / `vendor-sell`, `open-window` / `close-window`, `set-view-range`, `logout`. |

## Input

| Path | Kind | Change |
|---|---|---|
| `cuo:input/mouse` · `cuo:input/keyboard` | R (ro) | keep (held state + this frame's edges) |
| `cuo:input/hotkey` | E | keep |
| `cuo:input/host-hotkeys` | R (ro) | keep |
| `cuo:input/mod-hotkeys` | R | keep — your mod's own key bindings |
| `cuo:input/world-single-click` | R (ro) | keep |
| **`cuo:input/consume`** `{mouse?, key?}` | A | *new* — replaces the `consume-mouse` / `consume-key` commands |

## Windows (gumps)

| Path | Kind | Change |
|---|---|---|
| `cuo:gump/<name>` (paperdoll, container, journal, skills, spellbook, health-bar, …) | C (markers) | keep — find / close host windows |
| `cuo:gump/container-opened` · `container-closed` · `container-slot` | E | keep |
| `cuo:gump/server` | C (ro) | keep |
| `cuo:gump/server-opened` · `server-closed` · `context-menu` | E | keep |
| `cuo:gump/container-tag` · `cuo:ui/container-item` · `cuo:gump/container-positions` | C / R | keep — make your window a real container window |
| `cuo:gump/grid-container` · `cuo:gump/grid-loot` · `cuo:ui/grid-pinned` | C | **rename**: `grid-pinned` → **`cuo:ui/no-pickup`**; the other two keep (gumps.xml + corpse highlight read them) |

## UI

| Path | Kind | Change |
|---|---|---|
| `cuo:ui/node` · `text` · `text-font` · `text-color` · `text-hue` · `text-spans` · `text-wrap` | C | keep |
| `cuo:ui/bg-color` · `border-color` · `border-radius` · `hover-tint` · `custom` · `global-z` | C | keep |
| `cuo:ui/interaction` · `name` · `tooltip` · `tooltip-serial` · `button` · `stat-lock-button` | C | keep |
| `cuo:ui/scroll` · `scrollbar` · `computed` *(ro)* | C | keep |
| `cuo:ui/movable` · `resizable` · `movable-no-drag` · `no-window-drag` · `no-right-click-close` · `popup` · `no-blur` · `contains-by-bounds` · `supersedes` | C (markers) | keep |
| `cuo:ui/text-input` · `editable-text` · `masked-text` | C | keep |
| `cuo:ui/focused-input` · `surface` *(ro)* · `pick` *(ro)* · `text-caret` *(ro)* · `text-completion` · `text-completion-state` *(ro)* · `text-completion-pick` · `text-clipboard-action` · `clipboard-set` | R | keep |
| `cuo:ui/topbar-*` · `options-window` · `statusbar-window` | C (markers) | keep |
| `cuo:ui/clicked` · `right-clicked` · `hovered` | — | *cut* → events **`cuo:ui/click`**, **`cuo:ui/right-click`** `{x, y}`, **`cuo:ui/hover`** `{over}` (observe them on your entity) |

## Scenes

| Path | Kind | Change |
|---|---|---|
| `cuo:scene/login` · `server-selection` · `character-selection` · `character-creation` · `login-error` · `game` | C (markers) | keep |
| `cuo:scene/login-request` | A | keep |
| `cuo:scene/server-list` · `login-error-info` · `character-list` | E | keep |

## Gone without replacement

| Path / API | Why |
|---|---|
| `cuo:test/counter` | test fixture |
| `cuo:modding/owned` | host bookkeeping, not mod data |
| `cuo:modding/state` | keep your own state in your mod |
| `component-get` / `resource-get` / `entity-parent` / `entity-children` imports | read through system parameters only |
| `spawn-named` / `entity(name)` | `spawn` returns the entity id |
| `every-ms` | check `Res<Time>` in the system |
| `tile` / `hue-ramp` / `gump-size` / `cliloc` host functions | moved to `assets` (`static-tile`, `hue-ramp`, `image-size`, `cliloc` with args) |
| `net-send` host function, `filter-packet` / `on-incoming-packet` export (block-only, every packet) | → `on-packet(direction, ids)` observer trigger returning pass / block / replace; `packets` interface keeps `send-to-server`, `send-to-client` (new: inject incoming) |
