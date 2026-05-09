//! Paper Strategy — minimal Bevy + bevy_matchbox 2-player scaffold.
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

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    // Read room ID once at startup so every system that needs it can use the
    // RoomId resource instead of re-reading the URL each frame.
    let room = room_id();

    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .add_message::<ActionTaken>()
        .insert_resource(RoomId(room))
        .insert_resource(NetState::default())
        .insert_resource(TurnState::default())
        .insert_resource(CameraSettings::default())
        .add_systems(Startup, (setup_ui, open_socket, spawn_camera))
        .add_systems(
            Update,
            (
                camera_control,
                draw_grid.after(camera_control),
                // Ordering matters: networking first so ActionTaken events are
                // available to game logic in the same frame they arrive.
                handle_socket,
                handle_local_input.after(handle_socket),
                update_status_text.after(handle_socket),
            ),
        )
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
/// shuffle, etc.) *both* sides call `rng.next_u32()` in the same order — the
/// result is identical on both machines without ever transmitting it.
#[derive(Resource)]
struct GameRng(ChaCha8Rng);

#[derive(Resource, Default)]
struct TurnState {
    /// True when it is the local player's turn to act.
    my_turn: bool,
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
        Self {
            focus: Vec3::ZERO,
            distance: 600.0,
            yaw: 0.0,
            pitch: PI / 2.0 - 0.05,
            smooth_focus: Vec3::ZERO,
            smooth_distance: 600.0,
            smooth_yaw: 0.0,
            smooth_pitch: PI / 2.0 - 0.05,
        }
    }
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
            max_distance: 2000.0,
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
    bincode::serialize(msg).unwrap().into_boxed_slice()
}

fn decode(raw: &[u8]) -> Option<NetMsg> {
    bincode::deserialize(raw).ok()
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
                socket.channel_mut(0).send(enc_msg(&NetMsg::Seed(seed)), peer);
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
///   2. Call `rng.0.next_u32()` for any die rolls (opponent does the same
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
) {
    // Guard: only act when it's our turn and the RNG is seeded.
    if !turn.my_turn {
        return;
    }
    let Some(peer) = net.peer else { return };
    let Some(mut rng) = rng_opt else { return }; // not ready until seed arrives

    if keys.just_pressed(KeyCode::Space) {
        // Example: roll a d6 using the shared RNG.
        // The opponent calls next_u32() when they process our Action message,
        // so both sides stay in sync without transmitting the roll result.
        let roll = rng.0.next_u32() % 6 + 1;
        info!(roll, "local roll");

        if let Ok(mut socket) = socket_q.single_mut() {
            socket.channel_mut(0).send(enc_msg(&NetMsg::Action(roll)), peer);
        }

        ev_action.write(ActionTaken {
            by_me: true,
            data: roll,
        });
        turn.my_turn = false;
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
            if turn.my_turn {
                "Your turn - press SPACE".into()
            } else {
                "Opponent's turn…".into()
            }
        }
    });
}

// ── Grid ──────────────────────────────────────────────────────────────────────

fn draw_grid(mut gizmos: Gizmos) {
    let dim = 200.0;
    let step = 10.0;
    let color = Color::srgba(0.25, 0.25, 0.25, 0.4);
    let mut x = -dim;
    while x <= dim {
        gizmos.line(
            Vec3::new(x, 0.0, -dim),
            Vec3::new(x, 0.0, dim),
            color,
        );
        x += step;
    }
    let mut z = -dim;
    while z <= dim {
        gizmos.line(
            Vec3::new(-dim, 0.0, z),
            Vec3::new(dim, 0.0, z),
            color,
        );
        z += step;
    }
}

// ── Camera systems ────────────────────────────────────────────────────────────

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        RtsCamera,
        RtsCameraState::default(),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection::default()),
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
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) { pan.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) { pan.y -= 1.0; }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowRight) { pan.x += 1.0; }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowLeft) { pan.x -= 1.0; }
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
        state.distance = (state.distance * factor).clamp(settings.min_distance, settings.max_distance);
    }

    if keys.pressed(KeyCode::KeyQ) { state.yaw += settings.rotate_speed_keys * dt; }
    if keys.pressed(KeyCode::KeyE) { state.yaw -= settings.rotate_speed_keys * dt; }

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
