use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::GameMap;
use omdurman_net::{Control, NetMsg, NetState};
use std::borrow::Cow;

use crate::{
    AppState, CursorPositions, EditorMode, GamePhaseApp, GameTurn, HoveredHex, PendingEdits,
    RoomId, browser, camera::RtsCamera, editor, settings,
};

#[derive(Component)]
pub(crate) struct StatusPane;

#[derive(Component)]
pub(crate) struct StatusText;

#[derive(Component)]
pub(crate) struct HexCoordLabel;

#[derive(Component)]
struct HexCoordPane;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        use crate::{dice, event_viewer, lobby, units};

        app.insert_resource(settings::SettingsOverlay::default())
            .insert_resource(settings::LocalPlayerSettings::default())
            .insert_resource(settings::PlayerInfoMap::default())
            .insert_resource(units::UnitViewer::load_or_default())
            .insert_resource(browser::SpriteBrowser::new())
            .insert_resource(browser::SpriteMetaClipboard::default())
            .insert_resource(dice::DiceSimulator::default())
            .insert_resource(event_viewer::EventViewerState::default())
            .add_systems(
                Startup,
                (
                    setup_ui,
                    configure_egui_touch,
                    maximize_primary_window,
                    units::spawn_units_plane,
                    browser::spawn_sprite_browser,
                ),
            )
            .add_systems(
                Update,
                (
                    setup_egui_fonts,
                    update_status_text,
                    update_hex_coord_display,
                    handle_mode_shortcuts,
                    units::draw_unit_grids,
                    browser::scroll_sprite_browser,
                    browser::handle_sprite_clicks,
                    browser::update_sprite_selection_marker,
                    browser::navigate_sprite_selection,
                ),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    cursor_overlay_ui.run_if(map_mode_active_state),
                    mode_toolbar,
                    game_hud,
                    game_log_panel,
                    victory_modal,
                    units::unit_grids_ui,
                    units::unit_grid_labels,
                    browser::sprite_meta_editor_ui,
                    dice::dice_sim_ui,
                    event_viewer::event_viewer_ui,
                    settings::settings_ui,
                    lobby::lobby_ui,
                ),
            );
    }
}

pub(crate) fn maximize_primary_window(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.set_maximized(true);
}

pub(crate) fn setup_egui_fonts(mut contexts: EguiContexts, mut done: Local<bool>) {
    if *done {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
    ctx.add_font(FontInsert::new(
        "EBGaramond-Regular",
        egui::FontData::from_static(include_bytes!("../../assets/fonts/EBGaramond-Regular.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name("Garamond".into()),
            priority: FontPriority::Highest,
        }],
    ));
    *done = true;
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut, unused_variables))]
pub(crate) fn configure_egui_touch(mut contexts: EguiContexts) {
    #[cfg(target_arch = "wasm32")]
    {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        ctx.style_mut(|style| {
            style.spacing.interact_size = egui::vec2(40.0, 40.0);
            style.spacing.slider_width = 120.0;
        });
    }
}

pub(crate) fn setup_ui(mut commands: Commands) {
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
            Text::new("Connecting..."),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
        ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(14.0),
                right: Val::Px(14.0),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            HexCoordPane,
        ))
        .with_child((
            HexCoordLabel,
            Text::new(""),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
        ));
}

pub(crate) fn update_status_text(
    state: Res<State<AppState>>,
    room: Res<RoomId>,
    game_state: Option<Res<crate::GameStateResource>>,
    factions: Res<crate::PlayerFactions>,
    net: Res<NetState>,
    mut query: Query<&mut Text, With<StatusText>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    let new = match state.get() {
        AppState::Connecting => {
            Cow::Owned(format!("Waiting for players - share: ?room={}", room.0))
        }
        AppState::Lobby => Cow::Borrowed("Lobby -- choose your faction"),
        // In game, status follows the rules engine's active player and the
        // local faction binding: the turn advances via the End Phase button,
        // not any key. (Movement/fire/melee are gated on this same condition.)
        AppState::InGame => match game_state.as_deref() {
            Some(gs) => {
                let active = gs.0.active_player;
                let label = match active {
                    omdurman_rules::Player::AngloEgyptian => "Anglo-Egyptian",
                    omdurman_rules::Player::Dervish => "Dervish",
                };
                if factions.local_may_act(&net, active) {
                    Cow::Owned(format!("Your turn ({label}) -- act, then End Phase"))
                } else {
                    Cow::Owned(format!("{label}'s turn..."))
                }
            }
            None => Cow::Borrowed("Setting up..."),
        },
    };
    if text.as_str() != new.as_ref() {
        *text = Text::new(new.into_owned());
    }
}

