//! UI interaction trace: what was clicked, selected, deselected, plotted,
//! and which panel buttons were pressed — so the UI can be *observed* as a
//! game progresses (debugging playtests, replay sessions, multiplayer).
//!
//! Two recording styles cooperate:
//!
//! * **Explicit sites** — the board-click funnel
//!   ([`handle_picker_clicks`](crate::picker::handle_picker_clicks)), the
//!   sidebar pick, the path confirm/undo/cancel systems, and pure-UI buttons
//!   record events with precise semantics at the place they happen.
//! * **Watchers** — [`observe_picker_state`] summarizes the picker selection
//!   state machine every frame and records a [`UiTraceEvent::Selection`]
//!   whenever its shape changes, whatever the cause (click, right-click
//!   cancel, phase change, turn change, replay install). Deselection can
//!   thus never escape notice, even via a code path that predates the trace.
//!
//! Events land in the bounded [`UiTrace`] ring buffer (viewable in-app via
//! the **U** key — same idea as the event log's **V** viewer) and are echoed
//! to `tracing` under the `ui_trace` target for native log capture.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use std::collections::VecDeque;

use crate::picker::{MovementPath, PickerState, PlacedUnit};

/// Ring-buffer capacity. Old entries are dropped oldest-first; the viewer
/// shows what remains.
const TRACE_CAP: usize = 600;

/// A resolved selection shape — the *observable* summary of
/// [`PickerState`], with entity references resolved to unit ids and hexes.
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

/// Hex coordinate rendered as `(q, r)` in summaries and event lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HexLabel {
    pub q: i32,
    pub r: i32,
}

