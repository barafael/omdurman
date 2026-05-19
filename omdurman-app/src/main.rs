//! Remember Gordon! Battle of Omdurman.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

/// Early-return guard: require a specific `EditorMode`.
/// Usage: `guard_mode!(contexts, mode, Overlay);`
///        `guard_mode!(contexts, mode, Editor, clip);`  — also clears `clip.right_sidebar`
macro_rules! guard_mode {
    ($contexts:expr, $mode:expr, $variant:ident) => {{
        let Ok(ctx) = $contexts.ctx_mut() else { return };
        if *$mode != EditorMode::$variant { return; }
        ctx
    }};
    ($contexts:expr, $mode:expr, $variant:ident, $clip:expr) => {{
        let Ok(ctx) = $contexts.ctx_mut() else { return };
        if *$mode != EditorMode::$variant {
            $clip.right_sidebar = None;
            return;
        }
        ctx
    }};
}

mod browser;
mod dice;
mod editor;
mod picker;
mod render;
mod secret;
mod units;
mod util;

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input::touch::Touches;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::gizmos::config::{DefaultGizmoConfigGroup, GizmoConfigStore};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_matchbox::prelude::*;
use omdurman_hex::HexLayout;
use omdurman_map::{GameMap, clip_hexes_to_overlay, load_annotations_from_str};
use omdurman_net::{
    GameRng, NetMsg, NetState, RoomId, decode, enc_msg, new_seed, open_socket, room_id,
};
use omdurman_types::HexCoord;
use strum::FromRepr;
use std::borrow::Cow;
use std::f32::consts::PI;

#[derive(Resource, Default)]
struct ShortcutsOverlay {
    visible: bool,
}

#[derive(Resource, Default)]
struct PendingEdits {
    items: Vec<NetMsg>,
    targeted: Vec<(NetMsg, PeerId)>,
    retry: Vec<(NetMsg, PeerId)>,
}

#[derive(Resource, Default)]
struct PendingIncoming(Vec<NetMsg>);

#[derive(Resource, Default)]
pub struct SidebarClip {
    pub right_sidebar: Option<egui::Rect>,
}

fn main() {
    let room = room_id();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(EguiPlugin::default())
        .init_state::<AppState>()
        .add_message::<ActionTaken>()
        .insert_resource(RoomId(room))
        .insert_resource(NetState::default())
        .insert_resource(TurnState::default())
        .insert_resource(CameraSettings::default())
        .insert_resource(CameraDragState::default())
        .insert_resource(GameMap::default())
        .insert_resource(render::HexOverlay::default())
        .insert_resource(editor::HexEditor::default())
        .insert_resource(EditorMode::Normal)
        .insert_resource(units::UnitViewer::load_or_default())
        .insert_resource(browser::SpriteBrowser::new())
        .insert_resource(browser::SpriteMetaClipboard::default())
        .insert_resource(dice::DiceSimulator::default())
        .insert_resource(secret::SecretState::default())
        .insert_resource(ShortcutsOverlay::default())
        .insert_resource(PendingEdits::default())
        .insert_resource(PendingIncoming::default())
        .insert_resource(SidebarClip::default())
        .insert_resource(picker::UnitPicker::default())
        .insert_resource(picker::PickerState::default())
        .insert_resource(HexLayout::calibrated(
            omdurman_types::Orientation::Pointy,
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
                units::spawn_units_plane,
                browser::spawn_sprite_browser,
                picker::spawn_picker_assets,
                load_annotations,
                init_gizmo_config,
                configure_egui_touch,
            ),
        )
        .add_systems(
            Update,
            (
                camera_control,
                render::draw_hex_debug,
                render::update_selection_marker,
                editor::editor_terrain_keys,
                editor::handle_hex_editor_click,
                editor::draw_editor_highlight,
                despawn_dice,
                handle_socket,
                apply_pending_placement.after(handle_socket),
                handle_local_input.after(handle_socket),
                update_status_text.after(handle_socket),
                units::draw_unit_grids,
                picker::placement_preview_gizmo,
                picker::handle_picker_clicks,
                picker::movement_overlay_gizmo,
                picker::animate_unit_movement,
                picker::cancel_placement,
                mode_shortcuts,
                flush_pending,
                sync_mode_visibilities,
            ),
        )
        .add_systems(
            Update,
            (
                browser::scroll_sprite_browser,
                browser::handle_sprite_clicks,
                browser::update_sprite_selection_marker,
            ),
        )
        .add_systems(Update, (browser::navigate_sprite_selection,))
        .add_systems(
            EguiPrimaryContextPass,
            (
                mode_toolbar,
                render::overlay_ui,
                editor::editor_ui,
                units::unit_grids_ui,
                units::unit_grid_labels,
                browser::sprite_meta_editor_ui,
                dice::dice_sim_ui,
                picker::unit_picker_ui,
                secret::secret_ui,
                shortcuts_ui,
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

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr)]
#[repr(u8)]
pub enum EditorMode {
    #[default]
    Normal,
    Overlay,
    Editor,
    Units,
    Sprites,
    Dice,
    Secret,
}

impl EditorMode {
    fn from_u8(v: u8) -> Self {
        Self::from_repr(v).unwrap_or(Self::Normal)
    }
}

#[derive(Resource)]
struct TurnState {
    my_index: usize,
    current_turn: usize,
    pending_roll: Option<u32>,
    game_started: bool,
}

impl Default for TurnState {
    fn default() -> Self {
        Self {
            my_index: 0,
            current_turn: 0,
            pending_roll: None,
            game_started: false,
        }
    }
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

#[derive(Resource, Default)]
struct CameraDragState {
    active: bool,
    last_cursor: Vec2,
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
struct StatusPane;

#[derive(Component)]
struct StatusText;

fn init_gizmo_config(mut store: ResMut<GizmoConfigStore>) {
    let (config, _) = store.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -0.01;
    config.line.width = 2.0;
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut, unused_variables))]
fn configure_egui_touch(mut contexts: EguiContexts) {
    #[cfg(target_arch = "wasm32")]
    {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        ctx.style_mut(|style| {
            style.spacing.interact_size = egui::vec2(40.0, 40.0);
            style.spacing.slider_width = 120.0;
        });
    }
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(14.0),
                left: Val::Px(14.0),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            StatusPane,
        ))
        .with_child((
            StatusText,
            Text::new("Connecting…"),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
        ));
}

