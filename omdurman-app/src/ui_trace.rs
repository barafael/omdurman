//! UI interaction tracing: what was clicked, selected, deselected, plotted,
//! and which panel buttons were pressed — emitted as plain `tracing` events
//! so the UI can be *observed* as the game progresses by watching the log.
//!
//! All events use the `ui_trace` target; show them (native, with Bevy's
//! `LogPlugin`) via the log filter, e.g. `RUST_LOG=ui_trace=info`, or leave
//! the default filter in place — the events are `info` level and pass it.
//!
//! Two recording styles cooperate:
//!
//! * **Explicit sites** — the board-click router
//!   ([`route_board_clicks`](crate::board_click::route_board_clicks)), the
//!   sidebar pick, the path confirm/undo/cancel systems, and pure-UI buttons
//!   call the [`emit`] helpers where the interaction happens, with precise
//!   semantics and reasons.
//! * **Watchers** — [`observe_picker_state`] summarizes the picker selection
//!   state machine every frame and emits a `selection` event whenever its
//!   shape changes, whatever the cause (click, right-click cancel, phase
//!   change, turn change, replay install). Deselection can thus never
//!   escape notice, even via a code path that predates the trace.
//!
//! Every event carries the rules-engine turn and phase when a game is live
//! (the log subscriber adds the wall-clock timestamp itself).

use bevy::prelude::*;
use omdurman_types::HexCoord;

use crate::picker::{MovementPath, PickerState, PlacedUnit};

/// `tracing` target shared by every UI-trace event, so the stream is
/// filterable (`ui_trace=info`) and greppable.
pub const TARGET: &str = "ui_trace";

/// Turn/phase stamp attached to events emitted where a game state is at
/// hand. `None` fields render as `-` — no live game (menu, editor).
#[derive(Clone, Debug, Default)]
pub struct Stamp {
    pub turn: Option<u8>,
    pub phase: Option<String>,
}

impl Stamp {
    pub fn of(gs: Option<&crate::GameStateResource>) -> Self {
        gs.map(|gs| Self {
            turn: Some(gs.0.current_turn.value()),
            phase: Some(format!("{:?}", gs.0.phase)),
        })
        .unwrap_or_default()
    }

    pub const NONE: Self = Self {
        turn: None,
        phase: None,
    };
}

/// Hex coordinate rendered as `(q, r)` in event fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HexLabel {
    pub q: i32,
    pub r: i32,
}

impl HexLabel {
    pub fn of(coord: HexCoord) -> Self {
        Self {
            q: coord.q,
            r: coord.r,
        }
    }
}

impl std::fmt::Display for HexLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.q, self.r)
    }
}

/// A resolved selection shape — the *observable* summary of
/// [`PickerState`], with entity references resolved to unit labels and
/// hexes.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectionSummary {
    Idle,
    Placing {
        unit: String,
        drag: bool,
    },
    /// One counter selected (movement plot or combat actor).
    Single {
        unit: String,
        hex: HexLabel,
    },
    /// A double-clicked stack moving as one group.
    Stack {
        units: usize,
        hex: HexLabel,
    },
    /// A combat-phase whole-tile selection (fire / melee actors).
    Tile {
        units: Vec<String>,
        hex: HexLabel,
    },
}

impl SelectionSummary {
    /// Short phrase for event fields ("select stack of 2 at (3,4)").
    pub fn describe_change(&self) -> String {
        match self {
            SelectionSummary::Idle => "nothing".to_string(),
            SelectionSummary::Placing { unit, drag } => {
                format!("in hand: {unit}{}", if *drag { " (drag)" } else { "" })
            }
            SelectionSummary::Single { unit, hex } => format!("unit {unit} at {hex}"),
            SelectionSummary::Stack { units, hex } => {
                format!("stack of {units} at {hex}")
            }
            SelectionSummary::Tile { units, hex } => {
                format!("tile of {} at {hex}", units.len())
            }
        }
    }
}

// -- Emit helpers (the explicit call sites use these) -------------------------

/// Emit one `tracing` event on the UI-trace target. The single seam between
/// the call sites and the log format.
fn emit(stamp: &Stamp, message: &str, fields: impl FnOnce() -> Vec<String>) {
    let turn = stamp
        .turn
        .map(|t| t.to_string())
        .unwrap_or_else(|| "-".to_string());
    let phase = stamp.phase.clone().unwrap_or_else(|| "-".to_string());
    info!(
        target: TARGET,
        turn = %turn,
        phase = %phase,
        "{message} {}",
        fields().join(" "),
    );
}

