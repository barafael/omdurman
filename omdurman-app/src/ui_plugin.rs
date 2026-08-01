use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_net::NetState;
use std::borrow::Cow;

use crate::{
    AppState, CursorPositions, GameTurn, HoveredHex, RoomId, browser, camera::RtsCamera, settings,
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
        use crate::{event_viewer, lobby, units};

        app.insert_resource(settings::LocalPlayerSettings::default())
            .insert_resource(settings::PlayerInfoMap::default())
            .insert_resource(units::UnitViewer::load_or_default())
            .insert_resource(browser::SpriteBrowser::new())
            .insert_resource(browser::SpriteMetaClipboard::default())
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
                    crate::scenario_setup::auto_trigger_scenario_setup
                        .run_if(bevy::prelude::in_state(crate::AppState::InGame)),
                ),
            )
            .add_systems(
                EguiPrimaryContextPass,
                    (
                        mode_toolbar_ui.run_if(not(bevy::prelude::in_state(crate::AppMode::Menu))),
                        cursor_overlay_ui.run_if(crate::map_view_active),
                        overlay_toggles_ui.run_if(crate::map_view_active),
                        // In-game HUD/overlays: only while actually in a game, so
                    // they don't show over the lobby.
                    (
                        game_log_panel,
                        victory_modal,
                        crate::fire::fire_combat_preview_ui,
                        crate::melee::melee_combat_preview_ui,
                        friendlies_transport_ui,
                        special_actions_ui,
                        chat_ui,
                        optional_rule_setup_ui,
                    )
                        .run_if(in_state(AppState::InGame)),
                    units::unit_grids_ui,
                    units::unit_grid_labels,
                    browser::sprite_meta_editor_ui,
                    event_viewer::event_viewer_ui,
                    lobby::lobby_ui.run_if(in_state(AppState::Lobby)),
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

    // -- Inter: sans-serif UI font -------------------------------------------
    // Medium (500) is the primary weight for all UI text.
    ctx.add_font(FontInsert::new(
        "Inter-Medium",
        egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter-Medium.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Proportional,
            priority: FontPriority::Highest,
        }],
    ));

    // -- Merriweather: serif font for the splash screen ----------------------
    // Registered under "Garamond" family name so every existing reference
    // (splash screen, quoted titles) picks it up without code changes.
    ctx.add_font(FontInsert::new(
        "Merriweather-Regular",
        egui::FontData::from_static(include_bytes!("../../assets/fonts/Merriweather-Regular.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name("Garamond".into()),
            priority: FontPriority::Highest,
        }],
    ));
    // A real italic face, registered as its own family.  Italic text (the
    // splash quote, book titles) selects this family rather than egui's
    // synthetic italic -- epaint fakes italics by shearing the upright glyphs
    // without fixing advances, which left uneven gaps.  A genuine italic has
    // correct metrics.
    ctx.add_font(FontInsert::new(
        "Merriweather-Italic",
        egui::FontData::from_static(include_bytes!("../../assets/fonts/Merriweather-Italic.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name("GaramondItalic".into()),
            priority: FontPriority::Highest,
        }],
    ));
    ctx.add_font(FontInsert::new(
        "Merriweather-Bold",
        egui::FontData::from_static(include_bytes!("../../assets/fonts/Merriweather-Bold.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name("Garamond".into()),
            priority: FontPriority::Lowest,
        }],
    ));

    // -- Noto Sans Symbols 2: icon fallback ----------------------------------
    // Covers miscellaneous icons (arrows, checkmarks, warning signs, media
    // controls, emoji) that the text fonts lack.  Registered at lowest priority
    // so it only kicks in for missing glyphs.
    ctx.add_font(FontInsert::new(
        "NotoSansSymbols2",
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/NotoSansSymbols2-Regular.ttf"
        )),
        vec![
            InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: FontPriority::Lowest,
            },
            InsertFontFamily {
                family: egui::FontFamily::Monospace,
                priority: FontPriority::Lowest,
            },
        ],
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
        AppState::Lobby => Cow::Owned(format!(
            "Lobby -- choose your faction (share: ?room={})",
            room.as_str()
        )),
        // In game, the phase banner provides full turn/phase/sequence info.
        // The status line is a minimal complement showing server info.
        AppState::InGame => Cow::Owned(format!(
            "Room: {}  |  {}",
            room.as_str(),
            match game_state.as_deref() {
                Some(gs) => {
                    let active = gs.0.active_player;
                    let label = match active {
                        omdurman_types::Player::AngloEgyptian => "Anglo-Egyptian",
                        omdurman_types::Player::Dervish => "Dervish",
                    };
                    if factions.local_may_act(&net, active) {
                        format!("Your turn ({label})")
                    } else {
                        format!("{label}'s turn — waiting...")
                    }
                }
                None => "Setting up...".into(),
            },
        )),
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

