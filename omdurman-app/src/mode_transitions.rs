//! Menu-driven mode switching with independent per-mode state.
//!
//! Pressing **M** from any mode saves that mode's state into a snapshot
//! resource and transitions to [`AppMode::Menu`]. Entering a mode from the
//! menu restores its snapshot (or initialises fresh if never visited).
//!
//! The four modes (Lobby, Game, Sandbox, Editor) each have independent,
//! persistent state. The menu is the single hub for navigation.

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use omdurman_hexmap::{HexLayout, hex_world_pos};
use omdurman_types::Scenario;

use crate::editor::{EditorBoard, PendingMapLoad};
use crate::net_plugin::PlayerFactions;
use crate::picker::{PlacedUnit, UnitPicker, collect_placed_units, spawn_placed_unit};
use crate::render::HexOverlay;
use crate::sandbox::SandboxSettings;
use crate::state::*;
use crate::{GameStateResource, GameTurn, PendingEdits};

pub struct ModeTransitionsPlugin;

impl Plugin for ModeTransitionsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameSnapshot::default())
            .insert_resource(SandboxSnapshot::default())
            .insert_resource(LobbySnapshot::default())
            .insert_resource(EditorSnapshot::default())
            .add_systems(Update, handle_menu_key)
            .add_systems(OnEnter(AppMode::Lobby), restore_lobby_from_snapshot)
            .add_systems(OnEnter(AppMode::Game), restore_game_from_snapshot)
            .add_systems(OnEnter(AppMode::Sandbox), restore_sandbox_from_snapshot)
            .add_systems(OnEnter(AppMode::Editor), restore_editor_from_snapshot);
    }
}

// -- M key handler -----------------------------------------------------------

/// Snapshot-data resources bundled to keep `handle_menu_key` under Bevy's
/// 16-parameter system limit.
#[derive(bevy::ecs::system::SystemParam)]
struct MenuSaveParams<'w> {
    game: ResMut<'w, GameSnapshot>,
    sandbox: ResMut<'w, SandboxSnapshot>,
    lobby: ResMut<'w, LobbySnapshot>,
    editor: ResMut<'w, EditorSnapshot>,
}

/// Watch for the **M** key and return to the menu from any mode.
fn handle_menu_key(
    mode: Res<State<AppMode>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut snapshots: MenuSaveParams,
    game_state: Option<Res<GameStateResource>>,
    factions: Res<PlayerFactions>,
    game_turn: Option<Res<GameTurn>>,
    placed_entities: Query<Entity, With<PlacedUnit>>,
    placed_units: Query<&PlacedUnit>,
    mut sandbox_settings: ResMut<SandboxSettings>,
    lobby_scenario: Option<Res<crate::LobbyScenario>>,
    local_faction: Option<Res<crate::LocalFaction>>,
    local_spectator: Option<Res<crate::LocalSpectator>>,
    editor_board: Res<EditorBoard>,
    mut commands: Commands,
    mut next_mode: ResMut<NextState<AppMode>>,
) {
    if **mode == AppMode::Menu {
        return;
    }
    let over_ui = contexts
        .ctx_mut()
        .map(|c| c.egui_wants_keyboard_input())
        .unwrap_or(false);
    if over_ui || !keys.just_pressed(KeyCode::KeyM) {
        return;
    }

    info!(from = ?mode.get(), "menu key pressed — saving state and returning to menu");

    match **mode {
        AppMode::Game => {
            save_game_snapshot(
                &mut snapshots.game,
                game_state.as_deref(),
                &factions,
                game_turn.as_deref(),
                &placed_units,
            );
        }
        AppMode::Sandbox => {
            save_sandbox_snapshot(
                &mut snapshots.sandbox,
                game_state.as_deref(),
                &sandbox_settings,
                &placed_units,
            );
        }
        AppMode::Lobby => {
            save_lobby_snapshot(
                &mut snapshots.lobby,
                lobby_scenario.as_deref(),
                local_faction.as_deref(),
                local_spectator.as_deref(),
            );
        }
        AppMode::Editor => {
            save_editor_snapshot(&mut snapshots.editor, &editor_board);
        }
        AppMode::Menu => unreachable!(),
    }

    // When leaving a play mode, despawn placed units so they don't linger
    // behind the menu overlay (they'll be respawned from the snapshot on
    // re-entry).
    if mode.is_play() {
        for entity in &placed_entities {
            commands.entity(entity).despawn();
        }
    }

    next_mode.set(AppMode::Menu);
}

