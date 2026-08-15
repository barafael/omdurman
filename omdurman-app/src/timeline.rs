//! Spectator timeline scrubber (§spectator).
//!
//! Reviews a recorded [`GameRecord`] — the in-memory game or one loaded from a
//! `games/*/events.jsonl` file — by rebuilding rules/map state to an arbitrary event
//! index. Rewind is *replay-from-start*: to show event `N` we reset to the
//! record's seed and re-apply events `0..=N` via
//! [`crate::rebuild_state_to`]. `ChaCha8Rng` can't resume mid-stream, so
//! reseeding every rebuild (rather than snapshotting the RNG) is deliberate.
//!
//! While [`AppState::Spectating`] is active there is no live socket; the net
//! systems are gated off and the scrubber owns the world state.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use bevy_matchbox::prelude::PeerId;
use omdurman_hexmap::{GameMap, load_map_data};
use omdurman_net::{GameEvent, GameRecord};
use omdurman_rules::board::BoardInfo;
use omdurman_rules::board_data;
use omdurman_rules::effects::GameState;

use crate::{
    AppState, GameRng, LoadedAnnotations, PendingIncoming, PendingMapLoad,
    editor::HexEditor, game_apply, map_kind_for_scenario, render::HexOverlay, units::UnitViewer,
};

/// Mutable state bundle for [`rebuild_state_to`].
///
/// Groups the 11 `&mut` parameters so the function signature stays short.
/// NOT a `SystemParam` — this is a plain struct for a non-system function.
pub(crate) struct RebuildState<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub game_map: &'a mut GameMap,
    pub overlay: &'a mut HexOverlay,
    pub editor: &'a mut HexEditor,
    pub viewer: &'a mut UnitViewer,
    pub replay: &'a mut Vec<(GameEvent, PeerId)>,
    pub game_state: &'a mut GameState,
    pub queued_factions: &'a mut crate::peers::QueuedFactions,
    pub loaded_annotations: &'a mut LoadedAnnotations,
    pub pending_map_load: &'a mut PendingMapLoad,
}

/// The record under review plus the scrubber's cursor and playback state.
/// Absent (`record: None`) until a game is opened for review.
#[derive(Resource, Default)]
pub struct SpectatorTimeline {
    /// The event log being reviewed. `None` when not spectating a record.
    pub record: Option<GameRecord>,
    /// Index of the last event applied to the currently-shown state.
    pub cursor: usize,
    /// `true` while auto-advancing; the play loop steps `cursor` on a timer.
    pub playing: bool,
    /// Seconds since the last auto-advance step (playback pacing).
    pub play_accum: f32,
    /// Set when `cursor` changed and the world must be rebuilt to match. The
    /// rebuild is deferred to [`scrub_teardown`]/[`scrub_rebuild`] so the heavy
    /// work runs in normal systems with full world access, not inside the egui
    /// pass.
    pub dirty: bool,
    /// Where a loaded record came from, for the panel header. Empty for the
    /// in-memory game.
    pub source_label: String,
}

impl SpectatorTimeline {
    /// Begin reviewing `record` from its final event, marking a rebuild.
    pub fn open(&mut self, record: GameRecord, source_label: String) {
        self.cursor = record.events.len().saturating_sub(1);
        self.record = Some(record);
        self.playing = false;
        self.play_accum = 0.0;
        self.source_label = source_label;
        self.dirty = true;
    }

    /// Number of events in the open record (0 if none).
    pub fn len(&self) -> usize {
        self.record.as_ref().map_or(0, |r| r.events.len())
    }

    /// Move the cursor to `idx` (clamped) and mark a rebuild if it changed.
    fn seek(&mut self, idx: usize) {
        let max = self.len().saturating_sub(1);
        let idx = idx.min(max);
        if idx != self.cursor {
            self.cursor = idx;
            self.dirty = true;
        }
    }
}

/// Seconds between auto-advance steps while playing.
const PLAY_STEP_SECS: f32 = 0.6;

/// Auto-advance the cursor while playing; stops at the end.
pub fn advance_timeline_playback(time: Res<Time>, mut timeline: ResMut<SpectatorTimeline>) {
    if !timeline.playing || timeline.record.is_none() {
        return;
    }
    let last = timeline.len().saturating_sub(1);
    if timeline.cursor >= last {
        timeline.playing = false;
        return;
    }
    timeline.play_accum += time.delta_secs();
    if timeline.play_accum >= PLAY_STEP_SECS {
        timeline.play_accum = 0.0;
        let next = timeline.cursor + 1;
        timeline.seek(next);
    }
}

