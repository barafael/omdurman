use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use bevy_matchbox::prelude::PeerId;
use omdurman_net::NetState;
use std::borrow::Cow;

use crate::peers::{LocalPeer, Peers};
use crate::{AppState, GameTurn, HoveredHex, RoomId, camera::RtsCamera, settings};

// -- Map-input gating (shared with the map editor) ---------------------------
//
// The pointer predicate, its per-frame snapshot, the [`MapPointerInputSet`] /
// [`PanelUiSet`] sets, and the `ui_wants_pointer` run condition are the app's
// design, generalized into `omdurman-board-ui::panels` so the editor uses the
// identical gating. The names are re-exported here so `crate::ui_plugin::*`
// paths keep working.

pub use omdurman_board_ui::panels::{
    EguiPointerOverUi, MapPointerInputSet, PanelUiSet, sync_egui_pointer_over_ui, ui_wants_pointer,
};
// (Directly referenced only by the ui_gating tests in non-inline paths.)
#[cfg_attr(not(test), allow(unused_imports))]
pub use omdurman_board_ui::panels::egui_wants_pointer_input;

/// Schedule set grouping the left-rail panel systems (unit picker, unit
/// overview) so consumers (the game log) can order against the rail without
/// naming a system that is registered more than once (the overview runs in
/// both the InGame and Spectating states, and Bevy refuses to order against
/// an ambiguous system type).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LeftRailSet;

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
        use crate::{event_viewer, lobby};

        app.insert_resource(settings::LocalPlayerSettings::default())
            .insert_resource(event_viewer::EventViewerState::default())
            .insert_resource(FontsInstalled::default())
            .insert_resource(EguiPointerOverUi::default())
            .init_resource::<crate::ScreenLayout>()
            // Whole-set gate: map-interaction systems in this set are skipped
            // whenever the pointer is over UI (see `MapPointerInputSet`).
            .configure_sets(Update, MapPointerInputSet.run_if(not(ui_wants_pointer)))
            // Clear the chrome layout ledger before any egui surface draws.
            .add_systems(
                First,
                (
                    sync_egui_pointer_over_ui,
                    crate::layout::reset_screen_layout,
                ),
            )
            .add_systems(
                Startup,
                (setup_ui, configure_egui_touch, maximize_primary_window),
            )
            .add_systems(
                Update,
                (
                    // Retries until the egui context exists (a camera is up),
                    // then installs fonts exactly once.
                    setup_egui_fonts,
                    update_status_text,
                    update_hex_coord_display,
                    crate::scenario_setup::auto_trigger_scenario_setup
                        .run_if(bevy::prelude::in_state(crate::AppState::InGame)),
                ),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    mode_toolbar_ui.run_if(not(bevy::prelude::in_state(crate::AppMode::Menu))),
                    cursor_overlay_ui.run_if(crate::map_view_active),
                    // (ZOC/LOS toggles live in the left rail's Overlays
                    // section -- see overview::unit_overview_ui.)
                    // In-game HUD/overlays: only while actually in a game, so
                    // they don't show over the lobby. The top-center cards
                    // stack below the phase banner (see `stacked_card`), so
                    // they chain in that order and must run after it; the
                    // game log reads the left-rail inset, so it runs after
                    // the rail panels.
                    (
                        // Fire preview in a fire sub-phase (offensive *or*
                        // defensive, §6.41/§6.42) ...
                        crate::fire::fire_combat_preview_ui.run_if(
                            crate::ui_phase_state::in_defensive_fire_phase
                                .or_else(crate::ui_phase_state::in_offensive_fire_phase),
                        ),
                        // §6.63 artillery breach: fire-phase sibling of the
                        // Movement-phase special-actions card — one battery,
                        // the wall hexsides it can reach.
                        artillery_breach_ui.run_if(
                            crate::ui_phase_state::in_defensive_fire_phase
                                .or_else(crate::ui_phase_state::in_offensive_fire_phase),
                        ),
                        // ... melee preview in Melee (§7) ...
                        crate::melee::melee_combat_preview_ui
                            .run_if(crate::ui_phase_state::in_melee_phase), // Movement-phase action panels (§5.21 transport,
                        // §5.3 zariba / §6.53 demolition). The mirror state
                        // machine (see `ui_phase_state`) gates the phase;
                        // selection/eligibility stays inside.
                        (friendlies_transport_ui, special_actions_ui)
                            .chain()
                            .run_if(crate::ui_phase_state::in_movement_phase),
                        crate::fok_panel::gordon_badge_ui,
                    )
                        .chain()
                        .after(crate::phase_banner::phase_banner_ui)
                        .run_if(in_state(AppState::InGame)),
                    victory_modal.run_if(in_state(AppState::InGame)),
                    game_log_panel
                        .run_if(in_state(AppState::InGame))
                        .after(LeftRailSet),
                    // (Not a run condition: its not-in-Setup branch clears the
                    // staged mine/chain placement on the transition out of
                    // §10 setup -- cleanup a `run_if` would skip.)
                    optional_rule_setup_ui
                        .run_if(in_state(AppState::InGame))
                        .after(crate::charts::chart_sheet_ui),
                    event_viewer::event_viewer_ui
                        .run_if(in_state(AppState::InGame).or_else(in_state(AppState::Spectating))),
                    event_viewer::event_viewer_toggle
                        .run_if(in_state(AppState::InGame).or_else(in_state(AppState::Spectating))),
                    lobby::lobby_ui
                        .in_set(PanelUiSet)
                        .run_if(in_state(AppState::Lobby)),
                ),
            );
    }
}