fn handle_socket(
    mut socket_q: Query<&mut MatchboxSocket>,
    mut net: ResMut<NetState>,
    mut pending: ResMut<PendingEdits>,
    mut turn: ResMut<TurnState>,
    mut commands: Commands,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut current: ResMut<EditorMode>,
    mut editor: ResMut<editor::HexEditor>,
    mut browser: ResMut<browser::SpriteBrowser>,
    mut game_map: ResMut<GameMap>,
    mut overlay: ResMut<render::HexOverlay>,
    mut annotations: Option<ResMut<browser::SpriteAnnotationsResource>>,
    mut viewer: ResMut<units::UnitViewer>,
    mut incoming: ResMut<PendingIncoming>,
    placed_units: Query<&picker::PlacedUnit>,
) {
    let Ok(mut socket) = socket_q.single_mut() else {
        return;
    };

    let Ok(peer_updates) = socket.try_update_peers() else {
        return;
    };
    let mut new_peers: Vec<PeerId> = Vec::new();
    for (peer, peer_state) in peer_updates {
        if peer_state == PeerState::Connected && !net.peers.contains(&peer) {
            net.peers.push(peer);
            new_peers.push(peer);
        }
    }

    if let Some(my_id) = socket.id() {
        if !net.peers.is_empty() {
            let mut all_peers: Vec<PeerId> = net.peers.clone();
            all_peers.push(my_id);
            all_peers.sort();
            turn.my_index = all_peers.iter().position(|&id| id == my_id).unwrap();
            net.is_host = all_peers[0] == my_id;
        }

        if !turn.game_started && !net.peers.is_empty() {
            turn.game_started = true;
            turn.current_turn = 0;

            if net.is_host {
                let seed = new_seed();
                net.current_seed = Some(seed);
                info!(seed, "host: sending seed");
                pending.items.push(NetMsg::Seed(seed));
                commands.insert_resource(GameRng::from_seed(seed));
            } else {
                info!("guest: requesting snapshot from host");
                net.needs_snapshot = true;
                pending.items.push(NetMsg::RequestSnapshot);
            }

            next_state.set(AppState::InGame);
        } else if turn.game_started && net.is_host {
            for &p in &new_peers {
                if let Some(seed) = net.current_seed {
                    info!("host: syncing late joiner with full snapshot");
                    let snapshot = NetMsg::FullStateSnapshot(omdurman_net::GameStateSnapshot {
                        hexes: game_map.hexes.clone(),
                        overlay: game_map.overlay.clone(),
                        editor_mode: *current as u8,
                        annotations: annotations
                            .as_ref()
                            .map(|a| a.0.clone())
                            .unwrap_or_default(),
                        unit_grids: viewer.grids.clone(),
                        show_terrain_overlay: editor.show_terrain_overlay,
                        placed_units: placed_units
                            .iter()
                            .map(|u| omdurman_net::PlacedUnitSnapshot {
                                section_name: u.section_name.clone(),
                                col: u.col,
                                row: u.row,
                                coord_q: u.coord.q,
                                coord_r: u.coord.r,
                                movement: u.movement,
                                is_boat: u.is_boat,
                            })
                            .collect(),
                        seed,
                        current_turn: turn.current_turn,
                    });
                    net.snapshot_pending.push(p);
                    pending.targeted.push((snapshot, p));
                }
            }
        }
    }

    if *state.get() != AppState::InGame {
        return;
    }

    let mut targeted: Vec<(NetMsg, PeerId)> = Vec::new();
    for (_peer, raw) in socket.channel_mut(0).receive() {
        match decode(&raw) {
            Some(NetMsg::Seed(seed)) => {
                info!(seed, "guest: received seed, game ready");
                commands.insert_resource(GameRng::from_seed(seed));
            }
            Some(NetMsg::Action(data)) => {
                info!(data, "action received");
                let total = 1 + net.peers.len();
                turn.current_turn = (turn.current_turn + 1) % total;
            }
            Some(NetMsg::MapEdit {
                q,
                r,
                terrain,
                name,
            }) => {
                info!(q, r, "remote map edit");
                let coord = omdurman_types::HexCoord::new(q, r);
                let terrain_val = omdurman_types::Terrain::from_u8(terrain);
                game_map.hexes.insert(
                    coord,
                    omdurman_types::HexData {
                        terrain: terrain_val,
                        location: None,
                        name: if name.is_empty() { None } else { Some(name) },
                    },
                );
            }
            Some(NetMsg::ModeSwitch(mode)) => {
                info!(mode, "remote mode switch");
                apply_mode(EditorMode::from_u8(mode), &mut *current, &mut *editor, &mut *browser, &game_map);
            }
            Some(NetMsg::OverlayUpdate(params)) => {
                info!("remote overlay update");
                overlay.params = params.clone();
                game_map.overlay = params;
                clip_hexes_to_overlay(&mut game_map);
            }
            Some(NetMsg::AnnotateSprite {
                section_name,
                col,
                row,
                annotation,
            }) => {
                info!("remote sprite annotation");
                if let Some(ref mut ann) = annotations {
                    ann.0.units
                        .entry(section_name)
                        .or_default()
                        .insert((col, row), annotation);
                }
            }
            Some(msg @ (NetMsg::PlaceUnit { .. } | NetMsg::MoveUnit { .. })) => {
                incoming.0.push(msg);
            }
            Some(NetMsg::ShowTerrainOverlay(v)) => {
                editor.show_terrain_overlay = v;
            }
            Some(NetMsg::UpdateUnitGrids(grids)) => {
                info!("remote unit grids update");
                viewer.grids = grids;
                units::save_unit_grids(&viewer.grids);
            }
            Some(NetMsg::SyncState { seed, current_turn }) => {
                info!(seed, "late joiner: received sync state");
                commands.insert_resource(GameRng::from_seed(seed));
                turn.current_turn = current_turn;
            }
            Some(NetMsg::RequestSnapshot) => {
                info!("host: late joiner requested snapshot");
                if net.is_host && turn.game_started {
                    if let Some(seed) = net.current_seed {
                        let snapshot = NetMsg::FullStateSnapshot(omdurman_net::GameStateSnapshot {
                            hexes: game_map.hexes.clone(),
                            overlay: game_map.overlay.clone(),
                            editor_mode: *current as u8,
                            annotations: annotations
                                .as_ref()
                                .map(|a| a.0.clone())
                                .unwrap_or_default(),
                            unit_grids: viewer.grids.clone(),
                            show_terrain_overlay: editor.show_terrain_overlay,
                            placed_units: placed_units
                                .iter()
                                .map(|u| omdurman_net::PlacedUnitSnapshot {
                                    section_name: u.section_name.clone(),
                                    col: u.col,
                                    row: u.row,
                                    coord_q: u.coord.q,
                                    coord_r: u.coord.r,
                                    movement: u.movement,
                                    is_boat: u.is_boat,
                                })
                                .collect(),
                            seed,
                            current_turn: turn.current_turn,
                        });
                        targeted.push((snapshot, _peer));
                    }
                }
            }
            Some(NetMsg::SnapshotReceived) => {
                info!("host: late joiner acknowledged snapshot");
                net.snapshot_pending.retain(|&p| p != _peer);
            }
            Some(NetMsg::FullStateSnapshot(snap)) => {
                if !net.needs_snapshot {
                    info!(seed = snap.seed, "ignoring duplicate snapshot");
                    continue;
                }
                net.needs_snapshot = false;
                info!(seed = snap.seed, "late joiner: received full state snapshot");
                targeted.push((NetMsg::SnapshotReceived, _peer));
                game_map.hexes = snap.hexes;
                game_map.overlay = snap.overlay.clone();
                overlay.params = snap.overlay;
                clip_hexes_to_overlay(&mut game_map);
                *current = EditorMode::from_u8(snap.editor_mode);
                if let Some(ref mut ann) = annotations {
                    ann.0 = snap.annotations;
                } else {
                    commands.insert_resource(browser::SpriteAnnotationsResource(snap.annotations));
                }
                viewer.grids = snap.unit_grids;
                editor.show_terrain_overlay = snap.show_terrain_overlay;
                for u in &snap.placed_units {
                    incoming.0.push(NetMsg::PlaceUnit {
                        section_name: u.section_name.clone(),
                        col: u.col,
                        row: u.row,
                        coord_q: u.coord_q,
                        coord_r: u.coord_r,
                        is_boat: u.is_boat,
                    });
                }
                commands.insert_resource(GameRng::from_seed(snap.seed));
                turn.current_turn = snap.current_turn;
            }
            None => warn!("unknown message, ignoring"),
        }
    }
    // queue targeted sends (flushed by flush_pending later)
    for (msg, peer) in targeted {
        pending.targeted.push((msg, peer));
    }
}

