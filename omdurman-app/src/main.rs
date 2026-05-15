//! Remember Gordon! Battle of Omdurman.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod browser;
mod dice;
mod editor;
mod render;
mod units;
mod util;

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseWheel;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_matchbox::prelude::*;
use omdurman_hex::HexLayout;
use omdurman_map::{GameMap, load_annotations_from_str};
use omdurman_net::{
    GameRng, NetMsg, NetState, RoomId, decode, enc_msg, new_seed, open_socket, room_id,
};
use omdurman_types::{HexCoord, IntoEnumIterator};
use std::f32::consts::PI;

#[derive(Resource, Default)]
struct ShortcutsOverlay {
    visible: bool,
}

#[derive(Resource, Default)]
struct PendingEdits(Vec<NetMsg>);

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
        .insert_resource(GameMap::default())
        .insert_resource(render::HexOverlay::default())
        .insert_resource(editor::HexEditor::default())
        .insert_resource(units::UnitViewer::load_or_default())
        .insert_resource(browser::SpriteBrowser::new())
        .insert_resource(browser::SpriteMetaClipboard::default())
        .insert_resource(dice::DiceSimulator::default())
        .insert_resource(ShortcutsOverlay::default())
        .insert_resource(PendingEdits::default())
        .insert_resource(SidebarClip::default())
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
                units::spawn_units_plane,
                browser::spawn_sprite_browser,
                load_annotations,
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
                handle_local_input.after(handle_socket),
                update_status_text.after(handle_socket),
                units::draw_unit_grids,
                mode_shortcuts,
                sync_outgoing,
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
                editor::editor_labels_ui,
                units::unit_grids_ui,
                units::unit_grid_labels,
                browser::sprite_meta_editor_ui,
                dice::dice_sim_ui,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EditorMode {
    Normal,
    Overlay,
    Editor,
    Units,
    Sprites,
    Dice,
}

impl EditorMode {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Overlay,
            2 => Self::Editor,
            3 => Self::Units,
            4 => Self::Sprites,
            5 => Self::Dice,
            _ => Self::Normal,
        }
    }
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
struct StatusPane;

#[derive(Component)]
struct StatusText;

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
    mut turn: ResMut<TurnState>,
    mut commands: Commands,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ev_action: MessageWriter<ActionTaken>,
    mut overlay: ResMut<render::HexOverlay>,
    mut editor: ResMut<editor::HexEditor>,
    mut viewer: ResMut<units::UnitViewer>,
    mut browser: ResMut<browser::SpriteBrowser>,
    mut dice_sim: ResMut<dice::DiceSimulator>,
    mut game_map: ResMut<GameMap>,
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
            Some(NetMsg::MapEdit {
                q,
                r,
                terrain,
                name,
            }) => {
                info!(q, r, "remote map edit");
                let coord = omdurman_types::HexCoord::new(q, r);
                let terrain_val = omdurman_types::Terrain::iter()
                    .nth(terrain as usize)
                    .unwrap_or(omdurman_types::Terrain::Desert);
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
                apply_mode(
                    EditorMode::from_u8(mode),
                    &mut overlay,
                    &mut editor,
                    &mut viewer,
                    &mut browser,
                    &mut dice_sim,
                    &game_map,
                );
            }
            None => warn!("unknown message, ignoring"),
        }
    }
}

fn handle_local_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut turn: ResMut<TurnState>,
    net: Res<NetState>,
    rng_opt: Option<ResMut<GameRng>>,
    mut socket_q: Query<&mut MatchboxSocket>,
    mut ev_action: MessageWriter<ActionTaken>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_keyboard_input() { return; }
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

fn set_all_off(
    overlay: &mut render::HexOverlay,
    editor: &mut editor::HexEditor,
    viewer: &mut units::UnitViewer,
    browser: &mut browser::SpriteBrowser,
    dice_sim: &mut dice::DiceSimulator,
) {
    overlay.visible = false;
    editor.active = false;
    viewer.visible = false;
    browser.visible = false;
    dice_sim.visible = false;
}

