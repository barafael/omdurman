//! Spectator timeline scrubber (§spectator).
//!
//! Reviews a recorded [`GameRecord`] — the in-memory game or one loaded from a
//! `games/*/events.jsonl` file — by rebuilding the rules state to an arbitrary
//! event index. The dice need no rewinding: every effect carries its
//! pre-rolled dice, so a replay never draws a random number. Scrubbing is
//! incremental: a step forward applies the next event to the shown state,
//! and any other seek restarts from the nearest [checkpoint](CHECKPOINT_EVERY)
//! at or before the cursor instead of from event 0.
//!
//! **The review owns only what the board shows.** The reviewed position is
//! held in the board's resources ([`crate::GameStateResource`], the press,
//! the movement routes; the board itself follows the reviewed scenario), so
//! every renderer shows it unchanged. Everything else a replay writes -- the
//! seat table, the setup-ready flag, the AI driver -- goes to the review's
//! private [`ReviewSide`], so the session's seats, peers and AI are never
//! touched.
//!
//! **A live session goes on underneath.** Opening a review from a running
//! session (the lobby's "Review current game", the result card's "Review
//! timeline") moves the live board state aside into [`LiveStash`] (see
//! [`stash_live_world`]). The net layer keeps running: the host keeps
//! sequencing, every peer keeps recording, but the receive path leaves the
//! board's resources to the review, and [`catch_up_live_stash`] applies each
//! newly recorded event to the stash instead -- through the same
//! [`game_apply::apply_game_event`] as the live echo. The host's AI and the
//! struck-mine roll read the live state through [`LiveGame`], so play goes
//! on while a player reviews. Leaving the review ([`leave_review`]) puts the
//! caught-up live state back. An *offline* review (opened from the title
//! screen, no session) has no stash, and the net systems stay gated off.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::{GameRecord, RecordedEvent};
use omdurman_rules::effects::GameState;

use crate::{GameRng, game_apply};

/// A scrub restarts from a checkpoint at most this many events before the
/// cursor (memory: one engine-state clone per this many events). Counted in
/// events because an event is one order of bounded replay cost (the
/// `GameEvent` contract); if orders ever grow into something a scrub feels,
/// space checkpoints by replay cost (turn boundaries are the natural
/// anchors) rather than by count.
pub(crate) const CHECKPOINT_EVERY: usize = 64;

/// The log-derived state a replay writes that the board does not show: kept
/// private to the review, so a scrub never rewrites the session's seat
/// table, setup-ready flag or AI driver.
#[derive(Default)]
pub(crate) struct ReviewSide {
    seats: crate::seats::Seats,
    local_setup_ready: crate::peers::LocalSetupReady,
    bot_driver: crate::bot_player::BotDriver,
}

/// The reviewed state after one event, kept to restart a scrub from.
struct Checkpoint {
    game_state: GameState,
    seats: crate::seats::Seats,
    press: crate::telegram::TelegramLog,
    unit_paths: crate::picker::UnitPaths,
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
    /// rebuild is deferred to [`scrub_rebuild`] so the heavy work runs in a
    /// normal system with full world access, not inside the egui pass.
    pub dirty: bool,
    /// Where a loaded record came from, for the panel header. Empty for the
    /// in-memory game.
    pub source_label: String,
    /// Bumped every time [`open`](Self::open) swaps in a record, so the
    /// combat-marker spawner's (label, cursor, generation) key cannot
    /// suppress the marker of a re-opened record parked on the same event.
    pub generation: u32,
    /// The event the board's state reflects (`None`: nothing of this record
    /// is shown yet), so a step forward applies one event.
    shown: Option<usize>,
    /// The review's own seat table & co. (see [`ReviewSide`]).
    side: ReviewSide,
    /// Reviewed states by event index (every [`CHECKPOINT_EVERY`]th event).
    checkpoints: std::collections::BTreeMap<usize, Checkpoint>,
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
        self.shown = None;
        self.checkpoints.clear();
    }

    /// Stop reviewing: drop the record and everything rebuilt from it.
    pub fn close(&mut self) {
        self.record = None;
        self.playing = false;
        self.dirty = false;
        self.shown = None;
        self.checkpoints.clear();
        self.side = ReviewSide::default();
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

/// What a scrub writes: the board's resources (the reviewed position is what
/// the board shows) and the picker selection it resets. Never the seat
/// table, the peers, the net state or the AI driver (see [`ReviewSide`]).
#[derive(SystemParam)]
pub struct ScrubRebuild<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub game_state: ResMut<'w, crate::GameStateResource>,
    /// The reviewed scenario's board, loaded on the next frame.
    pub pending_map_load: ResMut<'w, crate::PendingMapLoad>,
    /// Movement routes, rebuilt by the replay.
    pub unit_paths: ResMut<'w, crate::picker::UnitPaths>,
    /// The press (telegrams / Gazette), rebuilt by the replay.
    pub press: ResMut<'w, crate::telegram::TelegramLog>,
    pub picker: ResMut<'w, crate::picker::UnitPicker>,
    pub picker_state: ResMut<'w, crate::picker::PickerState>,
}

