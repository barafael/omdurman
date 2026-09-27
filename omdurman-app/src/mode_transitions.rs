//! Menu-driven mode switching.
//!
//! Pressing **M** from any mode returns to [`AppMode::Menu`]. The Game view
//! keeps no snapshot: the engine state is a pure function of the event log
//! (which keeps applying while the menu is shown) and the board counters are
//! re-derived from it by `picker::reconcile_unit_sprites`, so returning to the
//! board simply shows the live game. The Lobby's local picks (faction,
//! command, scenario) are snapshotted and restored on re-entry, and
//! re-announced to the peers.

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use omdurman_types::Scenario;

use crate::PendingEdits;
use crate::state::*;

pub struct ModeTransitionsPlugin;

impl Plugin for ModeTransitionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_menu_key);
        add_lobby_snapshot_systems(app);
    }
}

/// The lobby half of mode switching: snapshot the lobby picks on leaving the
/// Lobby mode, restore (and re-announce) them on entering it. Nothing is
/// registered for the Game mode -- entering it must never touch the engine
/// state.
pub(crate) fn add_lobby_snapshot_systems(app: &mut App) {
    app.insert_resource(LobbySnapshot::default())
        .add_systems(OnExit(AppMode::Lobby), save_lobby_snapshot)
        .add_systems(OnEnter(AppMode::Lobby), restore_lobby_from_snapshot);
}

// -- M key handler -----------------------------------------------------------

/// Watch for the **M** key and return to the menu from any mode (the same
/// transition as the toolbar's Menu button).
fn handle_menu_key(
    mode: Res<State<AppMode>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
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
    info!(from = ?mode.get(), "menu key pressed — returning to menu");
    next_mode.set(AppMode::Menu);
}

// -- Lobby snapshot ----------------------------------------------------------

/// Save the lobby's local picks whenever the Lobby mode is left, by any path
/// (M key, toolbar, a started game).
fn save_lobby_snapshot(
    mut snapshot: ResMut<LobbySnapshot>,
    scenario: Option<Res<crate::LobbyScenario>>,
    local_faction: Option<Res<crate::LocalFaction>>,
    local_spectator: Option<Res<crate::LocalSpectator>>,
    local_command: Option<Res<crate::lobby::LocalCommand>>,
) {
    snapshot.scenario = scenario.map_or(Scenario::Campaign, |s| s.0);
    snapshot.local_faction = local_faction.and_then(|f| f.0);
    snapshot.local_spectator = local_spectator.is_some_and(|s| s.0);
    snapshot.local_command = local_command.and_then(|c| c.0.clone());
    snapshot.has_data = true;
    info!("saved lobby snapshot");
}

// -- Restore helpers (OnEnter handlers) --------------------------------------

/// Restore lobby state from snapshot when entering Lobby mode.
fn restore_lobby_from_snapshot(
    snapshot: Res<LobbySnapshot>,
    mut lobby_scenario: Option<ResMut<crate::LobbyScenario>>,
    mut local_faction: Option<ResMut<crate::LocalFaction>>,
    mut local_spectator: Option<ResMut<crate::LocalSpectator>>,
    mut local_command: Option<ResMut<crate::lobby::LocalCommand>>,
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
    if let Some(ref mut c) = local_command {
        c.0 = snapshot.local_command.clone();
    }

    if let Some(faction) = snapshot.local_faction {
        pending
            .outgoing_broadcast
            .push(omdurman_net::NetMsg::Ephemeral(
                omdurman_net::Ephemeral::FactionChoice(Some(faction)),
            ));
    }
    if let Some(command) = snapshot.local_command.clone() {
        pending
            .outgoing_broadcast
            .push(omdurman_net::NetMsg::Ephemeral(
                omdurman_net::Ephemeral::CommandChoice(Some(command)),
            ));
    }
}