/// Floating overlay-toggle buttons (top-right). Currently just the ZOC ring
/// toggle so the player can show/hide enemy zones of control on demand.
pub(crate) fn overlay_toggles_ui(
    mut contexts: EguiContexts,
    mut zoc: ResMut<crate::zoc::ZocOverlay>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::Area::new(egui::Id::new("overlay_toggles"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 48.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let btn = egui::Button::new(
                egui::RichText::new("ZOC")
                    .size(12.0)
                    .monospace()
                    .color(if zoc.visible {
                        egui::Color32::from_rgb(0xFF, 0xDD, 0x44)
                    } else {
                        egui::Color32::from_rgb(0xAA, 0x99, 0x66)
                    }),
            )
            .fill(if zoc.visible {
                egui::Color32::from_rgba_unmultiplied(80, 70, 30, 200)
            } else {
                egui::Color32::from_rgba_unmultiplied(40, 36, 28, 180)
            });
            if ui.add(btn).on_hover_text("Toggle enemy ZOC ring overlay (§5.41)").clicked() {
                zoc.visible = !zoc.visible;
            }
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

/// Bundle of everything the game-control section needs, so `unit_overview_ui`
/// can host it without blowing past the system parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct GameControl<'w> {
    pub game_turn: Option<Res<'w, GameTurn>>,
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
        use omdurman_rules::VpSource;
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

        // VP breakdown by source category (§9.14). Collapsible to keep the
        // sidebar compact; defaults to collapsed.
        ui.collapsing("Breakdown", |ui| {
            ui.style_mut().override_font_id =
                Some(egui::FontId::proportional(11.0));
            // Tally per VpSource for each player. VpSource is not Hash, so we
            // iterate the known variants and sum directly from the event log.
            let sources = [
                VpSource::MahdisTomb,
                VpSource::IsaZachneihEliminated,
                VpSource::KhalifaEliminated,
                VpSource::DervishUnitEliminated,
                VpSource::BritishLeaderEliminated,
                VpSource::BritishGunboatSunk,
                VpSource::FriendliesEastBankEliminated,
                VpSource::FriendliesWestBankEliminated,
                VpSource::AngloEgyptianLandUnitEliminated,
            ];
            let tally = |src: VpSource| -> i32 {
                state
                    .0
                    .victory
                    .events
                    .iter()
                    .filter(|e| e.source == src)
                    .map(|e| e.source.points().value())
                    .sum()
            };
            let ae_sources = [
                VpSource::MahdisTomb,
                VpSource::IsaZachneihEliminated,
                VpSource::KhalifaEliminated,
                VpSource::DervishUnitEliminated,
            ];
            let dv_sources = [
                VpSource::BritishLeaderEliminated,
                VpSource::BritishGunboatSunk,
                VpSource::FriendliesEastBankEliminated,
                VpSource::FriendliesWestBankEliminated,
                VpSource::AngloEgyptianLandUnitEliminated,
            ];
            let has_ae = ae_sources.iter().any(|s| tally(*s) > 0);
            let has_dv = dv_sources.iter().any(|s| tally(*s) > 0);
            if has_ae {
                ui.colored_label(
                    egui::Color32::from_rgb(120, 180, 220),
                    "Anglo-Egyptian:",
                );
                for src in &ae_sources {
                    let pts = tally(*src);
                    if pts > 0 {
                        ui.label(format!("  {src}: {pts}"));
                    }
                }
            }
            if has_dv {
                ui.colored_label(
                    egui::Color32::from_rgb(220, 150, 100),
                    "Dervish:",
                );
                for src in &dv_sources {
                    let pts = tally(*src);
                    if pts > 0 {
                        ui.label(format!("  {src}: {pts}"));
                    }
                }
            }
            if !has_ae && !has_dv {
                ui.colored_label(egui::Color32::from_gray(150), "No scoring yet.");
            }
            let _ = sources; // kept for completeness/debugging

            // Last 5 VP events (most recent last).
            let recent: Vec<&omdurman_rules::VpEvent> =
                state.0.victory.events.iter().rev().take(5).collect();
            if !recent.is_empty() {
                ui.add_space(2.0);
                ui.colored_label(egui::Color32::from_gray(170), "Recent:");
                for ev in recent.iter().rev() {
                    let who = ev.source.who_scores();
                    let pts = ev.source.points().value();
                    let color = match who {
                        omdurman_types::Player::AngloEgyptian => egui::Color32::from_rgb(120, 180, 220),
                        omdurman_types::Player::Dervish => egui::Color32::from_rgb(220, 150, 100),
                    };
                    ui.colored_label(
                        color,
                        format!("  T{}: {} (+{pts})", ev.turn.value(), ev.source),
                    );
                }
            }
        });

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

}

/// The Setup-phase controls: per-faction deployed/target counts and the local
/// player's one-way "Ready" confirmation. Setup is concurrent -- both sides
/// deploy at once and each confirms independently; the engine auto-advances to
/// Movement once both are ready (§9.2/§9.3), so there's no explicit "advance"
/// click. An unbound session (no faction binding) keeps a single "Begin battle"
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
        // Unbound session (single seat, no faction binding): one button starts
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
        .is_some_and(|t| !t.entries.is_empty());
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
                    if let Some(t) = telegram_log.as_ref()
                        && !t.entries.is_empty() {
                            for (turn, text) in t.entries.iter().rev().take(2) {
                                ui.colored_label(
                                    egui::Color32::from_rgb(180, 210, 180),
                                    format!("[Turn {}] {}", turn, text.lines().next().unwrap_or("")),
                                );
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

                        // LLM-generated body paragraphs.
                        let body_color = egui::Color32::from_rgb(190, 180, 150);
                        if let Some(r) = report.as_ref()
                            && !r.paragraphs.is_empty() {
                                for para in &r.paragraphs {
                                    ui.label(
                                        egui::RichText::new(para)
                                            .size(12.0)
                                            .color(body_color),
                                    );
                                    ui.add_space(4.0);
                                }
                            }
                    });
                });
        });
}

