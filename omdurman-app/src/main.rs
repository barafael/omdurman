//! Remember Gordon! Battle of Omdurman.

mod actions_panel;
mod activity;
mod board_click;
mod board_state;
mod bot_player;
mod camera;
mod charts;
mod combat_card;
mod combat_predict;
mod combat_ui;
mod debug_capture;
mod desertion;
mod dev_inspector;
mod dispatch;
mod event_viewer;
mod events;
mod fire;
mod fire_allocation;
mod fok_panel;
mod frame_stats;
mod fx;
mod game_apply;
mod game_record;
mod hexside_layer;
mod hotkeys;
mod hover_tooltip;
mod keepsakes;
mod lobby;
mod melee;
mod mode_transitions;
mod net_plugin;
mod net_socket;
mod newspaper;
mod overlay;
mod overview;
mod params;
mod peers;
mod picker;
mod picking;
#[cfg(not(target_arch = "wasm32"))]
mod player_key_store;
mod reinforce;
mod render;
mod retreat;
mod river_placement;
mod rulebook;
mod submit;

mod layout;
mod los;
mod scenario_setup;
mod seat_arbiter;
mod seats;
mod seats_ui;
mod settings;
mod splash;
mod state;
mod telegram;
#[cfg(test)]
mod tests;
mod timeline;
mod turn_track_ui;
mod ui;
mod ui_phase_state;
mod ui_plugin;
mod ui_trace;
mod zoc;

// Re-export items moved out of main.rs into their owning modules so existing
// `crate::Foo` paths continue to resolve throughout the crate.
pub(crate) use board_state::{ActiveEditMap, LoadedAnnotations, PendingMapLoad};
pub(crate) use layout::ScreenLayout;
pub(crate) use lobby::{LobbyScenario, LobbyTab, LocalFaction, LocalOptionalRule, LocalSpectator};
pub(crate) use net_plugin::{PendingEdits, PendingIncoming, TurnState};
pub(crate) use params::{BoardGeometry, DirectionArrowCtx, GameStateParams, HexRender};
pub(crate) use render::{HoveredHex, HoveredUnit};
pub(crate) use scenario_setup::map_kind_for_scenario;
pub(crate) use settings::ReconnectRoom;
pub(crate) use state::*;
pub(crate) use timeline::rebuild_state_to;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use omdurman_net::{RoomId, room_id};
use omdurman_rules::effects::GameState;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    dotenvy::dotenv().ok();

    let room = room_id();

    let mut app = App::new();
    picker::register_sprite_source(&mut app);
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // Start windowed but maximized (see `maximize_primary_window`).
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(bevy::log::LogPlugin {
                // A broken audio device (an ALSA stream in POLLERR) makes
                // rodio's stream callback log one error per poll -- thousands
                // a second, gigabytes in minutes. Sound still plays when the
                // device is fine; `RUST_LOG=rodio::stream=error` re-enables
                // it (env directives are added on top of these defaults).
                filter: format!("{}rodio::stream=off", bevy::log::DEFAULT_FILTER),
                ..default()
            }),
    );
    add_game(&mut app, room);
    app.run();
}