/// A board click (left press) with the units standing under the cursor —
/// including clicks that change nothing, so rejected clicks stay visible.
pub fn board_click(hex: HexLabel, units: &[String], stamp: &Stamp) {
    let hex = hex.to_string();
    emit(stamp, "click", || {
        if units.is_empty() {
            vec![format!("on {hex} (empty)")]
        } else {
            vec![format!("on {hex} at {}", units.join(","))]
        }
    });
}

/// The picker selection state machine changed shape (any cause).
pub fn selection(from: &SelectionSummary, to: &SelectionSummary, stamp: &Stamp) {
    let (from, to) = (from.describe_change(), to.describe_change());
    emit(stamp, "select", || vec![format!("{from} -> {to}")]);
}

/// A path leg was plotted onto the pending movement path.
pub fn path_plot(from: HexLabel, to: HexLabel, cost: i16, stamp: &Stamp) {
    let (from, to) = (from.to_string(), to.to_string());
    emit(stamp, "path", || {
        vec![format!("{from} -> {to} ({cost} MP)")]
    });
}

/// Enter confirmed the pending path (units moved via MoveUnit events).
pub fn path_confirmed(legs: usize, cost: i16, stamp: &Stamp) {
    emit(stamp, "path CONFIRMED", || {
        vec![format!("({legs} legs, {cost} MP)")]
    });
}

/// Backspace removed the last plotted leg.
pub fn path_undone(legs_left: usize, stamp: &Stamp) {
    emit(stamp, "path undo", || {
        vec![format!("({legs_left} legs left)")]
    });
}

/// The pending path was dropped (cancel / deselect / turn change).
pub fn path_cleared(reason: &str, stamp: &Stamp) {
    emit(stamp, "path cleared", || vec![format!(": {reason}")]);
}

/// A counter was picked from the sidebar into the hand.
pub fn placement_pick(unit: &str, via: &str, stamp: &Stamp) {
    emit(stamp, "pick", || {
        vec![format!("{unit} from sidebar ({via})")]
    });
}

/// A pure-UI control was pressed (overlay toggles, remove-unit, mode
/// switches). Buttons that dispatch game effects are *not* recorded here —
/// they are visible in the network event log already.
pub fn button(id: &str) {
    emit(&Stamp::NONE, "button", || vec![format!(": {id}")]);
}

// -- Selection summarization ---------------------------------------------------

/// Summarize a [`PickerState`] into its observable shape, resolving placed
/// entities to human-readable unit labels via the world's placed units, and
/// the counter in hand via the sidebar tray (auto-next picks it without a
/// sidebar click, so no `pick` event names it).
pub fn summarize_selection(
    state: &PickerState,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    picker: Option<&crate::picker::UnitPicker>,
    game_state: Option<&crate::GameStateResource>,
) -> SelectionSummary {
    summarize_selection_with(
        state,
        |entity| {
            placed_units
                .get(entity)
                .ok()
                .map(|(_, placed)| placed_label(placed, game_state))
                .unwrap_or_else(|| "?".to_string())
        },
        |idx| {
            let tray = format!("tray#{idx}");
            match picker.and_then(|p| p.available.get(idx)) {
                Some(u) => format!(
                    "{tray} ({} {},{})",
                    u.section_name.display_name(),
                    u.col,
                    u.row
                ),
                None => tray,
            }
        },
    )
}

/// [`summarize_selection`] with injectable label lookups (placed entity,
/// tray index) — the testable core. An entity the lookup can't resolve
/// renders as `?` (a just-despawned counter mid-transition).
pub fn summarize_selection_with(
    state: &PickerState,
    mut label_of: impl FnMut(Entity) -> String,
    tray_label: impl Fn(usize) -> String,
) -> SelectionSummary {
    match state {
        PickerState::Idle => SelectionSummary::Idle,
        PickerState::Placing {
            unit_idx,
            drag_drop,
            ..
        } => SelectionSummary::Placing {
            unit: tray_label(*unit_idx),
            drag: *drag_drop,
        },
        PickerState::Selected {
            source,
            start_coord,
            ..
        } => SelectionSummary::Single {
            unit: label_of(*source),
            hex: HexLabel::of(*start_coord),
        },
        PickerState::SelectedStack(sel) => SelectionSummary::Stack {
            units: sel.sources.len(),
            hex: HexLabel::of(sel.start_coord),
        },
        PickerState::SelectedTile(sel) => SelectionSummary::Tile {
            units: sel.sources.iter().map(|&e| label_of(e)).collect(),
            hex: HexLabel::of(sel.start_coord),
        },
    }
}