/// Resources needed for the teardown phase of a timeline scrub: despawn
/// placed-unit entities, clear movement paths, and reset the picker.
#[derive(SystemParam)]
pub struct ScrubTeardown<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub placed_units: Query<'w, 's, Entity, With<crate::picker::PlacedUnit>>,
    pub unit_paths: ResMut<'w, crate::picker::UnitPaths>,
    pub picker: ResMut<'w, crate::picker::UnitPicker>,
    pub picker_state: ResMut<'w, crate::picker::PickerState>,
    /// Despawned so the rebuild starts with an empty peer set; the reviewed
    /// record's `StartGame` binding is re-applied via `QueuedFactions`.
    pub peer_entities: Query<'w, 's, Entity, With<crate::peers::Peer>>,
}

/// Resources needed for the rebuild phase of a timeline scrub: reset the map,
/// overlay, editor, rules state, and annotations, then replay events.
#[derive(SystemParam)]
pub struct ScrubRebuild<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub game_map: ResMut<'w, omdurman_hexmap::GameMap>,
    pub overlay: ResMut<'w, crate::render::HexOverlay>,
    pub editor: ResMut<'w, crate::editor::HexEditor>,
    pub viewer: ResMut<'w, crate::units::UnitViewer>,
    pub game_state: ResMut<'w, crate::GameStateResource>,
    pub queued_factions: ResMut<'w, crate::peers::QueuedFactions>,
    pub loaded_annotations: ResMut<'w, crate::LoadedAnnotations>,
    pub pending_map_load: ResMut<'w, crate::PendingMapLoad>,
    /// The review shows the play board (the board itself is (re)loaded from
    /// `pending_map_load`, and follows the reviewed scenario via the play-view
    /// board reconciler, §dual-map).
    pub next_app_mode: ResMut<'w, NextState<crate::AppMode>>,
}

/// When the timeline cursor is dirty, rebuild the whole world to that event.
///
/// Because a re-scrub runs over an *already populated* world (unlike the live
/// late-joiner path, which starts empty), it first tears down the ephemeral
/// state that [`crate::rebuild_state_to`] does not itself reset — placed-unit
/// entities, movement paths, and the picker — then replays `0..=cursor`. The
/// queued placement events are spawned by `apply_pending_placement` on the
/// following frame, exactly as in live replay.
///
/// Split into two chained systems [`scrub_teardown`] → [`scrub_rebuild`] so
/// each SystemParam bundle stays focused on a single phase.
pub fn scrub_teardown(
    mut timeline: ResMut<SpectatorTimeline>,
    mut incoming: ResMut<PendingIncoming>,
    mut teardown: ScrubTeardown,
) {
    if !timeline.dirty {
        return;
    }
    if timeline.record.is_none() {
        timeline.dirty = false;
        return;
    }
    // Tear down populated ephemeral state so the rebuild starts clean.
    for entity in &teardown.placed_units {
        teardown.commands.entity(entity).despawn();
    }
    for entity in &teardown.peer_entities {
        teardown.commands.entity(entity).despawn();
    }
    teardown.unit_paths.0.clear();
    teardown.picker.reset_available();
    *teardown.picker_state = crate::picker::PickerState::Idle;
    // The replay queue and any pending live placements are stale across a scrub.
    incoming.replay.clear();
    incoming.live.clear();
}

/// Rebuild phase of the timeline scrub: replays events `0..=cursor` and
/// switches to the game view. Runs after [`scrub_teardown`].
pub fn scrub_rebuild(
    mut timeline: ResMut<SpectatorTimeline>,
    mut incoming: ResMut<PendingIncoming>,
    mut rebuild: ScrubRebuild,
) {
    if !timeline.dirty {
        return;
    }
    let Some(record) = timeline.record.clone() else {
        timeline.dirty = false;
        return;
    };
    timeline.dirty = false;

    let history_peer = PeerId(uuid::Uuid::nil());
    {
        let mut state = RebuildState {
            commands: &mut rebuild.commands,
            game_map: &mut rebuild.game_map,
            overlay: &mut rebuild.overlay,
            editor: &mut rebuild.editor,
            viewer: &mut rebuild.viewer,
            replay: &mut incoming.replay,
            game_state: &mut rebuild.game_state.0,
            queued_factions: &mut rebuild.queued_factions,
            loaded_annotations: &mut rebuild.loaded_annotations,
            pending_map_load: &mut rebuild.pending_map_load,
        };
        rebuild_state_to(&record, Some(timeline.cursor), history_peer, &mut state);
    }

    // Show the reviewed game on the play board (rebuild_state_to queued the
    // board data via PendingMapLoad; the reconciler keeps it on the reviewed
    // scenario's map while in a play view).
    rebuild.next_app_mode.set(crate::AppMode::Game);
}

