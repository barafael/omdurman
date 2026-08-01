# UI Implementation Plan

Based on `docs/ui-states.md` and an audit of the existing `omdurman-app` codebase
(47 source files across `src/` and `src/editor/`).

---

## Phase 1 — Communication & Connection Awareness

### 1.1 Chat panel

**Files:** `src/chat.rs` (new), `omdurman-net/src/lib.rs` (modify), `src/ui_plugin.rs` (modify)

New module with:

- **Resource:** `ChatLog { messages: Vec<ChatMessage>, max_len: usize }`
  where `ChatMessage { sender: String, text: String, turn: u32, timestamp: f64 }`.
- **Network message:** `NetMsg::Chat(String)` — sent as unreliable* ephemeral (not
  recorded in the event log; late joiners don't get chat history).
- **UI:** Collapsible egui panel anchored to the right sidebar, below the
  action-guide panel. Text input field + "Send" button at the bottom; scrollable
  message list above, coloured by sender faction. Persist ~50 messages.

*Could use reliable broadcast for guaranteed delivery — decide during review.

### 1.2 Connection status dot

**Files:** `src/state.rs` (modify), `src/ui_plugin.rs` (modify)

- **Resource:** `NetConnectionState` enum — `Connected { room_id: String } |
  Connecting | Disconnected`.
- **UI:** Colored circle (green/yellow/red) rendered in the status text area next
  to the room code. Hooked into `NetSocketPlugin` state changes.

---

## Phase 2 — Menu & Lobby Flow

### 2.1 Main menu buttons

**Files:** `src/splash.rs` (modify)

Replace the three-button layout (Lobby / Game / Editor) with a five-button layout:

- **Host Game** — creates a new room, auto-hosts, enters lobby
- **Join Game** — inline room-code text field + "Connect" button → enters lobby
- **Editor** — unchanged
- **Review saved game** — opens the saved-games file browser, transitions to
  `AppState::Spectating` on selection
- **Continue Game** — previously "Game", disabled unless a snapshot exists

The `M` key still returns to menu from any mode.

### 2.2 Lobby "Back" button

**Files:** `src/lobby.rs` (modify)

- Add a "Back" button (top-left or bottom) that sends a disconnect message and
  transitions to `AppMode::Menu`.
- Currently the only way to leave the lobby is pressing `M`.

### 2.3 Loading bar / spinner

**Files:** `src/splash.rs` (modify)

- Replace `"Loading..."` text with a centered progress bar.
- Track asset progress via `AssetServer` event reader or a counter of
  loaded-vs-total assets.
- Spinner animation for indeterminate progress (first launch).

---

## Phase 3 — Visual Feedback & Combat Immersion

### 3.1 Deployment zone overlay

**Files:** `src/render.rs` (modify) or `src/deployment_overlay.rs` (new)

- Read deployment-zone hexes from the scenario setup data.
- Render a tinted hex overlay: green for the local player's zone, blue for the
  opponent's.
- Only visible during `UiPhaseState::Setup`.
- Use the same pooled-mesh pattern as the existing ZOC rings.

### 3.2 Night crescent-moon icon

**Files:** `src/ui_plugin.rs` (modify)

- Replace the text-only "Night rules (§8)" note with a moon icon + text.
- Use the `☾` Unicode character in egui `RichText`, sized to match the phase
  banner font.

### 3.3 3D dice roll animation

**Files:** `src/dice_animation.rs` (new), `src/combat_card.rs` (modify),
`src/main.rs` (modify — add plugin)

Optional visual flair during combat resolution:

- **When:** A `GameEffect::FireCombat` or `GameEffect::MeleeCombat` is about to
  be applied locally (the result is already known — this is purely cosmetic).
- **What:** Spawn avian3d rigid-body die entities, apply a random torque, let
  them settle on a random face for ~1.5 s.
- **Cleanup:** `DiceAnimation` resource with a `Timer`; despawn entities and
  show result text when timer expires.
- **Toggle:** `LocalPlayerSettings.show_dice_animations` checkbox in settings.
- **Fallback:** If toggled off, skip straight to result toast.

### 3.4 Howitzer scattergram map overlay

**Files:** `src/fire.rs` (modify), `src/render.rs` (modify)

- When a howitzer target is selected, render the 6 scatter-destination hexes
  as a coloured overlay pattern around the target hex.
- On resolution, draw an arrow from the target hex to the actual impact hex.
- Use the same ring/arrow rendering pattern as the movement path arrows.

### 3.5 Advance-after-combat prompt

**Files:** `src/combat_card.rs` (modify), `src/dispatch.rs` (modify)

The engine already supports `can_advance_after_combat`. What's missing is a
visible floating prompt:

- After an offensive fire or melee attack vacates a hex, show:
  > "Advance into [hex name]?"
  > [Advance: Unit A] [Advance: Unit B] [Skip all]
- One prompt per eligible participating unit.
- Gate on the phase type (offensive fire / melee) and the advance rules
  (artillery excluded, wall/khor restrictions).

---

## Phase 4 — Always-Visible Toolbar

### 4.1 Top toolbar

**Files:** `src/ui_plugin.rs` (modify)

- Render an egui `TopBottomPanel::top` in all non-menu, non-splash states.
- **Left section:** mode indicator ("Game" / "Editor") — clicking is a no-op
  (switch via M key as before).
- **Center section:** current phase banner + turn counter (moved from the
  bottom status pane).
- **Right section:** connection dot + room code (from 1.2).
- Keep the existing bottom status pane as a condensed fallback.

### File change summary

| File | Action | Phase |
|------|--------|-------|
| `src/chat.rs` | New | 1 |
| `src/dice_animation.rs` | New | 3 |
| `omdurman-net/src/lib.rs` | Add `NetMsg::Chat` | 1 |
| `src/state.rs` | Add `NetConnectionState` | 1 |
| `src/settings.rs` | Add `show_dice_animations` | 3 |
| `src/splash.rs` | Modify menu buttons, loading bar | 2 |
| `src/lobby.rs` | Add Back button | 2 |
| `src/ui_plugin.rs` | Chat panel, connection dot, night icon, toolbar | 1, 3, 4 |
| `src/render.rs` | Deployment zone, scattergram overlay | 3 |
| `src/fire.rs` | Scattergram visual | 3 |
| `src/combat_card.rs` | Advance-after-combat prompt | 3 |
| `src/dispatch.rs` | Advance-after-combat prompt | 3 |
| `src/main.rs` | Add `DiceAnimationPlugin` | 3 |

### Dependencies between phases

- Phase 1 has no dependencies on later phases.
- Phase 2 has no dependencies on later phases.
- Phase 3.5 (advance prompt) depends on the combat-card infrastructure that
  already exists.
- Phase 4.1 (toolbar) is independent but can absorb the connection dot from
  1.2 and the phase banner from existing code.

Each phase can be implemented independently and merged in any order.