pub(crate) fn maximize_primary_window(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.set_maximized(true);
}

/// Set once [`setup_egui_fonts`] has installed the fonts (it retries until
/// the egui context exists, then must not re-insert every frame).
#[derive(Resource, Default)]
pub(crate) struct FontsInstalled(bool);

pub(crate) fn setup_egui_fonts(mut contexts: EguiContexts, mut installed: ResMut<FontsInstalled>) {
    if installed.0 {
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
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/Merriweather-Regular.ttf"
        )),
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
    // too far, so egui keeps its default neutrals. What *is* applied is the
    // minimal accent pass below: luminance-matched warm shifts plus brass
    // selection/hyperlink accents. Per-surface colours are inlined where
    // needed (panel backgrounds via `crate::ui::panel_bg`).

    // -- Period accent pass: brass instead of egui blue ----------------------
    // Every value here sits within a few luminance points of the egui dark
    // default it replaces, so contrast is unchanged -- only the hue warms up,
    // echoing the gold turn indicators and sepia chrome elsewhere in the game.
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        let v = &mut style.visuals;
        v.hyperlink_color = egui::Color32::from_rgb(196, 158, 90);
        v.selection.bg_fill = egui::Color32::from_rgb(110, 84, 30);
        v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(235, 210, 150));
        v.faint_bg_color = egui::Color32::from_rgb(35, 31, 26);
        v.extreme_bg_color = egui::Color32::from_rgb(14, 13, 11);
        v.panel_fill = egui::Color32::from_rgb(29, 27, 24);
        v.window_fill = egui::Color32::from_rgb(29, 27, 24);
        let w = &mut v.widgets;
        w.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(74, 66, 54));
        w.inactive.weak_bg_fill = egui::Color32::from_rgb(62, 56, 47);
        w.inactive.bg_fill = egui::Color32::from_rgb(62, 56, 47);
        w.hovered.weak_bg_fill = egui::Color32::from_rgb(74, 67, 56);
        w.hovered.bg_fill = egui::Color32::from_rgb(74, 67, 56);
        w.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 132, 100));
        w.active.weak_bg_fill = egui::Color32::from_rgb(58, 52, 42);
        w.active.bg_fill = egui::Color32::from_rgb(58, 52, 42);
    });
    installed.0 = true;
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
    peers: Peers,
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
                    if peers.may_act(active) {
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

pub(crate) fn cursor_overlay_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    local: Res<settings::LocalPlayerSettings>,
    local_peer: Res<LocalPeer>,
    mut peers: crate::peers::PeerCursorQuery,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
) {
    if !local.show_other_cursors {
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

    let mut visible: Vec<(PeerId, Vec2, egui::Color32, String)> = Vec::new();
    for (entity, key, name, color, mut cursor) in &mut peers {
        // Skip the local player's own cursor (that's the mouse pointer).
        if local_peer.0 == Some(entity) {
            continue;
        }
        let Some(pos) = cursor.current else {
            continue;
        };
        let t = if cursor.last_update > 0.0 {
            let elapsed = now - cursor.last_update;
            (elapsed / 0.1).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let prev = cursor.previous.unwrap_or(pos);
        let target = prev.lerp(pos, t as f32);
        let display = cursor.display.get_or_insert(target);
        *display = display.lerp(target, alpha);

        let color = color.map(|c| c.0).unwrap_or(egui::Color32::WHITE);
        let name = name
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("{:?}", key.0));
        visible.push((key.0, *display, color, name));
    }

    if visible.is_empty() {
        return;
    }

    egui::Area::new(egui::Id::new("cursor_overlay"))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            for (_, world_xz, color, label) in &visible {
                let world = Vec3::new(world_xz.x, 0.0, world_xz.y);
                let Ok(viewport) = camera.world_to_viewport(cam_transform, world) else {
                    continue;
                };
                let screen = egui::pos2(viewport.x, viewport.y);
                painter.circle_filled(screen, 5.0, *color);
                painter.text(
                    screen + egui::Vec2::new(8.0, -4.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(12.0),
                    *color,
                );
            }
        });
}

