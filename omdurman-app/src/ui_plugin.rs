use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_net::{Control, NetMsg, NetState};
use std::borrow::Cow;

use crate::{
    AppMode, AppState, CursorPositions, EditorBoard, EditorTab, GameTurn, HoveredHex,
    PendingEdits, RoomId, browser, camera::RtsCamera, settings,
};

// -- UI resources -----------------------------------------------------------

#[derive(Resource, Default)]
pub struct SidebarClip {
    pub right_sidebar: Option<egui::Rect>,
}

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
                    cursor_overlay_ui.run_if(crate::map_view_active),
                    mode_toolbar,
                    // In-game HUD/overlays: only while actually in a game, so
                    // they don't show over the lobby.
                    (
                        game_log_panel,
                        victory_modal,
                        crate::fire::fire_combat_preview_ui,
                    )
                        .run_if(in_state(AppState::InGame)),
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
    // A real italic face, registered as its own family. Italic text (the splash
    // quote, book titles) selects this family rather than egui's synthetic
    // italic -- epaint fakes italics by shearing the upright glyphs without
    // fixing advances, which left uneven gaps (e.g. "tran quillity"). A genuine
    // italic has correct metrics.
    ctx.add_font(FontInsert::new(
        "EBGaramond-Italic",
        egui::FontData::from_static(include_bytes!("../../assets/fonts/EBGaramond-Italic.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name("GaramondItalic".into()),
            priority: FontPriority::Highest,
        }],
    ));
    // NOTE: a full-app paper-skin override was tried and dropped UI contrast
    // too far, so egui keeps its default visuals. Per-surface colours are
    // inlined where needed (panel backgrounds via `crate::ui::panel_bg`).
    *done = true;
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut, unused_variables))]
pub(crate) fn configure_egui_touch(mut contexts: EguiContexts) {
    #[cfg(target_arch = "wasm32")]
    {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        ctx.style_mut_of(egui::Theme::Dark, |style| {
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
                font_size: FontSize::Px(22.0),
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
                font_size: FontSize::Px(16.0),
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
        AppState::Splash => Cow::Borrowed(""),
        AppState::Connecting => {
            Cow::Owned(format!("Waiting for players - share: ?room={}", room.as_str()))
        }
        AppState::Lobby => Cow::Borrowed("Lobby -- choose your faction"),
        // In game, status follows the rules engine's active player and the
        // local faction binding: the turn advances via the End Phase button,
        // not any key. (Movement/fire/melee are gated on this same condition.)
        AppState::InGame => match game_state.as_deref() {
            Some(gs) => {
                let active = gs.0.active_player;
                let label = match active {
                    omdurman_types::Player::AngloEgyptian => "Anglo-Egyptian",
                    omdurman_types::Player::Dervish => "Dervish",
                };
                if factions.local_may_act(&net, active) {
                    Cow::Owned(format!("Your turn ({label}) -- act, then End Phase"))
                } else {
                    Cow::Owned(format!("{label}'s turn..."))
                }
            }
            None => Cow::Borrowed("Setting up..."),
        },
        AppState::Spectating => Cow::Borrowed("Reviewing game -- use the timeline"),
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

/// Resources consumed by the mode toolbar, bundled to keep
/// [`mode_toolbar`] under Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct ModeToolbarContext<'w> {
    pub editor_board: ResMut<'w, EditorBoard>,
    pub net: ResMut<'w, omdurman_net::NetState>,
    pub pending: ResMut<'w, PendingEdits>,
    pub zoc_overlay: ResMut<'w, crate::zoc::ZocOverlay>,
}

/// One action produced by the mode toolbar, applied after the egui closure so
/// the borrow of the state resources is released first.
enum ModeAction {
    /// Enter the game lane (networked play / lobby).
    Game,
    /// Enter (or re-open) the sandbox.
    Sandbox,
    /// Enter the editor.
    Editor,
    /// Voluntarily go to the lobby (from an active game in the game lane).
    Lobby,
    /// Switch the editor tab.
    Tab(EditorTab),
    /// Switch the editor board (scenario).
    Board(omdurman_types::Scenario),
}

/// Top-left mode picker: the three top-level modes (Lobby/Game, Sandbox,
/// Editor). While in the editor it also renders the horizontal tab bar and, for
/// board-specific tabs, a board (scenario) picker. Selection is UI-only; there
/// are no keyboard shortcuts for mode switching.
pub(crate) fn mode_toolbar(
    mut contexts: EguiContexts,
    mode: Res<State<AppMode>>,
    tab: Res<State<EditorTab>>,
    app_state: Res<State<AppState>>,
    mut next_mode: ResMut<NextState<AppMode>>,
    mut next_tab: ResMut<NextState<EditorTab>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut toolbar: ModeToolbarContext,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let cur_mode = **mode;
    let cur_tab = **tab;
    let in_lobby = *app_state.get() == AppState::Lobby;
    let mut action: Option<ModeAction> = None;

    // A full-width docked bar along the top edge -- a real tab bar rather than
    // a floating card. In the editor the tab row (Terrain/Overlay/...) is the
    // primary top row; the mode lane (Game/Sandbox/Editor) and board picker sit
    // beneath it. Registered before the side panels (see the system tuple in
    // `UiPlugin::build`), so those tuck under this bar.
    let mut __ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("mode_toolbar"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    egui::Panel::top("mode_toolbar")
        .frame(
            egui::Frame::new()
                .fill(crate::ui::panel_bg())
                .inner_margin(egui::Margin::symmetric(10, 6)),
        )
        .show(&mut __ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));

            // -- Editor tab bar (the actual tabs) -- topmost row --------------
            if cur_mode == AppMode::Editor {
                ui.horizontal_wrapped(|ui| {
                    for t in EditorTab::ALL {
                        if ui
                            .add(egui::Button::selectable(cur_tab == t, t.label()))
                            .clicked()
                            && cur_tab != t
                        {
                            action = Some(ModeAction::Tab(t));
                        }
                    }
                });
                ui.separator();
            }

            // -- Top-level mode row + board picker ---------------------------
            ui.horizontal(|ui| {
                // The game lane's label reflects the networking state:
                // "Lobby" while in the lobby, "Game" while playing.
                let game_label = if in_lobby { "Lobby" } else { "Game" };
                let game_selected = cur_mode == AppMode::Game;
                if ui
                    .add(egui::Button::selectable(game_selected, game_label))
                    .clicked()
                {
                    // From another mode, return to the game lane. Already
                    // in it and playing -> go to the lobby (voluntary).
                    action = Some(if game_selected && !in_lobby {
                        ModeAction::Lobby
                    } else {
                        ModeAction::Game
                    });
                }
                if ui
                    .add(egui::Button::selectable(
                        cur_mode == AppMode::Sandbox,
                        "Sandbox",
                    ))
                    .clicked()
                {
                    action = Some(ModeAction::Sandbox);
                }
                if ui
                    .add(egui::Button::selectable(
                        cur_mode == AppMode::Editor,
                        "Editor",
                    ))
                    .clicked()
                {
                    action = Some(ModeAction::Editor);
                }

                // Board picker rides on the mode row (editor + board-specific tab).
                if cur_mode == AppMode::Editor && cur_tab.is_board_specific() {
                    ui.separator();
                    ui.label("Board:");
                    for scenario in omdurman_types::Scenario::ALL {
                        let selected = toolbar.editor_board.0 == scenario;
                        if ui.add(egui::Button::selectable(selected, scenario.label())).clicked() && !selected
                        {
                            action = Some(ModeAction::Board(scenario));
                        }
                    }
                }

                // ZOC overlay toggle -- only in play modes (Game / Sandbox).
                if matches!(cur_mode, AppMode::Game | AppMode::Sandbox) && !in_lobby {
                    ui.separator();
                    let zoc_on = toolbar.zoc_overlay.visible;
                    if ui
                        .add(egui::Button::selectable(zoc_on, "ZOC"))
                        .on_hover_text("Toggle zone-of-control overlay (\u{00a7}5.41)")
                        .clicked()
                    {
                        toolbar.zoc_overlay.visible = !zoc_on;
                    }
                }
            });
        });

    match action {
        Some(ModeAction::Game) => {
            next_mode.set(AppMode::Game);
        }
        Some(ModeAction::Sandbox) => {
            next_mode.set(AppMode::Sandbox);
        }
        Some(ModeAction::Editor) => {
            next_mode.set(AppMode::Editor);
        }
        Some(ModeAction::Lobby) => {
            info!("entering lobby (voluntary)");
            next_app_state.set(AppState::Lobby);
            request_snapshot_if_guest(&mut toolbar.net, &mut toolbar.pending);
        }
        Some(ModeAction::Tab(t)) => {
            next_tab.set(t);
        }
        Some(ModeAction::Board(scenario)) => {
            toolbar.editor_board.0 = scenario;
        }
        None => {}
    }
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

/// Bundle of everything the game-control section needs, so `unit_overview_ui`
/// can host it without blowing past the system parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct GameControl<'w> {
    pub game_turn: Option<Res<'w, GameTurn>>,
    pub game_map: Option<Res<'w, omdurman_hexmap::GameMap>>,
    pub gate: crate::FactionGate<'w>,
    pub pending: Option<ResMut<'w, crate::PendingEdits>>,
}

/// Render the game-control section (turn/phase/day-night line, turn indicator,
/// End Phase button, one-shot scenario set-up) into an existing `ui`. Lives in
/// the right sidebar's "Game control" section; a no-op unless a game is active.
pub(crate) fn game_control_section(
    ui: &mut egui::Ui,
    state: &crate::GameStateResource,
    control: &mut GameControl,
) {
    let Some(turn) = control.game_turn.as_deref() else {
        return;
    };
    let Some(pending) = control.pending.as_deref_mut() else {
        return;
    };

    let day_night_str = match state.0.day_night {
        omdurman_types::DayNight::Day => "Day",
        omdurman_types::DayNight::Night => "Night",
    };
    let active_player_str = match state.0.active_player {
        omdurman_types::Player::AngloEgyptian => "A-E",
        omdurman_types::Player::Dervish => "Dervish",
    };

    // Whose turn it is, from the local player's point of view.
    let my_turn = control.gate.may_act(state.0.active_player);
    let is_host = control.gate.net.is_host;
    let in_setup = matches!(state.0.phase, omdurman_rules::Phase::Setup);

    ui.colored_label(
        egui::Color32::from_rgb(200, 200, 150),
        format!("Turn {}  {}  {}", **turn, state.0.phase.top_level_name(), day_night_str),
    );

    // Turn indicator -- only meaningful once play has begun. Setup is *not* a
    // turn: both players deploy concurrently, so a "your turn / waiting on"
    // indicator would be misleading. It's suppressed during Setup, where the
    // deployment status below tells each player what to do instead.
    if !in_setup {
        if my_turn {
            ui.colored_label(
                egui::Color32::from_rgb(230, 200, 110),
                format!("\u{25b6} Your turn ({active_player_str})"),
            );
        } else {
            ui.colored_label(
                egui::Color32::from_gray(150),
                format!("Waiting on {active_player_str}"),
            );
        }
    }

    ui.add_space(4.0);

    // -- Victory-point scoreboard (§9.14) --
    if !in_setup {
        let ae_vp = state.0.victory.total_for(omdurman_types::Player::AngloEgyptian).value();
        let dv_vp = state.0.victory.total_for(omdurman_types::Player::Dervish).value();
        let net = ae_vp - dv_vp;
        let net_color = if net > 0 {
            egui::Color32::from_rgb(120, 200, 120)
        } else if net < 0 {
            egui::Color32::from_rgb(200, 120, 120)
        } else {
            egui::Color32::from_gray(170)
        };
        ui.label(
            egui::RichText::new("Score")
                .strong()
                .color(egui::Color32::from_rgb(200, 200, 150)),
        );
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("A-E: {ae_vp}"))
                    .color(egui::Color32::from_rgb(120, 180, 220)),
            );
            ui.label(
                egui::RichText::new(format!("Dervish: {dv_vp}"))
                    .color(egui::Color32::from_rgb(220, 150, 100)),
            );
        });
        ui.colored_label(net_color, format!("Net: {net:+}"));
        ui.add_space(4.0);
    }

    // -- Night-effects reminder (§8) --
    if !in_setup && state.0.day_night == omdurman_types::DayNight::Night {
        ui.label(
            egui::RichText::new("Night rules (§8)")
                .strong()
                .color(egui::Color32::from_rgb(160, 180, 220)),
        );
        ui.label(
            egui::RichText::new(
                "\u{2022} A-E movement halved\n\
                 \u{2022} Fire ranges halved (min 1)\n\
                 \u{2022} No howitzer fire",
            )
            .small()
            .color(egui::Color32::from_gray(180)),
        );
        ui.add_space(4.0);
    }

    if in_setup {
        setup_control_section(ui, state, &control.gate, pending);
    } else if my_turn && ui.button("End Phase").clicked() {
        // Each player ends their *own* turn: the End Phase button is shown only
        // to whoever controls the active faction.
        pending.outgoing_broadcast.push(omdurman_net::NetMsg::Game(
            omdurman_net::GameEvent::Effect(omdurman_rules::effects::GameEffect::AdvancePhase),
        ));
    }

    // Scenario set-up is the host's prerogative, and only until it's done: once
    // the fixed units are on the board the button disappears (re-placing would
    // be redundant).
    if is_host && let Some(map) = control.game_map.as_deref() {
        let plan = crate::scenario_setup::build_setup_plan(state.0.scenario, map);
        let already_placed = plan
            .placements
            .iter()
            .all(|ev| setup_unit_already_on_board(ev, &state.0, map));
        if !plan.placements.is_empty()
            && !already_placed
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
}

