//! Remember Gordon! Battle of Omdurman.

mod actions_panel;
mod board_state;
mod camera;
mod charts;
mod combat_card;
mod combat_predict;
mod debug_capture;
mod desertion;
mod dev_inspector;
mod dispatch;
mod event_viewer;
mod events;
mod fire;
mod fire_allocation;
mod fok_entry;
mod fok_panel;
mod game_apply;
mod game_record;
mod hover_tooltip;
mod input;
mod llm;
mod lobby;
mod melee;
mod mode_transitions;
mod net_plugin;
mod net_socket;
mod newspaper;
mod overview;
mod params;
mod peers;
mod phase_banner;
mod picker;
mod picking;
mod placement;
mod render;
mod retreat;
mod river_placement;
mod rulebook;

mod layout;
mod los;
mod scenario_setup;
mod settings;
mod splash;
mod sprites;
mod state;
mod telegram;
#[cfg(test)]
mod tests;
mod timeline;
mod turn_track_ui;
mod ui;
mod ui_phase_state;
mod ui_plugin;
mod zoc;

// Re-export items moved out of main.rs into their owning modules so existing
// `crate::Foo` paths continue to resolve throughout the crate.
pub(crate) use board_state::{ActiveEditMap, LoadedAnnotations, PendingMapLoad};
pub(crate) use layout::ScreenLayout;
pub(crate) use lobby::{LobbyScenario, LobbyTab, LocalFaction, LocalOptionalRule, LocalSpectator};
pub(crate) use net_plugin::{PendingEdits, PendingIncoming, TurnState};
pub(crate) use params::{
    BoardGeometry, DirectionArrowCtx, GameStateParams, HexRender, PlacementContext,
};
pub(crate) use placement::apply_pending_placement;
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
            }),
    )
    .add_plugins(EguiPlugin::default())
    .add_plugins(camera::CameraPlugin)
    .add_plugins(omdurman_hexmap::HexMapPlugin)
    .add_plugins(board_state::BoardStatePlugin)
    .add_plugins(render::RenderPlugin)
    .add_plugins(picker::GamePlugin)
    .add_plugins(picking::BoardPickingPlugin)
    .add_plugins(ui_plugin::UiPlugin)
    .add_plugins(net_plugin::NetPlugin)
    .add_plugins(net_socket::NetSocketPlugin)
    .add_plugins(splash::SplashPlugin)
    .add_plugins(mode_transitions::ModeTransitionsPlugin)
    .add_plugins(charts::ChartsPlugin)
    .add_plugins(dispatch::DispatchPlugin)
    .add_plugins(combat_card::CombatCardPlugin)
    .add_plugins(hover_tooltip::HoverTooltipPlugin)
    .add_plugins(debug_capture::DebugCapturePlugin)
    // Dev-only egui world inspector: `cargo run -p omdurman-app --features dev`.
    // Never part of release or wasm builds (off by default).
    .add_plugins(dev_inspector::DevInspectorPlugin)
    .init_state::<AppState>()
    .init_state::<AppMode>()
    // The Bevy mirror of the rules engine's §4 turn machine (Setup →
    // Movement → fire subphases → Melee → next turn; see
    // `ui_phase_state::UiPhaseState`). Gameplay/UI systems gate on it with
    // `in_state`-style run conditions instead of matching the engine phase.
    .init_state::<ui_phase_state::UiPhaseState>()
    .add_systems(Last, ui_phase_state::sync_ui_phase_state)
    .add_message::<events::LocalAction>()
    .add_message::<events::ObservationEvent>()
    .configure_sets(
        Update,
        (
            // Gameplay systems (picker, combat overlays, movement) run only on a
            // play view (Game) *and* while actually in a game -- never
            // in the lobby/connecting.
            GameSet.run_if(in_state(AppState::InGame).and_then(in_state(AppMode::Game))),
        ),
    )
    .insert_resource(RoomId::new(room))
    .insert_resource(GameStateResource(GameState::new(
        omdurman_types::Scenario::Campaign,
    )))
    .insert_resource(game_record::GameRecorder::default())
    .insert_resource(LoadedAnnotations::default())
    .insert_resource(ActiveEditMap::default())
    .insert_resource(fire_allocation::FireAllocationState::default())
    .insert_resource(ui_plugin::DemolitionSelection::default())
    .insert_resource(ui_plugin::OptionalRulePlacement::default())
    .insert_resource(PendingMapLoad::default())
    .insert_resource(GameTurn::default())
    .insert_resource(phase_banner::PhaseBannerAnimation::default())
    .insert_resource(timeline::SpectatorTimeline::default())
    // (HexLayout comes from the shared board bootstrap: `load_annotations`
    // calibrates it from the embedded Fall-of-Khartoum board data at startup.)
    .add_systems(Startup, spawn_lights)
    .init_resource::<crate::los::LosOverlay>()
    .add_systems(Startup, timeline::spawn_spectator_marker_assets)
    // The ZOC and LOS overlays run on any board view: the live game (GameSet
    // hosts the gameplay scheduling) *and* the spectator timeline, where
    // there is no local player and both sides' ZOC are drawn instead.
    .add_systems(
        Update,
        (crate::zoc::zoc_overlay_mesh, crate::los::los_overlay_mesh).run_if(
            in_state(AppState::InGame)
                .or_else(in_state(AppState::Spectating))
                .and_then(in_state(AppMode::Game)),
        ),
    )
    .add_systems(
        Update,
        (
            events::forward_local_actions.before(net_plugin::flush_pending),
            // Timeline scrub: advance playback, then rebuild world state to
            // the cursor *before* apply_pending_placement drains the replay
            // queue it fills.
            timeline::advance_timeline_playback,
            timeline::scrub_teardown
                .after(timeline::advance_timeline_playback)
                .before(apply_pending_placement),
            timeline::scrub_rebuild
                .after(timeline::scrub_teardown)
                .before(apply_pending_placement),
            // Combat markers for the event at the timeline cursor (fire
            // arrows / melee triangles); after the rebuild so firer
            // positions match the scrubbed state. Spawned once per event,
            // then animated out by `animate_spectator_combat_markers`.
            timeline::spectator_combat_markers
                .run_if(in_state(AppState::Spectating))
                .after(timeline::scrub_rebuild),
            timeline::animate_spectator_combat_markers.run_if(in_state(AppState::Spectating)),
            phase_banner::update_phase_banner_animation,
        ),
    )
    .add_systems(
        bevy_egui::EguiPrimaryContextPass,
        (
            phase_banner::phase_banner_ui.run_if(in_state(AppState::InGame)),
            // "Back to lobby" lives in the mode toolbar (ui_plugin) now.
            timeline::timeline_ui
                .in_set(ui_plugin::PanelUiSet)
                .run_if(in_state(AppState::Spectating)),
        ),
    )
    // The saved-games list is cached and refreshed on entering the lobby,
    // then rendered inside the lobby's "Saved games" sub-tab (native has
    // files on disk; the cache stays empty on wasm).
    .insert_resource(game_record::SavedGamesCache::default())
    .insert_resource(telegram::TelegramLog::default())
    .insert_resource(newspaper::NewspaperReport::default())
    .insert_resource(newspaper::NewspaperLlmState::default())
    .insert_resource(llm::LlmConfig::default())
    .insert_resource(llm::PendingCompletions::default())
    .add_systems(
        OnEnter(AppState::Lobby),
        game_record::refresh_saved_games_on_lobby,
    )
    .add_systems(
        Update,
        (
            telegram::generate_telegrams,
            telegram::poll_telegram_completions,
            telegram::save_telegram_artifacts,
            newspaper::generate_newspaper,
            newspaper::poll_newspaper_completion,
            newspaper::save_newspaper_artifact,
        )
            .run_if(in_state(AppState::InGame)),
    );

    app.run();
}

fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadow_maps_enabled: true,
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
