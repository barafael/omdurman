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
use omdurman_hexmap::{GameMap, load_map_data};
use omdurman_net::GameRecord;

use crate::{GameRng, game_apply};

/// Mutable state bundle for [`rebuild_state_to`]: the command queue (to
/// reseed the shared RNG), the live board, and the event-application sinks.
/// NOT a `SystemParam` — this is a plain struct for a non-system function.
pub(crate) struct RebuildState<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub game_map: &'a mut GameMap,
    pub sinks: game_apply::EventSinks<'a>,
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
    /// Bumped every time [`open`](Self::open) swaps in a record, so the
    /// combat-marker spawner's (label, cursor, generation) key cannot
    /// suppress the marker of a re-opened record parked on the same event.
    pub generation: u32,
}

impl SpectatorTimeline {
    /// Begin reviewing `record` from its final event, marking a rebuild.
    pub fn open(&mut self, record: GameRecord, source_label: String) {
        self.cursor = record.events.len().saturating_sub(1);
        self.record = Some(record);
        self.playing = false;
        self.play_accum = 0.0;
        self.source_label = source_label;
        self.generation += 1;
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
pub fn advance_timeline_playback(
    time: Res<Time>,
    mut timeline: ResMut<SpectatorTimeline>,
    mut activity: ResMut<crate::activity::Activity>,
) {
    if !timeline.playing || timeline.record.is_none() {
        return;
    }
    // Playback steps on the clock: keep the frames coming while it plays.
    activity.keep_running();
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

/// Resources needed for the teardown phase of a timeline scrub: reset the
/// picker and drop the peer entities. Placed units are kept and reconciled
/// against the rebuilt state (see [`scrub_teardown`]).
#[derive(SystemParam)]
pub struct ScrubTeardown<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub picker: ResMut<'w, crate::picker::UnitPicker>,
    pub picker_state: ResMut<'w, crate::picker::PickerState>,
    /// Despawned so the rebuild starts with an empty peer set; the reviewed
    /// record's `StartGame` seats are re-applied by the rebuild.
    pub peer_entities: Query<'w, 's, Entity, With<crate::peers::Peer>>,
}

/// Resources needed for the rebuild phase of a timeline scrub: reset the map,
/// overlay, and rules state, then replay events.
#[derive(SystemParam)]
pub struct ScrubRebuild<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub game_map: ResMut<'w, omdurman_hexmap::GameMap>,
    pub game_state: ResMut<'w, crate::GameStateResource>,
    pub seats: ResMut<'w, crate::seats::Seats>,
    pub local_setup_ready: ResMut<'w, crate::peers::LocalSetupReady>,
    /// The always-present AI driver (reseeded per StartGame in
    /// [`game_apply::apply_start_game`]).
    pub bot_driver: ResMut<'w, crate::bot_player::BotDriver>,
    pub loaded_annotations: ResMut<'w, crate::LoadedAnnotations>,
    pub pending_map_load: ResMut<'w, crate::PendingMapLoad>,
    /// Movement routes, rebuilt by the replay.
    pub unit_paths: ResMut<'w, crate::picker::UnitPaths>,
    /// The press (telegrams / Gazette), rebuilt by the replay.
    pub press: ResMut<'w, crate::telegram::TelegramLog>,
    /// The review shows the play board (the board itself is (re)loaded from
    /// `pending_map_load`, and follows the reviewed scenario via the play-view
    /// board reconciler, §dual-map).
    pub next_app_mode: ResMut<'w, NextState<crate::AppMode>>,
}

/// When the timeline cursor is dirty, rebuild the whole world to that event.
///
/// A re-scrub runs over an *already populated* world, so it first resets the
/// picker selection and the peer entities, then replays `0..=cursor`
/// synchronously ([`rebuild_state_to`]). Placed-unit entities are
/// intentionally kept: `picker::reconcile_unit_sprites` moves / spawns /
/// despawns them against the rebuilt engine state in the same frame, so
/// playback steps don't blank the board.
///
/// Split into two chained systems [`scrub_teardown`] → [`scrub_rebuild`] so
/// each SystemParam bundle stays focused on a single phase.
pub fn scrub_teardown(mut timeline: ResMut<SpectatorTimeline>, mut teardown: ScrubTeardown) {
    if !timeline.dirty {
        return;
    }
    if timeline.record.is_none() {
        timeline.dirty = false;
        return;
    }
    for entity in &teardown.peer_entities {
        teardown.commands.entity(entity).despawn();
    }
    teardown.picker.reset_available();
    *teardown.picker_state = crate::picker::PickerState::Idle;
}

/// Rebuild phase of the timeline scrub: replays events `0..=cursor` and
/// switches to the game view. Runs after [`scrub_teardown`].
///
/// A step to the next event plays out on the board (counters glide, fade,
/// spin); any other seek is a jump, and the counters snap
/// ([`crate::fx::SnapSprites`]).
pub fn scrub_rebuild(
    mut timeline: ResMut<SpectatorTimeline>,
    mut rebuild: ScrubRebuild,
    mut last_shown: Local<Option<(u32, usize)>>,
) {
    if !timeline.dirty {
        return;
    }
    // Borrow, don't clone: this runs on every playback step and the record
    // holds the entire game's events. (`dirty` is reset after the rebuild,
    // once the borrowed record's last use is behind us.)
    let Some(record) = timeline.record.as_ref() else {
        timeline.dirty = false;
        return;
    };

    {
        let ScrubRebuild {
            commands,
            game_map,
            game_state,
            seats,
            local_setup_ready,
            bot_driver,
            loaded_annotations,
            pending_map_load,
            unit_paths,
            press,
            ..
        } = &mut rebuild;
        let mut state = RebuildState {
            commands,
            game_map,
            sinks: game_apply::EventSinks {
                game_state: &mut game_state.0,
                seats,
                local_setup_ready,
                bot_driver,
                loaded_annotations,
                pending_map_load,
                unit_paths,
                press,
            },
        };
        rebuild_state_to(record, Some(timeline.cursor), &mut state);
    }
    timeline.dirty = false;
    let stepped = last_shown.is_some_and(|(generation, cursor)| {
        generation == timeline.generation && timeline.cursor == cursor + 1
    });
    if !stepped {
        rebuild.commands.insert_resource(crate::fx::SnapSprites);
    }
    *last_shown = Some((timeline.generation, timeline.cursor));

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
/// Every event goes through [`game_apply::apply_game_event`] -- the same
/// function the live echo uses -- synchronously and in record order, so the
/// rebuilt engine state equals the live one. The sprites follow the engine
/// state via `picker::reconcile_unit_sprites`.
///
/// Observations produced while replaying history are discarded: they describe
/// the past, and leaving them in the engine would flood the next live effect's
/// drain with stale combat cards and dispatch slips.
pub(crate) fn rebuild_state_to(
    record: &GameRecord,
    upto: Option<usize>,
    state: &mut RebuildState<'_, '_, '_>,
) {
    let upto = upto.unwrap_or(record.events.len().saturating_sub(1));
    info!(
        upto,
        total = record.events.len(),
        "rebuilding state from log"
    );

    // Clear the map -- the event stream is canonical so we rebuild from a
    // known state. The dice need no rewinding: every effect carries its
    // pre-rolled dice, so replay never draws from `GameRng`. Reseed it from
    // fresh entropy instead of the record seed: restarting the record-seeded
    // stream at position 0 would make this peer's next rolls repeat the
    // rolls already made at the start of the game.
    state
        .commands
        .insert_resource(GameRng::from_seed(omdurman_net::new_seed()));
    state.game_map.hexes.clear();
    state.sinks.unit_paths.0.clear();
    // The seat table is part of the log's state too: only the replayed
    // `StartGame` (and seat events) may populate it.
    state.sinks.seats.0.clear();

    // Seed LoadedAnnotations from the board RON data and load the default
    // board (Fall-of-Khartoum) into the live map; the replayed `StartGame`
    // then queues the scenario's own board via `PendingMapLoad`.
    *state.sinks.loaded_annotations = crate::board_state::LoadedAnnotations::from_board_ron();
    load_map_data(
        state
            .sinks
            .loaded_annotations
            .map(omdurman_types::MapKind::FallOfKhartoum),
        &mut *state.game_map,
    );

    let end = (upto + 1).min(record.events.len());
    for event in &record.events[..end] {
        game_apply::apply_game_event(&event.payload, &mut state.sinks);
    }
    let _stale = state.sinks.game_state.drain_observations();
}

/// The timeline scrubber panel: a slider over the event log, play/step controls,
/// and the current event's summary. Shown only while [`AppState::Spectating`]
/// (gated at the system registration site).
pub fn timeline_ui(mut contexts: EguiContexts, mut timeline: ResMut<SpectatorTimeline>) {
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
    // Click-sensed full-rect blocker, registered *before* the content so the
    // panel's own widgets stay above it in egui's hit-test (see
    // `register_panel_blocker`). Makes `egui_wants_pointer_input` true over
    // blank panel areas too, which is what gates map input.
    omdurman_board_ui::panels::register_panel_blocker(
        &mut __ui,
        "timeline_panel",
        egui::Rect::from_min_max(
            egui::pos2(ctx.viewport_rect().min.x, ctx.viewport_rect().max.y - 48.0),
            ctx.viewport_rect().max,
        ),
    );
    let __panel = egui::Panel::bottom("timeline_panel")
        .frame(
            egui::Frame::default()
                .fill(crate::ui::palette::NEUTRAL_BG)
                .inner_margin(egui::Margin::symmetric(12, 8)),
        )
        .show(&mut __ui, |ui| {
            ui.horizontal(|ui| {
                if !timeline.source_label.is_empty() {
                    ui.label(
                        egui::RichText::new(&timeline.source_label)
                            .color(crate::ui::palette::TEXT_MUTED),
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
                        .color(crate::ui::palette::TEXT_DIM),
                );
            }
        });
}
