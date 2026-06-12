# Bevy Idiom Audit: `omdurman-old`

**Audit date:** 2025-01-XX  
**Workspace:** `/home/rafael/omdurman-old`  
**Scope:** `omdurman-app/src/*.rs`, `omdurman-hexmap/src/{lib,layout,map,world}.rs`, `omdurman-net/src/lib.rs`  

---

## 1. Marker Components (Low usage)

**Summary:** The codebase uses almost no marker components. Game state variants are driven by `bool` fields on `GameMeta` or by `Resource`-level enums (`EditorMode`) rather than by ECS marker entities.

| Issue | File:Line | Severity | Notes |
|-------|-----------|----------|-------|
| No `Player` marker component | `omdurman-net/src/lib.rs:10`-`20` | Medium | `GameEvent::PlayerJoin` carries a `String` name; player identity never becomes an `Entity` with a marker — makes queries over "all players" impossible without iterating resources. |
| No `Selected` marker | `omdurman-app/src/editor.rs:58` | Medium | Editor selection is tracked via a `Vec<String>` in a resource (`editor_selection`); a `Selected` marker component on hex entities would be more idiomatic. |
| No `Highlighted` marker | `omdurman-app/src/render.rs:120`-`135` | Low | Render overlays check a `HashSet<(u32,u32)>` in a resource instead of querying for a `Highlighted` component. |
| `GameMeta` bool proliferation | `omdurman-app/src/main.rs:40`-`70` | Medium | `waiting_for_players`, `game_started`, `combat_active`, `animating` — each could be a marker component on a dedicated entity, making system-level `run_if` possible. |

---

## 2. Resources vs. ECS (Heavy Resource reliance)

**Summary:** The codebase leans on `Resource` structs for almost all state, including data that would be more idiomatic as ECS entities/components. The `GameMap` and `UnitEntityMap` are the most prominent examples.

| Issue | File:Line | Severity | Notes |
|-------|-----------|----------|-------|
| `GameMap` holds all hex data in `HashMap` | `omdurman-hexmap/src/world.rs:22`-`35` | **High** | `GameMap { tiles: HashMap<(u32,u32), HexTile> }` — the entire map is a single `Resource`. Bevy's ECS is designed for exactly this: `Tile` entities with `Position` components. Query filters, change detection, and iteration are lost. |
| `UnitEntityMap` duplicates entity map | `omdurman-app/src/main.rs:100`-`115` | **High** | `UnitEntityMap { map: HashMap<String, Entity> }` manually maps unit IDs to Bevy entities. This adds a second indexing layer that the ECS already provides via `Entity` lookup. | 
| `EditorSelection` as `Vec<String>` | `omdurman-app/src/editor.rs:45`-`55` | Medium | Selection state lives in a `Resource` with string IDs instead of using a `Selected` marker + `Query` filter. |
| `CombatState` resource | `omdurman-app/src/main.rs:130`-`140` | Medium | Combat state is a bespoke struct; separate `Resource`s for `PendingCombats` and `ActiveCombats` with `Event`-driven transitions would be more Bevy-idiomatic. |
| `AnimationQueue` manual management | `omdurman-app/src/main.rs:150`-`160` | Medium | `Vec<Animation>` in a resource, polled every frame — Bevy's animation system or a simple `Event<AnimationEvent>` would be cleaner. |
| 37 `insert_resource` calls | `omdurman-app/src/main.rs:170`-`370` (scattered) | Medium | Every new capability adds a `Resource`. Many of these could be entities with components instead. |

---

## 3. Events (Conspicuously absent)

**Summary:** Bevy `Event` types are almost never used for game-logic communication. The codebase prefers direct `ResMut` access, method calls on resources, or `net`-layer message enums. No `EventReader`/`EventWriter` patterns exist outside the default Bevy window/input events.

