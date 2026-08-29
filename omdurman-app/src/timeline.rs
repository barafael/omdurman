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
use omdurman_hexmap::{GameMap, hex_world_pos, load_map_data};
use omdurman_net::{GameEvent, GameRecord};
use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_types::HexCoord;

use crate::{GameRng, LoadedAnnotations, PendingIncoming, PendingMapLoad, game_apply};

/// Mutable state bundle for [`rebuild_state_to`].
///
/// Groups the 11 `&mut` parameters so the function signature stays short.
/// NOT a `SystemParam` — this is a plain struct for a non-system function.
pub(crate) struct RebuildState<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub game_map: &'a mut GameMap,
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

/// Resources needed for the teardown phase of a timeline scrub: clear movement
/// paths, reset the picker, and drop the peer entities. Placed units are kept
/// and reconciled against the rebuilt state (see [`scrub_teardown`]).
#[derive(SystemParam)]
pub struct ScrubTeardown<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub unit_paths: ResMut<'w, crate::picker::UnitPaths>,
    pub picker: ResMut<'w, crate::picker::UnitPicker>,
    pub picker_state: ResMut<'w, crate::picker::PickerState>,
    /// Despawned so the rebuild starts with an empty peer set; the reviewed
    /// record's `StartGame` binding is re-applied via `QueuedFactions`.
    pub peer_entities: Query<'w, 's, Entity, With<crate::peers::Peer>>,
}

/// Resources needed for the rebuild phase of a timeline scrub: reset the map,
/// overlay, and rules state, then replay events.
#[derive(SystemParam)]
pub struct ScrubRebuild<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub game_map: ResMut<'w, omdurman_hexmap::GameMap>,
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
/// state that [`crate::rebuild_state_to`] does not itself reset — movement
/// paths and the picker — then replays `0..=cursor`. Placed-unit entities are
/// intentionally kept and reconciled (see the comment in the body) so playback
/// steps don't blank the board. The queued placement events are spawned by
/// `apply_pending_placement` on the following frame, exactly as in live replay.
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
    // Placed-unit entities are deliberately NOT despawned here: a scrub
    // happens on every playback step, and despawn + respawn (deferred one
    // frame for effect-only records, whose sprites come from
    // `sync_spectator_units`) blanked the whole board for a frame -- the
    // "cards twitch" on each step. Instead the re-queued PlaceUnit events
    // are deduplicated in `apply_pending_placement` (a unit already on the
    // board is not re-spawned) and `sync_spectator_units` reconciles
    // positions/eliminations against the rebuilt engine state.
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

// -- Combat markers: brief fire arrows + melee triangles (§spectator) --------

/// Marker for a transient spectator combat visual: a red fire arrow (§6) or
/// a red melee triangle (§7), spawned when the timeline cursor lands on the
/// matching event and animated out by [`animate_spectator_combat_markers`].
#[derive(Component)]
pub(crate) struct SpectatorCombatMarker {
    /// Full-size scale; the animation lerps the live scale toward this.
    base_scale: Vec3,
    /// Seconds since spawn.
    age: f32,
    /// Lifetime: the marker grows in, holds, then shrinks away.
    ttl: f32,
}

/// How long a combat marker stays on screen (seconds): long enough to read
/// during playback, short enough to feel like a muzzle flash / clash.
const MARKER_TTL: f32 = 1.4;