// -- Save helpers ------------------------------------------------------------

fn save_game_snapshot(
    snapshot: &mut GameSnapshot,
    game_state: Option<&GameStateResource>,
    factions: &PlayerFactions,
    game_turn: Option<&GameTurn>,
    placed_units: &Query<&PlacedUnit>,
) {
    snapshot.game_state = game_state.map(|gs| gs.0.clone());
    snapshot.factions = factions.by_peer.iter().map(|(&k, &v)| (k, v)).collect();
    snapshot.game_turn = game_turn.map_or(1, |gt| **gt);
    snapshot.placed_units = collect_placed_units(placed_units);
    snapshot.has_data = true;
    info!(
        units = snapshot.placed_units.len(),
        "saved game snapshot"
    );
}

fn save_sandbox_snapshot(
    snapshot: &mut SandboxSnapshot,
    game_state: Option<&GameStateResource>,
    settings: &SandboxSettings,
    placed_units: &Query<&PlacedUnit>,
) {
    snapshot.game_state = game_state.map(|gs| gs.0.clone());
    snapshot.settings_scenario = settings.scenario;
    snapshot.settings_started = settings.started;
    snapshot.placed_units = collect_placed_units(placed_units);
    snapshot.has_data = true;
    info!(
        units = snapshot.placed_units.len(),
        "saved sandbox snapshot"
    );
}

fn save_lobby_snapshot(
    snapshot: &mut LobbySnapshot,
    scenario: Option<&crate::LobbyScenario>,
    local_faction: Option<&crate::LocalFaction>,
    local_spectator: Option<&crate::LocalSpectator>,
) {
    snapshot.scenario = scenario.map_or(Scenario::Campaign, |s| s.0);
    snapshot.local_faction = local_faction.and_then(|f| f.0);
    snapshot.local_spectator = local_spectator.map_or(false, |s| s.0);
    snapshot.has_data = true;
    info!("saved lobby snapshot");
}

fn save_editor_snapshot(snapshot: &mut EditorSnapshot, board: &EditorBoard) {
    snapshot.board = board.0;
    snapshot.has_data = true;
    info!("saved editor snapshot");
}

// -- Restore helpers (OnEnter handlers) --------------------------------------

/// Design note: these hooks restore snapshot *data* only. They deliberately do
/// not touch `AppState`. `AppMode` and `AppState` are independent axes: a
/// single `AppMode` can pair with different `AppState`s (e.g. `AppMode::Game`
/// pairs with `AppState::InGame` for live play but `AppState::Spectating` for
/// timeline review). Driving `AppState` from these hooks would clobber that
/// distinction. Each call site that sets `AppMode::X` owns the corresponding
/// `AppState::set(...)` — see `splash::menu_ui`, `net_socket::handle_socket`
/// (StartGame), and `timeline::scrub_rebuild`.

/// Restore game state from snapshot when entering Game mode.
fn restore_game_from_snapshot(
    snapshot: Res<GameSnapshot>,
    mut game_state: Option<ResMut<GameStateResource>>,
    mut factions: ResMut<PlayerFactions>,
    mut game_turn: Option<ResMut<GameTurn>>,
    mut pending_map: ResMut<PendingMapLoad>,
    mut commands: Commands,
    placed_units: Query<Entity, With<PlacedUnit>>,
    mut picker: ResMut<UnitPicker>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !snapshot.has_data {
        return;
    }

    info!("restoring game from snapshot");

    if let Some(ref gs) = snapshot.game_state {
        if let Some(ref mut res) = game_state {
            res.0 = gs.clone();
            let map_kind = crate::map_kind_for_scenario(gs.scenario);
            pending_map.0 = Some(map_kind);
        }
    }

    factions.by_peer.clear();
    for &(pid, player) in &snapshot.factions {
        factions.by_peer.insert(pid, player);
    }

    if let Some(ref mut turn) = game_turn {
        turn.0 = snapshot.game_turn;
    }

    for entity in &placed_units {
        commands.entity(entity).despawn();
    }

    respawn_placed_units(
        &snapshot.placed_units,
        &mut commands,
        &mut picker,
        &layout,
        &overlay,
        &mut meshes,
        &mut materials,
    );
}

