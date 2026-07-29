//! Remember Gordon! Battle of Omdurman.

mod actions_panel;
mod browser;
mod camera;
mod charts;
mod combat_card;
mod combat_predict;
mod debug_capture;
mod desertion;
mod dispatch;
mod editor;
mod event_viewer;
mod events;
mod fire;
mod fok_entry;
mod game_apply;
mod game_record;
mod hover_tooltip;
mod input;
mod lobby;
mod llm;
mod mode_transitions;
mod melee;
mod net_plugin;
mod net_socket;
mod newspaper;
mod overview;
mod params;
mod picker;
mod placement;
pub(crate) mod prelude;
mod render;
mod retreat;
mod rulebook;

mod scenario_setup;
mod settings;
mod splash;
mod state;
#[cfg(test)]
mod tests;
mod telegram;
mod timeline;
mod turn_track_ui;
mod ui;
mod ui_plugin;
mod units;
mod zoc;
mod util;

// Re-export items moved out of main.rs into their owning modules so existing
// `crate::Foo` paths continue to resolve throughout the crate.
pub(crate) use editor::{
    ActiveEditMap, EditorBoard, LoadedAnnotations, PendingMapLoad,
};
pub(crate) use lobby::{LobbyChoices, LobbyScenario, LobbyTab, LocalFaction, LocalSpectator};
pub(crate) use net_plugin::{
    CursorPositions, PendingEdits, PendingIncoming, PlayerFactions, TurnState,
};
pub(crate) use params::{DirectionArrowCtx, FactionGate, GameStateParams, HexRender, MoveGate, PlacementContext};
pub(crate) use placement::apply_pending_placement;
pub(crate) use render::HoveredHex;
pub(crate) use scenario_setup::map_kind_for_scenario;
pub(crate) use settings::ReconnectRoom;
pub(crate) use state::*;
pub(crate) use timeline::rebuild_state_to;
pub(crate) use ui_plugin::SidebarClip;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use omdurman_hexmap::HexLayout;
use omdurman_net::{RoomId, room_id};
use omdurman_rules::effects::GameState;

fn main() {
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
    .add_plugins(editor::EditorPlugin)
    .add_plugins(render::RenderPlugin)
    .add_plugins(picker::GamePlugin)
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
    .init_state::<AppState>()
    .init_state::<AppMode>()
    .init_state::<EditorTab>()
    .add_message::<events::LocalAction>()
    .add_message::<events::ObservationEvent>()
    .configure_sets(
        Update,
        (
            EditorSet.run_if(in_state(AppMode::Editor).and_then(in_state(EditorTab::Terrain))),
            OverlaySet.run_if(in_state(AppMode::Editor).and_then(in_state(EditorTab::Overlay))),
            HexsideSet.run_if(in_state(AppMode::Editor).and_then(in_state(EditorTab::Hexside))),
            // Gameplay systems (picker, combat overlays, movement) run only on a
            // play view (Game) *and* while actually in a game -- never
            // in the lobby/connecting/editor.
            GameSet.run_if(
                in_state(AppState::InGame).and_then(in_state(AppMode::Game)),
            ),
        ),
    )
    .insert_resource(RoomId::new(room))
    .insert_resource(GameStateResource(GameState::new(
        omdurman_types::Scenario::Campaign,
    )))
    .insert_resource(game_record::GameRecorder::default())
    .insert_resource(LoadedAnnotations::default())
    .insert_resource(ActiveEditMap::default())
    .insert_resource(EditorBoard::default())
    .insert_resource(PendingMapLoad::default())
    .insert_resource(GameTurn::default())
    .insert_resource(timeline::SpectatorTimeline::default())
    .insert_resource(HexLayout::calibrated(
        omdurman_types::Orientation::Pointy,
        omdurman_hexmap::CalibrationAnchor {
            px: Vec2::new(736.0, 420.0),
            hex: omdurman_types::HexCoord::new(0, 0),
        },
        omdurman_hexmap::CalibrationAnchor {
            px: Vec2::new(1178.0, 572.0),
            hex: omdurman_types::HexCoord::new(5, -1),
        },
        Vec2::new(omdurman_hexmap::IMG_W, omdurman_hexmap::IMG_H),
    ))
    .add_systems(Startup, spawn_lights)
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
        ),
    )
    .add_systems(
        bevy_egui::EguiPrimaryContextPass,
        (timeline::timeline_ui, timeline::exit_review_ui)
            .run_if(in_state(AppState::Spectating)),
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
            newspaper::generate_newspaper,
            newspaper::poll_newspaper_completion,
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