/// The Setup-phase controls: per-faction deployed/target counts and the local
/// player's one-way "Ready" confirmation. Setup is concurrent -- both sides
/// deploy at once and each confirms independently; the engine auto-advances to
/// Movement once both are ready (§9.2/§9.3), so there's no explicit "advance"
/// click. An unbound sandbox (no faction binding) keeps a single "Begin battle"
/// that drives the same `AdvancePhase` for both sides.
fn setup_control_section(
    ui: &mut egui::Ui,
    state: &crate::GameStateResource,
    gate: &crate::FactionGate,
    pending: &mut crate::PendingEdits,
) {
    use omdurman_types::Player;

    ui.label(
        egui::RichText::new("Deployment -- place your forces, then Ready.")
            .size(12.0)
            .color(egui::Color32::from_gray(190)),
    );

    // Per-faction deployed/target + ready status, for both sides.
    for (player, label) in [(Player::AngloEgyptian, "A-E"), (Player::Dervish, "Dervish")] {
        let deployed = state.0.setup_deployed_count(player);
        let count = match state.0.setup_target(player) {
            Some(target) => format!("{deployed}/{target}"),
            None => format!("{deployed}"),
        };
        let ready = state.0.setup_ready(player);
        let mark = if ready { "  \u{2713} ready" } else { "" };
        let color = if ready {
            egui::Color32::from_rgb(230, 200, 110)
        } else {
            egui::Color32::from_gray(190)
        };
        ui.colored_label(color, format!("{label}: {count}{mark}"));
    }

    ui.add_space(2.0);

    let local = gate.factions.local(&gate.net);
    match local {
        // Bound player: confirm ready for *your* faction (one-way).
        Some(player) => {
            if state.0.setup_ready(player) {
                ui.colored_label(
                    egui::Color32::from_rgb(230, 200, 110),
                    "\u{2713} You are ready -- waiting for the other side.",
                );
            } else if state.0.setup_target_met(player) {
                if ui.button("Ready").clicked() {
                    pending.outgoing_broadcast.push(omdurman_net::NetMsg::Game(
                        omdurman_net::GameEvent::Effect(
                            omdurman_rules::effects::GameEffect::ConfirmSetupReady { player },
                        ),
                    ));
                }
            } else {
                let reason = "Deploy your forces before confirming ready.";
                ui.add_enabled(false, egui::Button::new("Ready"))
                    .on_disabled_hover_text(reason);
                ui.colored_label(egui::Color32::from_rgb(220, 180, 90), reason);
            }
        }
        // Unbound sandbox (single seat, no faction binding): one button starts
        // the battle for both sides once deployment is complete.
        None => match state.0.setup_complete() {
            Ok(()) => {
                if ui.button("Begin battle").clicked() {
                    pending.outgoing_broadcast.push(omdurman_net::NetMsg::Game(
                        omdurman_net::GameEvent::Effect(
                            omdurman_rules::effects::GameEffect::AdvancePhase,
                        ),
                    ));
                }
            }
            Err(reason) => {
                let reason = reason.to_string();
                ui.add_enabled(false, egui::Button::new("Begin battle"))
                    .on_disabled_hover_text(&reason);
                ui.colored_label(egui::Color32::from_rgb(220, 180, 90), &reason);
            }
        },
    }
}