// -- Friendlies transport UI (§5.21) ----------------------------------------

/// Floating panel for the §5.21 "Friendlies" transport actions: Load, Cross, Disembark.
/// Appears during Movement phase when transport conditions are met.
pub(crate) fn friendlies_transport_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    state: Res<crate::picker::PickerState>,
    placed_units: Query<(Entity, &crate::picker::PlacedUnit)>,
    mut pending: ResMut<crate::PendingEdits>,
    factions: Res<crate::PlayerFactions>,
    net: Res<NetState>,
) {
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, omdurman_rules::Phase::Movement) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let local = factions.local(&net);
    let is_host = net.is_host;

    // Determine transport state and eligible action.
    let transport = gs.0.friendlies_transport;
    let action_label = match &transport {
        None => {
            // Check if selected unit can load: must be a non-disrupted unit
            // adjacent to a gunboat with no existing transport.
            if let Some((uid, _)) = crate::picker::selected_unit_id(&state, &placed_units)
                && let Some(unit) = gs.0.find_unit(uid)
                && !unit.state.disrupted
                && !unit.state.constructing_zariba
                && !unit.state.demolishing
                && is_friendlies_unit(&unit.profile.identity)
            {
                // Check adjacency to any Anglo-Egyptian gunboat.
                let has_adjacent_gunboat = unit.position.neighbors().iter().any(|n| {
                    gs.0.units.iter().any(|u| {
                        u.position == *n
                            && matches!(
                                u.profile.identity,
                                omdurman_rules::UnitIdentity::AngloEgyptianGunboat(_)
                            )
                            && u.profile.identity.owner() == unit.profile.identity.owner()
                    })
                });
                if has_adjacent_gunboat && gs.0.isa_zachneih_eliminated {
                    Some("Load onto Gunboat")
                } else {
                    None
                }
            } else {
                None
            }
        }
        Some(omdurman_rules::TransportState::Loaded { unit, gunboat }) => {
            // Show "Cross Nile" for the gunboat's owner.
            let _ = (unit, gunboat);
            if local.is_some() || is_host {
                Some("Cross Nile (§5.21)")
            } else {
                None
            }
        }
        Some(omdurman_rules::TransportState::Crossing { unit, gunboat, .. }) => {
            let _ = (unit, gunboat);
            Some("Disembark (§5.21)")
        }
        Some(omdurman_rules::TransportState::ReadyToDisembark { .. }) => {
            Some("Disembark (§5.21)")
        }
    };

    let Some(label) = action_label else { return };

    egui::Area::new(egui::Id::new("friendlies_transport"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 80.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(40, 50, 30, 210))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.style_mut().override_font_id =
                        Some(egui::FontId::proportional(13.0));
                    ui.colored_label(
                        egui::Color32::from_rgb(180, 220, 180),
                        "\u{1f6a2} Friendlies Transport",
                    );
                        if ui.button(label).clicked() {
                        let effect = match transport {
                            None => {
                                // Build Load action from selected unit + adjacent gunboat.
                                if let Some((uid, _)) = crate::picker::selected_unit_id(&state, &placed_units)
                                    && let Some(unit) = gs.0.find_unit(uid)
                                {
                                    unit.position.neighbors().iter().find_map(|n| {
                                        gs.0.units.iter().find(|u| {
                                            u.position == *n
                                                && matches!(
                                                    u.profile.identity,
                                                    omdurman_rules::UnitIdentity::AngloEgyptianGunboat(_)
                                                )
                                                && u.profile.identity.owner() == unit.profile.identity.owner()
                                        }).map(|u| omdurman_rules::effects::GameEffect::FriendliesTransport(
                                            omdurman_rules::FriendliesAction::Load { unit: uid, gunboat: u.id },
                                        ))
                                    })
                                } else {
                                    None
                                }
                            }
                            Some(omdurman_rules::TransportState::Loaded { unit, gunboat }) => {
                                Some(omdurman_rules::effects::GameEffect::FriendliesTransport(
                                    omdurman_rules::FriendliesAction::Cross {
                                        unit,
                                        gunboat,
                                        to: gunboat_pos(&gs.0, gunboat).unwrap_or(unit_pos(&gs.0, unit).unwrap_or(omdurman_types::HexCoord::new(0, 0))),
                                    },
                                ))
                            }
                            Some(omdurman_rules::TransportState::Crossing { unit, gunboat, .. }) => {
                                Some(omdurman_rules::effects::GameEffect::FriendliesTransport(
                                    omdurman_rules::FriendliesAction::Disembark { unit, gunboat },
                                ))
                            }
                            Some(omdurman_rules::TransportState::ReadyToDisembark { unit, gunboat }) => {
                                Some(omdurman_rules::effects::GameEffect::FriendliesTransport(
                                    omdurman_rules::FriendliesAction::Disembark { unit, gunboat },
                                ))
                            }
                        };
                        if let Some(effect) = effect {
                            pending.outgoing_broadcast.push(omdurman_net::NetMsg::Game(
                                omdurman_net::GameEvent::Effect(effect),
                            ));
                        }
                    }
                });
        });
}