pub(crate) fn update_hex_coord_display(
    hovered: Res<HoveredHex>,
    mut query: Query<&mut Text, With<HexCoordLabel>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    let new = match hovered.0 {
        Some(coord) => format!("({}, {})", coord.q, coord.r),
        None => String::new(),
    };
    if text.as_str() != new {
        *text = Text::new(new);
    }
}

fn apply_mode(
    mode: EditorMode,
    editor: &mut editor::HexEditor,
    browser: &mut browser::SpriteBrowser,
    game_map: &GameMap,
) {
    use omdurman_types::HexCoord;
    match mode {
        EditorMode::FallOfKhartoumMap | EditorMode::CampaignMap => {
            editor.selection.clear();
            editor.anchor = None;
        }
        EditorMode::Editor | EditorMode::CampaignEditor => {
            let coord = HexCoord { q: 0, r: 0 };
            if game_map.hexes.contains_key(&coord) {
                editor.selection.clear();
                editor.selection.insert(coord);
                editor::load_anchor(coord, editor, game_map);
            }
        }
        EditorMode::EventViewer => {}
        EditorMode::CampaignTiming => {
            editor.selection.clear();
            editor.anchor = None;
        }
        EditorMode::Units => {
            if browser.selected_sprite.is_none()
                && let Some(section) = browser.sections.first()
                && let Some(sprite) = section.sprites.first()
            {
                browser.selected_sprite = Some(browser::SpriteSelection {
                    section: 0,
                    sprite: 0,
                    section_name: section.name,
                    unit_name: section.name.display_name().to_string(),
                    col: sprite.col,
                    row: sprite.row,
                });
            }
        }
        _ => {}
    }
}

pub(crate) fn handle_mode_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<EditorMode>>,
    mut editor: ResMut<editor::HexEditor>,
    mut browser: ResMut<browser::SpriteBrowser>,
    game_map: Res<GameMap>,
    mut contexts: EguiContexts,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_keyboard_input() {
        return;
    }
    let ctrl = crate::util::ctrl_held(&keys);
    if !ctrl {
        return;
    }
    let new_mode = if keys.just_pressed(KeyCode::Digit1) {
        Some(EditorMode::FallOfKhartoumMap)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(EditorMode::CampaignMap)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(EditorMode::Overlay)
    } else if keys.just_pressed(KeyCode::Digit4) {
        Some(EditorMode::Editor)
    } else if keys.just_pressed(KeyCode::Digit5) {
        Some(EditorMode::UnitSheet)
    } else if keys.just_pressed(KeyCode::Digit6) {
        Some(EditorMode::Units)
    } else if keys.just_pressed(KeyCode::Digit7) {
        Some(EditorMode::Dice)
    } else if keys.just_pressed(KeyCode::Digit8) {
        Some(EditorMode::EventViewer)
    } else if keys.just_pressed(KeyCode::Digit9) {
        Some(EditorMode::CampaignTiming)
    } else {
        None
    };
    if let Some(m) = new_mode {
        apply_mode(m, &mut editor, &mut browser, &game_map);
        next.set(m);
        info!(mode = ?m, "mode switch via keyboard shortcut");
    }
}

fn request_snapshot_if_guest(net: &mut NetState, pending: &mut PendingEdits) {
    if !net.is_host && !net.peers.is_empty() {
        net.needs_snapshot = true;
        net.snapshot_retry_timer = 0.0;
        if let Some(host) = net.host_id() {
            pending
                .outgoing_targeted
                .push((NetMsg::Control(Control::RequestSnapshot), host));
        }
    }
}