/// Whether the unit a `PlaceUnit` set-up event would place is already on the
/// board, so the one-shot "Set up scenario" button can hide itself once used.
fn setup_unit_already_on_board(
    ev: &omdurman_net::GameEvent,
    state: &omdurman_rules::effects::GameState,
    map: &omdurman_hexmap::GameMap,
) -> bool {
    let omdurman_net::GameEvent::PlaceUnit { sprite, .. } = ev else {
        return true; // not a placement -- nothing to wait on
    };
    let Some(uid) = omdurman_rules::unit_id_for_section_pos(
        sprite.section_name,
        sprite.col as u8,
        sprite.row as u8,
    ) else {
        // No stable id (shouldn't happen for set-up units) -- fall back to "not
        // placed" so the button stays available.
        let _ = map;
        return false;
    };
    state.find_unit(uid).is_some()
}

/// Combat/event feed: the most recent military telegrams. The structured
/// `turn_events` / `observations` on `GameState` are surfaced elsewhere; this
/// panel now shows only the flavour telegrams.
// TODO(A-rules-4): render the event feed from `turn_events` + `observations`
// now that the human-readable `log` field has been removed.
pub(crate) fn game_log_panel(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    telegram_log: Option<Res<crate::telegram::TelegramLog>>,
) {
    let Some(_state) = game_state else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let has_telegrams = telegram_log
        .as_ref()
        .map_or(false, |t| !t.entries.is_empty());
    if !has_telegrams {
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
                    // Military telegrams — most recent two, newest first.
                    if let Some(t) = telegram_log.as_ref() {
                        if !t.entries.is_empty() {
                            for (turn, text) in t.entries.iter().rev().take(2) {
                                ui.colored_label(
                                    egui::Color32::from_rgb(180, 210, 180),
                                    format!("[Turn {}] {}", turn, text.lines().next().unwrap_or("")),
                                );
                            }
                        }
                    }
                });
        });
}