/// Restore sandbox state from snapshot when entering Sandbox mode.
fn restore_sandbox_from_snapshot(
    snapshot: Res<SandboxSnapshot>,
    mut game_state: Option<ResMut<GameStateResource>>,
    mut sandbox_settings: ResMut<SandboxSettings>,
    mut pending_map: ResMut<PendingMapLoad>,
    mut commands: Commands,
    placed_units: Query<Entity, With<PlacedUnit>>,
    mut picker: ResMut<UnitPicker>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !snapshot.has_data {
        sandbox_settings.open = true;
        sandbox_settings.started = false;
        return;
    }

    info!("restoring sandbox from snapshot");

    sandbox_settings.scenario = snapshot.settings_scenario;
    sandbox_settings.started = snapshot.settings_started;
    sandbox_settings.open = false;

    if let Some(ref gs) = snapshot.game_state {
        if let Some(ref mut res) = game_state {
            res.0 = gs.clone();
            let map_kind = crate::map_kind_for_scenario(gs.scenario);
            pending_map.0 = Some(map_kind);
        }
    }

    for entity in &placed_units {
        commands.entity(entity).despawn();
    }

    respawn_placed_units(
        &snapshot.placed_units,
        &mut commands,
        &mut picker,
        &layout,
        &overlay,
        &mut meshes,
        &mut materials,
    );
}

/// Restore lobby state from snapshot when entering Lobby mode.
fn restore_lobby_from_snapshot(
    snapshot: Res<LobbySnapshot>,
    mut lobby_scenario: Option<ResMut<crate::LobbyScenario>>,
    mut local_faction: Option<ResMut<crate::LocalFaction>>,
    mut local_spectator: Option<ResMut<crate::LocalSpectator>>,
    mut pending: ResMut<PendingEdits>,
) {
    if !snapshot.has_data {
        return;
    }

    info!("restoring lobby from snapshot");

    if let Some(ref mut s) = lobby_scenario {
        s.0 = snapshot.scenario;
    }
    if let Some(ref mut f) = local_faction {
        f.0 = snapshot.local_faction;
    }
    if let Some(ref mut s) = local_spectator {
        s.0 = snapshot.local_spectator;
    }

    if let Some(faction) = snapshot.local_faction {
        pending.outgoing_broadcast.push(omdurman_net::NetMsg::Ephemeral(
            omdurman_net::Ephemeral::FactionChoice(Some(faction)),
        ));
    }
}

/// Restore editor state from snapshot when entering Editor mode.
fn restore_editor_from_snapshot(
    snapshot: Res<EditorSnapshot>,
    mut editor_board: ResMut<EditorBoard>,
    mut pending_map: ResMut<PendingMapLoad>,
) {
    if !snapshot.has_data {
        return;
    }

    info!("restoring editor from snapshot");

    editor_board.0 = snapshot.board;
    let map_kind = crate::map_kind_for_scenario(snapshot.board);
    pending_map.0 = Some(map_kind);
}

// -- Unit respawn -----------------------------------------------------------

/// Respawn placed units from snapshot data.
fn respawn_placed_units(
    data: &[PlacedUnitData],
    commands: &mut Commands,
    picker: &mut UnitPicker,
    layout: &HexLayout,
    overlay: &HexOverlay,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    if data.is_empty() {
        return;
    }
    let origin = layout.adjusted_origin(&overlay.params);
    for unit_data in data {
        let Some(handle) = picker
            .all
            .iter()
            .find(|(sn, col, row, _, _)| {
                *sn == unit_data.section_name && *col == unit_data.col && *row == unit_data.row
            })
            .map(|(_, _, _, h, _)| h.clone())
        else {
            warn!(
                ?unit_data.section_name,
                unit_data.col,
                unit_data.row,
                "respawn: texture handle not found in picker"
            );
            continue;
        };

        let pos = hex_world_pos(unit_data.coord, origin, &overlay.params);

        if let Some(idx) = picker.available.iter().position(|u| {
            u.section_name == unit_data.section_name
                && u.col == unit_data.col
                && u.row == unit_data.row
        }) {
            picker.available.remove(idx);
        }

        spawn_placed_unit(
            commands,
            meshes,
            materials,
            handle,
            overlay,
            pos,
            PlacedUnit {
                coord: unit_data.coord,
                section_name: unit_data.section_name,
                col: unit_data.col,
                row: unit_data.row,
                is_boat: unit_data.is_boat,
                unit_id: unit_data.unit_id,
                disrupted: unit_data.disrupted,
            },
        );
    }
    info!(count = data.len(), "respawned placed units from snapshot");
}