pub(crate) fn mode_toolbar(
    mut contexts: EguiContexts,
    current: Res<State<EditorMode>>,
    app_state: Res<State<AppState>>,
    mut next: ResMut<NextState<EditorMode>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut editor: ResMut<editor::HexEditor>,
    mut browser: ResMut<browser::SpriteBrowser>,
    game_map: Res<GameMap>,
    mut net: ResMut<NetState>,
    mut pending: ResMut<PendingEdits>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let mut clicked = None;
    let mut clicked_lobby = false;
    let mut selected = **current;

    egui::Area::new(egui::Id::new("mode_toolbar"))
        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_gray(45))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
                    let mode_label = selected.to_string();
                    egui::ComboBox::from_id_salt("mode_selector")
                        .selected_text(mode_label)
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            macro_rules! mode_btn {
                                ($variant:ident, $label:expr) => {{
                                    if ui
                                        .selectable_value(
                                            &mut selected,
                                            EditorMode::$variant,
                                            $label,
                                        )
                                        .clicked()
                                    {
                                        clicked = Some(EditorMode::$variant);
                                    }
                                }};
                            }
                            mode_btn!(FallOfKhartoumMap, "Fall Of Khartoum Map");
                            mode_btn!(CampaignMap, "Campaign Map");
                            mode_btn!(Overlay, "Overlay");
                            mode_btn!(Editor, "Editor");
                            mode_btn!(Hexside, "Hexsides");
                            mode_btn!(CampaignOverlay, "Campaign Overlay");
                            mode_btn!(CampaignEditor, "Campaign Editor");
                            mode_btn!(CampaignHexside, "Campaign Hexsides");
                            mode_btn!(CampaignTiming, "Campaign Timing");
                            mode_btn!(UnitSheet, "Unit Sheet");
                            mode_btn!(Units, "Units");
                            mode_btn!(Dice, "Dice");
                            mode_btn!(EventViewer, "EventViewer");
                            ui.separator();
                            if ui
                                .selectable_label(*app_state.get() == AppState::Lobby, "Lobby")
                                .clicked()
                            {
                                clicked_lobby = true;
                            }
                        });
                    if let Some(m) = clicked {
                        apply_mode(m, &mut editor, &mut browser, &game_map);
                        next.set(m);
                        if *app_state.get() == AppState::Lobby {
                            next_app_state.set(AppState::InGame);
                        }
                    } else if clicked_lobby {
                        info!("entering lobby (voluntary)");
                        next_app_state.set(AppState::Lobby);
                        request_snapshot_if_guest(&mut net, &mut pending);
                    }
                });
        });
}

pub(crate) fn cursor_overlay_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    local: Res<settings::LocalPlayerSettings>,
    mut cursor_positions: ResMut<CursorPositions>,
    player_info: Res<settings::PlayerInfoMap>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
) {
    if !local.show_other_cursors || cursor_positions.current.is_empty() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };

    let now = time.elapsed_secs_f64();
    let dt = time.delta_secs();

    const SMOOTH: f32 = 6.0;
    let alpha = 1.0 - (-SMOOTH * dt).exp();

    let peers: Vec<_> = cursor_positions.current.keys().copied().collect();

    for peer in &peers {
        let pos = cursor_positions.current[peer];
        let t = match cursor_positions.last_update.get(peer) {
            Some(&last) if last > 0.0 => {
                let elapsed = now - last;
                (elapsed / 0.1).clamp(0.0, 1.0)
            }
            _ => 1.0,
        };
        let prev = cursor_positions.previous.get(peer).copied().unwrap_or(pos);
        let target = prev.lerp(pos, t as f32);
        let display = cursor_positions.display.entry(*peer).or_insert(target);
        *display = display.lerp(target, alpha);
    }

    egui::Area::new(egui::Id::new("cursor_overlay"))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            for peer in &peers {
                let Some(&world_xz) = cursor_positions.display.get(peer) else {
                    continue;
                };
                let world = Vec3::new(world_xz.x, 0.0, world_xz.y);
                let Ok(viewport) = camera.world_to_viewport(cam_transform, world) else {
                    continue;
                };
                let screen = egui::pos2(viewport.x, viewport.y);

                let color = player_info
                    .peers
                    .get(peer)
                    .map(|p| p.color)
                    .unwrap_or(egui::Color32::WHITE);
                painter.circle_filled(screen, 5.0, color);
                let label = player_info
                    .peers
                    .get(peer)
                    .map(|p| p.name.as_str())
                    .unwrap_or("?");
                painter.text(
                    screen + egui::Vec2::new(8.0, -4.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(12.0),
                    color,
                );
            }
        });
}

fn map_mode_active(mode: EditorMode) -> bool {
    mode.is_map_mode() || mode.is_overlay() || mode.is_editor() || mode.is_hexside()
}

pub(crate) fn map_mode_active_state(mode: Res<State<EditorMode>>) -> bool {
    map_mode_active(**mode)
}

/// Hover text for the "Set up scenario" button: how many counters will be
/// placed, and a warning for any anchor that could not be resolved on the
/// current board (so a missing landmark is surfaced, never silently dropped).
fn setup_hover(plan: &crate::scenario_setup::SetupPlan) -> String {
    let mut s = format!("Place {} fixed-hex unit(s)", plan.placements.len());
    if !plan.unresolved.is_empty() {
        s.push_str(&format!(
            "\nUnresolved (anchor not on this map): {}",
            plan.unresolved.join(", ")
        ));
    }
    s
}