fn is_friendlies_unit(identity: &omdurman_rules::UnitIdentity) -> bool {
    matches!(
        identity,
        omdurman_rules::UnitIdentity::AngloEgyptianInfantry { brigade, .. }
            if matches!(brigade.nationality, omdurman_types::BrigadeNationality::Friendlies)
    )
}

fn gunboat_pos(gs: &omdurman_rules::effects::GameState, id: omdurman_rules::UnitId) -> Option<omdurman_types::HexCoord> {
    gs.find_unit(id).map(|u| u.position)
}

fn unit_pos(gs: &omdurman_rules::effects::GameState, id: omdurman_rules::UnitId) -> Option<omdurman_types::HexCoord> {
    gs.find_unit(id).map(|u| u.position)
}

// -- Special actions UI: Zariba construction + Royal Engineers demolition ------

/// Floating panel for §5.3 Zariba construction and §6.53 Demolition actions.
/// Appears during Movement phase when a relevant unit is selected.
/// Transient selection state for demolition target picking (§6.53).
#[derive(Resource, Default)]
pub(crate) struct DemolitionSelection {
    pub target: Option<omdurman_rules::DemolitionTarget>,
}

/// Placement mode for optional-rule river mines/chains (§10.11, §10.21).
/// Active during Setup phase for the Dervish player.
#[derive(Resource, Default)]
pub(crate) struct OptionalRulePlacement {
    /// `None` = idle. `Some(coord)` = a pending mine placement at that hex
    /// (emitted on next frame via the placement system).
    pub pending_mine: Option<omdurman_types::HexCoord>,
    /// Chain hexes being built up during placement (max 4).
    pub chain_hexes: Vec<omdurman_types::HexCoord>,
    /// Whether we are currently in chain-placement mode.
    pub placing_chain: bool,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn special_actions_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    state: Res<crate::picker::PickerState>,
    placed_units: Query<(Entity, &crate::picker::PlacedUnit)>,
    mut pending: ResMut<crate::PendingEdits>,
    factions: Res<crate::PlayerFactions>,
    net: Res<NetState>,
    editor: Res<crate::editor::HexEditor>,
    mut demolition_sel: ResMut<DemolitionSelection>,
    game_map: Res<omdurman_hexmap::GameMap>,
) {
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, omdurman_rules::Phase::Movement) {
        return;
    }
    let Some((uid, _)) = crate::picker::selected_unit_id(&state, &placed_units) else {
        return;
    };
    let Some(unit) = gs.0.find_unit(uid) else { return };
    if unit.state.disrupted {
        return;
    }

    // Only the active player may take special actions.
    let local = factions.local(&net);
    if local.is_none() && !net.is_host {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Zariba construction (§5.3): engineers or adjacent units.
    let can_construct = matches!(
        unit.profile.identity,
        omdurman_rules::UnitIdentity::RoyalEngineers
    );
    // Demolition (§6.53): Royal Engineers adjacent to a zariba hexside.
    let can_demolish = matches!(
        unit.profile.identity,
        omdurman_rules::UnitIdentity::RoyalEngineers
    ) && !unit.state.constructing_zariba;

    if !can_construct && !can_demolish {
        return;
    }

    let has_construct_button = can_construct && !unit.state.constructing_zariba && !unit.state.demolishing;
    let has_demolish_button = can_demolish && gs.0.can_demolition(uid).is_ok();

    // Discover adjacent demolition targets (§6.53)
    let unit_hex = unit.position;
    let neighbors = unit_hex.neighbors();
    let adjacent_forts: Vec<omdurman_rules::UnitId> = gs.0.units.iter()
        .filter(|u| neighbors.contains(&u.position) && matches!(u.profile.kind, omdurman_types::UnitKind::Fort { .. }))
        .map(|u| u.id)
        .collect();
    let adjacent_walls: Vec<omdurman_types::HexsideRef> = neighbors.iter()
        .filter_map(|&n| {
            let edge = omdurman_types::HexsideRef::new(unit_hex, n);
            game_map.hexsides.get(&edge).and_then(|kind| {
                if *kind == omdurman_types::HexsideKind::Wall { Some(edge) } else { None }
            })
        })
        .collect();
    let has_targets = !adjacent_forts.is_empty() || !adjacent_walls.is_empty();
    let has_demolish_button_full = has_demolish_button && has_targets;

    if !has_construct_button && !has_demolish_button_full {
        // Clear stale demolition selection when no eligible targets
        if !has_targets {
            demolition_sel.target = None;
        }
        return;
    }

    egui::Area::new(egui::Id::new("special_actions"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 110.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(50, 40, 30, 210))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.style_mut().override_font_id =
                        Some(egui::FontId::proportional(13.0));

                    if has_construct_button {
                        ui.colored_label(
                            egui::Color32::from_rgb(200, 180, 140),
                            "Construct Zariba (§5.3)",
                        );
                        ui.label(
                            egui::RichText::new(
                                "Place a zariba hexside adjacent to the unit's hex."
                            )
                            .size(11.0)
                            .color(egui::Color32::from_rgb(160, 150, 130)),
                        );
                        if let Some(hexside) = editor.selected_hexside {
                            if ui.button("Begin Construction").clicked() {
                                pending.outgoing_broadcast.push(omdurman_net::NetMsg::Game(
                                    omdurman_net::GameEvent::Effect(
                                        omdurman_rules::effects::GameEffect::ConstructZariba {
                                            unit_ids: vec![uid],
                                            hexside,
                                        },
                                    ),
                                ));
                            }
                        } else {
                            ui.label(
                                egui::RichText::new("Select a hexside in the Hexside editor tab\nbefore constructing.")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(180, 140, 100)),
                            );
                        }
                    }

                    if has_demolish_button_full {
                        if has_construct_button {
                            ui.add_space(4.0);
                        }
                        ui.colored_label(
                            egui::Color32::from_rgb(200, 160, 120),
                            "Royal Engineers Demolition (§6.53)",
                        );
                        ui.label(
                            egui::RichText::new(
                                "Destroy adjacent fort or wall. Resolved at end of turn."
                            )
                            .size(11.0)
                            .color(egui::Color32::from_rgb(160, 150, 130)),
                        );
                        ui.add_space(2.0);

                        // Fort targets
                        for &fort_id in &adjacent_forts {
                            let label = format!("Fort at {}", {
                                if let Some(f) = gs.0.find_unit(fort_id) {
                                    format!("({}, {})", f.position.q, f.position.r)
                                } else {
                                    "?".to_string()
                                }
                            });
                            let selected = matches!(demolition_sel.target, Some(omdurman_rules::DemolitionTarget::Fort(id)) if id == fort_id);
                            if ui.selectable_label(selected, label).clicked() {
                                demolition_sel.target = Some(omdurman_rules::DemolitionTarget::Fort(fort_id));
                            }
                        }

                        // Wall targets
                        for &edge in &adjacent_walls {
                            let label = format!("Wall ({},{})–({},{})",
                                edge.a.q, edge.a.r, edge.b.q, edge.b.r);
                            let selected = matches!(demolition_sel.target, Some(omdurman_rules::DemolitionTarget::WallHexside(e)) if e == edge);
                            if ui.selectable_label(selected, label).clicked() {
                                demolition_sel.target = Some(omdurman_rules::DemolitionTarget::WallHexside(edge));
                            }
                        }

                        ui.add_space(4.0);
                        if demolition_sel.target.is_some() {
                            if ui.button("Commit to Demolition").clicked()
                                && let Some(target) = demolition_sel.target {
                                    pending.outgoing_broadcast.push(omdurman_net::NetMsg::Game(
                                        omdurman_net::GameEvent::Effect(
                                            omdurman_rules::effects::GameEffect::Demolition {
                                                unit_id: uid,
                                                target,
                                            },
                                        ),
                                    ));
                                    demolition_sel.target = None;
                                }
                        } else {
                            ui.label(
                                egui::RichText::new("Select a target above.")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(180, 140, 100)),
                            );
                        }
                    }
                });
        });
}