/// Everything the game adds on top of Bevy's default plugins: its plugins,
/// resources and systems. Split out of `main` so a headless test can build
/// the same app (see `tests::every_system_has_valid_parameters`).
fn add_game(app: &mut App, room: String) {
    app.add_plugins(EguiPlugin::default())
        // Reactive frame pacing: no busy loop while nothing happens.
        .add_plugins(activity::ActivityPlugin)
        // Frame counts and times on the log, with `OMDURMAN_FRAME_STATS`.
        .add_plugins(frame_stats::FrameStatsPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(omdurman_hexmap::HexMapPlugin)
        .add_plugins(board_state::BoardStatePlugin)
        .add_plugins(render::RenderPlugin)
        .add_plugins(hexside_layer::HexsideLayerPlugin)
        .add_plugins(picker::GamePlugin)
        .add_plugins(ui_trace::UiTracePlugin)
        .add_plugins(reinforce::ReinforcePlugin)
        .add_plugins(picking::BoardPickingPlugin)
        .add_plugins(ui_plugin::UiPlugin)
        .add_plugins(net_plugin::NetPlugin)
        .add_plugins(net_socket::NetSocketPlugin)
        .add_plugins(splash::SplashPlugin)
        .add_plugins(mode_transitions::ModeTransitionsPlugin)
        .add_plugins(charts::ChartsPlugin)
        .add_plugins(dispatch::DispatchPlugin)
        .add_plugins(combat_card::CombatCardPlugin)
        .add_plugins(fx::FxPlugin)
        .add_plugins(hover_tooltip::HoverTooltipPlugin)
        .add_plugins(debug_capture::DebugCapturePlugin)
        .add_plugins(debug_capture::HexProbePlugin)
        // Dev-only egui world inspector: `cargo run -p omdurman-app --features dev`.
        // Never part of release or wasm builds (off by default).
        .add_plugins(dev_inspector::DevInspectorPlugin)
        .init_state::<AppState>()
        .init_state::<AppMode>()
        // The screen that is up, derived from the two (see `state::Screen`).
        .add_computed_state::<Screen>()
        // The Bevy mirror of the rules engine's §4 turn machine (Setup →
        // Movement → fire subphases → Melee → next turn; see
        // `ui_phase_state::UiPhaseState`). Gameplay/UI systems gate on it with
        // `in_state`-style run conditions instead of matching the engine phase.
        .init_state::<ui_phase_state::UiPhaseState>()
        .add_systems(Last, ui_phase_state::sync_ui_phase_state)
        // In-game AI commanders (Kitchener/Khalifa): the host plays any faction
        // whose seats are AI seats, paced for live spectating.
        .init_resource::<bot_player::BotDriver>()
        .add_systems(
            Update,
            bot_player::bot_player_act
                .run_if(in_state(Screen::Board))
                .before(net_plugin::flush_pending),
        )
        .add_message::<events::LocalAction>()
        .add_message::<events::ObservationEvent>()
        .configure_sets(
            Update,
            (
                // Gameplay systems (picker, combat overlays, movement) run only on a
                // play view (Game) *and* while actually in a game -- never
                // in the lobby/connecting.
                GameSet.run_if(in_state(Screen::Board)),
            ),
        )
        .insert_resource(RoomId::new(room))
        .insert_resource(GameStateResource(GameState::new(
            omdurman_types::Scenario::Campaign,
        )))
        .insert_resource(game_record::GameRecorder::default())
        // Present from the first frame (reseeded by `init_game_record` and
        // every rebuild): a system requiring a resource that a command
        // inserts later panics if it ever runs first.
        .insert_resource(GameRng::from_seed(omdurman_net::new_seed()))
        .insert_resource(LoadedAnnotations::default())
        .insert_resource(ActiveEditMap::default())
        .insert_resource(fire_allocation::FireAllocationState::default())
        .insert_resource(ui_plugin::DemolitionSelection::default())
        .insert_resource(ui_plugin::OptionalRulePlacement::default())
        .insert_resource(PendingMapLoad::default())
        .insert_resource(timeline::SpectatorTimeline::default())
        // (HexLayout comes from the shared board bootstrap: `load_annotations`
        // calibrates it from the embedded Fall-of-Khartoum board data at startup.)
        .add_systems(Startup, spawn_lights)
        .init_resource::<crate::los::LosOverlay>()
        .init_resource::<crate::los::LosAnalysis>()
        // Legal-fire-target enumeration shared by the target overlay, the
        // actions-panel count, the hover preview, and the artillery panel.
        .init_resource::<fire::FireTargetCache>()
        // The ZOC and LOS overlays run on any board view: the live game (GameSet
        // hosts the gameplay scheduling) *and* the spectator timeline, where
        // there is no local player and both sides' ZOC are drawn instead.
        .add_systems(
            Update,
            (
                crate::zoc::zoc_overlay_mesh,
                crate::los::update_los_analysis,
                crate::los::los_overlay_mesh,
                crate::los::los_blocked_labels,
            )
                .chain()
                .run_if(on_board),
        )
        .add_systems(
            Update,
            (
                events::forward_local_actions.before(net_plugin::flush_pending),
                // Timeline scrub: advance playback, then rebuild world state to
                // the cursor (the sprite reconcile runs after the rebuild).
                timeline::advance_timeline_playback,
                timeline::scrub_teardown.after(timeline::advance_timeline_playback),
                timeline::scrub_rebuild.after(timeline::scrub_teardown),
            ),
        )
        .add_systems(
            bevy_egui::EguiPrimaryContextPass,
            (
                // The pause notice stacks under the top bar.
                seats_ui::pause_card_ui
                    .after(ui_plugin::mode_toolbar_ui)
                    .run_if(in_state(Screen::Board)),
                // A spectator's way into a running game, below the pause card.
                seats_ui::join_panel_ui
                    .after(seats_ui::pause_card_ui)
                    .run_if(in_state(Screen::Board)),
                // A seat vote waits for nobody: on every screen of a live
                // session, the menu included (the menu's native buttons
                // take no clicks through it: `EguiPointerOverUi`).
                seats_ui::vote_popup_ui.run_if(in_state(AppState::InGame)),
                // "Back to lobby" lives in the mode toolbar (ui_plugin) now.
                timeline::timeline_ui
                    .in_set(ui_plugin::PanelUiSet)
                    .run_if(in_state(Screen::Review)),
            ),
        )
        // The saved-games list is cached and refreshed on entering the lobby,
        // then rendered inside the lobby's "Saved games" sub-tab (native has
        // files on disk; the cache stays empty on wasm).
        .insert_resource(game_record::SavedGamesCache::default())
        .insert_resource(telegram::TelegramLog::default())
        .insert_resource(newspaper::NewspaperReport::default())
        .init_resource::<keepsakes::Keepsakes>()
        .add_systems(Update, keepsakes::take_keepsakes)
        .add_systems(
            OnEnter(AppState::Lobby),
            game_record::refresh_saved_games_on_lobby,
        )
        // Every peer composes the end-of-game front page, live or when
        // reviewing a finished record.
        .add_systems(
            Update,
            newspaper::compose_newspaper
                .before(newspaper::save_newspaper_artifact)
                .run_if(in_state(AppState::InGame).or_else(in_state(AppState::Spectating))),
        )
        .add_systems(
            Update,
            (
                telegram::generate_telegrams,
                telegram::save_telegram_artifacts,
                newspaper::save_newspaper_artifact,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        );
}

fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            // The board and the counters are unlit, so no shadow ever shows;
            // cascaded shadow maps would re-render every mesh each frame for
            // nothing.
            shadow_maps_enabled: false,
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