/// Render the game-control section (turn/phase/day-night line, turn indicator,
/// End Phase button, one-shot scenario set-up) into an existing `ui`. Lives in
/// the right sidebar's "Game control" section; a no-op unless a game is active.
pub(crate) fn game_control_section(
    ui: &mut egui::Ui,
    state: &crate::GameStateResource,
    game_turn: Option<&GameTurn>,
    peers: &Peers,
    pending: Option<&mut crate::PendingEdits>,
) {
    let Some(turn) = game_turn else {
        return;
    };
    let Some(pending) = pending else {
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
    let my_turn = peers.may_act(state.0.active_player);
    let in_setup = matches!(state.0.phase, omdurman_rules::Phase::Setup);

    ui.colored_label(
        crate::ui::palette::HEADING,
        format!(
            "Turn {}  {}  {}",
            **turn,
            state.0.phase.top_level_name(),
            day_night_str
        ),
    );

    // Turn indicator -- only meaningful once play has begun. Setup is *not* a
    // turn: both players deploy concurrently, so a "your turn / waiting on"
    // indicator would be misleading. It's suppressed during Setup, where the
    // deployment status below tells each player what to do instead.
    if !in_setup {
        if my_turn {
            ui.colored_label(
                crate::ui::palette::GOLD,
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

    // -- Scoreboard / victory progress --
    // FoK uses a different victory scheme (§9.35: GORDON's fate + Dervish
    // losses) that the §9.14 VP ladder does not model -- the generic scoreboard
    // would read "No scoring yet." for the whole game. Replace it with the
    // FoK-specific panel during a Fall-of-Khartoum session.
    if !in_setup {
        if crate::fok_panel::is_fok(state) {
            crate::fok_panel::fok_status_section(ui, state);
        } else {
            victory_point_scoreboard(ui, state);
        }
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
        setup_control_section(ui, state, peers, pending);
    } else if my_turn && ui.button("End Phase").clicked() {
        // Each player ends their *own* turn: the End Phase button is shown only
        // to whoever controls the active faction.
        pending.submit_game(omdurman_net::GameEvent::Effect(
            omdurman_rules::effects::GameEffect::AdvancePhase,
        ));
    }
}

/// The Campaign/Historical §9.14 victory-point scoreboard. Extracted from
/// `game_control_section` so the FoK scenario can swap in its own
/// victory-progress panel ([`crate::fok_panel::fok_status_section`]) instead.
fn victory_point_scoreboard(ui: &mut egui::Ui, state: &crate::GameStateResource) {
    use omdurman_rules::VpSource;
    let ae_vp = state
        .0
        .victory
        .total_for(omdurman_types::Player::AngloEgyptian)
        .value();
    let dv_vp = state
        .0
        .victory
        .total_for(omdurman_types::Player::Dervish)
        .value();
    let net = ae_vp - dv_vp;
    let net_color = if net > 0 {
        crate::ui::palette::GOOD
    } else if net < 0 {
        crate::ui::palette::BAD
    } else {
        egui::Color32::from_gray(170)
    };
    ui.label(
        egui::RichText::new("Score")
            .strong()
            .color(crate::ui::palette::HEADING),
    );
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("A-E: {ae_vp}")).color(crate::ui::palette::AE));
        ui.label(
            egui::RichText::new(format!("Dervish: {dv_vp}")).color(crate::ui::palette::DERVISH),
        );
    });
    ui.colored_label(net_color, format!("Net: {net:+}"));

    // VP breakdown by source category (§9.14). Collapsible to keep the
    // sidebar compact; defaults to collapsed.
    ui.collapsing("Breakdown", |ui| {
        ui.style_mut().override_font_id = Some(egui::FontId::proportional(11.0));
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
            ui.colored_label(crate::ui::palette::AE, "Anglo-Egyptian:");
            for src in &ae_sources {
                let pts = tally(*src);
                if pts > 0 {
                    ui.label(format!("  {src}: {pts}"));
                }
            }
        }
        if has_dv {
            ui.colored_label(crate::ui::palette::DERVISH, "Dervish:");
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
                    omdurman_types::Player::AngloEgyptian => crate::ui::palette::AE,
                    omdurman_types::Player::Dervish => crate::ui::palette::DERVISH,
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

/// The Setup-phase controls: per-faction deployed/target counts and the local
/// player's one-way "Ready" confirmation. Setup is concurrent -- both sides
/// deploy at once and each confirms independently; the engine auto-advances to
/// Movement once both are ready (§9.2/§9.3), so there's no explicit "advance"
/// click. An unbound session (no faction binding) keeps a single "Begin battle"
/// that drives the same `AdvancePhase` for both sides.
fn setup_control_section(
    ui: &mut egui::Ui,
    state: &crate::GameStateResource,
    peers: &Peers,
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
            crate::ui::palette::GOLD
        } else {
            egui::Color32::from_gray(190)
        };
        ui.colored_label(color, format!("{label}: {count}{mark}"));
    }

    ui.add_space(2.0);

    let local = peers.local();
    match local {
        // Bound player: confirm ready for *your* faction (one-way).
        Some(player) => {
            if state.0.setup_ready(player) {
                ui.colored_label(
                    crate::ui::palette::GOLD,
                    "\u{2713} You are ready -- waiting for the other side.",
                );
            } else if state.0.setup_target_met(player) {
                if ui.button("Ready").clicked() {
                    pending.submit_game(omdurman_net::GameEvent::Effect(
                        omdurman_rules::effects::GameEffect::ConfirmSetupReady { player },
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
                    pending.submit_game(omdurman_net::GameEvent::Effect(
                        omdurman_rules::effects::GameEffect::AdvancePhase,
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
    layout: Res<crate::ScreenLayout>,
) {
    let Some(_state) = game_state else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let has_telegrams = telegram_log.as_ref().is_some_and(|t| !t.entries.is_empty());
    if !has_telegrams {
        return;
    }
    crate::ui::anchored_card(
        ctx,
        egui::Id::new("game_log"),
        egui::Align2::LEFT_BOTTOM,
        // Clear of the left rail (see `ScreenLayout::left_inset`).
        egui::Vec2::new(layout.left_inset + 8.0, -8.0),
        egui::Frame::new()
            .fill(egui::Color32::from_black_alpha(180))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(8, 6)),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));
            ui.set_max_width(460.0);
            // Military telegrams — most recent two, newest first.
            if let Some(t) = telegram_log.as_ref()
                && !t.entries.is_empty()
            {
                for (turn, text) in t.entries.iter().rev().take(2) {
                    ui.colored_label(
                        egui::Color32::from_rgb(180, 210, 180),
                        format!("[Turn {}] {}", turn, text.lines().next().unwrap_or("")),
                    );
                }
            }
        },
    );
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

    crate::ui::anchored_card(
        ctx,
        egui::Id::new("victory_modal"),
        egui::Align2::CENTER_CENTER,
        egui::Vec2::ZERO,
        egui::Frame::new()
            .fill(paper_bg)
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(32, 24))
            .stroke(egui::Stroke::new(2.0, paper_border)),
        |ui| {
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
                    && !r.paragraphs.is_empty()
                {
                    for para in &r.paragraphs {
                        ui.label(egui::RichText::new(para).size(12.0).color(body_color));
                        ui.add_space(4.0);
                    }
                }
            });
        },
    );
}

// -- Friendlies transport UI (§5.21) ----------------------------------------

#[allow(clippy::too_many_arguments)]
/// Floating panel for the §5.21 "Friendlies" transport actions: Load, Cross, Disembark.
/// Appears during Movement phase when transport conditions are met (the phase
/// gate is the `in_movement_phase` run condition; see `ui_phase_state`).
pub(crate) fn friendlies_transport_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    state: Res<crate::picker::PickerState>,
    placed_units: Query<(Entity, &crate::picker::PlacedUnit)>,
    mut pending: ResMut<crate::PendingEdits>,
    peers: crate::peers::Peers,
    net: Res<NetState>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    let Some(gs) = game_state else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let local = peers.local();
    let is_host = net.is_host;

    // Eligibility and effect construction live on the rules engine (§5.21);
    // this system only decides the label and who may act.
    let selected = crate::picker::selected_unit_id(&state, &placed_units).map(|(uid, _)| uid);
    let action = gs.0.friendlies_transport_offer(selected);
    let action_label = match action {
        Some(omdurman_rules::FriendliesAction::Load { .. }) => Some("Load onto Gunboat"),
        // Show "Cross Nile" for the gunboat's owner.
        Some(omdurman_rules::FriendliesAction::Cross { .. }) => {
            (local.is_some() || is_host).then_some("Cross Nile (§5.21)")
        }
        Some(omdurman_rules::FriendliesAction::Disembark { .. }) => Some("Disembark (§5.21)"),
        None => None,
    };
    let Some(label) = action_label else { return };

    crate::ui::stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("friendlies_transport"),
        egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(40, 50, 30, 210))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(10, 6)),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(13.0));
            ui.colored_label(
                egui::Color32::from_rgb(180, 220, 180),
                "\u{1f6a2} Friendlies Transport",
            );
            if ui.button(label).clicked()
                && let Some(action) = action
            {
                pending.submit_game(omdurman_net::GameEvent::Effect(
                    omdurman_rules::effects::GameEffect::FriendliesTransport(action),
                ));
            }
        },
    );
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
/// Floating panel for §5.3 Zariba construction and §6.53 Demolition actions.
/// Appears during Movement phase when a relevant unit is selected (the phase
/// gate is the `in_movement_phase` run condition; see `ui_phase_state`).
pub(crate) fn special_actions_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    state: Res<crate::picker::PickerState>,
    placed_units: Query<(Entity, &crate::picker::PlacedUnit)>,
    mut pending: ResMut<crate::PendingEdits>,
    peers: crate::peers::Peers,
    net: Res<NetState>,
    mut demolition_sel: ResMut<DemolitionSelection>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    let Some(gs) = game_state else { return };
    let Some((uid, _)) = crate::picker::selected_unit_id(&state, &placed_units) else {
        return;
    };
    let Some(unit) = gs.0.find_unit(uid) else {
        return;
    };
    if unit.state.disrupted {
        return;
    }

    // Only the active player may take special actions.
    let local = peers.local();
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

    let has_construct_button =
        can_construct && !unit.state.constructing_zariba && !unit.state.demolishing;
    let has_demolish_button = can_demolish && gs.0.can_demolition(uid).is_ok();

    // Adjacent demolition targets (§6.53), discovered by the rules engine.
    let targets = gs.0.demolition_targets(uid);
    let adjacent_forts: Vec<omdurman_rules::UnitId> = targets
        .iter()
        .filter_map(|t| match t {
            omdurman_rules::DemolitionTarget::Fort(id) => Some(*id),
            _ => None,
        })
        .collect();
    let adjacent_walls: Vec<omdurman_types::HexsideRef> = targets
        .iter()
        .filter_map(|t| match t {
            omdurman_rules::DemolitionTarget::WallHexside(edge) => Some(*edge),
            _ => None,
        })
        .collect();
    let has_targets = !targets.is_empty();
    let has_demolish_button_full = has_demolish_button && has_targets;
    let unit_hex = unit.position;

    if !has_construct_button && !has_demolish_button_full {
        // Clear stale demolition selection when no eligible targets
        if !has_targets {
            demolition_sel.target = None;
        }
        return;
    }

    crate::ui::stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("special_actions"),
        egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(50, 40, 30, 210))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(10, 6)),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(13.0));

            if has_construct_button {
                ui.colored_label(
                    egui::Color32::from_rgb(200, 180, 140),
                    "Construct Zariba (§5.3)",
                );
                ui.label(
                    egui::RichText::new("Place a zariba hexside adjacent to the unit's hex.")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(160, 150, 130)),
                );
                // Pick the construction side among the unit hex's six
                // neighbours (canonical `neighbors()` order = compass
                // directions East..NorthEast).
                const DIR_LABELS: [&str; 6] = ["E", "SE", "SW", "W", "NW", "NE"];
                ui.label(
                    egui::RichText::new("Construct on side:")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(160, 150, 130)),
                );
                ui.horizontal(|ui| {
                    for (idx, n) in unit_hex.neighbors().into_iter().enumerate() {
                        if ui.small_button(DIR_LABELS[idx]).clicked() {
                            let hexside = omdurman_types::HexsideRef::new(unit_hex, n);
                            pending.submit_game(omdurman_net::GameEvent::Effect(
                                omdurman_rules::effects::GameEffect::ConstructZariba {
                                    unit_ids: vec![uid],
                                    hexside,
                                },
                            ));
                        }
                    }
                });
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
                    egui::RichText::new("Destroy adjacent fort or wall. Resolved at end of turn.")
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
                        demolition_sel.target =
                            Some(omdurman_rules::DemolitionTarget::Fort(fort_id));
                    }
                }

                // Wall targets
                for &edge in &adjacent_walls {
                    let label = format!(
                        "Wall ({},{})–({},{})",
                        edge.a.q, edge.a.r, edge.b.q, edge.b.r
                    );
                    let selected = matches!(demolition_sel.target, Some(omdurman_rules::DemolitionTarget::WallHexside(e)) if e == edge);
                    if ui.selectable_label(selected, label).clicked() {
                        demolition_sel.target =
                            Some(omdurman_rules::DemolitionTarget::WallHexside(edge));
                    }
                }

                ui.add_space(4.0);
                if demolition_sel.target.is_some() {
                    if ui.button("Commit to Demolition").clicked()
                        && let Some(target) = demolition_sel.target
                    {
                        pending.submit_game(omdurman_net::GameEvent::Effect(
                            omdurman_rules::effects::GameEffect::Demolition {
                                unit_id: uid,
                                target,
                            },
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
        },
    );
}

#[allow(clippy::too_many_arguments)]
/// §6.63 Artillery Breach: with an artillery/howitzer unit selected during a
/// fire sub-phase, list every Wall hexside the engine's `can_fire_at_wall`
/// accepts for it; a click pre-rolls the d10 and broadcasts
/// [`GameEffect::ArtilleryBreachWall`]. The CRT cell (Eliminate ≥ 2) decides
/// the breach on the echo — the button is an attempt, not a promise. This is
/// the attacker's standard way into the walled city (the Royal Engineers'
/// §6.53 demolition is the other), so it is the one fire action that targets
/// a *hexside* rather than a hex.
pub(crate) fn artillery_breach_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    state: Res<crate::picker::PickerState>,
    placed_units: Query<(Entity, &crate::picker::PlacedUnit)>,
    mut pending: ResMut<crate::PendingEdits>,
    peers: crate::peers::Peers,
    mut game_rng: ResMut<crate::GameRng>,
    mut dispatches: ResMut<crate::dispatch::Dispatches>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    use omdurman_rules::WeaponClass;
    use omdurman_rules::effects::{GameEffect, RuleError};
    use omdurman_types::HexsideKind;

    let Some(gs) = game_state else { return };
    if !matches!(
        gs.0.phase,
        omdurman_rules::Phase::OffensiveFire(_) | omdurman_rules::Phase::DefensiveFire(_)
    ) {
        return;
    }
    let firing_player = match gs.0.phase {
        omdurman_rules::Phase::OffensiveFire(_) => gs.0.active_player,
        omdurman_rules::Phase::DefensiveFire(_) => gs.0.active_player.opponent(),
        _ => return,
    };
    if !peers.may_act(firing_player) {
        return;
    }
    let Some((uid, _)) = crate::picker::selected_unit_id(&state, &placed_units) else {
        return;
    };
    let Some(unit) = gs.0.find_unit(uid) else {
        return;
    };
    if unit.profile.identity.owner() != firing_player || unit.state.disrupted {
        return;
    }
    if !matches!(
        unit.profile.weapon,
        WeaponClass::Artillery | WeaponClass::Howitzer
    ) {
        return;
    }
    if gs.0.units_fired_this_phase.contains(&uid) {
        return;
    }

    // Every wall hexside this battery may currently fire at, nearest first.
    // (Engine re-validates on the echo; this list is just the clickable set.)
    let mut targets: Vec<(omdurman_types::HexsideRef, u16)> =
        gs.0.board
            .hexsides
            .iter()
            .filter(|(_, kind)| **kind == HexsideKind::Wall)
            .filter_map(|(edge, _)| {
                match gs.0.can_fire_at_wall(uid, *edge) {
                    Ok((_, range, _)) => Some((*edge, range.value())),
                    // Out-of-range walls are the common case; surface them as
                    // disabled buttons only when nothing is in range (below).
                    Err(RuleError::OutOfRange { .. } | RuleError::OutOfRangeAtNight { .. }) => {
                        Some((*edge, u16::MAX))
                    }
                    Err(_) => None,
                }
            })
            .collect();
    targets.sort_by_key(|(edge, range)| (*range, edge.a.q, edge.a.r, edge.b.q, edge.b.r));
    if targets.is_empty() {
        return;
    }
    let in_range = || targets.iter().any(|(_, range)| *range != u16::MAX);

    let Ok(ctx) = contexts.ctx_mut() else { return };
    crate::ui::stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("artillery_breach"),
        egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(50, 35, 30, 210))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(10, 6)),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(13.0));
            ui.colored_label(
                egui::Color32::from_rgb(210, 170, 140),
                "Artillery Breach (§6.63)",
            );
            ui.label(
                egui::RichText::new("Fire at a wall hexside. A CRT result of Eliminate 2+ breaches it; any enemy adjacent to the wall is eliminated.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(160, 150, 130)),
            );
            ui.add_space(2.0);
            if !in_range() {
                ui.label(
                    egui::RichText::new("No wall in range (night halves artillery range).")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(180, 120, 100)),
                );
            }
            for (edge, range) in &targets {
                if *range == u16::MAX {
                    continue;
                }
                let label = format!(
                    "Wall ({},{})–({},{})  [range {}]",
                    edge.a.q, edge.a.r, edge.b.q, edge.b.r, range
                );
                if ui.small_button(label).clicked() {
                    let roll = game_rng.roll_d10();
                    dispatches.push(
                        "Artillery Breach",
                        format!(
                            "Firing at wall ({},{})–({},{}) — roll {}",
                            edge.a.q,
                            edge.a.r,
                            edge.b.q,
                            edge.b.r,
                            roll.value(),
                        ),
                    );
                    pending.submit_game(omdurman_net::GameEvent::Effect(
                        GameEffect::ArtilleryBreachWall {
                            firers: vec![uid],
                            target: *edge,
                            roll,
                        },
                    ));
                }
            }
        },
    );
}