fn apply_pending_placement(
    mut incoming: ResMut<PendingIncoming>,
    mut picker: ResMut<picker::UnitPicker>,
    layout: Res<HexLayout>,
    overlay: Res<render::HexOverlay>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut placed_units: Query<(Entity, &mut picker::PlacedUnit)>,
) {
    for msg in incoming.0.drain(..) {
        match msg {
            NetMsg::PlaceUnit {
                section_name,
                col,
                row,
                coord_q,
                coord_r,
                is_boat,
            } => {
                let coord = omdurman_types::HexCoord::new(coord_q, coord_r);
                if placed_units.iter().any(|(_, u)| u.section_name == section_name && u.col == col && u.row == row && u.coord == coord) {
                    continue;
                }
                let unit_idx = picker.available.iter().position(|u| u.section_name == section_name && u.col == col && u.row == row);
                if let Some(idx) = unit_idx {
                    let unit = picker.available.remove(idx);
                    let origin = crate::util::adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
                    let pos = crate::util::hex_world_pos(coord, origin, &overlay.params);
                    let sprite_size = overlay.params.hex_size * 1.05;
                    let material = materials.add(StandardMaterial {
                        base_color_texture: Some(unit.handle.clone()),
                        unlit: true,
                        alpha_mode: AlphaMode::Mask(0.1),
                        ..default()
                    });
                    commands.spawn((
                        picker::PlacedUnit {
                            coord,
                            section_name,
                            col,
                            row,
                            movement: picker::DEFAULT_MOVEMENT,
                            is_boat,
                        },
                        Mesh3d(meshes.add(Rectangle::new(sprite_size, sprite_size))),
                        MeshMaterial3d(material),
                        Transform::from_xyz(pos.x, 1.0, pos.z)
                            .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
                        Visibility::Visible,
                    ));
                }
            }
            NetMsg::MoveUnit {
                section_name,
                col,
                row,
                to_q,
                to_r,
            } => {
                let target = omdurman_types::HexCoord::new(to_q, to_r);
                for (entity, mut placed) in placed_units.iter_mut() {
                    if placed.section_name == section_name && placed.col == col && placed.row == row {
                        let origin = crate::util::adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
                        let pos = crate::util::hex_world_pos(target, origin, &overlay.params);
                        placed.coord = target;
                        commands.entity(entity)
                            .insert(Transform::from_xyz(pos.x, 1.0, pos.z)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)));
                        commands.entity(entity).remove::<picker::MovementAnimation>();
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}

fn handle_local_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut turn: ResMut<TurnState>,
    net: Res<NetState>,
    rng_opt: Option<ResMut<GameRng>>,
    mut pending: ResMut<PendingEdits>,
    mut ev_action: MessageWriter<ActionTaken>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_keyboard_input() {
        return;
    }
    if turn.current_turn != turn.my_index {
        return;
    }
    if net.peers.is_empty() {
        return;
    }
    let Some(mut rng) = rng_opt else { return };
    let mut local_rng = rand::rng();

    if keys.just_pressed(KeyCode::Space) && turn.pending_roll.is_none() {
        let roll = rng.random_u32() % 10 + 1;
        info!(roll, "rolled");

        turn.pending_roll = Some(roll);

        let radius = 60.0;
        let height = 120.0;
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

    if keys.just_pressed(KeyCode::Enter)
        && let Some(roll) = turn.pending_roll.take()
    {
        info!(roll, "sending action");

        pending.items.push(NetMsg::Action(roll));

        ev_action.write(ActionTaken {
            by_me: true,
            data: roll,
        });
        let total = 1 + net.peers.len();
        turn.current_turn = (turn.current_turn + 1) % total;
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
    let new = match state.get() {
        AppState::Connecting => Cow::Owned(format!("Waiting for players - share: ?room={}", room.0)),
        AppState::InGame if turn.current_turn == turn.my_index && turn.pending_roll.is_none() => {
            Cow::Borrowed("Your turn - SPACE to roll")
        }
        AppState::InGame if turn.current_turn == turn.my_index && turn.pending_roll.is_some() => {
            Cow::Borrowed("ENTER to confirm")
        }
        AppState::InGame => Cow::Owned(format!("Player {}'s turn...", turn.current_turn)),
    };
    if text.as_str() != new.as_ref() {
        *text = Text::new(new.into_owned());
    }
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

fn apply_mode(
    mode: EditorMode,
    current: &mut EditorMode,
    editor: &mut editor::HexEditor,
    browser: &mut browser::SpriteBrowser,
    game_map: &omdurman_map::GameMap,
) {
    *current = mode;
    match mode {
        EditorMode::Normal => editor.selected = None,
        EditorMode::Editor => {
            let coord = HexCoord { q: 0, r: 0 };
            if let Some(data) = game_map.hexes.get(&coord) {
                editor.selected = Some(coord);
                editor.name = data.name.clone().unwrap_or_default();
                editor.terrain = data.terrain;
            }
        }
        EditorMode::Secret => {}
        EditorMode::Sprites => {
            if browser.selected_sprite.is_none()
                && let Some(section) = browser.sections.first()
                && let Some(sprite) = section.sprites.first()
            {
                browser.selected_sprite = Some(browser::SpriteSelection {
                    section: 0,
                    sprite: 0,
                    section_name: section.name.clone(),
                    unit_name: section.name.replace('_', " "),
                    col: sprite.col,
                    row: sprite.row,
                });
            }
        }
        _ => {}
    }
}

fn sync_mode_visibilities(
    mode: Res<EditorMode>,
    mut vis_set: ParamSet<(
        Query<&mut Visibility, With<units::UnitsPlane>>,
        Query<&mut Visibility, With<render::MapPlane>>,
        Query<&mut Visibility, With<browser::SpriteBrowserRoot>>,
        Query<&mut Visibility, With<StatusPane>>,
        Query<&mut Visibility, With<picker::PlacedUnit>>,
    )>,
) {
    if let Ok(mut vis) = vis_set.p0().single_mut() {
        *vis = if *mode == EditorMode::Units {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = vis_set.p1().single_mut() {
        *vis = if matches!(*mode, EditorMode::Units | EditorMode::Sprites | EditorMode::Secret) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
    if let Ok(mut vis) = vis_set.p2().single_mut() {
        *vis = if *mode == EditorMode::Sprites {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = vis_set.p3().single_mut() {
        *vis = if matches!(*mode, EditorMode::Normal) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut vis in vis_set.p4().iter_mut() {
        *vis = if *mode == EditorMode::Normal {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn mode_toolbar(
    mut contexts: EguiContexts,
    mut current: ResMut<EditorMode>,
    mut editor: ResMut<editor::HexEditor>,
    mut browser: ResMut<browser::SpriteBrowser>,
    game_map: Res<omdurman_map::GameMap>,
    mut pending: ResMut<PendingEdits>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if *current == EditorMode::Secret {
        return;
    }

    egui::Area::new(egui::Id::new("mode_toolbar"))
        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_gray(45))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
                    let label = format!("{:?}", *current);
                    let mut clicked = None;
                    egui::ComboBox::from_id_salt("mode_selector")
                        .selected_text(label)
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_value(&mut *current, EditorMode::Normal, "Normal")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Normal);
                            }
                            if ui
                                .selectable_value(&mut *current, EditorMode::Overlay, "Overlay")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Overlay);
                            }
                            if ui
                                .selectable_value(&mut *current, EditorMode::Editor, "Editor")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Editor);
                            }
                            if ui
                                .selectable_value(&mut *current, EditorMode::Units, "Units")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Units);
                            }
                            if ui
                                .selectable_value(&mut *current, EditorMode::Sprites, "Sprites")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Sprites);
                            }
                            if ui
                                .selectable_value(&mut *current, EditorMode::Dice, "Dice")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Dice);
                            }
                        });
                    if let Some(m) = clicked {
                        apply_mode(m, &mut *current, &mut *editor, &mut *browser, &game_map);
                        pending.items.push(NetMsg::ModeSwitch(m as u8));
                    }
                });
        });
}

