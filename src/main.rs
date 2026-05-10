//! Remember Gordon! Battle of Omdurman.
//!
//! What's wired up:
//!   • WebRTC P2P connection via a matchbox signaling server
//!   • Room code from the URL hash (#abc123) — share the URL to invite
//!   • Deterministic role assignment (no extra round-trip)
//!   • Seeded shared RNG — die rolls stay in sync without sending results
//!   • Alternating turn state
//!   • A status text UI
//!
//! What you add:
//!   • Replace `u32` in `ActionTaken` with your real game action type
//!   • Replace `handle_local_input` with your actual board/input logic
//!   • Extend `update_status_text` and `setup_ui` with your game's visuals

mod map;

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_matchbox::prelude::*;
use map::layout::HexLayout;
use map::{GameMap, HexCoord};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    // Read room ID once at startup so every system that needs it can use the
    // RoomId resource instead of re-reading the URL each frame.
    let room = room_id();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(EguiPlugin::default())
        .init_state::<AppState>()
        .add_message::<ActionTaken>()
        .insert_resource(RoomId(room))
        .insert_resource(NetState::default())
        .insert_resource(TurnState::default())
        .insert_resource(CameraSettings::default())
        .insert_resource(GameMap::default())
        .insert_resource(map::render::HexOverlay::default())
        .insert_resource(map::editor::MapHexes::default())
        .insert_resource(map::editor::HexEditor::default())
        .insert_resource(HexLayout::calibrated(
            // Austrian Mission  pixel → axial (0, 0)
            Vec2::new(736.0, 420.0), HexCoord::new(0, 0),
            // Barracks          pixel → axial (5, −1)
            Vec2::new(1178.0, 572.0), HexCoord::new(5, -1),
        ))
        .add_systems(
            Startup,
            (
                setup_ui,
                open_socket,
                spawn_camera,
                spawn_ground,
                spawn_lights,
                map::render::spawn_map_plane,
                map::render::spawn_hover_coord_text,
                map::editor::compute_map_hexes,
                map::editor::load_saved_map,
            ),
        )
        .add_systems(
            Update,
            (
                camera_control,
                map::render::draw_hex_debug,
                map::render::hex_overlay_controls,
                map::editor::editor_controls,
                map::editor::handle_hex_editor_click,
                map::editor::draw_editor_highlight,
                despawn_dice,
                // Ordering matters: networking first so ActionTaken events are
                // available to game logic in the same frame they arrive.
                handle_socket,
                handle_local_input.after(handle_socket),
                update_status_text.after(handle_socket),
            ),
        )
        .add_systems(EguiPrimaryContextPass, (
            map::render::overlay_ui,
            map::editor::editor_ui,
            map::editor::editor_labels_ui,
        ))
        .run();
}

// ── App state ─────────────────────────────────────────────────────────────────

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
enum AppState {
    #[default]
    Connecting, // waiting for peer to join the room
    InGame, // both players connected, game running
}

// ── Resources ─────────────────────────────────────────────────────────────────

/// The WebRTC room name, read once from the URL hash on startup.
#[derive(Resource)]
struct RoomId(String);

#[derive(Resource, Default)]
struct NetState {
    /// The remote peer we are matched with.
    peer: Option<PeerId>,
    /// True if our PeerId sorts lower — used to break symmetry cheaply.
    is_host: bool,
}

/// Deterministic RNG shared between both players.
///
/// The host generates a seed and sends it once.  Both sides initialise this
/// RNG from that seed.  Whenever the game needs randomness (a die roll, a
/// shuffle, etc.) *both* sides call the RNG in the same order — the
/// result is identical on both machines without ever transmitting it.
#[derive(Resource)]
struct GameRng(ChaCha8Rng);

#[derive(Resource, Default)]
struct TurnState {
    my_turn: bool,
    pending_roll: Option<u32>,
}

#[derive(Component)]
struct RtsCamera;