// -- Top bar (cross-cutting) -------------------------------------------------

#[allow(clippy::too_many_arguments)]
/// The full-width top bar: mode-switching controls on the left and
/// phase/turn info beside them. Publishes its measured height to
/// [`crate::ScreenLayout::top_bar_height`] so every band below it (left rail,
/// stacked cards, charts sheet) starts clear of it. Visible in all non-Menu
/// states. While spectating a replay, it also hosts the "Back to lobby" exit
/// button as its first item.
pub(crate) fn mode_toolbar_ui(
    mut contexts: EguiContexts,
    mode: Res<State<crate::AppMode>>,
    app_state: Res<State<crate::AppState>>,
    mut next_mode: ResMut<NextState<crate::AppMode>>,
    mut next_app_state: ResMut<NextState<crate::AppState>>,
    mut timeline: ResMut<crate::timeline::SpectatorTimeline>,
    game_state: Option<Res<crate::GameStateResource>>,
    phase_machine: Option<Res<State<crate::ui_phase_state::UiPhaseState>>>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let mut bar_height = None;

    egui::Area::new(egui::Id::new("mode_toolbar"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            // Full-width chrome: one continuous bar across the top instead of
            // a floating island (everything below starts under it).
            ui.set_min_width(ctx.content_rect().width());
            let inner = egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(40, 40, 50, 220))
                .corner_radius(0.0)
                .inner_margin(egui::Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Spectator exit: leave review mode back to the lobby.
                        if *app_state.get() == crate::AppState::Spectating {
                            if ui.button("\u{2b05} Back to lobby").clicked() {
                                timeline.record = None;
                                next_mode.set(crate::AppMode::Lobby);
                                next_app_state.set(crate::AppState::Lobby);
                            }
                            ui.separator();
                        }

                        // Mode label
                        ui.label(
                            egui::RichText::new(match **mode {
                                crate::AppMode::Menu => "Menu",
                                crate::AppMode::Lobby => "Lobby",
                                crate::AppMode::Game => "Game",
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

                        // Phase/turn info when in Game mode (from the
                        // mirrored §4 machine; see `ui_phase_state`)
                        if let (Some(gs), Some(machine)) =
                            (game_state.as_ref(), phase_machine.as_ref())
                        {
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("Turn {}", gs.0.current_turn.value()))
                                    .size(13.0),
                            );
                            ui.label(egui::RichText::new(machine.get().phase_label()).size(13.0));
                        }
                    });
                });
            bar_height = Some(inner.response.rect.height());
        });
    if let Some(height) = bar_height {
        layout.top_bar_height = height.max(crate::layout::TOP_BAR_HEIGHT);
    }
}