fn mode_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut current: ResMut<EditorMode>,
    mut editor: ResMut<editor::HexEditor>,
    mut browser: ResMut<browser::SpriteBrowser>,
    game_map: Res<omdurman_map::GameMap>,
    mut shortcuts: ResMut<ShortcutsOverlay>,
    mut pending: ResMut<PendingEdits>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_keyboard_input() {
        return;
    }
    if !keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        return;
    }

    if keys.just_pressed(KeyCode::Digit0) {
        let m = EditorMode::Secret;
        apply_mode(m, &mut *current, &mut *editor, &mut *browser, &game_map);
        pending.items.push(NetMsg::ModeSwitch(m as u8));
        return;
    }

    if keys.just_pressed(KeyCode::Digit9) {
        shortcuts.visible = !shortcuts.visible;
        return;
    }

    let next = if keys.just_pressed(KeyCode::Digit1) && *current != EditorMode::Normal {
        Some(EditorMode::Normal)
    } else if keys.just_pressed(KeyCode::Digit2) && *current != EditorMode::Overlay {
        Some(EditorMode::Overlay)
    } else if keys.just_pressed(KeyCode::Digit3) && *current != EditorMode::Editor {
        Some(EditorMode::Editor)
    } else if keys.just_pressed(KeyCode::Digit4) && *current != EditorMode::Units {
        Some(EditorMode::Units)
    } else if keys.just_pressed(KeyCode::Digit5) && *current != EditorMode::Sprites {
        Some(EditorMode::Sprites)
    } else if keys.just_pressed(KeyCode::Digit6) && *current != EditorMode::Dice {
        Some(EditorMode::Dice)
    } else {
        None
    };

    if let Some(m) = next {
        apply_mode(m, &mut *current, &mut *editor, &mut *browser, &game_map);
        pending.items.push(NetMsg::ModeSwitch(m as u8));
    }
}