/// When the timeline cursor is dirty, rebuild the shown state to that event.
///
/// A step to the next event applies that one event: it plays out on the
/// board (counters glide, fade, spin). Any other seek restarts from the
/// nearest checkpoint at or before the cursor (or from event 0) and is a
/// jump: the counters snap ([`crate::fx::SnapSprites`]). Placed-unit entities
/// are kept: `picker::reconcile_unit_sprites` moves / spawns / despawns them
/// against the rebuilt engine state in the same frame, so playback steps
/// don't blank the board. Runs only on the review screen.
pub fn scrub_rebuild(mut timeline: ResMut<SpectatorTimeline>, mut rebuild: ScrubRebuild) {
    if !timeline.dirty {
        return;
    }
    timeline.dirty = false;
    let SpectatorTimeline {
        record,
        cursor,
        shown,
        side,
        checkpoints,
        ..
    } = &mut *timeline;
    // Borrow, don't clone: this runs on every playback step and the record
    // holds the entire game's events.
    let Some(record) = record.as_ref() else {
        return;
    };
    if record.events.is_empty() {
        return;
    }
    let target = (*cursor).min(record.events.len() - 1);
    let stepped = *shown == Some(target.wrapping_sub(1)) && target > 0;
    rebuild.picker.reset_available();
    *rebuild.picker_state = crate::picker::PickerState::Idle;

    let ScrubRebuild {
        game_state,
        pending_map_load,
        unit_paths,
        press,
        ..
    } = &mut rebuild;
    // Where to start: from whichever of the shown event (when moving forward)
    // and the nearest checkpoint at or before the target is closer, else
    // afresh.
    let forward = shown.filter(|&at| at <= target);
    let checkpoint = checkpoints
        .range(..=target)
        .next_back()
        .filter(|(at, _)| forward.is_none_or(|shown_at| **at > shown_at));
    let start = match (forward, checkpoint) {
        (_, Some((&at, point))) => {
            game_state.0 = point.game_state.clone();
            side.seats = point.seats.clone();
            **press = point.press.clone();
            unit_paths.0 = point.unit_paths.0.clone();
            at + 1
        }
        (Some(at), None) => at + 1,
        (None, None) => {
            game_state.0 = GameState::new(omdurman_types::Scenario::Campaign);
            side.seats.0.clear();
            **press = crate::telegram::TelegramLog::default();
            unit_paths.0.clear();
            0
        }
    };
    let mut sinks = game_apply::EventSinks {
        game_state: &mut game_state.0,
        seats: &mut side.seats,
        local_setup_ready: &mut side.local_setup_ready,
        bot_driver: &mut side.bot_driver,
        pending_map_load,
        unit_paths,
        press,
    };
    for (idx, event) in record
        .events
        .iter()
        .enumerate()
        .take(target + 1)
        .skip(start)
    {
        game_apply::apply_game_event(&event.payload, &mut sinks);
        if (idx + 1).is_multiple_of(CHECKPOINT_EVERY) && !checkpoints.contains_key(&idx) {
            checkpoints.insert(
                idx,
                Checkpoint {
                    game_state: sinks.game_state.clone(),
                    seats: sinks.seats.clone(),
                    press: sinks.press.clone(),
                    unit_paths: crate::picker::UnitPaths(sinks.unit_paths.0.clone()),
                },
            );
        }
    }
    // The past's observations would flood the live drain with stale cards.
    let _stale = sinks.game_state.drain_observations();
    *shown = Some(target);
    if !stepped {
        rebuild.commands.insert_resource(crate::fx::SnapSprites);
    }
}

// -- The live session underneath a review ------------------------------------

/// The live session's board state, moved aside while the board shows a
/// review (see the module docs). Present exactly while a review runs on top
/// of a session.
#[derive(Resource)]
pub(crate) struct LiveStash {
    game_state: GameState,
    press: crate::telegram::TelegramLog,
    unit_paths: crate::picker::UnitPaths,
    /// How many of the record's events `game_state` reflects...
    applied: usize,
    /// ...the last of them (seq, uid)...
    last: Option<(u32, Option<u64>)>,
    /// ...and of which record (`GameRecorder::generation`).
    record_generation: u32,
}