// -- Optional-rule setup UI (§10.11, §10.21) --------------------------------

/// Setup-phase panel for Dervish river mine / chain placement.
/// Hex-click handling is done by `handle_optional_rule_click` in the picker
/// pipeline.
pub(crate) fn optional_rule_setup_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    peers: Peers,
    net: Res<NetState>,
    mut placement: ResMut<OptionalRulePlacement>,
    mut pending: ResMut<crate::PendingEdits>,
    layout: Res<crate::ScreenLayout>,
) {
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, omdurman_rules::Phase::Setup) {
        placement.pending_mine = None;
        placement.placing_chain = false;
        placement.chain_hexes.clear();
        return;
    }

    // Only the Dervish player (or unbound host) can place mines/chains.
    let local = peers.local();
    let is_dervish = match local {
        Some(p) => p == omdurman_types::Player::Dervish,
        None => net.is_host,
    };
    if !is_dervish {
        return;
    }

    let has_mines =
        gs.0.optional_rules
            .contains(&omdurman_rules::OptionalRule::RiverMines);
    let has_chain =
        gs.0.optional_rules
            .contains(&omdurman_rules::OptionalRule::RiverChain);
    if !has_mines && !has_chain {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    crate::ui::anchored_card(
        ctx,
        egui::Id::new("optional_rule_setup"),
        egui::Align2::RIGHT_TOP,
        // Clear of the charts sheet / peek tab (see `right_inset`).
        egui::vec2(-(layout.right_inset + 10.0), 380.0),
        egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(40, 30, 40, 210))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(10, 6)),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(12.0));

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
                        placement.pending_mine = Some(omdurman_types::HexCoord::new(99, 99));
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
                        egui::RichText::new(format!(
                            "Selecting hex {}/4...",
                            placement.chain_hexes.len() + 1
                        ))
                        .size(11.0)
                        .color(egui::Color32::from_gray(180)),
                    );
                    if ui.button("Finish Chain").clicked() && !placement.chain_hexes.is_empty() {
                        pending.submit_game(omdurman_net::GameEvent::Effect(
                            omdurman_rules::effects::GameEffect::PlaceChain {
                                hexes: std::mem::take(&mut placement.chain_hexes),
                            },
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
                            .color(crate::ui::palette::GOOD),
                    );
                }
            }
        },
    );
}