/// Transient combat visuals for the event at the timeline cursor:
///
/// - `FireCombat`/`HowitzerFire`: one translucent red arrow per firing hex,
///   aimed at the attack's target hex (§6). Howitzer bombardments (§6.64)
///   draw their arrow at the *aim* hex; scatter is visible on the board.
/// - `DeclareMelee`: a red triangle between the warring counters, point
///   toward the defenders (§7).
///
/// Firer positions are read from the rebuilt engine state, which is exactly
/// "after this event", i.e. where the firers stood when they fired (fire
/// combat does not move units).
///
/// Markers are spawned once per (record, cursor) -- not rebuilt every
/// frame -- so the grow/hold/shrink animation plays out; they fade on their
/// own even while the cursor parks on the event.
pub(crate) fn spectator_combat_markers(
    mut commands: Commands,
    timeline: Res<SpectatorTimeline>,
    game_state: Option<Res<crate::GameStateResource>>,
    marker_assets: Res<SpectatorMarkerAssets>,
    render: crate::DirectionArrowCtx,
    existing: Query<Entity, With<SpectatorCombatMarker>>,
    // (record label, cursor) of the last spawn, so a re-scrub of the same
    // event (or playback stepping onto it) doesn't reset the animation.
    mut last_spawned: Local<Option<(String, usize)>>,
) {
    let existing: Vec<Entity> = existing.iter().collect();

    let Some(record) = timeline.record.as_ref() else {
        crate::ui::despawn_all(&mut commands, &existing);
        *last_spawned = None;
        return;
    };
    let key = (timeline.source_label.clone(), timeline.cursor);
    if last_spawned.as_ref() == Some(&key) {
        return; // same event: let the animation run
    }
    *last_spawned = Some(key);
    crate::ui::despawn_all(&mut commands, &existing);

    let Some(event) = record.events.get(timeline.cursor) else {
        return;
    };
    let GameEvent::Effect(effect) = &event.payload else {
        return;
    };
    let Some(gs) = game_state else { return };
    debug!(
        cursor = timeline.cursor,
        ?effect,
        "spectator: combat marker check"
    );

    let crate::DirectionArrowCtx {
        arrow_assets,
        hex:
            crate::HexRender {
                assets: hex_assets,
                layout,
                overlay,
            },
    } = render;
    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;

    let spawn_marker = |commands: &mut Commands,
                        mesh: Handle<Mesh>,
                        material: Handle<StandardMaterial>,
                        transform: Transform| {
        commands.spawn((
            SpectatorCombatMarker {
                base_scale: transform.scale,
                age: 0.0,
                ttl: MARKER_TTL,
            },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform {
                scale: Vec3::splat(0.001),
                ..transform
            },
            Visibility::Visible,
        ));
    };

    match effect {
        GameEffect::FireCombat { attack, .. } | GameEffect::HowitzerFire { attack, .. } => {
            // One arrow per distinct firing hex: a stacked combined attack
            // (§6.14) draws a single arrow instead of N overlapping ones.
            let mut firer_hexes: Vec<HexCoord> = Vec::new();
            for id in &attack.firers {
                if let Some(unit) = gs.0.find_unit(*id)
                    && !firer_hexes.contains(&unit.position)
                {
                    firer_hexes.push(unit.position);
                }
            }
            let target = hex_world_pos(attack.target_hex, origin, &overlay.params);
            for from_hex in &firer_hexes {
                let from = hex_world_pos(*from_hex, origin, &overlay.params);
                // Same construction as the live `fire_direction_arrow`:
                // inset from both ends, the arrow mesh points +Z, rotated
                // onto the heading.
                let delta = Vec3::new(target.x - from.x, 0.0, target.z - from.z);
                let len = delta.length();
                if len < f32::EPSILON {
                    continue;
                }
                let dir = delta / len;
                let inset = size * 0.18;
                let draw_len = (len - inset).max(len * 0.4);
                let tail = from + dir * ((len - draw_len) * 0.5);
                spawn_marker(
                    &mut commands,
                    arrow_assets.mesh.clone(),
                    hex_assets.fire_arrow.clone(),
                    Transform::from_xyz(tail.x, 1.55, tail.z)
                        .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                        .with_scale(Vec3::new(size * 0.5, 1.0, draw_len)),
                );
            }
        }
        GameEffect::DeclareMelee { attack, .. } => {
            // A red triangle clashing between the warring counters: placed
            // at the midpoint of the two hexes, point toward the defenders.
            let a = hex_world_pos(attack.attacker_hex, origin, &overlay.params);
            let d = hex_world_pos(attack.defender_hex, origin, &overlay.params);
            let delta = Vec3::new(d.x - a.x, 0.0, d.z - a.z);
            let len = delta.length();
            if len < f32::EPSILON {
                return;
            }
            let dir = delta / len;
            let mid = (a + d) / 2.0 - dir * (size * 0.1);
            spawn_marker(
                &mut commands,
                marker_assets.melee_triangle.clone(),
                hex_assets.melee_red.clone(),
                Transform::from_xyz(mid.x, 1.6, mid.z)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                    .with_scale(Vec3::splat(size * 0.45)),
            );
        }
        _ => {}
    }
}