fn flush_pending(
    mut pending: ResMut<PendingEdits>,
    net: Res<NetState>,
    mut socket_q: Query<&mut MatchboxSocket>,
) {
    if pending.items.is_empty() && pending.targeted.is_empty() && pending.retry.is_empty() {
        return;
    }
    let Ok(mut socket) = socket_q.single_mut() else {
        return;
    };

    let peers = &net.peers;

    // 1. retry previously failed sends
    let mut new_retry: Vec<(NetMsg, PeerId)> = Vec::new();
    for (msg, peer) in pending.retry.drain(..) {
        if socket.channel_mut(0).try_send(enc_msg(&msg), peer).is_err() {
            new_retry.push((msg, peer));
        }
    }

    // 2. targeted sends (single peer)
    for (msg, peer) in pending.targeted.drain(..) {
        if socket.channel_mut(0).try_send(enc_msg(&msg), peer).is_err() {
            new_retry.push((msg, peer));
        }
    }

    // 3. broadcast items (all peers)
    for msg in pending.items.drain(..) {
        let encoded = enc_msg(&msg);
        for &peer in peers {
            if socket.channel_mut(0).try_send(encoded.clone(), peer).is_err() {
                new_retry.push((msg.clone(), peer));
            }
        }
    }

    pending.retry = new_retry;
}