#[derive(Component)]
struct RtsCameraState {
    focus: Vec3,
    distance: f32,
    yaw: f32,
    pitch: f32,
    smooth_focus: Vec3,
    smooth_distance: f32,
    smooth_yaw: f32,
    smooth_pitch: f32,
}

impl Default for RtsCameraState {
    fn default() -> Self {
        // At pitch ≈ π/2 the camera is directly above the focus point.
        // The map is 1200 world units tall; with the default 45° FOV we need
        // roughly 1450 units of altitude to see it all — use 1500 for margin.
        Self {
            focus: Vec3::ZERO,
            distance: 1500.0,
            yaw: 0.0,
            pitch: PI / 2.0 - 0.05,
            smooth_focus: Vec3::ZERO,
            smooth_distance: 1500.0,
            smooth_yaw: 0.0,
            smooth_pitch: PI / 2.0 - 0.05,
        }
    }
}

#[derive(Component)]
struct Dice {
    timer: Timer,
}

#[derive(Resource)]
struct CameraSettings {
    pan_speed: f32,
    rotate_speed_keys: f32,
    rotate_speed_mouse: f32,
    min_distance: f32,
    max_distance: f32,
    min_pitch: f32,
    max_pitch: f32,
    smoothing: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            pan_speed: 600.0,
            rotate_speed_keys: 0.8,
            rotate_speed_mouse: 0.003,
            min_distance: 100.0,
            max_distance: 3000.0,
            min_pitch: 0.25,
            max_pitch: PI / 2.0 - 0.05,
            smoothing: 6.0,
        }
    }
}

// ── Events ────────────────────────────────────────────────────────────────────

/// Fired whenever any player (local or remote) completes an action.
///
/// Replace `u32` with your game's action enum once you know what moves look
/// like.  Everything else stays the same.
#[derive(Message, Debug)]
pub struct ActionTaken {
    pub by_me: bool,
    /// Placeholder — swap with your real action type.
    pub data: u32,
}

// ── Wire protocol ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
enum NetMsg {
    Seed(u64),
    Action(u32),
}

fn enc_msg(msg: &NetMsg) -> Box<[u8]> {
    postcard::to_allocvec(msg).unwrap().into_boxed_slice()
}

fn decode(raw: &[u8]) -> Option<NetMsg> {
    postcard::from_bytes(raw).ok()
}

// ── Startup systems ───────────────────────────────────────────────────────────

#[derive(Component)]
struct StatusText;

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("Connecting…"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(14.0),
            left: Val::Px(18.0),
            ..default()
        },
        StatusText,
    ));
}

const SIGNALING_SERVER: &str = if let Some(s) = option_env!("MATCHBOX_SERVER") {
    s
} else {
    "wss://omdurman-matchbox.fly.dev"
};

fn open_socket(mut commands: Commands, room: Res<RoomId>) {
    // `?next=2` tells the signaling server to hold until 2 peers are present,
    // then exchange their ICE candidates.  After that, all traffic is direct
    // peer-to-peer — the signaling server is no longer involved.
    //
    // Override the server at build time: MATCHBOX_SERVER=ws://localhost:3536 trunk serve
    let url = format!("{}/{}?next=2", SIGNALING_SERVER, room.0);
    info!(room = %room.0, %url, "opening matchbox socket");
    commands.spawn(MatchboxSocket::new_reliable(url));
}

// ── Core networking system ────────────────────────────────────────────────────