impl LiveStash {
    /// The live engine state.
    pub(crate) fn game_state(&self) -> &GameState {
        &self.game_state
    }
}

/// The live session's engine state wherever the board is: the stash while
/// the board shows a review, else the board's own. For the systems that act
/// in the session on any screen (the host's AI, the struck-mine roll, seat
/// presence).
#[derive(SystemParam)]
pub(crate) struct LiveGame<'w> {
    shown: Res<'w, crate::GameStateResource>,
    stash: Option<Res<'w, LiveStash>>,
}

impl LiveGame<'_> {
    pub(crate) fn state(&self) -> &GameState {
        match &self.stash {
            Some(stash) => stash.game_state(),
            None => &self.shown.0,
        }
    }
}

/// Run condition: the board's resources hold a live session's state, or a
/// review runs on top of one -- anything but an offline review (no session
/// at all), where the session systems have nothing to act on.
pub(crate) fn session_live(
    state: Res<State<crate::AppState>>,
    stash: Option<Res<LiveStash>>,
) -> bool {
    *state.get() != crate::AppState::Spectating || stash.is_some()
}

/// Whether the review on screen runs on top of a live session.
pub(crate) fn reviewing_live(stash: Option<Res<LiveStash>>) -> bool {
    stash.is_some()
}

/// The record's identity at `applied` events: its generation and last event.
fn record_mark(
    recorder: &crate::game_record::GameRecorder,
    applied: usize,
) -> (u32, Option<(u32, Option<u64>)>) {
    let last = applied
        .checked_sub(1)
        .and_then(|i| recorder.record.as_ref()?.events.get(i))
        .map(|e| (e.seq, e.uid));
    (recorder.generation(), last)
}

/// Entering a review from a running session (`InGame` or `Lobby` →
/// `Spectating`): set the live board state aside, so the session goes on
/// underneath (see the module docs). Copied rather than moved: the board
/// keeps showing it until the first scrub.
pub(crate) fn stash_live_world(
    mut commands: Commands,
    game_state: Res<crate::GameStateResource>,
    press: Res<crate::telegram::TelegramLog>,
    unit_paths: Res<crate::picker::UnitPaths>,
    recorder: Res<crate::game_record::GameRecorder>,
) {
    let applied = recorder.record.as_ref().map_or(0, |r| r.events.len());
    let (record_generation, last) = record_mark(&recorder, applied);
    info!(
        applied,
        "review opened over a live session; live state set aside"
    );
    commands.insert_resource(LiveStash {
        game_state: game_state.0.clone(),
        press: press.clone(),
        unit_paths: crate::picker::UnitPaths(unit_paths.0.clone()),
        applied,
        last,
        record_generation,
    });
}

/// Entering any review: show the board (the review screen is the board of
/// `Spectating`).
pub(crate) fn enter_review(mut next_mode: ResMut<NextState<crate::AppMode>>) {
    next_mode.set(crate::AppMode::Game);
}

/// The session state a live event writes besides the stash itself: the
/// seat table, the setup-ready flag and the AI driver stay in their
/// resources (a review never touches them).
#[derive(SystemParam)]
pub(crate) struct LiveSinks<'w> {
    recorder: Res<'w, crate::game_record::GameRecorder>,
    seats: ResMut<'w, crate::seats::Seats>,
    local_setup_ready: ResMut<'w, crate::peers::LocalSetupReady>,
    bot_driver: ResMut<'w, crate::bot_player::BotDriver>,
}

/// Bring the stash up to the record: apply every event recorded since, or --
/// when the record was replaced (a history install, a resync reset) or no
/// longer holds the applied prefix -- replay it from the start.
fn catch_up(stash: &mut LiveStash, live: &mut LiveSinks) {
    let events: &[RecordedEvent] = live
        .recorder
        .record
        .as_ref()
        .map_or(&[], |r| r.events.as_slice());
    let same_line = live.recorder.generation() == stash.record_generation
        && events.len() >= stash.applied
        && record_mark(&live.recorder, stash.applied).1 == stash.last;
    let from = if same_line { stash.applied } else { 0 };
    if same_line && from == events.len() {
        return;
    }
    // (Seats & co. are touched only now: a resource written every frame
    // would wake every `Peers::changed` reader every frame.)
    let mut scratch_map_load = crate::PendingMapLoad::default();
    let mut sinks = game_apply::EventSinks {
        game_state: &mut stash.game_state,
        seats: &mut live.seats,
        local_setup_ready: &mut live.local_setup_ready,
        bot_driver: &mut live.bot_driver,
        pending_map_load: &mut scratch_map_load,
        unit_paths: &mut stash.unit_paths,
        press: &mut stash.press,
    };
    if !same_line {
        info!(
            events = events.len(),
            "live record replaced during a review; rebuilding the live state"
        );
        *sinks.game_state = GameState::new(omdurman_types::Scenario::Campaign);
        sinks.seats.0.clear();
        sinks.unit_paths.0.clear();
    }
    for event in &events[from..] {
        game_apply::apply_game_event(&event.payload, &mut sinks);
    }
    // Live-only presentation (combat cards, slips) of events the player was
    // not watching: dropped, as a rebuild drops them.
    let _unseen = sinks.game_state.drain_observations();
    stash.applied = events.len();
    let (generation, last) = record_mark(&live.recorder, stash.applied);
    stash.record_generation = generation;
    stash.last = last;
}

