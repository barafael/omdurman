//! Remember Gordon! Battle of Omdurman.

mod annotate;
mod editor;
mod render;

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseWheel;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_matchbox::prelude::*;
use omdurman_hex::HexLayout;
use omdurman_map::{GameMap, load_saved_map};
use omdurman_net::{
    GameRng, NetMsg, NetState, RoomId, decode, enc_msg, new_seed, open_socket, room_id,
};
use std::f32::consts::PI;

fn main() {
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
        .insert_resource(render::HexOverlay::default())
        .insert_resource(editor::HexEditor::default())
        .insert_resource(annotate::AnnotationSession::default())
        .insert_resource(HexLayout::calibrated(
            Vec2::new(736.0, 420.0),
            omdurman_types::HexCoord::new(0, 0),
            Vec2::new(1178.0, 572.0),
            omdurman_types::HexCoord::new(5, -1),
        ))
        .add_systems(
            Startup,
            (
                setup_ui,
                open_socket,
                spawn_camera,
                spawn_ground,
                spawn_lights,
                render::spawn_map_plane,
                render::spawn_selection_marker,
                load_saved_map,
            ),
        )
        .add_systems(
            Update,
            (
                camera_control,
                render::draw_hex_debug,
                render::update_selection_marker,
                render::hex_overlay_controls,
                editor::editor_controls,
                editor::handle_hex_editor_click,
                editor::draw_editor_highlight,
                despawn_dice,
                handle_socket,
                handle_local_input.after(handle_socket),
                update_status_text.after(handle_socket),
                annotate::toggle_annotation_mode,
                annotate::handle_annotation_click,
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                render::overlay_ui,
                editor::editor_ui,
                editor::editor_labels_ui,
            ),
        )
        .run();
}

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
enum AppState {
    #[default]
    Connecting,
    InGame,
}

#[derive(Resource, Default)]
struct TurnState {
    my_turn: bool,
    pending_roll: Option<u32>,
}

#[derive(Component)]
pub struct RtsCamera;

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
            distance: 1500.0,
            yaw: 0.0,
            pitch: PI / 2.0 - 0.02,
            smooth_focus: Vec3::ZERO,
            smooth_distance: 1500.0,
            smooth_yaw: 0.0,
            smooth_pitch: PI / 2.0 - 0.02,
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
            min_distance: 100.0,
            max_distance: 3000.0,
            min_pitch: PI / 6.0,
            max_pitch: PI / 2.0 - 0.02,
            smoothing: 6.0,
        }
    }
}

#[derive(Message, Debug)]
pub struct ActionTaken {
    pub by_me: bool,
    pub data: u32,
}

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

    let Ok(peer_updates) = socket.try_update_peers() else {
        return;
    };
    for (peer, peer_state) in peer_updates {
        if peer_state == PeerState::Connected && net.peer.is_none() {
            let my_id = socket.id().expect("socket id present once a peer connects");
            let is_host = my_id.0 < peer.0;

            net.peer = Some(peer);
            net.is_host = is_host;
            turn.my_turn = is_host;

            if is_host {
                let seed = new_seed();
                info!(seed, "host: sending seed");
                let _ = socket
                    .channel_mut(0)
                    .try_send(enc_msg(&NetMsg::Seed(seed)), peer);
                commands.insert_resource(GameRng::from_seed(seed));
            } else {
                info!("guest: waiting for seed from host");
            }

            next_state.set(AppState::InGame);
        }
    }

    if *state.get() != AppState::InGame {
        return;
    }

    for (_peer, raw) in socket.channel_mut(0).receive() {
        match decode(&raw) {
            Some(NetMsg::Seed(seed)) => {
                info!(seed, "guest: received seed, game ready");
                commands.insert_resource(GameRng::from_seed(seed));
            }
            Some(NetMsg::Action(data)) => {
                info!(data, "opponent action received");
                ev_action.write(ActionTaken { by_me: false, data });
                turn.my_turn = true;
            }
            None => warn!("unknown message, ignoring"),
        }
    }
}

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

    if keys.just_pressed(KeyCode::Space) && turn.pending_roll.is_none() {
        let roll = rng.random_u32() % 10 + 1;
        info!(roll, "rolled");

        turn.pending_roll = Some(roll);

        let radius = 20.0;
        let height = 40.0;
        let throw_dir = Vec3::new(
            rand::RngExt::random_range(&mut local_rng, -1.0..1.0),
            0.0,
            rand::RngExt::random_range(&mut local_rng, -1.0..1.0),
        )
        .normalize_or_zero();
        let initial_spin = throw_dir.cross(Vec3::Y) * 3.0
            + Vec3::new(
                rand::RngExt::random_range(&mut local_rng, -0.75..0.75),
                rand::RngExt::random_range(&mut local_rng, -0.75..0.75),
                rand::RngExt::random_range(&mut local_rng, -0.75..0.75),
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
                    rand::RngExt::random_range(&mut local_rng, 0.0..core::f32::consts::TAU),
                    rand::RngExt::random_range(&mut local_rng, 0.0..core::f32::consts::TAU),
                    rand::RngExt::random_range(&mut local_rng, 0.0..core::f32::consts::TAU),
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
        AppState::Connecting => format!("Waiting for opponent - share: #{}", room.0),
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

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        RtsCamera,
        RtsCameraState::default(),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        Tonemapping::None,
    ));
}

/// Arrow keys pan, scroll wheel zooms, PageUp/Down tilt pitch.
fn camera_control(
    time: Res<Time>,
    settings: Res<CameraSettings>,
    keys: Res<ButtonInput<KeyCode>>,
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

    let pitch_step = dt * 0.8;
    if keys.pressed(KeyCode::PageUp) {
        state.pitch = (state.pitch + pitch_step).min(settings.max_pitch);
    }
    if keys.pressed(KeyCode::PageDown) {
        state.pitch = (state.pitch - pitch_step).max(settings.min_pitch);
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

fn spawn_ground(mut commands: Commands) {
    commands.spawn((RigidBody::Static, Collider::half_space(Vec3::Y)));
}

fn despawn_dice(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Dice)>) {
    for (entity, mut dice) in query.iter_mut() {
        dice.timer.tick(time.delta());
        if dice.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
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