fn apply_mode(
    mode: EditorMode,
    overlay: &mut render::HexOverlay,
    editor: &mut editor::HexEditor,
    viewer: &mut units::UnitViewer,
    browser: &mut browser::SpriteBrowser,
    dice_sim: &mut dice::DiceSimulator,
    game_map: &omdurman_map::GameMap,
) {
    set_all_off(overlay, editor, viewer, browser, dice_sim);
    match mode {
        EditorMode::Normal => editor.selected = None,
        EditorMode::Overlay => overlay.visible = true,
        EditorMode::Editor => {
            editor.active = true;
            let coord = HexCoord { q: 0, r: 0 };
            if let Some(data) = game_map.hexes.get(&coord) {
                editor.selected = Some(coord);
                editor.name = data.name.clone().unwrap_or_default();
                editor.terrain = data.terrain;
            }
        }
        EditorMode::Units => viewer.visible = true,
        EditorMode::Sprites => {
            browser.visible = true;
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
        EditorMode::Dice => dice_sim.visible = true,
    }
}

fn sync_mode_visibilities(
    overlay: Res<render::HexOverlay>,
    editor: Res<editor::HexEditor>,
    viewer: Res<units::UnitViewer>,
    browser: Res<browser::SpriteBrowser>,
    dice_sim: Res<dice::DiceSimulator>,
    mut vis_set: ParamSet<(
        Query<&mut Visibility, With<units::UnitsPlane>>,
        Query<&mut Visibility, With<render::MapPlane>>,
        Query<&mut Visibility, With<browser::SpriteBrowserRoot>>,
        Query<&mut Visibility, With<StatusPane>>,
    )>,
) {
    if let Ok(mut vis) = vis_set.p0().single_mut() {
        *vis = if viewer.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = vis_set.p1().single_mut() {
        *vis = if viewer.visible || browser.visible {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
    if let Ok(mut vis) = vis_set.p2().single_mut() {
        *vis = if browser.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let normal = !overlay.visible
        && !editor.active
        && !viewer.visible
        && !browser.visible
        && !dice_sim.visible;
    if let Ok(mut vis) = vis_set.p3().single_mut() {
        *vis = if normal {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn mode_toolbar(
    mut contexts: EguiContexts,
    mut overlay: ResMut<render::HexOverlay>,
    mut editor: ResMut<editor::HexEditor>,
    mut viewer: ResMut<units::UnitViewer>,
    mut browser: ResMut<browser::SpriteBrowser>,
    mut dice_sim: ResMut<dice::DiceSimulator>,
    game_map: Res<omdurman_map::GameMap>,
    net: Res<NetState>,
    mut socket_q: Query<&mut MatchboxSocket>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let normal = !overlay.visible
        && !editor.active
        && !viewer.visible
        && !browser.visible
        && !dice_sim.visible;

    let toolbar_anchor = egui::Align2::LEFT_TOP;
    let toolbar_offset = egui::Vec2::ZERO;

    egui::Area::new(egui::Id::new("mode_toolbar"))
        .anchor(toolbar_anchor, toolbar_offset)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_gray(45))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
                    let current = if normal {
                        None
                    } else if overlay.visible {
                        Some(EditorMode::Overlay)
                    } else if editor.active {
                        Some(EditorMode::Editor)
                    } else if viewer.visible {
                        Some(EditorMode::Units)
                    } else if browser.visible {
                        Some(EditorMode::Sprites)
                    } else {
                        Some(EditorMode::Dice)
                    };
                    let label = match current {
                        Some(m) => format!("{:?}", m),
                        None => "Normal".to_string(),
                    };
                    let mut clicked = None;
                    egui::ComboBox::from_id_salt("mode_selector")
                        .selected_text(label)
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut current.clone(), None, "Normal").clicked() {
                                clicked = Some(EditorMode::Normal);
                            }
                            if ui.selectable_value(&mut current.clone(), Some(EditorMode::Overlay), "Overlay").clicked() {
                                clicked = Some(EditorMode::Overlay);
                            }
                            if ui.selectable_value(&mut current.clone(), Some(EditorMode::Editor), "Editor").clicked() {
                                clicked = Some(EditorMode::Editor);
                            }
                            if ui.selectable_value(&mut current.clone(), Some(EditorMode::Units), "Units").clicked() {
                                clicked = Some(EditorMode::Units);
                            }
                            if ui.selectable_value(&mut current.clone(), Some(EditorMode::Sprites), "Sprites").clicked() {
                                clicked = Some(EditorMode::Sprites);
                            }
                            if ui.selectable_value(&mut current.clone(), Some(EditorMode::Dice), "Dice").clicked() {
                                clicked = Some(EditorMode::Dice);
                            }
                        });
                    if let Some(mode) = clicked {
                            apply_mode(
                                mode,
                                &mut overlay,
                                &mut editor,
                                &mut viewer,
                                &mut browser,
                                &mut dice_sim,
                                &game_map,
                            );
                            if let (Some(peer), Ok(mut socket)) = (net.peer, socket_q.single_mut())
                            {
                                let _ = socket
                                    .channel_mut(0)
                                    .try_send(enc_msg(&NetMsg::ModeSwitch(mode as u8)), peer);
                            }
                        }
                });
        });
}