/// Handles peer lifecycle and all incoming messages.
/// Runs every frame — `update_peers()` must be called regularly to drive the
/// WebRTC state machine even when no messages are expected.
fn handle_socket(
    mut socket_q: Query<&mut MatchboxSocket>,
    mut net: ResMut<NetState>,
    mut turn: ResMut<TurnState>,
    mut commands: Commands,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ev_action: MessageWriter<ActionTaken>,
) {
    let Ok(mut socket) = socket_q.single_mut() else {
        return;
    };

    // ── Peer lifecycle ──────────────────────────────────────────────────────

    let Ok(peer_updates) = socket.try_update_peers() else {
        return; // signaling channel closed after handshake — normal
    };
    for (peer, peer_state) in peer_updates {
        if peer_state == PeerState::Connected && net.peer.is_none() {
            // Both peers now know each other's PeerId from the signaling
            // exchange.  Comparing them gives a deterministic host/guest
            // assignment without any extra message round-trip.
            let my_id = socket.id().expect("socket id present once a peer connects");
            let is_host = my_id.0 < peer.0;

            net.peer = Some(peer);
            net.is_host = is_host;
            turn.my_turn = is_host; // host (= player 1) moves first

            if is_host {
                let seed = new_seed();
                info!(seed, "host: sending seed");
                let _ = socket
                    .channel_mut(0)
                    .try_send(enc_msg(&NetMsg::Seed(seed)), peer);
                // Host inserts GameRng immediately; guest waits for the
                // Seed message below before inserting theirs.
                commands.insert_resource(GameRng(ChaCha8Rng::seed_from_u64(seed)));
            } else {
                info!("guest: waiting for seed from host");
            }

            next_state.set(AppState::InGame);
        }
    }

    // ── Incoming messages ───────────────────────────────────────────────────

    // State transition takes effect at the *end* of this schedule run, so
    // we gate on the *current* state rather than the next one.
    if *state.get() != AppState::InGame {
        return;
    }

    for (_peer, raw) in socket.channel_mut(0).receive() {
        match decode(&raw) {
            Some(NetMsg::Seed(seed)) => {
                // Only the guest receives this.
                info!(seed, "guest: received seed, game ready");
                commands.insert_resource(GameRng(ChaCha8Rng::seed_from_u64(seed)));
            }
            Some(NetMsg::Action(data)) => {
                info!(data, "opponent action received");
                ev_action.write(ActionTaken { by_me: false, data });
                turn.my_turn = true; // opponent acted → now it's our turn
            }
            None => warn!("unknown message tag, ignoring"),
        }
    }
}

// ── Local input placeholder ───────────────────────────────────────────────────

/// Press SPACE to take your turn.
///
/// Replace the body of this system with your game's actual input and action
/// logic.  The pattern to follow:
///   1. Collect player input / decision.
///   2. Call the shared RNG for any die rolls (opponent does the same
///      automatically when they process your action on their side).
///   3. Send the action via the socket.
///   4. Fire `ActionTaken { by_me: true, data }`.
///   5. Set `turn.my_turn = false`.
fn handle_local_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut turn: ResMut<TurnState>,
    net: Res<NetState>,
    rng_opt: Option<ResMut<GameRng>>,
    mut socket_q: Query<&mut MatchboxSocket>,
    mut ev_action: MessageWriter<ActionTaken>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !turn.my_turn {
        return;
    }
    let Some(peer) = net.peer else { return };
    let Some(mut rng) = rng_opt else { return };
    let mut local_rng = rand::rng();

    // SPACE: roll the dice (visual only, stores the result)
    if keys.just_pressed(KeyCode::Space) && turn.pending_roll.is_none() {
        let roll = rng.0.random::<u32>() % 10 + 1;
        info!(roll, "rolled");

        turn.pending_roll = Some(roll);

        let radius = 20.0;
        let height = 40.0;
        let throw_dir = Vec3::new(
            local_rng.random_range(-1.0..1.0),
            0.0,
            local_rng.random_range(-1.0..1.0),
        )
        .normalize_or_zero();
        let initial_spin = throw_dir.cross(Vec3::Y) * 3.0
            + Vec3::new(
                local_rng.random_range(-0.75..0.75),
                local_rng.random_range(-0.75..0.75),
                local_rng.random_range(-0.75..0.75),
            );

        let collider_points = d10_collider_points(radius, height);
        commands.spawn((
            RigidBody::Dynamic,
            Collider::convex_hull(collider_points).unwrap(),
            Mass(1.0),
            GravityScale(30.0),
            Mesh3d(meshes.add(d10_mesh_colored(radius, height))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)).with_rotation(
                Quat::from_euler(
                    EulerRot::XYZ,
                    local_rng.random_range(0.0..core::f32::consts::TAU),
                    local_rng.random_range(0.0..core::f32::consts::TAU),
                    local_rng.random_range(0.0..core::f32::consts::TAU),
                ),
            ),
            LinearVelocity(throw_dir * 150.0 + Vec3::Y * 100.0),
            AngularVelocity(initial_spin),
            Restitution::new(0.3),
            Friction::new(0.8),
            Dice {
                timer: Timer::from_seconds(6.0, TimerMode::Once),
            },
        ));
    }

    // Enter: confirm the roll and end the turn
    if keys.just_pressed(KeyCode::Enter) {
        if let Some(roll) = turn.pending_roll.take() {
            info!(roll, "sending action");

            if let Ok(mut socket) = socket_q.single_mut() {
                let _ = socket
                    .channel_mut(0)
                    .try_send(enc_msg(&NetMsg::Action(roll)), peer);
            }

            ev_action.write(ActionTaken {
                by_me: true,
                data: roll,
            });
            turn.my_turn = false;
        }
    }
}