// -- Mode toolbar (cross-cutting) -------------------------------------------

/// Top toolbar showing the current mode and providing mode-switching buttons.
/// Visible in all non-Menu states.
pub(crate) fn mode_toolbar_ui(
    mut contexts: EguiContexts,
    mode: Res<State<crate::AppMode>>,
    mut next_mode: ResMut<NextState<crate::AppMode>>,
    game_state: Option<Res<crate::GameStateResource>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::Area::new(egui::Id::new("mode_toolbar"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(40, 40, 50, 220))
                .corner_radius(0.0)
                .inner_margin(egui::Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Mode label
                        ui.label(
                            egui::RichText::new(match **mode {
                                crate::AppMode::Menu => "Menu",
                                crate::AppMode::Lobby => "Lobby",
                                crate::AppMode::Game => "Game",
                                crate::AppMode::Editor => "Editor",
                            })
                            .strong()
                            .size(13.0),
                        );
                        ui.separator();

                        // Mode switching buttons
                        if ui.button("Menu").clicked() {
                            next_mode.set(crate::AppMode::Menu);
                        }
                        if **mode != crate::AppMode::Game && ui.button("Game").clicked() {
                            next_mode.set(crate::AppMode::Game);
                        }
                        if **mode != crate::AppMode::Editor && ui.button("Editor").clicked() {
                            next_mode.set(crate::AppMode::Editor);
                        }

                        // Phase/turn info when in Game mode
                        if let Some(gs) = game_state.as_ref() {
                            let ui_state = crate::ui_phase_state::UiPhaseState::derive(&gs.0);
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("Turn {}", gs.0.current_turn.value()))
                                    .size(13.0),
                            );
                            ui.label(
                                egui::RichText::new(ui_state.phase_label())
                                    .size(13.0),
                            );
                        }
                    });
                });
        });
}