/// Human label for a placed counter: rules identity when known, sprite cell
/// otherwise.
pub fn placed_label(placed: &PlacedUnit, game_state: Option<&crate::GameStateResource>) -> String {
    let cell = format!(
        "{} {},{}",
        placed.section_name.display_name(),
        placed.col,
        placed.row
    );
    let scenario = game_state.map_or(omdurman_types::Scenario::Campaign, |gs| gs.0.scenario);
    match placed
        .unit_id
        .and_then(|uid| game_state.and_then(|gs| gs.0.find_unit(uid)))
    {
        Some(unit) => unit.profile.identity.label_in(scenario),
        None => cell,
    }
}

/// What a pending-path change looks like in the trace: legs extending a
/// common prefix are plots; a differing first leg is noted as a replot.
/// Shrunk/emptied paths produce *no* diff events — every removal is
/// recorded by the explicit site that caused it (confirm, undo, cancel,
/// turn change), with the proper reason.
#[derive(Clone, Debug, PartialEq)]
pub enum PathDiff {
    Plot {
        from: HexLabel,
        to: HexLabel,
        cost: i16,
    },
    Replotted,
}

/// Diff two pending-path snapshots into the path events the watcher should
/// emit.
pub fn diff_path(prev: &MovementPath, next: &MovementPath) -> Vec<PathDiff> {
    let common = prev
        .legs
        .iter()
        .zip(next.legs.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let mut out: Vec<PathDiff> = next.legs[common..]
        .iter()
        .map(|&(from, to)| PathDiff::Plot {
            from: HexLabel::of(from),
            to: HexLabel::of(to),
            cost: next.cost_so_far,
        })
        .collect();
    if common == 0 && !next.legs.is_empty() && !prev.legs.is_empty() {
        // Replaced rather than extended — the new legs are plotted above;
        // note that the old prefix was discarded wholesale.
        out.push(PathDiff::Replotted);
    }
    out
}

// -- Watcher systems ------------------------------------------------------------

/// Watch the picker selection state machine and emit an event for every
/// shape change. Runs after every system that can mutate `PickerState`
/// (ordering declared in [`UiTracePlugin`]).
pub fn observe_picker_state(
    game_state: Option<Res<crate::GameStateResource>>,
    state: Res<PickerState>,
    picker: Option<Res<crate::picker::UnitPicker>>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    mut prev: Local<Option<SelectionSummary>>,
) {
    let current = summarize_selection(
        &state,
        &placed_units,
        picker.as_deref(),
        game_state.as_deref(),
    );
    if prev.as_ref() != Some(&current) {
        let from = prev.take().unwrap_or(SelectionSummary::Idle);
        selection(&from, &current, &Stamp::of(game_state.as_deref()));
        *prev = Some(current);
    }
}

/// Watch the pending movement path and emit plot events (removals and
/// confirms are emitted by their explicit systems with proper reasons).
pub fn observe_movement_path(
    game_state: Option<Res<crate::GameStateResource>>,
    path: Res<MovementPath>,
    mut prev: Local<Option<MovementPath>>,
) {
    if let Some(previous) = prev.as_ref() {
        let stamp = Stamp::of(game_state.as_deref());
        for diff in diff_path(previous, &path) {
            match diff {
                PathDiff::Plot { from, to, cost } => path_plot(from, to, cost, &stamp),
                PathDiff::Replotted => path_cleared("replotted", &stamp),
            }
        }
    }
    if path.is_changed() || prev.is_none() {
        *prev = Some(path.clone());
    }
}

/// Watch the top-level mode (Menu / Lobby / Game) for the trace.
pub fn observe_app_mode(
    mode: Res<State<crate::state::AppMode>>,
    mut prev: Local<Option<crate::state::AppMode>>,
) {
    if prev.as_ref() != Some(mode.get()) {
        prev.replace(*mode.get());
        let id = match mode.get() {
            crate::state::AppMode::Menu => "mode -> Menu",
            crate::state::AppMode::Lobby => "mode -> Lobby",
            crate::state::AppMode::Game => "mode -> Game",
        };
        button(id);
    }
}

pub struct UiTracePlugin;

impl Plugin for UiTracePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                observe_app_mode,
                // (The sidebar's own `PickerState` writes happen in
                // `EguiPrimaryContextPass`; a pick is therefore observed on
                // the next Update — one frame late, which is fine.)
                observe_picker_state
                    .after(crate::board_click::BoardClickHandlerSet)
                    .after(crate::picker::reset_selection_on_phase_change)
                    .after(crate::picker::clear_paths_on_turn_change)
                    .after(crate::picker::cancel_placement)
                    .after(crate::picker::confirm_movement_path)
                    .after(crate::picker::undo_movement_leg)
                    .after(crate::picker::delete_selected_unit),
                observe_movement_path
                    .after(crate::board_click::BoardClickHandlerSet)
                    .after(crate::picker::confirm_movement_path)
                    .after(crate::picker::undo_movement_leg)
                    .after(crate::picker::clear_paths_on_turn_change)
                    .after(crate::picker::cancel_placement)
                    .after(crate::picker::clear_movement_path_when_idle),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn coord(q: i32, r: i32) -> HexCoord {
        HexCoord::new(q, r)
    }

    fn selected(hex: HexCoord) -> PickerState {
        PickerState::Selected {
            source: Entity::PLACEHOLDER,
            start_coord: hex,
            remaining_mp: 4,
            forced_stop: false,
        }
    }

    #[test]
    fn summarize_resolves_labels_and_hex() {
        let mut labels = HashMap::new();
        labels.insert(Entity::PLACEHOLDER, "1B 1st Btn".to_string());
        let summary = summarize_selection_with(
            &selected(coord(3, 4)),
            |e| labels.get(&e).cloned().unwrap_or_else(|| "?".to_string()),
            |i| format!("tray#{i}"),
        );
        assert_eq!(
            summary,
            SelectionSummary::Single {
                unit: "1B 1st Btn".into(),
                hex: HexLabel { q: 3, r: 4 },
            }
        );
        assert_eq!(summary.describe_change(), "unit 1B 1st Btn at (3,4)");
    }

    #[test]
    fn summarize_unresolvable_entity_renders_as_unknown() {
        let summary =
            summarize_selection_with(&selected(coord(0, 0)), |_| "?".into(), |_| "?".into());
        assert!(matches!(summary, SelectionSummary::Single { ref unit, .. } if unit == "?"));
    }

    #[test]
    fn summarize_counts_stack_members_without_enumerating() {
        let state = PickerState::SelectedStack(crate::picker::StackSelection {
            sources: vec![Entity::PLACEHOLDER; 3],
            start_coord: coord(1, 2),
            remaining_mp: vec![3, 2, 5],
            initial_mp: vec![3, 2, 5],
            forced_stop: false,
        });
        let summary = summarize_selection_with(&state, |_| "x".into(), |_| "x".into());
        assert_eq!(
            summary,
            SelectionSummary::Stack {
                units: 3,
                hex: HexLabel { q: 1, r: 2 },
            }
        );
    }

    #[test]
    fn path_extension_plots_only_the_new_legs() {
        let mut prev = MovementPath::default();
        prev.legs.push((coord(0, 0), coord(1, 0)));
        prev.cost_so_far = 1;
        let mut next = prev.clone();
        next.legs.push((coord(1, 0), coord(2, 0)));
        next.cost_so_far = 3;
        let diffs = diff_path(&prev, &next);
        assert_eq!(
            diffs,
            vec![PathDiff::Plot {
                from: HexLabel::of(coord(1, 0)),
                to: HexLabel::of(coord(2, 0)),
                cost: 3,
            }]
        );
    }

    #[test]
    fn path_removal_is_left_to_the_explicit_sites() {
        // Undo/cancel/confirm each emit their own reasoned event; the
        // watcher's diff must stay quiet about the removal itself, so a
        // cleared path produces no unattributed events.
        let mut prev = MovementPath::default();
        prev.legs.push((coord(0, 0), coord(1, 0)));
        let diffs = diff_path(&prev, &MovementPath::default());
        assert!(diffs.is_empty());
    }

    #[test]
    fn path_replaced_from_the_first_leg_notes_the_replot() {
        let mut prev = MovementPath::default();
        prev.legs.push((coord(0, 0), coord(1, 0)));
        prev.legs.push((coord(1, 0), coord(2, 0)));
        let mut next = MovementPath::default();
        next.legs.push((coord(5, 5), coord(4, 5)));
        let diffs = diff_path(&prev, &next);
        assert!(diffs.contains(&PathDiff::Replotted));
        assert!(matches!(diffs.first(), Some(PathDiff::Plot { .. })));
    }

    #[test]
    fn hex_label_displays_as_axial_pair() {
        assert_eq!(HexLabel::of(coord(-1, 2)).to_string(), "(-1,2)");
    }
}