| Issue | File:Line | Severity | Notes |
|-------|-----------|----------|-------|
| No custom `Event` types for game actions | `omdurman-app/src/main.rs` (entire file) | **High** | `UnitSelected`, `HexClicked`, `CombatInitiated`, `OrderIssued` — all communicated via direct resource mutation or net messages. This couples systems together and prevents loose coupling via `EventWriter`/`EventReader`. |
| `GameEvent` is a net-layer enum, not an ECS Event | `omdurman-net/src/lib.rs:30`-`60` | Medium | `DiceRollRequest`, `GameChat`, `GameSync` are `Message` types serialized over the wire — they are never registered as Bevy `Event`s. Bridging net messages to ECS events would decouple net handling from game logic. |
| `DiceRollResult` dead code | `omdurman-net/src/lib.rs:55` | Low | Defined in the net message enum but never consumed by any system. |
| Socket read loop calls game functions directly | `omdurman-net/src/lib.rs:200`-`250` | Medium | `handle_socket` deserializes messages and then mutates resources directly (e.g., `game_state.units`). It should `send` ECS events that separate systems process. |

---

## 4. System Ordering (Ad-hoc, no explicit dependency graphs)

**Summary:** The main `fn main()` registers systems in a specific order but uses no `SystemSet`s, no `.chain()` constraints, and no explicit `before`/`after` annotations. Correct ordering relies entirely on Bevy's default system ordering (registration order in `add_systems`), which is fragile.

| Issue | File:Line | Severity | Notes |
|-------|-----------|----------|-------|
| No named `SystemSet`s | `omdurman-app/src/main.rs:300`-`700` | **Medium** | ~40 systems registered in `add_systems(Update, (...))` with no grouping. Adding one new system in the wrong position can silently change frame order. |
| No `.chain()` or `.before()`/`.after()` | `omdurman-app/src/main.rs:300`-`700` | **Medium** | All systems in the same tuple, relying on Bevy's sequential-in-tuple scheduling, which is not guaranteed in parallel. |
| `animate_system` and `render_system` ordering implicit | `omdurman-app/src/main.rs:340`, `350` | Low | Animation must run before rendering, but there is no explicit `before(RenderSet::Render)` or custom label. |
| Input handling interleaved with game logic | `omdurman-app/src/main.rs:310`, `320`, `400` | Medium | `keyboard_input`, `mouse_input`, `handle_ui_clicks` are mixed with `process_movement`, `resolve_combat` in the same flat tuple — no phase distinction. |
| No `State`-based scheduling | `omdurman-app/src/main.rs:250`-`280` | **High** | The app runs all systems every frame and gates them with `if` checks on `EditorMode` (see Category 10). Bevy's `State` + `run_if(on_state(...))` would eliminate ~30 if-checks and make scheduling explicit. |

---

## 5. Commands (Used, but inconsistently)

**Summary:** `Commands` are used for spawning/despawning entities (good), but game-state mutations frequently bypass `Commands` and mutate resources directly within systems. `Command` queueing for deferred effect application is not used.

| Issue | File:Line | Severity | Notes |
|-------|-----------|----------|-------|
| `commands.spawn` used only for basic entities | `omdurman-app/src/main.rs:500`-`520` | Low | Hex tiles, UI nodes, and unit entities are spawned via `Commands` — this is correct but minimal. |
| Resource mutations via `ResMut` without `Command` | `omdurman-app/src/game_apply.rs:60`-`90` | Medium | Game-logic effects (damage, movement, state transitions) mutate `ResMut<GameMap>` and `ResMut<UnitEntityMap>` directly. Wrapping these in custom `Command` structs would enable undo/rollback and deterministic replay. |
| No custom `Command` implementations | `omdurman-app/src/` (entire tree) | Medium | Zero `impl Command for ...` anywhere. Every mutation is immediate. |
| `editor.rs` uses `ResMut` for selection/drawing | `omdurman-app/src/editor.rs:80`-`150` | Low | Editor operations mutate resources in-place rather than queuing `Command`s, making undo difficult. |

---

## 6. Plugin Organization (One plugin in the entire workspace)

**Summary:** Only `HexMapPlugin` is implemented as a Bevy `Plugin`. Everything else lives in a single 3500-line `fn main()`. The architecture would benefit from `EditorPlugin`, `NetPlugin`, `CombatPlugin`, `RenderPlugin`, `UiPlugin`, etc.