// -- Chat panel (cross-cutting) ---------------------------------------------

/// Collapsible in-game chat panel. Shows messages and a text-input field.
/// Messages are broadcast as `Ephemeral::ChatMessage` (not recorded in the event log).
pub(crate) fn chat_ui(
    mut contexts: EguiContexts,
    mut log: ResMut<crate::net_plugin::ChatLog>,
    settings: Res<crate::settings::LocalPlayerSettings>,
    mut pending: ResMut<crate::PendingEdits>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::Window::new("Chat")
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(0.0, 0.0))
        .default_width(240.0)
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Scrollable message area
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for (sender, msg) in &log.messages {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{}:", sender))
                                        .strong()
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(180, 180, 200)),
                                );
                                ui.label(
                                    egui::RichText::new(msg.as_str())
                                        .size(11.0),
                                );
                            });
                        }
                    });

                ui.add_space(4.0);

                // Input field + send button
                let mut input = String::new();
                let mut send = false;
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut input)
                        .hint_text("Type a message…")
                        .desired_width(f32::INFINITY),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    send = true;
                }
                if ui.button("Send").clicked() {
                    send = true;
                }

                if send && !input.trim().is_empty() {
                    let msg = input.trim().to_string();
                    let name = settings.name.clone();
                    log.push(name.clone(), msg.clone());
                    pending.outgoing_broadcast.push(omdurman_net::NetMsg::Ephemeral(
                        omdurman_net::Ephemeral::ChatMessage { text: msg },
                    ));
                }
            });
        });
}

