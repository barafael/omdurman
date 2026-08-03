use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::{GameEvent, NetMsg};
use omdurman_rules::effects::GameEffect;
use omdurman_rules::{FireAttack, FireKind, Phase};
use crate::dispatch::Dispatches;
use crate::fire::{build_fire_attack, fire_kind_for};
use crate::input::CombatClickCtx;
use crate::peers::Peers;
use crate::picker::{PickerState, PlacedUnit, selected_unit_id};
use crate::{GameRng, GameStateResource, PendingEdits};

/// Tracks fire allocations before batch resolution (§6.41).
/// Resets each fire sub-phase.
#[derive(Resource, Default)]
pub struct FireAllocationState {
    /// Allocated attacks built when the player clicks valid targets.
    pub attacks: Vec<FireAttack>,
    /// True once "Execute All" has been triggered — locks further changes.
    pub committed: bool,
    /// Set by the UI panel; consumed by [`execute_fire_allocations`].
    pub execute_requested: bool,
}

/// Replace the old per-click fire resolution: build a `FireAttack` and store
/// it in the allocation list instead of pre-rolling and broadcasting.
pub fn handle_fire_allocation_click(
    mut click: CombatClickCtx,
    mut state: ResMut<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    peers: Peers,
    mut allocation: ResMut<FireAllocationState>,
    mut dispatches: ResMut<Dispatches>,
) {
    let Some(target) = click.clicked_hex() else {
        return;
    };
    let Some(gs) = game_state else { return };
    if !matches!(
        gs.0.phase,
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_)
    ) {
        return;
    }
    if allocation.committed {
        return;
    }
    let firing_player = match gs.0.phase {
        Phase::OffensiveFire(_) => gs.0.active_player,
        Phase::DefensiveFire(_) => gs.0.active_player.opponent(),
        _ => return,
    };
    if !peers.may_act(firing_player) {
        return;
    }
    let Some((firer, firer_hex)) = selected_unit_id(&state, &placed_units) else {
        return;
    };
    let Some(kind) = fire_kind_for(&gs.0, firer) else {
        return;
    };

    match gs.0.can_fire_at(firer, target, kind) {
        Ok(()) => {}
        Err(omdurman_rules::effects::RuleError::LineOfSightBlocked(_, _)) => {
            dispatches.push("Field Telegraph", "Fire refused — no line of sight (§6.3).");
            return;
        }
        Err(_) => return,
    }

    let Some(attack) = build_fire_attack(&gs.0, firer, firer_hex, target, kind) else {
        return;
    };

    // Skip if these firers already allocated.
    if allocation.attacks.iter().any(|a| a.firers == attack.firers) {
        dispatches.push("Fire Allocation", "These units have already allocated their fire.");
        *state = PickerState::Idle;
        return;
    }

    allocation.attacks.push(attack);

    let kind_str = match kind {
        FireKind::Direct => "Direct fire",
        FireKind::MaximSecondFire => "Maxim second fire",
        FireKind::Howitzer => "Howitzer",
    };
    let n = allocation.attacks.len();
    dispatches.push(
        "Fire Allocation",
        format!(
            "{kind_str} allocated to ({}, {}). {n} attack{} pending.",
            target.q,
            target.r,
            if n == 1 { "" } else { "s" },
        ),
    );

    *state = PickerState::Idle;
}