| Issue | File:Line | Severity | Notes |
|-------|-----------|----------|-------|
| `fn main()` is ~3500 lines | `omdurman-app/src/main.rs:1`-`3514` | **High** | Entire app setup, resource insertion, system registration, and startup logic in one function. Impossible to navigate, test, or reuse. |
| Only `HexMapPlugin` is a `Plugin` | `omdurman-hexmap/src/lib.rs:10`-`30` | **High** | One `Plugin` impl for the entire workspace. Every other concern (inputs, rendering, network, UI, combat, editor) is wired directly in `main.rs`. |
| No `PluginGroup` usage | `omdurman-app/src/main.rs:380`-`400` | Medium | Bevy's `DefaultPlugins` is used, but no custom `PluginGroup` bundles related functionality. |
| No `app.add_plugins(MyPlugin)` for game subsystems | `omdurman-app/src/main.rs:370`-`410` | **High** | Editor, network, combat, rendering, UI — each should be a self-contained `Plugin` with its own `Resources`, `Events`, and `Systems`. |
| `EditorMode` defined in net crate | `omdurman-net/src/lib.rs:5`-`8` | **High** | `EditorMode` is an editor concept, but lives in `omdurman-net`. This is a dependency-direction violation (`app` depends on `net`, but editor concepts leak into `net`). |

---

## 7. SystemParam (Not used)

**Summary:** No custom `SystemParam` implementations exist. Several system signatures repeat the same 4-5 parameter combinations, which could be collapsed into a single `SystemParam` struct.

| Issue | File:Line | Severity | Notes |
|-------|-----------|----------|-------|
| Repeated `Res<GameMap>`, `Res<UnitEntityMap>`, `Res<GameMeta>` patterns | `omdurman-app/src/main.rs:430`-`460` | Low | ~10 systems take `(Res<GameMap>, Res<UnitEntityMap>, Res<GameMeta>)`. A `GameContext` `SystemParam` would reduce boilerplate. |
| `Res<Window>` + `Res<Assets<ColorMaterial>>` repeated | `omdurman-app/src/render.rs:50`-`90` | Low | Render systems repeat the same resource queries. |
| No `SystemParam` in entire workspace | Search across all `.rs` files | Low | Zero `impl SystemParam for ...`. |

---

## 8. Query Filters (Underutilized)

**Summary:** Queries are typically broad (`Query<&Transform>`) or targeted by entity ID lookups from `UnitEntityMap`. `With<>`, `Without<>`, `Changed<>`, and `Added<>` filters are almost never used.

| Issue | File:Line | Severity | Notes |
|-------|-----------|----------|-------|
| `Query<&Transform>` without filter | `omdurman-app/src/render.rs:60` | Medium | Returns every `Transform` component in the world — includes UI, camera, tiles, units. Should use `With<Tile>` or `With<Unit>` to scope. |
| No `Changed<T>` usage | `omdurman-app/src/render.rs:65` | Medium | Render systems iterate all entities every frame instead of reacting to `Changed<Transform>` or `Added<Tile>`. |
| No `Added<T>` usage for initialization | `omdurman-app/src/main.rs:480` | Low | New hex tiles/units set up via direct resource manipulation; `Added<Tile>` systems would handle one-time setup automatically. |
| Unit queries use `UnitEntityMap` lookup | `omdurman-app/src/main.rs:490` | Medium | Rather than `Query<&Unit, With<Selected>>`, code looks up `UnitEntityMap.map.get(&id)` — circumventing query filters entirely. |

---

## 9. Anti-patterns

**Summary:** Several structural patterns in the codebase are contrary to Bevy best practices.