/// Victory modal: when the rules engine ends the game (`game_over`), show a
/// newspaper-styled panel with the final result and game stats. The panel is
/// anchored to center and uses period-appropriate styling (sepia tones, masthead,
/// headline, subhead, stats).
pub(crate) fn victory_modal(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    report: Option<Res<crate::newspaper::NewspaperReport>>,
) {
    let Some(state) = game_state else { return };
    if !state.0.game_over {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let paper_bg = egui::Color32::from_rgb(42, 36, 28);
    let paper_border = egui::Color32::from_rgb(180, 160, 110);
    let masthead_color = egui::Color32::from_rgb(200, 180, 120);
    let headline_color = egui::Color32::from_rgb(230, 210, 150);
    let subhead_color = egui::Color32::from_rgb(170, 155, 110);
    let dim_color = egui::Color32::from_rgb(140, 130, 100);

    egui::Area::new(egui::Id::new("victory_modal"))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(paper_bg)
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(32, 24))
                .stroke(egui::Stroke::new(2.0, paper_border))
                .show(ui, |ui| {
                    ui.set_max_width(520.0);
                    ui.vertical_centered(|ui| {
                        if let Some(r) = report.as_ref() {
                            // Masthead
                            ui.label(
                                egui::RichText::new(&r.masthead)
                                    .size(22.0)
                                    .strong()
                                    .color(masthead_color),
                            );
                            ui.label(
                                egui::RichText::new(&r.date_line)
                                    .size(11.0)
                                    .color(dim_color),
                            );
                        } else {
                            // Fallback before the report is populated.
                            ui.label(
                                egui::RichText::new("GAME OVER")
                                    .size(28.0)
                                    .strong()
                                    .color(headline_color),
                            );
                        }

                        ui.add_space(6.0);
                        // Horizontal rule
                        let rect = ui.available_rect_before_wrap();
                        let y = rect.min.y;
                        ui.painter().line_segment(
                            [
                                egui::pos2(rect.min.x + 8.0, y),
                                egui::pos2(rect.max.x - 8.0, y),
                            ],
                            egui::Stroke::new(1.0, paper_border),
                        );
                        ui.add_space(6.0);

                        // Headline
                        if let Some(r) = report.as_ref() {
                            ui.label(
                                egui::RichText::new(&r.headline)
                                    .size(20.0)
                                    .strong()
                                    .color(headline_color),
                            );
                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new(&r.subhead)
                                    .size(13.0)
                                    .italics()
                                    .color(subhead_color),
                            );
                        }

                        ui.add_space(6.0);
                        // Horizontal rule
                        let rect = ui.available_rect_before_wrap();
                        let y = rect.min.y;
                        ui.painter().line_segment(
                            [
                                egui::pos2(rect.min.x + 8.0, y),
                                egui::pos2(rect.max.x - 8.0, y),
                            ],
                            egui::Stroke::new(0.5, paper_border),
                        );
                        ui.add_space(8.0);

                        // Stats block
                        if let Some(r) = report.as_ref() {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Scenario: {}   |   Turns played: {}   |   Result: {}",
                                    r.scenario, r.turns_played, r.result_key,
                                ))
                                .size(11.0)
                                .color(dim_color),
                            );
                            ui.add_space(6.0);
                        }

                        // TODO(A-rules-4): render the closing summary from
                        // `turn_summaries` / `observations` now that the
                        // human-readable `log` field has been removed.
                    });
                });
        });
}