fn shortcuts_ui(mut contexts: EguiContexts, shortcuts: Res<ShortcutsOverlay>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !shortcuts.visible {
        return;
    }
    egui::Window::new("keyboard shortcuts")
        .default_pos([300.0, 100.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
            ui.label("Ctrl+0  normal mode");
            ui.label("Ctrl+1  hex overlay");
            ui.label("Ctrl+2  hex editor");
            ui.label("Ctrl+3  unit viewer");
            ui.label("Ctrl+4  sprite browser");
            ui.label("Ctrl+5  dice simulator");
            ui.label("Ctrl+0  secret");
            ui.label("Ctrl+9  this screen");
            ui.separator();
            ui.label("Ctrl+scroll  pitch camera");
            ui.label("Scroll       zoom");
            ui.label("Arrows       pan");
            ui.label("PageUp/Down  tilt pitch");
            ui.separator();
            ui.label("U/Y   hex size +/-");
            ui.label("I/K   hex offset y");
            ui.label("J/L   hex offset x");
            ui.separator();
            ui.label("B/D/F/K    BlueNile/Desert/Fortress/Khartoum");
            ui.label("H/M/N/P/S  Hogali/FortMakran/NorthFort/Palm/Shrubs");
            ui.label("T/U/W/1    Tuti/Buri/WhiteNile/FortBuri");
        });
}