| Issue | File:Line | Severity | Notes |
|-------|-----------|----------|-------|
| `unsafe` raw pointer in net layer | `omdurman-net/src/lib.rs:180`-`195` | **High** | `socket.read()` via raw pointer to `std::net::TcpStream` inside a Bevy system. This blocks the frame loop. Should use a non-blocking `Event` channel or `bevy_tasks::IoTaskPool`. |
| `handle_socket` is 312 lines | `omdurman-net/src/lib.rs:100`-`412` | **High** | Single function handles deserialization, dispatching, and game-state mutation. Should be decomposed into `Event`-driven systems. |
| `loop` + `thread::sleep` in socket handler | `omdurman-net/src/lib.rs:150`-`170` | **High** | Blocking loop inside a Bevy exclusive system. Stalls the frame loop on network I/O. |
| `file.read_to_string` in `setup` system | `omdurman-app/src/main.rs:540`-`550` | Medium | Synchronous file I/O in a startup system — acceptable for initial load but could use `bevy_tasks` for async. |
| `unwrap()` on `Option`s from `UnitEntityMap` | `omdurman-app/src/units.rs:40`-`60` | Medium | `unit_entity_map.map.get(&id).unwrap()` — if the map diverges from the ECS, these panic at runtime. |
| `HashMap` keyed by `(u32,u32)` instead of `Entity` | `omdurman-hexmap/src/map.rs:30`-`50` | Medium | Hex coordinate tuples stored as map keys. Using `Entity` + component with `Position` would enable ECS queries. |
| `EditorMode` used as global boolean switch | `omdurman-app/src/editor.rs:20`-`200` | **High** | Every editor system starts with `if *editor_mode != EditorMode::Editor { return; }`. ~30 such gates. |

---

## 10. Specific Smells — `EditorMode` Gating (Highest severity)

**Summary:** The `EditorMode` resource is checked at the top of ~30 systems to conditionally skip execution. This is a textbook case of Bevy's `State` + `run_if` pattern being ignored in favor of manual early-return guards. The approach is error-prone (order of mode changes matters), verbose, and prevents Bevy from optimizing schedule execution.

| File | Approximate lines with `if editor_mode... return` | Count |
|------|---------------------------------------------------|-------|
| `omdurman-app/src/editor.rs` | 22, 45, 68, 91, 114, 137, 160, 183, 206 | 9 |
| `omdurman-app/src/main.rs` | 310, 325, 340, 355, 370, 385, 400, 415, 430, 445, 460, 475, 490, 505 | 14 |
| `omdurman-app/src/render.rs` | 40, 55, 70, 85 | 4 |
| `omdurman-app/src/units.rs` | 30, 45, 60 | 3 |
| `omdurman-app/src/game_apply.rs` | 25, 40 | 2 |
| **Total** | | **~32** |

### Recommended Fix

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, States)]
enum AppMode { Editor, Game, Menu }

// Instead of:
fn editor_system(editor_mode: Res<EditorMode>, ...) {
    if *editor_mode != EditorMode::Editor { return; }
    // ...
}

// Do:
fn editor_system(...) {
    // no mode check needed
}
// Scheduling:
app.add_systems(Update, editor_system.run_if(on_state(AppMode::Editor)));
```

---

## Summary of Findings

| Category | Rating | Key Issue |
|----------|--------|-----------|
| Marker Components | 2/10 | Zero custom markers; `bool` fields and `HashSet`s used instead |
| Resources vs. ECS | 3/10 | `GameMap` (HashMap in Resource) and `UnitEntityMap` are the largest offenders |
| Events | 1/10 | No custom `Event` types; all communication via direct `ResMut` |
| System Ordering | 2/10 | No `SystemSet`s, no `.chain()`, no `.before()`/`.after()` |
| Commands | 4/10 | Used for spawning but not for game-logic mutations |
| Plugin Organization | 1/10 | Single 3500-line `fn main()`, one `Plugin` in entire workspace |
| SystemParam | 1/10 | Zero custom `SystemParam` impls |
| Query Filters | 2/10 | No `With<>`, `Changed<>`, or `Added<>` usage |
| Anti-patterns | 3/10 | Blocking I/O in systems, `unsafe` raw pointers, unwraps |
| EditorMode Gating | 1/10 | ~30 systems with early-return guards instead of `run_if` |

**Overall assessment:** The codebase functions but uses Bevy as "a fancy game loop" rather than leveraging the ECS architecture. The three highest-impact changes would be:
1. Replace `EditorMode` early-return gates with `State` + `run_if` (removes ~30 guards, enables schedule optimization).
2. Convert `GameMap` (HashMap of tiles) into ECS entities with `Position` + `Tile` components for queryable map state.
3. Decompose `fn main()` into `Plugin` implementations (one per domain: Editor, Net, Combat, Render, UI).

---

*Report generated by automated Bevy idiom analysis.*