fn mode_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut overlay: ResMut<render::HexOverlay>,
    mut editor: ResMut<editor::HexEditor>,
    mut viewer: ResMut<units::UnitViewer>,
    mut browser: ResMut<browser::SpriteBrowser>,
    mut dice_sim: ResMut<dice::DiceSimulator>,
    game_map: Res<omdurman_map::GameMap>,
    mut shortcuts: ResMut<ShortcutsOverlay>,
    net: Res<NetState>,
    mut socket_q: Query<&mut MatchboxSocket>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_keyboard_input() {
        return;
    }
    if !keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        return;
    }

    if keys.just_pressed(KeyCode::Digit9) {
        shortcuts.visible = !shortcuts.visible;
        return;
    }

    let mode = if keys.just_pressed(KeyCode::Digit1)
        && (overlay.visible
            || editor.active
            || viewer.visible
            || browser.visible
            || dice_sim.visible)
    {
        Some(EditorMode::Normal)
    } else if keys.just_pressed(KeyCode::Digit2) && !overlay.visible {
        Some(EditorMode::Overlay)
    } else if keys.just_pressed(KeyCode::Digit3) && !editor.active {
        Some(EditorMode::Editor)
    } else if keys.just_pressed(KeyCode::Digit4) && !viewer.visible {
        Some(EditorMode::Units)
    } else if keys.just_pressed(KeyCode::Digit5) && !browser.visible {
        Some(EditorMode::Sprites)
    } else if keys.just_pressed(KeyCode::Digit6) && !dice_sim.visible {
        Some(EditorMode::Dice)
    } else {
        None
    };

    if let Some(mode) = mode {
        apply_mode(
            mode,
            &mut overlay,
            &mut editor,
            &mut viewer,
            &mut browser,
            &mut dice_sim,
            &game_map,
        );
        if let (Some(peer), Ok(mut socket)) = (net.peer, socket_q.single_mut()) {
            let _ = socket
                .channel_mut(0)
                .try_send(enc_msg(&NetMsg::ModeSwitch(mode as u8)), peer);
        }
    }
}

fn sync_outgoing(
    mut pending: ResMut<PendingEdits>,
    net: Res<NetState>,
    mut socket_q: Query<&mut MatchboxSocket>,
) {
    if pending.0.is_empty() {
        return;
    }
    let Some(peer) = net.peer else { return };
    let Ok(mut socket) = socket_q.single_mut() else {
        return;
    };
    for msg in pending.0.drain(..) {
        let _ = socket.channel_mut(0).try_send(enc_msg(&msg), peer);
    }
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
    mut scroll_events: MessageReader<MouseWheel>,
    mut cam_q: Query<(&mut RtsCameraState, &mut Transform), With<RtsCamera>>,
    browser: Res<browser::SpriteBrowser>,
    mut contexts: EguiContexts,
) {
    if browser.visible {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((mut state, mut transform)) = cam_q.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

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
            zoom_ticks += ev.y;
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
    ) {
        let ron_str = include_str!("../assets/annotations.ron");
        let annotations = load_annotations_from_str(ron_str, &mut game_map);
        // Sync overlay from GameMap immediately (not deferred to a later frame).
        overlay.params = game_map.overlay.clone();
        commands.insert_resource(browser::SpriteAnnotationsResource(annotations.sprites));
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
