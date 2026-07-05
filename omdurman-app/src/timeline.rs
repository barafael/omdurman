//! Spectator timeline scrubber (§spectator).
//!
//! Reviews a recorded [`GameRecord`] — the in-memory game or one loaded from a
//! `games/*.jsonl` file — by rebuilding rules/map state to an arbitrary event
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
use omdurman_net::GameRecord;

use crate::{AppState, PendingIncoming};

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
    /// rebuild is deferred to [`apply_timeline_scrub`] so the heavy work runs in
    /// a normal system with full world access, not inside the egui pass.
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

/// World state the scrub rebuild resets before replaying (§spectator). Bundled
/// so [`apply_timeline_scrub`] stays under the system-parameter limit.
#[derive(SystemParam)]
pub struct ScrubResetParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub game_map: ResMut<'w, omdurman_hexmap::GameMap>,
    pub overlay: ResMut<'w, crate::render::HexOverlay>,
    pub editor: ResMut<'w, crate::editor::HexEditor>,
    pub annotations: Option<ResMut<'w, crate::browser::SpriteAnnotationsResource>>,
    pub viewer: ResMut<'w, crate::units::UnitViewer>,
    pub game_state: ResMut<'w, crate::GameStateResource>,
    pub player_factions: ResMut<'w, crate::PlayerFactions>,
    pub loaded_annotations: ResMut<'w, crate::LoadedAnnotations>,
    pub pending_map_load: ResMut<'w, crate::PendingMapLoad>,
    pub unit_paths: ResMut<'w, crate::picker::UnitPaths>,
    pub picker: ResMut<'w, crate::picker::UnitPicker>,
    pub picker_state: ResMut<'w, crate::picker::PickerState>,
    pub placed_units: Query<'w, 's, Entity, With<crate::picker::PlacedUnit>>,
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
pub fn apply_timeline_scrub(
    mut timeline: ResMut<SpectatorTimeline>,
    mut incoming: ResMut<PendingIncoming>,
    mut reset: ScrubResetParams,
) {
    if !timeline.dirty {
        return;
    }
    let Some(record) = timeline.record.clone() else {
        timeline.dirty = false;
        return;
    };
    timeline.dirty = false;

    // Tear down populated ephemeral state so the rebuild starts clean.
    for entity in &reset.placed_units {
        reset.commands.entity(entity).despawn();
    }
    reset.unit_paths.0.clear();
    reset.picker.reset_available();
    *reset.picker_state = crate::picker::PickerState::Idle;
    reset.player_factions.by_peer.clear();
    // The replay queue and any pending live placements are stale across a scrub.
    incoming.replay.clear();
    incoming.live.clear();

    let history_peer = bevy_matchbox::prelude::PeerId(uuid::Uuid::nil());
    crate::rebuild_state_to(
        &record,
        Some(timeline.cursor),
        &mut reset.commands,
        &mut reset.game_map,
        &mut reset.overlay,
        &mut reset.editor,
        reset.annotations.as_deref_mut(),
        &mut reset.viewer,
        &mut incoming.replay,
        history_peer,
        &mut reset.game_state.0,
        &mut reset.player_factions,
        &mut reset.loaded_annotations,
        &mut reset.pending_map_load,
    );

    // Show the reviewed game on the play board (rebuild_state_to queued the
    // board data via PendingMapLoad; the reconciler keeps it on the reviewed
    // scenario's map while in a play view).
    reset.next_app_mode.set(crate::AppMode::Game);
}

/// Leave review mode back to the lobby. Shown while [`AppState::Spectating`].
pub fn exit_review_ui(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    mut timeline: ResMut<SpectatorTimeline>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if *state.get() != AppState::Spectating {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::Area::new(egui::Id::new("exit_review"))
        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(12.0, 12.0))
        .show(ctx, |ui| {
            if ui.button("\u{2b05} Back to lobby").clicked() {
                timeline.record = None;
                next_state.set(AppState::Lobby);
            }
        });
}

/// The timeline scrubber panel: a slider over the event log, play/step controls,
/// and the current event's summary. Shown only while [`AppState::Spectating`].
pub fn timeline_ui(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    mut timeline: ResMut<SpectatorTimeline>,
) {
    if *state.get() != AppState::Spectating {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let len = timeline.len();
    if len == 0 {
        return;
    }
    let last = len - 1;

    egui::TopBottomPanel::bottom("timeline_panel")
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(30))
                .inner_margin(egui::Margin::symmetric(12, 8)),
        )
        .show(ctx, |ui| {
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
}