/// Keep the stashed live state current while a review is on screen: apply
/// what the session recorded this frame (runs after the receive path).
pub(crate) fn catch_up_live_stash(mut stash: ResMut<LiveStash>, mut live: LiveSinks) {
    catch_up(&mut stash, &mut live);
}

/// The board resources a review hands back to the live session.
#[derive(SystemParam)]
pub(crate) struct ReviewExit<'w, 's> {
    commands: Commands<'w, 's>,
    timeline: ResMut<'w, SpectatorTimeline>,
    stash: Option<ResMut<'w, LiveStash>>,
    live: LiveSinks<'w>,
    game_state: ResMut<'w, crate::GameStateResource>,
    press: ResMut<'w, crate::telegram::TelegramLog>,
    unit_paths: ResMut<'w, crate::picker::UnitPaths>,
    pending_map_load: ResMut<'w, crate::PendingMapLoad>,
    picker: ResMut<'w, crate::picker::UnitPicker>,
    picker_state: ResMut<'w, crate::picker::PickerState>,
}

/// Leaving a review (any way out of `Spectating`): drop the reviewed record,
/// and when a session ran underneath, put its caught-up state back on the
/// board -- the counters snap to it, the board reloads for its scenario.
pub(crate) fn leave_review(exit: ReviewExit) {
    let ReviewExit {
        mut commands,
        mut timeline,
        stash,
        mut live,
        mut game_state,
        mut press,
        mut unit_paths,
        mut pending_map_load,
        mut picker,
        mut picker_state,
    } = exit;
    timeline.close();
    picker.reset_available();
    *picker_state = crate::picker::PickerState::Idle;
    let Some(mut stash) = stash else {
        return;
    };
    catch_up(&mut stash, &mut live);
    game_state.0 = std::mem::replace(
        &mut stash.game_state,
        GameState::new(omdurman_types::Scenario::Campaign),
    );
    *press = std::mem::take(&mut stash.press);
    unit_paths.0 = std::mem::take(&mut stash.unit_paths.0);
    pending_map_load.0 = Some(crate::map_kind_for_scenario(game_state.0.scenario));
    commands.remove_resource::<LiveStash>();
    commands.insert_resource(crate::fx::SnapSprites);
    info!(
        applied = stash.applied,
        "review closed; live state restored"
    );
}

/// Rebuild game + map state from the canonical event log, applying events
/// `0..=upto` (or all events when `upto` is `None`). The reset-from-seed + full
/// forward replay of the live late-joiner / history-install path.
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
    rng: &mut GameRng,
    sinks: &mut game_apply::EventSinks,
) {
    let upto = upto.unwrap_or(record.events.len().saturating_sub(1));
    info!(
        upto,
        total = record.events.len(),
        "rebuilding state from log"
    );

    // The event stream is canonical, so we rebuild from a known state. The
    // dice need no rewinding: every effect carries its pre-rolled dice, so
    // replay never draws from `GameRng`. Reseed it from fresh entropy
    // instead of the record seed: restarting the record-seeded stream at
    // position 0 would make this peer's next rolls repeat the rolls already
    // made at the start of the game. (The board needs no reset: the replayed
    // `StartGame` attaches the scenario's board and queues its load via
    // `PendingMapLoad`.)
    *rng = GameRng::from_seed(omdurman_net::new_seed());
    sinks.unit_paths.0.clear();
    // The seat table is part of the log's state too: only the replayed
    // `StartGame` (and seat events) may populate it.
    sinks.seats.0.clear();

    let end = (upto + 1).min(record.events.len());
    for event in &record.events[..end] {
        game_apply::apply_game_event(&event.payload, sinks);
    }
    let _stale = sinks.game_state.drain_observations();
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