/// Game HUD: turn/phase/day-night info bar + End Phase button.
/// Only visible when a game is active (GameStateResource exists).
pub(crate) fn game_hud(
    mut contexts: EguiContexts,
    game_turn: Option<Res<GameTurn>>,
    game_phase: Option<Res<GamePhaseApp>>,
    game_state: Option<Res<crate::GameStateResource>>,
    game_map: Option<Res<omdurman_hexmap::GameMap>>,
    mut pending: Option<ResMut<crate::PendingEdits>>,
) {
    let Some(state) = game_state else { return };
    let Some(turn) = game_turn else { return };
    let Some(phase) = game_phase else { return };
    let Some(pending) = pending.as_deref_mut() else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let day_night_str = match state.0.day_night {
        omdurman_rules::DayNight::Day => "Day",
        omdurman_rules::DayNight::Night => "Night",
    };
    let active_player_str = match state.0.active_player {
        omdurman_rules::Player::AngloEgyptian => "A-E",
        omdurman_rules::Player::Dervish => "Dervish",
    };

    egui::Area::new(egui::Id::new("game_hud"))
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-180.0, 6.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_gray(35))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::proportional(14.0));

                    ui.horizontal(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(200, 200, 150),
                            format!("Turn {}  {}  {}", **turn, *phase, day_night_str),
                        );
                        ui.label(format!("Active: {active_player_str}"));
                        ui.separator();
                        if ui.button("End Phase").clicked() {
                            pending.outgoing_broadcast.push(omdurman_net::NetMsg::Game(
                                omdurman_net::GameEvent::Effect(
                                    omdurman_rules::effects::GameEffect::AdvancePhase,
                                ),
                            ));
                        }
                        // Auto-place the scenario's fixed-hex units (Historical
                        // leaders; the FoK GORDON). Other units are placed by
                        // hand from the picker. Best-effort: re-placing is safe.
                        if let Some(map) = game_map.as_deref() {
                            let plan =
                                crate::scenario_setup::build_setup_plan(state.0.scenario, map);
                            if !plan.placements.is_empty()
                                && ui
                                    .button("Set up scenario")
                                    .on_hover_text(setup_hover(&plan))
                                    .clicked()
                            {
                                for ev in plan.placements {
                                    pending
                                        .outgoing_broadcast
                                        .push(omdurman_net::NetMsg::Game(ev));
                                }
                            }
                        }
                    });
                });
        });
}

/// Combat/event feed: the most recent lines of the rules engine's log (combat
/// results, eliminations, recoveries, victory). The engine writes these as it
/// applies effects; surfacing them gives players the "what just happened" that
/// was previously invisible (results only showed as counters changing).
pub(crate) fn game_log_panel(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
) {
    let Some(state) = game_state else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if state.0.log.is_empty() {
        return;
    }
    egui::Area::new(egui::Id::new("game_log"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::new(8.0, -8.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_black_alpha(180))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));
                    ui.set_max_width(460.0);
                    // Last few lines, oldest first, newest highlighted.
                    let lines: Vec<&String> = state.0.log.iter().rev().take(6).collect();
                    for (i, line) in lines.iter().rev().enumerate() {
                        let newest = i == lines.len() - 1;
                        let color = if newest {
                            egui::Color32::from_rgb(230, 230, 180)
                        } else {
                            egui::Color32::from_gray(160)
                        };
                        ui.colored_label(color, *line);
                    }
                });
        });
}

/// Victory modal: when the rules engine ends the game (`game_over`), show a
/// centered banner with the final result. The result line(s) are the tail of
/// the log written by `finish_game` (the scenario-specific verdict, e.g. the
/// §9.35 Fall-of-Khartoum level). Blocks nothing else; input gating on a
/// finished game is handled by the rules engine rejecting further effects.
pub(crate) fn victory_modal(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
) {
    let Some(state) = game_state else { return };
    if !state.0.game_over {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    // Show the "=== GAME OVER ===" marker and everything after it: the result.
    let tail: Vec<&String> = state
        .0
        .log
        .iter()
        .rev()
        .take_while(|l| !l.contains("GAME OVER"))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    egui::Area::new(egui::Id::new("victory_modal"))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 30, 40))
                .corner_radius(8.0)
                .inner_margin(egui::Margin::symmetric(24, 18))
                .stroke(egui::Stroke::new(
                    2.0,
                    egui::Color32::from_rgb(200, 180, 120),
                ))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("GAME OVER")
                                .size(28.0)
                                .strong()
                                .color(egui::Color32::from_rgb(220, 200, 140)),
                        );
                        ui.add_space(8.0);
                        for line in tail {
                            ui.label(egui::RichText::new(line).size(15.0));
                        }
                    });
                });
        });
}