// ── Status UI ─────────────────────────────────────────────────────────────────

fn update_status_text(
    state: Res<State<AppState>>,
    turn: Res<TurnState>,
    room: Res<RoomId>,
    mut query: Query<&mut Text, With<StatusText>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    *text = Text::new(match state.get() {
        AppState::Connecting => {
            format!("Waiting for opponent - share: #{}", room.0)
        }
        AppState::InGame => {
            if turn.my_turn && turn.pending_roll.is_none() {
                "Your turn - SPACE to roll".into()
            } else if turn.my_turn && turn.pending_roll.is_some() {
                "ENTER to confirm".into()
            } else {
                "Opponent's turn...".into()
            }
        }
    });
}

// ── Camera systems ────────────────────────────────────────────────────────────

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        RtsCamera,
        RtsCameraState::default(),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        Tonemapping::None,
    ));
}

fn camera_control(
    time: Res<Time>,
    settings: Res<CameraSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut scroll_events: MessageReader<MouseWheel>,
    mut cam_q: Query<(&mut RtsCameraState, &mut Transform), With<RtsCamera>>,
) {
    let Ok((mut state, mut transform)) = cam_q.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    let mut pan = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowUp) {
        pan.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        pan.y -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        pan.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowLeft) {
        pan.x += 1.0;
    }
    if pan != Vec2::ZERO {
        pan = pan.normalize() * settings.pan_speed * dt * (state.distance / 500.0).max(0.3);
        let fwd = Vec3::new(-state.yaw.sin(), 0.0, -state.yaw.cos());
        let right = Vec3::new(fwd.z, 0.0, -fwd.x);
        state.focus += fwd * pan.y + right * pan.x;
    }

    let mut zoom_ticks: f32 = 0.0;
    for ev in scroll_events.read() {
        zoom_ticks += ev.y;
    }
    if zoom_ticks != 0.0 {
        let factor = 1.0 - zoom_ticks.clamp(-5.0, 5.0) * 0.06;
        state.distance =
            (state.distance * factor).clamp(settings.min_distance, settings.max_distance);
    }

    if mouse_buttons.pressed(MouseButton::Middle) {
        for ev in mouse_motion.read() {
            state.yaw -= ev.delta.x * settings.rotate_speed_mouse;
            state.pitch = (state.pitch + ev.delta.y * settings.rotate_speed_mouse)
                .clamp(settings.min_pitch, settings.max_pitch);
        }
    } else {
        mouse_motion.clear();
    }

    let t = (settings.smoothing * dt).min(1.0);
    state.smooth_focus = state.smooth_focus.lerp(state.focus, t);
    state.smooth_distance = state.smooth_distance.lerp(state.distance, t);
    state.smooth_yaw = state.smooth_yaw.lerp(state.yaw, t);
    state.smooth_pitch = state.smooth_pitch.lerp(state.pitch, t);

    let hdist = state.smooth_distance * state.smooth_pitch.cos();
    let vert = state.smooth_distance * state.smooth_pitch.sin();
    let offset = Vec3::new(
        hdist * state.smooth_yaw.sin(),
        vert,
        hdist * state.smooth_yaw.cos(),
    );
    let eye = state.smooth_focus + offset;
    *transform = Transform::from_translation(eye).looking_at(state.smooth_focus, Vec3::Y);
}