/// Rebuild game + map state from the canonical event log, applying events
/// `0..=upto` (or all events when `upto` is `None`). The reset-from-seed + full
/// forward replay is the same mechanism the live late-joiner path uses; the
/// bounded form drives the spectator timeline scrubber (§spectator), which shows
/// the state as it was after event `upto`.
///
/// This rebuilds only the rules/map state and queues placement events into
/// `replay`; the caller is responsible for despawning any stale `PlacedUnit`
/// entities and clearing `UnitPaths`/`PickerState` before a *re-scrub* of an
/// already-populated world (the live path starts from an empty world, so it
/// needs no such reset).
pub(crate) fn rebuild_state_to(
    record: &GameRecord,
    upto: Option<usize>,
    history_peer: PeerId,
    state: &mut RebuildState<'_, '_, '_>,
) {
    let upto = upto.unwrap_or(record.events.len().saturating_sub(1));
    info!(
        upto,
        total = record.events.len(),
        "rebuilding state from log"
    );

    // Reset RNG + clear map -- the event stream is canonical so we rebuild
    // from a known state.
    state.commands.insert_resource(GameRng::from_seed(record.initial_state.seed));
    state.game_map.hexes.clear();

    // Seed LoadedAnnotations from compiled codegen data and load the default
    // board (Fall-of-Khartoum) into the live map so replay events (MapEdit,
    // PlaceUnit, etc.) have valid hexes to target. This replaces the old
    // LoadAnnotations network event that seeded the map at runtime.
    state.loaded_annotations.campaign = board_data::campaign_map_data();
    state.loaded_annotations.fall_of_khartoum = board_data::fall_of_khartoum_map_data();
    load_map_data(
        state.loaded_annotations.map(omdurman_types::MapKind::FallOfKhartoum),
        &mut *state.game_map,
    );

    let mut ctx = game_apply::GameApplyCtx {
        game_map: &mut *state.game_map,
        overlay: &mut *state.overlay,
        editor: &mut *state.editor,
        viewer: &mut *state.viewer,
        game_state: Some(&mut *state.game_state),
        loaded_annotations: Some(&mut *state.loaded_annotations),
        // Replay rebuilds from the default board; `apply_map_selection` reloads
        // the scenario's board from the accumulated `LoadedAnnotations` after
        // replay completes (§dual-map).
        active_map: omdurman_types::MapKind::FallOfKhartoum,
    };
    let end = (upto + 1).min(record.events.len());
    for event in &record.events[..end] {
        match &event.payload {
            GameEvent::PlaceUnit { .. }
            | GameEvent::MoveUnit { .. }
            | GameEvent::RemoveUnit { .. } => {
                state.replay.push((event.payload.clone(), history_peer));
                continue;
            }
            // Reconstruct the faction binding for a late joiner from the
            // recorded host commit (§lobby); the engine state's active player
            // is also seeded so the replayed game is consistent. The binding is
            // staged and applied to the peer entities by
            // `peers::apply_faction_bindings` (entities may not exist yet -- the
            // live set is gated off while spectating).
            GameEvent::StartGame {
                assignments,
                scenario,
                optional_rule: _,
            } => {
                state.queued_factions.0 = Some(
                    assignments
                        .iter()
                        .map(|(pid, faction)| (*pid, *faction))
                        .collect(),
                );
                let map_kind = map_kind_for_scenario(*scenario);
                if let Some(gs) = ctx.game_state.as_deref_mut() {
                    // `GameState::new` sets the scenario's first-moving player
                    // (§9.113/§9.212/§9.322); do not override it.
                    *gs = GameState::new(*scenario);
                    // Attach the scenario's board to the engine state *now*, so
                    // the replayed MoveUnit/PlaceUnit events (queued into
                    // `incoming.replay` and applied later by
                    // `apply_pending_placement`) are costed by terrain and
                    // checked for ZOC/Nile against the same board the live game
                    // used. Deferring only the *visual* map load left those moves
                    // briefly validated against an empty board -- diverging from
                    // live, especially now that movement cost accumulates
                    // (mp_spent_this_turn).
                    if let Some(loaded) = ctx.loaded_annotations.as_deref() {
                        gs.board = BoardInfo::from_map_data(loaded.map(map_kind));
                    }
                }
                // The *visual* board (map plane, overlay, camera) still loads
                // after replay completes, on the next frame (§dual-map).
                state.pending_map_load.0 = Some(map_kind);
                continue;
            }
            // All other variants fall through to apply_game_event.
            GameEvent::Effect(_)
            | GameEvent::TurnComplete(_)
            | GameEvent::MapEdit { .. }
            | GameEvent::OverlayUpdate { .. }
            | GameEvent::ExcludeHex { .. }
            | GameEvent::HexsideEdit { .. }
            | GameEvent::RoadEdit { .. }
            | GameEvent::SetupLetterEdit { .. }
            | GameEvent::ScattergramEdit { .. }
            | GameEvent::NamedAreaEdit { .. }
            | GameEvent::UpdateUnitGrids { .. }
            | GameEvent::ShowTerrainOverlay(_) => {}
        }
        game_apply::apply_game_event(&event.payload, &mut ctx);
    }
}

