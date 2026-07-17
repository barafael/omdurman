//! Sandbox mode: a local, unbound single-seat session for free experimentation.
//!
//! The sandbox is entered from the mode picker ([`crate::AppMode::Sandbox`]).
//! Unlike a networked game (which is set up by the host's `StartGame` in the
//! lobby), a sandbox is configured on its own **settings screen**: pick a
//! scenario/board, then *Open sandbox*. Opening:
//!
//! 1. **clears any previous sandbox** — resets the rules state, despawns every
//!    placed counter, and refills the unit picker;
//! 2. starts a fresh unbound game locally (empty [`crate::PlayerFactions`], so
//!    the single seat may drive both sides — see `local_may_act`), attaching the
//!    scenario's board and requesting the visual map load; and
//! 3. **auto-runs the scenario's fixed setup** (the same `build_setup_plan`
//!    placements as the game's "Set up scenario" button), once the board has
//!    loaded.
//!
//! The settings screen can be re-summoned at any time with **Escape**; opening a
//! new sandbox from it discards the current one. Placement in a sandbox is free:
//! any unit may be dropped on any terrain-valid, unoccupied hex in any phase
//! (the deployment-zone rings still draw during Setup, but are display-only —
//! the placement gate is relaxed for the unbound session; see
//! `picker::handle_picker_clicks`).

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::GameMap;
use omdurman_net::NetMsg;
use omdurman_rules::effects::GameState;
use omdurman_types::Scenario;

use crate::{AppMode, AppState, PendingEdits, SandboxContext};

/// Sandbox settings screen state. `open` gates the settings overlay; it is set
/// on first entering the sandbox and whenever Escape re-summons the screen.
#[derive(Resource)]
pub struct SandboxSettings {
    pub scenario: Scenario,
    pub open: bool,
    /// Whether a sandbox has ever been opened this session, so entering the
    /// sandbox mode the first time shows the settings screen rather than an
    /// empty board.
    started: bool,
}

impl Default for SandboxSettings {
    fn default() -> Self {
        Self {
            scenario: Scenario::Historical,
            open: false,
            started: false,
        }
    }
}

/// A deferred request to run the scenario's fixed setup once the board is
/// loaded. Set when a sandbox is opened; consumed by [`sandbox_auto_setup`]
/// after `apply_map_selection` has populated the `GameMap` for the scenario's
/// board (the plan resolves fixed-hex anchors against that map).
#[derive(Resource, Default)]
pub struct SandboxAutoSetup(pub Option<Scenario>);

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SandboxSettings::default())
            .insert_resource(SandboxAutoSetup::default())
            .add_systems(OnEnter(AppMode::Sandbox), open_settings_if_fresh)
            .add_systems(Update, (sandbox_escape, sandbox_auto_setup))
            .add_systems(EguiPrimaryContextPass, sandbox_settings_ui);
    }
}

/// Entering the sandbox with no session yet opens the settings screen (so the
/// player picks a scenario) rather than dropping onto an empty board.
fn open_settings_if_fresh(mut settings: ResMut<SandboxSettings>) {
    if !settings.started {
        settings.open = true;
    }
}

/// Escape re-summons the sandbox settings screen (sandbox mode only), so a new
/// sandbox can be configured without leaving the mode.
fn sandbox_escape(
    mode: Res<State<AppMode>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<SandboxSettings>,
    mut contexts: EguiContexts,
) {
    if **mode != AppMode::Sandbox {
        return;
    }
    let over_ui = contexts
        .ctx_mut()
        .map(|c| c.egui_wants_keyboard_input())
        .unwrap_or(false);
    if !over_ui && keys.just_pressed(KeyCode::Escape) {
        settings.open = true;
    }
}

/// The sandbox settings overlay: a scenario/board picker and an *Open sandbox*
/// button. Shown only in [`AppMode::Sandbox`] while `settings.open`.
fn sandbox_settings_ui(
    mut contexts: EguiContexts,
    mode: Res<State<AppMode>>,
    ctx: SandboxContext,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    let SandboxContext {
        mut settings,
        mut game_state,
        mut factions,
        loaded,
        mut pending_map,
        mut auto_setup,
        mut picker,
        placed_units,
    } = ctx;
    if **mode != AppMode::Sandbox || !settings.open {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let mut open_clicked = false;
    egui::Window::new("Sandbox")
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Set up a local sandbox -- drive both sides, place freely.")
                    .color(egui::Color32::from_gray(190)),
            );
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Scenario").strong());
            ui.horizontal(|ui| {
                for scenario in Scenario::ALL {
                    let selected = settings.scenario == scenario;
                    if ui.add(egui::Button::selectable(selected, scenario.label())).clicked() {
                        settings.scenario = scenario;
                    }
                }
            });
            ui.add_space(12.0);
            if ui
                .add(egui::Button::new(
                    egui::RichText::new("Open sandbox").size(16.0),
                ))
                .on_hover_text("Clears the current sandbox and starts fresh.")
                .clicked()
            {
                open_clicked = true;
            }
            if settings.started {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("(Escape re-opens this screen.)")
                        .weak()
                        .size(11.0),
                );
            }
        });

    if !open_clicked {
        return;
    }

    let scenario = settings.scenario;
    let map_kind = crate::map_kind_for_scenario(scenario);

    // 1. Clear the previous sandbox: fresh rules state, no faction binding
    //    (unbound = single seat may drive both sides), no placed counters.
    factions.by_peer.clear();
    game_state.0 = GameState::new(scenario);
    // Attach the scenario's board to the engine synchronously, so movement
    // costing / ZOC never see an empty board before the visual load (mirrors the
    // networked StartGame handler).
    game_state.0.board = omdurman_rules::board::BoardInfo::from_map_data(loaded.0.map(map_kind));
    for entity in &placed_units {
        commands.entity(entity).despawn();
    }
    picker.reset_available();

    // 2. Request the visual board load, and switch to the in-game view.
    pending_map.0 = Some(map_kind);
    next_app_state.set(AppState::InGame);

    // 3. Defer the fixed scenario setup until the board is loaded (the plan
    //    resolves anchors against the map, which loads next frame).
    auto_setup.0 = Some(scenario);

    settings.started = true;
    settings.open = false;
    info!(%scenario, "opened sandbox");
}

/// Once the sandbox's board has loaded, feed the scenario's fixed-hex setup
/// placements as ordinary local `PlaceUnit` events (the same plan the game's
/// "Set up scenario" button uses). Runs after `apply_map_selection`, so the
/// `GameMap` is populated for the scenario's board before the plan is built.
fn sandbox_auto_setup(
    mut auto_setup: ResMut<SandboxAutoSetup>,
    game_map: Res<GameMap>,
    mut pending: ResMut<PendingEdits>,
) {
    let Some(scenario) = auto_setup.0 else {
        return;
    };
    // Wait for the board to be present (map load is deferred a frame).
    if game_map.hexes.is_empty() {
        return;
    }
    let plan = crate::scenario_setup::build_setup_plan(scenario, &game_map);
    for ev in plan.placements {
        pending.outgoing_broadcast.push(NetMsg::Game(ev));
    }
    if !plan.unresolved.is_empty() {
        warn!(
            unresolved = ?plan.unresolved,
            "sandbox auto-setup: some fixed placements could not be resolved on this board"
        );
    }
    auto_setup.0 = None;
}