// ── Physics ───────────────────────────────────────────────────────────────────

fn spawn_ground(mut commands: Commands) {
    commands.spawn((RigidBody::Static, Collider::half_space(Vec3::Y)));
}

fn d10_collider_points(radius: f32, height: f32) -> Vec<Vec3> {
    let n = 5;
    let mut points = vec![
        Vec3::new(0.0, height / 2.0, 0.0),
        Vec3::new(0.0, -height / 2.0, 0.0),
    ];
    for k in 0..n {
        let a = core::f32::consts::TAU * k as f32 / n as f32;
        points.push(Vec3::new(radius * a.cos(), 0.0, radius * a.sin()));
    }
    points
}

fn d10_mesh_colored(radius: f32, height: f32) -> Mesh {
    let n = 5;
    let top = [0.0, height / 2.0, 0.0];
    let bot = [0.0, -height / 2.0, 0.0];

    let mut ring = Vec::new();
    for k in 0..n {
        let a = core::f32::consts::TAU * k as f32 / n as f32;
        ring.push([radius * a.cos(), 0.0, radius * a.sin()]);
    }

    let pastel_colors: [[f32; 4]; 10] = [
        [1.0, 0.4, 0.4, 1.0],
        [1.0, 0.6, 0.2, 1.0],
        [0.9, 0.9, 0.2, 1.0],
        [0.4, 0.9, 0.4, 1.0],
        [0.2, 0.8, 0.6, 1.0],
        [0.3, 0.7, 0.9, 1.0],
        [0.4, 0.4, 0.9, 1.0],
        [0.6, 0.3, 0.9, 1.0],
        [0.9, 0.4, 0.7, 1.0],
        [0.7, 0.5, 0.3, 1.0],
    ];

    let mut positions = Vec::new();
    let mut colors = Vec::new();

    for k in 0..n {
        let a = ring[k];
        let b = ring[(k + 1) % n];

        positions.extend_from_slice(&[top, b, a]);
        let c = pastel_colors[k];
        colors.extend_from_slice(&[c, c, c]);

        positions.extend_from_slice(&[bot, a, b]);
        let c = pastel_colors[k + 5];
        colors.extend_from_slice(&[c, c, c]);
    }

    let indices: Vec<u32> = (0..positions.len() as u32).collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.compute_normals();
    mesh
}

fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(50.0, 100.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            ..default()
        },
        Transform::from_xyz(-50.0, 50.0, -50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn despawn_dice(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Dice)>) {
    for (entity, mut dice) in query.iter_mut() {
        dice.timer.tick(time.delta());
        if dice.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// On WASM: read `#roomcode` from the URL.  If absent, generate a random 8-hex
/// code and write it back so the player can copy and share it.
///
/// On native (dev/testing): use the first CLI argument, defaulting to
/// `"dev-room"`.  Open two terminals with the same argument to test locally.
fn room_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let hash = web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .unwrap_or_default();
        let id = hash.trim_start_matches('#').to_string();
        if id.is_empty() {
            // Generate a short random code and put it in the URL hash so
            // player 1 just copies the whole URL to invite player 2.
            let new_id = format!("{:08x}", new_seed() as u32);
            if let Some(win) = web_sys::window() {
                let _ = win.location().set_hash(&new_id);
            }
            new_id
        } else {
            id
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "dev-room".to_string())
    }
}

/// Cryptographically random u64.
/// On WASM, `rand` delegates to `getrandom`, which calls `crypto.getRandomValues`.
/// The `getrandom = { features = ["js"] }` dep in Cargo.toml enables this.
fn new_seed() -> u64 {
    rand::random()
}