/// Grow-in / hold / shrink-out animation for spectator combat markers.
/// Scale-only (shared materials), so no per-instance material churn.
///
/// While playback is *paused* on the marker's event the marker is held at
/// full scale (frozen mid-hold): pausing on a fire event to study it keeps
/// the arrow on screen. During playback the 1.4s lifetime plays out,
/// spanning ~2 playback steps -- a brief flash. (When the cursor moves on
/// while paused, the spawner despawns the marker on its key change.)
pub(crate) fn animate_spectator_combat_markers(
    time: Res<Time>,
    timeline: Res<SpectatorTimeline>,
    mut commands: Commands,
    mut markers: Query<(Entity, &mut Transform, &mut SpectatorCombatMarker)>,
) {
    for (entity, mut transform, mut marker) in markers.iter_mut() {
        marker.age += time.delta_secs();
        if !timeline.playing {
            // Hold mid-animation while parked on the event.
            marker.age = marker.age.min(marker.ttl * 0.5);
        }
        let p = (marker.age / marker.ttl).clamp(0.0, 1.0);
        // Fast pop-in (~90ms), hold, then shrink away over the last 35%.
        let grow = (p / 0.07).min(1.0);
        let shrink = if p > 0.65 {
            1.0 - (p - 0.65) / 0.35
        } else {
            1.0
        };
        let s = (grow * shrink).clamp(0.0, 1.0);
        transform.scale = marker.base_scale * s.max(0.001);
        if p >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Shared assets for the melee triangle marker: the mesh is built once at
/// startup; the bright red material is the shared `HexRingAssets::melee_red`.
#[derive(Resource, Default)]
pub(crate) struct SpectatorMarkerAssets {
    /// Unit equilateral triangle in the XZ plane, pointing +Z (like the
    /// arrow convention), centered on its centroid.
    pub melee_triangle: Handle<Mesh>,
}

/// Startup: build the spectator marker mesh.
pub(crate) fn spawn_spectator_marker_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Equilateral triangle with side 1, pointing +Z, lying in XZ.
    let h = 3f32.sqrt() / 2.0; // height of a unit-side triangle
    let positions = vec![
        Vec3::new(0.0, 0.0, h * 2.0 / 3.0), // tip (+Z)
        Vec3::new(0.5, 0.0, -h / 3.0),      // base right
        Vec3::new(-0.5, 0.0, -h / 3.0),     // base left
    ];
    let mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_indices(bevy::mesh::Indices::U32(vec![0, 2, 1]))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::Y; 3])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![Vec2::ZERO; 3]);
    commands.insert_resource(SpectatorMarkerAssets {
        melee_triangle: meshes.add(mesh),
    });
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
    state
        .commands
        .insert_resource(GameRng::from_seed(record.initial_state.seed));
    state.game_map.hexes.clear();

    // Seed LoadedAnnotations from the board RON data and load the default
    // board (Fall-of-Khartoum) into the live map so replay events (PlaceUnit,
    // etc.) have valid hexes to target. This replaces the old
    // LoadAnnotations network event that seeded the map at runtime.
    *state.loaded_annotations = crate::board_state::LoadedAnnotations::from_board_ron();
    load_map_data(
        state
            .loaded_annotations
            .map(omdurman_types::MapKind::FallOfKhartoum),
        &mut *state.game_map,
    );

    let mut ctx = game_apply::GameApplyCtx {
        game_state: Some(&mut *state.game_state),
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
                optional_rule,
            } => {
                // Shared live/replay core: stage the binding, seed the engine
                // state (+ the committed optional rule, so replay matches the
                // live path exactly), attach the board synchronously, defer the
                // visual map load (§dual-map).
                game_apply::apply_start_game(
                    assignments,
                    *scenario,
                    *optional_rule,
                    ctx.game_state.as_deref_mut(),
                    state.queued_factions,
                    state.loaded_annotations,
                    state.pending_map_load,
                );
                continue;
            }
            // All other variants fall through to apply_game_event.
            GameEvent::Effect(_) => {}
        }
        game_apply::apply_game_event(&event.payload, &mut ctx);
    }
}

/// The timeline scrubber panel: a slider over the event log, play/step controls,
/// and the current event's summary. Shown only while [`AppState::Spectating`]
/// (gated at the system registration site).
pub fn timeline_ui(
    mut contexts: EguiContexts,
    mut timeline: ResMut<SpectatorTimeline>,
    mut panels: ResMut<crate::ui_plugin::PanelRects>,
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
    panels.push(__panel.response.rect);
}