// -- Optional-rule setup UI (§10.11, §10.21) --------------------------------

/// Setup-phase panel for Dervish river mine / chain placement.
/// Hex-click handling is done by `handle_optional_rule_click` in the picker
/// pipeline.
pub(crate) fn optional_rule_setup_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    gate: crate::FactionGate,
    mut placement: ResMut<OptionalRulePlacement>,
    mut pending: ResMut<crate::PendingEdits>,
) {
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, omdurman_rules::Phase::Setup) {
        placement.pending_mine = None;
        placement.placing_chain = false;
        placement.chain_hexes.clear();
        return;
    }

    // Only the Dervish player (or unbound host) can place mines/chains.
    let local = gate.factions.local(&gate.net);
    let is_dervish = match local {
        Some(p) => p == omdurman_types::Player::Dervish,
        None => gate.net.is_host,
    };
    if !is_dervish {
        return;
    }

    let has_mines = gs.0.optional_rules.contains(&omdurman_rules::OptionalRule::RiverMines);
    let has_chain = gs.0.optional_rules.contains(&omdurman_rules::OptionalRule::RiverChain);
    if !has_mines && !has_chain {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::Area::new(egui::Id::new("optional_rule_setup"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 380.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(40, 30, 40, 210))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.style_mut().override_font_id =
                        Some(egui::FontId::proportional(12.0));

                    if has_mines {
                        ui.colored_label(
                            egui::Color32::from_rgb(200, 170, 150),
                            "River Mines (§10.11)",
                        );
                        let mines_placed = gs.0.mines.len();
                        ui.label(
                            egui::RichText::new(format!("Placed: {mines_placed}/2"))
                                .size(11.0)
                                .color(egui::Color32::from_gray(180)),
                        );
                        if mines_placed < 2 {
                            if placement.pending_mine.is_some() {
                                if ui.button("Click a Nile hex to place").clicked() {
                                    placement.pending_mine = None;
                                }
                            } else if ui.button("Place River Mine").clicked() {
                                // Dummy coord; overwritten on hex click.
                                placement.pending_mine =
                                    Some(omdurman_types::HexCoord::new(99, 99));
                            }
                        }
                    }

                    if has_chain {
                        if has_mines {
                            ui.add_space(4.0);
                        }
                        ui.colored_label(
                            egui::Color32::from_rgb(180, 180, 150),
                            "River Chain (§10.21)",
                        );
                        let chain_placed = gs.0.chain.as_ref().map(|c| c.hexes.len()).unwrap_or(0);
                        let building = placement.placing_chain;
                        if building {
                            ui.label(
                                egui::RichText::new(
                                    format!("Selecting hex {}/4...",
                                        placement.chain_hexes.len() + 1)
                                )
                                .size(11.0)
                                .color(egui::Color32::from_gray(180)),
                            );
                            if ui.button("Finish Chain").clicked()
                                && !placement.chain_hexes.is_empty()
                            {
                                pending.outgoing_broadcast.push(omdurman_net::NetMsg::Game(
                                    omdurman_net::GameEvent::Effect(
                                        omdurman_rules::effects::GameEffect::PlaceChain {
                                            hexes: std::mem::take(&mut placement.chain_hexes),
                                        },
                                    ),
                                ));
                                placement.placing_chain = false;
                            }
                            if ui.button("Cancel").clicked() {
                                placement.chain_hexes.clear();
                                placement.placing_chain = false;
                            }
                        } else if chain_placed == 0 {
                            if ui.button("Place River Chain").clicked() {
                                placement.placing_chain = true;
                            }
                        } else {
                            ui.label(
                                egui::RichText::new("Chain placed")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(120, 200, 120)),
                            );
                        }
                    }
                });
        });
}