/// egui panel showing the current allocation list with remove buttons and
/// an "Execute All" button.
pub fn fire_allocation_review_ui(
    mut contexts: EguiContexts,
    mode: Res<State<crate::AppMode>>,
    game_state: Option<Res<GameStateResource>>,
    mut allocation: ResMut<FireAllocationState>,
    _placed_units: Query<(Entity, &PlacedUnit)>,
    peers: Peers,
) {
    if !mode.is_play() {
        return;
    }
    if allocation.attacks.is_empty() || allocation.committed {
        return;
    }
    let Some(gs) = game_state else { return };
    if !matches!(
        gs.0.phase,
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_)
    ) {
        return;
    }
    let firing_player = match gs.0.phase {
        Phase::OffensiveFire(_) => gs.0.active_player,
        Phase::DefensiveFire(_) => gs.0.active_player.opponent(),
        _ => return,
    };
    if !peers.may_act(firing_player) {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    let mut remove_self: Option<usize> = None;

    egui::Area::new(egui::Id::new("fire_allocation_panel"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, -100.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 40, 220))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.style_mut().override_font_id =
                        Some(egui::FontId::proportional(13.0));

                    ui.colored_label(
                        egui::Color32::from_rgb(200, 180, 140),
                        format!(
                            "Fire Allocations  ({} pending)",
                            allocation.attacks.len()
                        ),
                    );

                    ui.add_space(4.0);

                    for (i, attack) in allocation.attacks.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.set_min_height(20.0);
                            if ui
                                .add(
                                    egui::Button::new("×")
                                        .fill(egui::Color32::from_rgb(80, 30, 30))
                                        .min_size(egui::Vec2::splat(18.0)),
                                )
                                .clicked()
                            {
                                remove_self = Some(i);
                            }

                            let names: Vec<String> = attack
                                .firers
                                .iter()
                                .filter_map(|id| gs.0.find_unit(*id))
                                .map(|u| u.profile.identity.short_label())
                                .collect();
                            let kind = match attack.kind {
                                FireKind::Direct => "Dir",
                                FireKind::MaximSecondFire => "Max2",
                                FireKind::Howitzer => "How",
                            };
                            ui.colored_label(
                                egui::Color32::from_rgb(180, 180, 180),
                                format!(
                                    "{} → ({},{})  [{}]  net {:+}",
                                    names.join("+"),
                                    attack.target_hex.q,
                                    attack.target_hex.r,
                                    kind,
                                    attack.net_modifier(),
                                ),
                            );
                        });
                    }

                    ui.add_space(6.0);

                    if ui
                        .add(
                            egui::Button::new("Execute All")
                                .fill(egui::Color32::from_rgb(60, 80, 40))
                                .min_size(egui::Vec2::new(120.0, 28.0)),
                        )
                        .clicked()
                    {
                        allocation.execute_requested = true;
                    }
                });
        });

    if let Some(idx) = remove_self {
        allocation.attacks.remove(idx);
    }
}

/// Consume the [`FireAllocationState`] list: pre-roll dice for every
/// allocated attack and broadcast the corresponding [`GameEffect`].
/// Runs once when the player clicks "Execute All".
pub fn execute_fire_allocations(
    mut allocation: ResMut<FireAllocationState>,
    mut rng: Option<ResMut<GameRng>>,
    gs: Option<Res<GameStateResource>>,
    mut pending: ResMut<PendingEdits>,
    mut dispatches: ResMut<Dispatches>,
) {
    if !allocation.execute_requested || allocation.committed {
        return;
    }
    let Some(rng) = rng.as_mut() else { return };
    let Some(gs) = gs else { return };

    allocation.committed = true;

    let attacks = std::mem::take(&mut allocation.attacks);

    for attack in &attacks {
        let Some(firer_unit) = gs.0.find_unit(attack.firers[0]) else {
            continue;
        };
        let _firer_hex = firer_unit.position;

        let mut d10 = || rng.roll_d10();

        if attack.kind == FireKind::Howitzer {
            let combat_results_table_roll = d10();
            let impact_roll = d10();
            info!(
                target.q = attack.target_hex.q,
                target.r = attack.target_hex.r,
                crt_roll = %combat_results_table_roll,
                impact = %impact_roll,
                "howitzer fire (batch)",
            );
            pending.outgoing_broadcast.push(NetMsg::Game(
                GameEvent::Effect(GameEffect::HowitzerFire {
                    attack: attack.clone(),
                    combat_results_table_roll,
                    impact_roll,
                }),
            ));
        } else {
            let roll = d10();
            info!(
                target.q = attack.target_hex.q,
                target.r = attack.target_hex.r,
                roll = %roll,
                "fire (batch)",
            );
            pending.outgoing_broadcast.push(NetMsg::Game(
                GameEvent::Effect(GameEffect::FireCombat {
                    attack: attack.clone(),
                    roll,
                }),
            ));
        }
    }

    dispatches.push(
        "Fire Allocation",
        format!("Executed {} fire attack{s}.", attacks.len(), s = if attacks.len() == 1 { "" } else { "s" }),
    );

    allocation.committed = false;
    allocation.execute_requested = false;
}
