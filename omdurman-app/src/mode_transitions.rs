//! Menu-driven mode switching with independent per-mode state.
//!
//! Pressing **M** from any mode saves that mode's state into a snapshot
//! resource and transitions to [`AppMode::Menu`]. Entering a mode from the
//! menu restores its snapshot (or initialises fresh if never visited).
//!
//! The two modes (Lobby, Game) each have independent, persistent state. The
//! menu is the single hub for navigation.

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use omdurman_hexmap::{HexLayout, hex_world_pos};
use omdurman_types::Scenario;

use crate::board_state::PendingMapLoad;
use crate::peers::QueuedFactions;
use crate::picker::{PlacedUnit, UnitPicker, collect_placed_units, spawn_placed_unit};
use crate::render::HexOverlay;
use crate::state::*;
use crate::{GameStateResource, GameTurn, PendingEdits};

pub struct ModeTransitionsPlugin;

impl Plugin for ModeTransitionsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameSnapshot::default())
            .insert_resource(LobbySnapshot::default())
            .add_systems(Update, handle_menu_key)
            .add_systems(OnEnter(AppMode::Lobby), restore_lobby_from_snapshot)
            .add_systems(OnEnter(AppMode::Game), restore_game_from_snapshot);
    }
}

// -- M key handler -----------------------------------------------------------

/// Snapshot-data resources bundled to keep `handle_menu_key` under Bevy's
/// 16-parameter system limit.
#[derive(bevy::ecs::system::SystemParam)]
struct MenuSaveParams<'w> {
    game: ResMut<'w, GameSnapshot>,
    lobby: ResMut<'w, LobbySnapshot>,
}

/// Bundle of the mode/keys/contexts/next-mode inputs to [`handle_menu_key`].
#[derive(bevy::ecs::system::SystemParam)]
struct MenuKeyInput<'w, 's> {
    mode: Res<'w, State<AppMode>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    contexts: EguiContexts<'w, 's>,
    next_mode: ResMut<'w, NextState<AppMode>>,
}

/// Bundle of the read-only game state, peer set, and turn counter accessed
/// by [`handle_menu_key`] when snapshotting the play mode.
#[derive(bevy::ecs::system::SystemParam)]
struct GameReadState<'w, 's> {
    game_state: Option<Res<'w, GameStateResource>>,
    peers: crate::peers::Peers<'w, 's>,
    game_turn: Option<Res<'w, GameTurn>>,
}

/// Bundle of the lobby-scenario + local faction/spectator picks used by
/// [`handle_menu_key`] when snapshotting the lobby.
#[derive(bevy::ecs::system::SystemParam)]
struct LobbySettings<'w> {
    lobby_scenario: Option<Res<'w, crate::LobbyScenario>>,
    local_faction: Option<Res<'w, crate::LocalFaction>>,
    local_spectator: Option<Res<'w, crate::LocalSpectator>>,
}

/// Bundle of the two placed-unit queries (entities for despawn, components for
/// snapshot collection) used by [`handle_menu_key`].
#[derive(bevy::ecs::system::SystemParam)]
struct PlacedUnitQueries<'w, 's> {
    placed_entities: Query<'w, 's, Entity, With<PlacedUnit>>,
    placed_units: Query<'w, 's, &'static PlacedUnit>,
}

/// Bundle of the mutable game-state resources touched when restoring the play
/// mode from a snapshot, so [`restore_game_from_snapshot`] stays under the
/// system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
struct GameMutableState<'w> {
    game_state: Option<ResMut<'w, GameStateResource>>,
    queued_factions: ResMut<'w, QueuedFactions>,
    game_turn: Option<ResMut<'w, GameTurn>>,
    pending_map: ResMut<'w, PendingMapLoad>,
}

/// Bundle of the mesh + material asset stores so [`restore_game_from_snapshot`]
/// stays under the system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
struct AssetStores<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
}

/// Bundle of the read-only hex layout + overlay so [`restore_game_from_snapshot`]
/// stays under the system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
struct HexView<'w> {
    layout: Res<'w, HexLayout>,
    overlay: Res<'w, HexOverlay>,
}

/// Watch for the **M** key and return to the menu from any mode.
fn handle_menu_key(
    input: MenuKeyInput,
    mut snapshots: MenuSaveParams,
    game_read: GameReadState,
    lobby: LobbySettings,
    placed: PlacedUnitQueries,
    mut commands: Commands,
) {
    let MenuKeyInput {
        mode,
        keys,
        mut contexts,
        mut next_mode,
    } = input;
    let GameReadState {
        game_state,
        peers,
        game_turn,
    } = game_read;
    let LobbySettings {
        lobby_scenario,
        local_faction,
        local_spectator,
    } = lobby;
    let PlacedUnitQueries {
        placed_entities,
        placed_units,
    } = placed;
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
                &peers,
                game_turn.as_deref(),
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
    peers: &crate::peers::Peers,
    game_turn: Option<&GameTurn>,
    placed_units: &Query<&PlacedUnit>,
) {
    snapshot.game_state = game_state.map(|gs| gs.0.clone());
    snapshot.factions = peers.assignments();
    snapshot.game_turn = game_turn.map_or(1, |gt| **gt);
    snapshot.placed_units = collect_placed_units(placed_units);
    snapshot.has_data = true;
    info!(
        units = snapshot.placed_units.len(),
        "saved game snapshot"
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
    snapshot.local_spectator = local_spectator.is_some_and(|s| s.0);
    snapshot.has_data = true;
    info!("saved lobby snapshot");
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
///
/// Restore game state from snapshot when entering Game mode.
fn restore_game_from_snapshot(
    snapshot: Res<GameSnapshot>,
    game: GameMutableState,
    view: HexView,
    assets: AssetStores,
    placed_units: Query<Entity, With<PlacedUnit>>,
    mut picker: ResMut<UnitPicker>,
    mut commands: Commands,
) {
    let GameMutableState {
        mut game_state,
        mut queued_factions,
        mut game_turn,
        mut pending_map,
    } = game;
    let HexView { layout, overlay } = view;
    let AssetStores {
        mut meshes,
        mut materials,
    } = assets;
    if !snapshot.has_data {
        return;
    }

    info!("restoring game from snapshot");

    if let Some(ref gs) = snapshot.game_state
        && let Some(ref mut res) = game_state {
            res.0 = gs.clone();
            let map_kind = crate::map_kind_for_scenario(gs.scenario);
            pending_map.0 = Some(map_kind);
        }

    // The faction binding is staged and applied to the peer entities on the
    // next frame (`peers::apply_faction_bindings`), once `sync_peer_entities`
    // has re-spawned them from the live `NetState`.
    queued_factions.0 = Some(snapshot.factions.clone());

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