/// Leave review mode back to the lobby. Shown while [`AppState::Spectating`]
/// (gated at the system registration site).
pub fn exit_review_ui(
    mut contexts: EguiContexts,
    mut timeline: ResMut<SpectatorTimeline>,
    mut next_state: ResMut<NextState<AppState>>,
    mut next_mode: ResMut<NextState<crate::AppMode>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::Area::new(egui::Id::new("exit_review"))
        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(12.0, 12.0))
        .show(ctx, |ui| {
            if ui.button("\u{2b05} Back to lobby").clicked() {
                timeline.record = None;
                next_mode.set(crate::AppMode::Lobby);
                next_state.set(AppState::Lobby);
            }
        });
}

/// The timeline scrubber panel: a slider over the event log, play/step controls,
/// and the current event's summary. Shown only while [`AppState::Spectating`]
/// (gated at the system registration site).
pub fn timeline_ui(
    mut contexts: EguiContexts,
    mut timeline: ResMut<SpectatorTimeline>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let len = timeline.len();
    if len == 0 {
        return;
    }
    let last = len - 1;

    let mut __ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("timeline_panel"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    let __panel = egui::Panel::bottom("timeline_panel")
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(30))
                .inner_margin(egui::Margin::symmetric(12, 8)),
        )
        .show(&mut __ui, |ui| {
            ui.horizontal(|ui| {
                if !timeline.source_label.is_empty() {
                    ui.label(
                        egui::RichText::new(&timeline.source_label)
                            .color(egui::Color32::from_gray(170)),
                    );
                    ui.separator();
                }

                let play_label = if timeline.playing {
                    "\u{23f8} Pause"
                } else {
                    "\u{25b6} Play"
                };
                if ui.button(play_label).clicked() {
                    timeline.playing = !timeline.playing;
                    timeline.play_accum = 0.0;
                }
                if ui.button("|< Start").clicked() {
                    timeline.playing = false;
                    timeline.seek(0);
                }
                if ui.button("< Prev").clicked() {
                    timeline.playing = false;
                    let prev = timeline.cursor.saturating_sub(1);
                    timeline.seek(prev);
                }
                if ui.button("Next >").clicked() {
                    timeline.playing = false;
                    let next = timeline.cursor + 1;
                    timeline.seek(next);
                }
                if ui.button("End >|").clicked() {
                    timeline.playing = false;
                    timeline.seek(last);
                }

                let mut cursor = timeline.cursor;
                let resp = ui.add(egui::Slider::new(&mut cursor, 0..=last).text(format!(
                    "event {} / {}",
                    timeline.cursor + 1,
                    len
                )));
                if resp.changed() {
                    timeline.playing = false;
                    timeline.seek(cursor);
                }
            });

            // Summary of the event now at the cursor.
            if let Some(record) = timeline.record.as_ref()
                && let Some(ev) = record.events.get(timeline.cursor)
            {
                let name: &'static str = (&ev.payload).into();
                ui.label(
                    egui::RichText::new(format!("#{}  {}", ev.seq, name))
                        .size(12.0)
                        .color(egui::Color32::from_gray(150)),
                );
            }
        });
    crate::ui_plugin::register_panel_rect(ctx, __panel.response.rect);
}