fn camera_control(
    time: Res<Time>,
    settings: Res<CameraSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut scroll_events: MessageReader<MouseWheel>,
    mut drag_state: ResMut<CameraDragState>,
    windows: Query<&Window>,
    mut cam_q: Query<(&mut RtsCameraState, &mut Transform), With<RtsCamera>>,
    mode: Res<EditorMode>,
    mut contexts: EguiContexts,
    touches: Res<Touches>,
) {
    if matches!(*mode, EditorMode::Sprites | EditorMode::Secret) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((mut state, mut transform)) = cam_q.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    // ── Right-click drag pan ──────────────────────────────────────────────
    let cursor_pos = windows.single().ok().and_then(|w| w.cursor_position());
    if !ctx.wants_pointer_input() {
        if buttons.just_pressed(MouseButton::Right) {
            drag_state.active = true;
            if let Some(pos) = cursor_pos {
                drag_state.last_cursor = pos;
            }
        } else if buttons.just_released(MouseButton::Right) {
            drag_state.active = false;
        }
    } else {
        drag_state.active = false;
    }

    if drag_state.active
        && let (Some(pos), false) = (cursor_pos, ctx.wants_pointer_input())
    {
        let delta = Vec2::new(
            pos.x - drag_state.last_cursor.x,
            pos.y - drag_state.last_cursor.y,
        );
        if delta.length_squared() > 0.0 {
            // Convert screen-space drag delta to world-space focus delta.
            // At distance 500 the scale is ~1 world unit per pixel, tuned by feel.
            let scale = (state.distance / 500.0) * 0.6;
            let fwd = Vec3::new(-state.yaw.sin(), 0.0, -state.yaw.cos());
            let right = Vec3::new(fwd.z, 0.0, -fwd.x);
            state.focus += fwd * delta.y * scale + right * delta.x * scale;
        }
        drag_state.last_cursor = pos;
    }

    // ── Arrow-key pan ────────────────────────────────────────────────────
    let mut pan = Vec2::ZERO;
    if !ctx.wants_keyboard_input() {
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
    }
    if pan != Vec2::ZERO {
        pan = pan.normalize() * settings.pan_speed * dt * (state.distance / 500.0).max(0.3);
        let fwd = Vec3::new(-state.yaw.sin(), 0.0, -state.yaw.cos());
        let right = Vec3::new(fwd.z, 0.0, -fwd.x);
        state.focus += fwd * pan.y + right * pan.x;
    }

    let mut zoom_ticks: f32 = 0.0;
    if !ctx.wants_pointer_input() {
        for ev in scroll_events.read() {
            let notch_scale = match ev.unit {
                MouseScrollUnit::Pixel => 0.01,
                MouseScrollUnit::Line => 1.0,
            };
            zoom_ticks += ev.y * notch_scale;
        }
    }
    if zoom_ticks != 0.0 {
        if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
            state.pitch =
                (state.pitch + zoom_ticks * 0.1).clamp(settings.min_pitch, settings.max_pitch);
        } else {
            let factor = 1.0 - zoom_ticks.clamp(-5.0, 5.0) * 0.06;
            state.distance =
                (state.distance * factor).clamp(settings.min_distance, settings.max_distance);
        }
    }

    let pitch_step = dt * 0.8;
    if !ctx.wants_keyboard_input() {
        if keys.pressed(KeyCode::PageUp) {
            state.pitch = (state.pitch + pitch_step).min(settings.max_pitch);
        }
        if keys.pressed(KeyCode::PageDown) {
            state.pitch = (state.pitch - pitch_step).max(settings.min_pitch);
        }
    }

    // ── Touch gestures (pinch zoom + two-finger pitch) ────────────────────
    if !ctx.wants_pointer_input() {
        let mut touches_iter = touches.iter();
        if let (Some(t0), Some(t1)) = (touches_iter.next(), touches_iter.next()) {
            // pinch zoom
            let prev_dist = t0.previous_position().distance(t1.previous_position());
            let cur_dist = t0.position().distance(t1.position());
            let pinch_delta = cur_dist - prev_dist;
            if pinch_delta != 0.0 {
                let factor = 1.0 - pinch_delta.clamp(-30.0, 30.0) * 0.02;
                state.distance =
                    (state.distance * factor).clamp(settings.min_distance, settings.max_distance);
            }
            // two-finger vertical drag → pitch
            let prev_mid_y = (t0.previous_position().y + t1.previous_position().y) * 0.5;
            let cur_mid_y = (t0.position().y + t1.position().y) * 0.5;
            let pitch_delta = cur_mid_y - prev_mid_y;
            if pitch_delta != 0.0 {
                state.pitch =
                    (state.pitch - pitch_delta * 0.02).clamp(settings.min_pitch, settings.max_pitch);
            }
        }
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

fn load_annotations(
    mut commands: Commands,
    mut game_map: ResMut<GameMap>,
    mut overlay: ResMut<render::HexOverlay>,
    mut current: ResMut<EditorMode>,
) {
    let ron_str = include_str!("../assets/annotations.ron");
    let annotations = load_annotations_from_str(ron_str, &mut game_map);
    overlay.params = game_map.overlay.clone();
    commands.insert_resource(browser::SpriteAnnotationsResource(annotations.sprites));
    *current = EditorMode::Normal;
}

pub fn d10_collider_points(radius: f32, height: f32) -> Vec<Vec3> {
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

pub fn d10_mesh_colored(radius: f32, height: f32) -> Mesh {
    let n = 5;
    let top = [0.0, height / 2.0, 0.0];
    let bot = [0.0, -height / 2.0, 0.0];

    let mut ring = Vec::new();
    for k in 0..n {
        let a = core::f32::consts::TAU * k as f32 / n as f32;
        ring.push([radius * a.cos(), 0.0, radius * a.sin()]);
    }

    let pastel_colors: [[f32; 4]; 10] = [
        [0.95, 0.85, 0.65, 1.0],
        [0.95, 0.75, 0.45, 1.0],
        [0.95, 0.65, 0.25, 1.0],
        [0.90, 0.55, 0.20, 1.0],
        [0.95, 0.45, 0.15, 1.0],
        [0.90, 0.35, 0.10, 1.0],
        [0.85, 0.25, 0.10, 1.0],
        [0.80, 0.20, 0.10, 1.0],
        [1.00, 0.90, 0.60, 1.0],
        [0.85, 0.55, 0.20, 1.0],
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