impl HexLabel {
    pub fn of(coord: omdurman_types::HexCoord) -> Self {
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

impl SelectionSummary {
    /// Short phrase for event lines ("select stack of 2 at (3,4)").
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

/// One UI interaction observation.
#[derive(Clone, Debug, PartialEq)]
pub enum UiTraceEvent {
    /// A board click (left press) with what stood under the cursor.
    BoardClick { hex: HexLabel, units: Vec<String> },
    /// The picker selection state machine changed shape (any cause).
    Selection {
        from: SelectionSummary,
        to: SelectionSummary,
    },
    /// A path leg was plotted onto the pending movement path.
    PathPlot {
        from: HexLabel,
        to: HexLabel,
        cost: i16,
    },
    /// Enter confirmed the pending path (units moved via MoveUnit events).
    PathConfirmed { legs: usize, cost: i16 },
    /// Backspace removed the last plotted leg.
    PathUndone { legs_left: usize },
    /// The pending path was dropped (cancel / deselect / turn change).
    PathCleared { reason: &'static str },
    /// A counter was picked from the sidebar into the hand.
    PlacementPick { unit: String, via: &'static str },
    /// A pure-UI control was pressed (mode switch, ready, overlay toggles).
    Button { id: &'static str },
}

impl UiTraceEvent {
    /// Category filter key + viewer tint.
    pub fn category(&self) -> Category {
        match self {
            UiTraceEvent::BoardClick { .. } => Category::Clicks,
            UiTraceEvent::Selection { .. } => Category::Selection,
            UiTraceEvent::PathPlot { .. }
            | UiTraceEvent::PathConfirmed { .. }
            | UiTraceEvent::PathUndone { .. }
            | UiTraceEvent::PathCleared { .. } => Category::Path,
            UiTraceEvent::PlacementPick { .. } => Category::Placement,
            UiTraceEvent::Button { .. } => Category::Buttons,
        }
    }

    /// The one-line description shown in the viewer / trace log.
    pub fn line(&self) -> String {
        match self {
            UiTraceEvent::BoardClick { hex, units } => {
                if units.is_empty() {
                    format!("click {hex} (empty)")
                } else {
                    format!("click {hex} on {}", units.join(", "))
                }
            }
            UiTraceEvent::Selection { from, to } => {
                format!(
                    "select: {} -> {}",
                    from.describe_change(),
                    to.describe_change()
                )
            }
            UiTraceEvent::PathPlot { from, to, cost } => {
                format!("path leg {from} -> {to} ({cost} MP)")
            }
            UiTraceEvent::PathConfirmed { legs, cost } => {
                format!("path CONFIRMED ({legs} legs, {cost} MP)")
            }
            UiTraceEvent::PathUndone { legs_left } => {
                format!("path undo ({legs_left} legs left)")
            }
            UiTraceEvent::PathCleared { reason } => format!("path cleared: {reason}"),
            UiTraceEvent::PlacementPick { unit, via } => {
                format!("pick {unit} from sidebar ({via})")
            }
            UiTraceEvent::Button { id } => format!("button: {id}"),
        }
    }
}

/// Event categories, for the viewer's filters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Clicks,
    Selection,
    Path,
    Placement,
    Buttons,
}

const ALL_CATEGORIES: [Category; 5] = [
    Category::Clicks,
    Category::Selection,
    Category::Path,
    Category::Placement,
    Category::Buttons,
];

/// One trace entry: the event plus when/where it happened.
#[derive(Clone, Debug)]
pub struct UiTraceEntry {
    pub seq: u64,
    /// Seconds since app start (matches `Res<Time>`).
    pub at: f64,
    /// Rules-engine turn, when a game is live.
    pub turn: Option<u8>,
    /// Rules-engine phase, when a game is live.
    pub phase: Option<String>,
    pub event: UiTraceEvent,
}

/// Bounded in-memory log of UI interactions (see [module docs](self)).
#[derive(Resource)]
pub struct UiTrace {
    entries: VecDeque<UiTraceEntry>,
    next_seq: u64,
    /// Viewer window open (**U** toggles it).
    pub open: bool,
    /// Viewer freeze: record nothing while set (inspect a frozen window).
    pub paused: bool,
    /// Viewer category filters (parallel to [`ALL_CATEGORIES`]).
    pub visible: [bool; 5],
}

impl Default for UiTrace {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            next_seq: 0,
            open: false,
            paused: false,
            visible: [true; 5],
        }
    }
}

impl UiTrace {
    /// Record one event. No-op while [`paused`](UiTrace::paused) (the
    /// viewer's freeze button), so a frozen window stays frozen.
    pub fn record(
        &mut self,
        at: f64,
        turn: Option<u8>,
        phase: Option<String>,
        event: UiTraceEvent,
    ) {
        if self.paused {
            return;
        }
        let seq = self.next_seq;
        self.next_seq += 1;
        let line = event.line();
        info!(target: "ui_trace", "#{seq:04} [{line}]");
        self.entries.push_back(UiTraceEntry {
            seq,
            at,
            turn,
            phase,
            event,
        });
        while self.entries.len() > TRACE_CAP {
            self.entries.pop_front();
        }
    }

    /// Entries currently held, oldest first.
    pub fn entries(&self) -> impl Iterator<Item = &UiTraceEntry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    fn category_enabled(&self, category: Category) -> bool {
        ALL_CATEGORIES
            .iter()
            .position(|c| *c == category)
            .is_some_and(|i| self.visible[i])
    }
}

/// Summarize a [`PickerState`] into its observable shape, resolving placed
/// entities to human-readable unit labels via the world's placed units.
pub fn summarize_selection(
    state: &PickerState,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    game_state: Option<&crate::GameStateResource>,
) -> SelectionSummary {
    summarize_selection_with(state, |entity| {
        placed_units
            .get(entity)
            .ok()
            .map(|(_, placed)| placed_label(placed, game_state))
            .unwrap_or_else(|| "?".to_string())
    })
}

/// [`summarize_selection`] with an injectable label lookup — the testable
/// core. An entity the lookup can't resolve renders as `?` (a just-despawned
/// counter mid-transition).
pub fn summarize_selection_with(
    state: &PickerState,
    mut label_of: impl FnMut(Entity) -> String,
) -> SelectionSummary {
    match state {
        PickerState::Idle => SelectionSummary::Idle,
        PickerState::Placing {
            unit_idx,
            drag_drop,
            ..
        } => SelectionSummary::Placing {
            // The unit label needs the sidebar tray, which the watcher
            // doesn't hold; the pick event already named it. Here the tray
            // index is the stable reference.
            unit: format!("tray#{unit_idx}"),
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
    match placed
        .unit_id
        .and_then(|uid| game_state.and_then(|gs| gs.0.find_unit(uid)))
    {
        Some(unit) => unit.profile.identity.short_label(),
        None => cell,
    }
}

/// Diff two pending-path snapshots into the path events the watcher should
/// record. Legs extending a common prefix are plots; a differing first leg
/// is noted as a replot. Shrunk/emptied paths produce *no* events here —
/// every removal is recorded by the explicit site that caused it (confirm,
/// undo, cancel, turn change), with the proper reason.
pub fn diff_path(prev: &MovementPath, next: &MovementPath) -> Vec<UiTraceEvent> {
    let common = prev
        .legs
        .iter()
        .zip(next.legs.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let mut out = Vec::new();
    for &(from, to) in &next.legs[common..] {
        out.push(UiTraceEvent::PathPlot {
            from: HexLabel::of(from),
            to: HexLabel::of(to),
            cost: next.cost_so_far,
        });
    }
    if common == 0 && !next.legs.is_empty() && !prev.legs.is_empty() {
        // Replaced rather than extended — the new legs are plotted above;
        // note that the old prefix was discarded wholesale.
        out.push(UiTraceEvent::PathCleared {
            reason: "replotted",
        });
    }
    out
}

// -- Systems -----------------------------------------------------------------

/// Toggle the trace viewer with **U** (not while typing in egui).
pub fn ui_trace_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut trace: ResMut<UiTrace>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    if keys.just_pressed(KeyCode::KeyU) {
        trace.open = !trace.open;
    }
}

/// Watch the picker selection state machine and record every shape change.
/// Runs after every system that can mutate `PickerState` (ordering declared
/// in [`UiTracePlugin`]).
pub fn observe_picker_state(
    time: Res<Time>,
    game_state: Option<Res<crate::GameStateResource>>,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    mut prev: Local<Option<SelectionSummary>>,
    mut trace: ResMut<UiTrace>,
) {
    let current = summarize_selection(&state, &placed_units, game_state.as_deref());
    if prev.as_ref() != Some(&current) {
        let from = prev.take().unwrap_or(SelectionSummary::Idle);
        trace.record(
            time.elapsed_secs_f64(),
            game_state.as_deref().map(|gs| gs.0.current_turn.value()),
            game_state.as_deref().map(phase_label),
            UiTraceEvent::Selection {
                from,
                to: current.clone(),
            },
        );
        *prev = Some(current);
    }
}

/// Watch the pending movement path and record plotted legs (removals and
/// confirms are recorded by their explicit systems with proper reasons).
pub fn observe_movement_path(
    time: Res<Time>,
    game_state: Option<Res<crate::GameStateResource>>,
    path: Res<MovementPath>,
    mut prev: Local<Option<MovementPath>>,
    mut trace: ResMut<UiTrace>,
) {
    if let Some(previous) = prev.as_ref() {
        for event in diff_path(previous, &path) {
            if let UiTraceEvent::PathCleared { .. } = event {
                // Clears are recorded by their cause (confirm / undo /
                // cancel / turn change); a bare watcher clear would be
                // redundant noise for those and unattributed for the rest.
                continue;
            }
            trace.record(
                time.elapsed_secs_f64(),
                game_state.as_deref().map(|gs| gs.0.current_turn.value()),
                game_state.as_deref().map(phase_label),
                event,
            );
        }
    }
    if path.is_changed() || prev.is_none() {
        *prev = Some(path.clone());
    }
}

/// Watch the top-level mode (Menu / Game / Spectating) for the trace.
pub fn observe_app_mode(
    time: Res<Time>,
    mode: Res<State<crate::state::AppMode>>,
    mut prev: Local<Option<crate::state::AppMode>>,
    mut trace: ResMut<UiTrace>,
) {
    if prev.as_ref() != Some(mode.get()) {
        prev.replace(*mode.get());
        trace.record(
            time.elapsed_secs_f64(),
            None,
            None,
            UiTraceEvent::Button {
                id: match mode.get() {
                    crate::state::AppMode::Menu => "mode -> Menu",
                    crate::state::AppMode::Lobby => "mode -> Lobby",
                    crate::state::AppMode::Game => "mode -> Game",
                },
            },
        );
    }
}

fn phase_label(gs: &crate::GameStateResource) -> String {
    format!("{:?}", gs.0.phase)
}

// -- Viewer ------------------------------------------------------------------

pub struct UiTracePlugin;

impl Plugin for UiTracePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiTrace>()
            .add_systems(
                Update,
                (
                    observe_app_mode,
                    // (The sidebar's own `PickerState` writes happen in
                    // `EguiPrimaryContextPass`; a pick is therefore observed
                    // on the next Update — one frame late, which is fine.)
                    observe_picker_state
                        .after(crate::picker::handle_picker_clicks)
                        .after(crate::picker::reset_selection_on_phase_change)
                        .after(crate::picker::clear_paths_on_turn_change)
                        .after(crate::picker::cancel_placement)
                        .after(crate::picker::confirm_movement_path)
                        .after(crate::picker::undo_movement_leg)
                        .after(crate::picker::delete_selected_unit),
                    observe_movement_path
                        .after(crate::picker::handle_picker_clicks)
                        .after(crate::picker::confirm_movement_path)
                        .after(crate::picker::undo_movement_leg)
                        .after(crate::picker::clear_paths_on_turn_change)
                        .after(crate::picker::cancel_placement)
                        .after(crate::picker::clear_movement_path_when_idle),
                    ui_trace_toggle,
                ),
            )
            .add_systems(EguiPrimaryContextPass, ui_trace_ui);
    }
}

/// The **U**-toggled trace window: filterable, freezable, clearable.
pub fn ui_trace_ui(mut contexts: EguiContexts, mut trace: ResMut<UiTrace>) {
    if !trace.open {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let mut open = trace.open;
    egui::Window::new("UI trace (U)")
        .open(&mut open)
        .id(egui::Id::new("ui_trace_window"))
        .default_width(560.0)
        .default_height(420.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, category) in ALL_CATEGORIES.iter().enumerate() {
                    ui.toggle_value(&mut trace.visible[i], format!("{category:?}"));
                }
                ui.separator();
                if ui.button("Freeze").clicked() {
                    trace.paused = !trace.paused;
                }
                if trace.paused {
                    ui.colored_label(egui::Color32::from_rgb(230, 200, 110), "frozen");
                }
                if ui.button("Clear").clicked() {
                    trace.clear();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{} entries", trace.len()));
                });
            });
            ui.separator();
            egui::ScrollArea::vertical()
                .id_salt("ui_trace_list")
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(11.0));
                    for entry in trace.entries() {
                        if !trace.category_enabled(entry.event.category()) {
                            continue;
                        }
                        let tint = match entry.event.category() {
                            Category::Clicks => egui::Color32::from_rgb(200, 170, 120),
                            Category::Selection => egui::Color32::from_rgb(140, 190, 240),
                            Category::Path => egui::Color32::from_rgb(150, 210, 150),
                            Category::Placement => egui::Color32::from_rgb(210, 150, 220),
                            Category::Buttons => egui::Color32::from_gray(170),
                        };
                        let stamp = match (&entry.turn, &entry.phase) {
                            (Some(turn), Some(phase)) => format!("T{turn} {phase}"),
                            (None, Some(phase)) => phase.clone(),
                            _ => "-".to_string(),
                        };
                        ui.colored_label(
                            egui::Color32::from_gray(120),
                            format!("#{:04} {:7.1}s {:26}", entry.seq, entry.at, stamp),
                        );
                        ui.colored_label(tint, format!("    {}", entry.event.line()));
                    }
                });
        });
    // The Window's own close button (X) writes through `open` — sync it
    // back so the U key toggles from the right state.
    trace.open = open;
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_types::HexCoord;
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
        let summary = summarize_selection_with(&selected(coord(3, 4)), |e| {
            labels.get(&e).cloned().unwrap_or_else(|| "?".to_string())
        });
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
        let summary = summarize_selection_with(&selected(coord(0, 0)), |_| "?".into());
        assert!(matches!(summary, SelectionSummary::Single { ref unit, .. } if unit == "?"));
    }

    #[test]
    fn summarize_counts_stack_members_without_enumerating() {
        let state = PickerState::SelectedStack(crate::picker::StackSelection {
            sources: vec![
                Entity::PLACEHOLDER,
                Entity::PLACEHOLDER,
                Entity::PLACEHOLDER,
            ],
            start_coord: coord(1, 2),
            remaining_mp: vec![3, 2, 5],
            initial_mp: vec![3, 2, 5],
            forced_stop: false,
        });
        let summary = summarize_selection_with(&state, |_| "x".into());
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
        let events = diff_path(&prev, &next);
        assert_eq!(
            events,
            vec![UiTraceEvent::PathPlot {
                from: HexLabel::of(coord(1, 0)),
                to: HexLabel::of(coord(2, 0)),
                cost: 3,
            }]
        );
    }

    #[test]
    fn path_removal_is_left_to_the_explicit_sites() {
        // Undo/cancel/confirm each record their own reasoned event; the
        // watcher's diff must stay quiet about the removal itself, so a
        // cleared path produces no unattributed events.
        let mut prev = MovementPath::default();
        prev.legs.push((coord(0, 0), coord(1, 0)));
        let events = diff_path(&prev, &MovementPath::default());
        assert!(events.is_empty());
    }

    #[test]
    fn path_replaced_from_the_first_leg_notes_the_replot() {
        let mut prev = MovementPath::default();
        prev.legs.push((coord(0, 0), coord(1, 0)));
        prev.legs.push((coord(1, 0), coord(2, 0)));
        let mut next = MovementPath::default();
        next.legs.push((coord(5, 5), coord(4, 5)));
        let events = diff_path(&prev, &next);
        assert!(events.contains(&UiTraceEvent::PathCleared {
            reason: "replotted",
        }));
        assert!(matches!(
            events.first(),
            Some(UiTraceEvent::PathPlot { .. })
        ));
    }

    #[test]
    fn ring_buffer_evicts_oldest_beyond_capacity() {
        let mut trace = UiTrace::default();
        for i in 0..(TRACE_CAP as u64 + 25) {
            trace.record(i as f64, None, None, UiTraceEvent::Button { id: "spam" });
        }
        assert_eq!(trace.len(), TRACE_CAP);
        let first = trace.entries().next().unwrap();
        assert_eq!(first.seq, 25, "oldest entries were evicted first");
        assert_eq!(trace.next_seq, TRACE_CAP as u64 + 25, "seqs stay monotonic");
    }

    #[test]
    fn pause_freezes_recording_but_keeps_existing_entries() {
        let mut trace = UiTrace::default();
        trace.record(1.0, None, None, UiTraceEvent::Button { id: "before" });
        trace.paused = true;
        trace.record(2.0, None, None, UiTraceEvent::Button { id: "frozen-out" });
        assert_eq!(trace.len(), 1);
        assert_eq!(
            trace.entries().next().unwrap().event,
            UiTraceEvent::Button { id: "before" }
        );
    }

    #[test]
    fn category_filters_start_all_visible() {
        let trace = UiTrace::default();
        assert!(trace.visible.iter().all(|v| *v));
        for category in ALL_CATEGORIES {
            assert!(trace.category_enabled(category));
        }
    }
}
