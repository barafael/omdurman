use crate::board_click::FireClick;
use crate::dispatch::Dispatches;
use crate::fire::{fire_group_kinds, fire_selection};
use crate::peers::Peers;
use crate::picker::{PickerState, PlacedUnit};
use crate::{GameRng, GameStateResource};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::GameEvent;
use omdurman_rules::effects::GameEffect;
use omdurman_rules::{FireAttack, FireKind, Phase};

/// Tracks fire allocations before batch resolution (§6.41).
/// Resets each fire sub-phase (see [`reset_fire_allocation_on_phase_change`]).
#[derive(Resource, Default)]
pub struct FireAllocationState {
    /// Allocated attacks built when the player clicks valid targets.
    pub attacks: Vec<FireAttack>,
    /// True once "Fire" has been triggered — locks further changes
    /// until the fire phase changes and the state resets.
    pub committed: bool,
    /// Set by the UI panel; consumed by [`execute_fire_allocations`].
    pub execute_requested: bool,
    /// Whether the resolution overlay is open. Auto-opens on the first
    /// allocation; the player can toggle it with the actions panel's
    /// "Review allocations" button (Esc / right-click closes it).
    pub panel_open: bool,
}

/// Marker for one persistent red arrow drawn from an allocated attack's firer
/// hex to its target hex (§6.41 allocation preview). Rebuilt each frame from
/// [`FireAllocationState`], so removing/executing an allocation clears its
/// arrow immediately.
#[derive(Component)]
pub struct AllocationArrow;

/// True while the engine is in a fire sub-phase.
fn in_fire_phase(gs: &GameStateResource) -> bool {
    matches!(
        gs.0.phase,
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_)
    )
}

/// Replace the old per-click fire resolution: build a `FireAttack` and store
/// it in the allocation list instead of pre-rolling and broadcasting.
///
/// The attack set comes from the fire selection ([`fire_selection`]): the
/// combat tile selection (double-click on the hex) allocates every armed
/// unit that sees the target, split into one attack per weapon kind
/// (§6.14/§6.42); a single-clicked counter allocates exactly its own fire
/// (§6.13/§6.15). The selection is deliberately *kept* after allocating, so
/// the target rings, the direction arrow, and the pending-allocation arrows
/// all stay around while the player reviews or adds further shots; clicking
/// the group's own hex dismisses it.
pub fn handle_fire_allocation_click(
    mut clicks: bevy::ecs::message::MessageReader<FireClick>,
    state: ResMut<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    peers: Peers,
    mut allocation: ResMut<FireAllocationState>,
    mut dispatches: ResMut<Dispatches>,
) {
    // Routed by `board_click::route_board_clicks` (fire-phase release).
    let Some(&FireClick(target)) = clicks.read().last() else {
        return;
    };
    let Some(gs) = game_state else { return };
    if !in_fire_phase(&gs) {
        return;
    }
    let firing_player = gs.0.phase_player();
    if !peers.may_act(firing_player) {
        return;
    }
    let Some(group) = fire_selection(&state, &placed_units, &gs.0) else {
        return;
    };
    // A release on a friendly or empty hex (e.g. the selecting click itself)
    // is not an attempted shot: stay quiet about it below.
    let enemy_there =
        gs.0.units.iter().any(|u| {
            u.position == target && u.profile.identity.owner() == firing_player.opponent()
        });
    if allocation.committed {
        // §6.41: a sub-phase's fire is allocated, then resolved -- after the
        // resolution nothing more may be allocated. Say so rather than
        // ignore the shot.
        if enemy_there {
            dispatches.push(
                "Fire Allocation",
                format!(
                    "Fire refused — {}.",
                    crate::actions_panel::FIRE_ALREADY_RESOLVED_HINT
                ),
            );
        }
        return;
    }
    let kinds = fire_group_kinds(&gs.0, &group);
    if kinds.is_empty() {
        return;
    }

    let attacks = crate::fire::click_attacks(&gs.0, &group, &kinds, target, &allocation.attacks);
    if attacks.is_empty() {
        if !enemy_there {
            return;
        }
        // No unit of the group may fire at this hex: say why (the engine's
        // reason for the first unit -- out of range, no line of sight, a
        // wrong weapon for the target...) instead of ignoring the click.
        let Some(&(firer, kind)) = kinds.first() else {
            return;
        };
        let reason = match gs.0.can_fire_at(firer, target, kind) {
            Err(omdurman_rules::effects::RuleError::LineOfSightBlocked(_, _)) => {
                "no line of sight (§6.3)".to_string()
            }
            Err(error) => error.to_string(),
            Ok(()) => "the selected units cannot fire there".to_string(),
        };
        dispatches.push("Field Telegraph", format!("Fire refused — {reason}."));
        return;
    }

    for attack in &attacks {
        // §6.13/§6.14: a unit fires once per phase -- refuse a group any of
        // whose units is already allocated.
        if crate::fire::uses_allocated_weapon(attack, &allocation.attacks) {
            dispatches.push(
                "Fire Allocation",
                "These units have already allocated their fire.",
            );
            continue;
        }
        // §6.14: a hex is fired at once per phase, so fire at an already
        // targeted hex joins that attack instead of opening a second one
        // (which the engine would refuse at resolution).
        if let Some(existing) = allocation
            .attacks
            .iter_mut()
            .find(|a| a.target_hex == attack.target_hex && a.kind == attack.kind)
            && let Some(combined) =
                omdurman_rules::effects::combine_fire_attacks(&gs.0, existing, attack)
        {
            *existing = combined;
            continue;
        }
        allocation.attacks.push(attack.clone());
    }
    let allocated = allocation.attacks.len();
    if allocated == 0 {
        return;
    }
    allocation.panel_open = true;

    let kind_str = match attacks[0].kind {
        FireKind::Direct if attacks[0].firers.is_empty() => "Gunboat Maxim fire",
        FireKind::MaximSecondFire if attacks[0].firers.is_empty() => "Gunboat Maxim second fire",
        FireKind::Direct => "Direct fire",
        FireKind::MaximSecondFire => "Maxim second fire",
        FireKind::Howitzer => "Howitzer",
    };
    let n = allocated;
    dispatches.push(
        "Fire Allocation",
        format!(
            "{kind_str} allocated to {}. {n} attack{} pending.",
            target_label(&gs.0, target),
            if n == 1 { "" } else { "s" },
        ),
    );
}

/// A player-readable name for a fire target hex: the units standing there
/// (at most two named), else the landmark, else the terrain — followed by the
/// coordinate for cross-reference with the board.
pub(crate) fn target_label(
    gs: &omdurman_rules::effects::GameState,
    hex: omdurman_types::HexCoord,
) -> String {
    // The units fire can hit: an Anglo-Egyptian leader is never a fire
    // casualty (§6.51, §9.346), so naming GORDON as a target misleads.
    let names: Vec<String> = gs
        .units
        .iter()
        .filter(|u| {
            u.position == hex
                && !matches!(
                    u.profile.kind,
                    omdurman_types::UnitKind::BritishLeader { .. }
                )
        })
        .map(|u| u.profile.identity.label_in(gs.scenario))
        .collect();
    let what = match names.len() {
        0 => gs
            .board
            .location_at(hex)
            .map(|l| l.to_string())
            .or_else(|| gs.board.terrain_at(hex).map(|t| t.to_string()))
            .unwrap_or_else(|| "hex".to_string()),
        1 | 2 => names.join(", "),
        n => format!("{}, {} +{} more", names[0], names[1], n - 2),
    };
    format!("{what} ({}, {})", hex.q, hex.r)
}

/// egui panel showing the current allocation list with remove buttons and
/// a "Fire" button. Each attack is one row (scrollable when the list
/// is long) with per-firer factors and the specific die modifiers (§6.24,
/// §5.54, §6.23, §9.231/§9.232) that attack carries.
pub fn fire_allocation_review_ui(
    mut contexts: EguiContexts,
    mode: Res<State<crate::AppMode>>,
    game_state: Option<Res<GameStateResource>>,
    mut allocation: ResMut<FireAllocationState>,
    peers: Peers,
) {
    if !mode.is_play() {
        return;
    }
    if !allocation.panel_open || allocation.committed {
        return;
    }
    let Some(gs) = game_state else { return };
    if !matches!(
        gs.0.phase,
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_)
    ) {
        return;
    }
    let firing_player = gs.0.phase_player();
    if !peers.may_act(firing_player) {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    let mut remove_self: Option<usize> = None;
    let mut toggle_aim: Option<usize> = None;

    crate::ui::anchored_card(
        ctx,
        egui::Id::new("fire_allocation_panel"),
        egui::Align2::CENTER_BOTTOM,
        egui::Vec2::new(0.0, -100.0),
        crate::ui::frames::card(crate::ui::palette::CARD_ALLOCATION),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(13.0));

            ui.horizontal(|ui| {
                ui.colored_label(
                    crate::ui::palette::CARD_TITLE_TAN,
                    format!(
                        "Fire combat resolutions  ({} pending)",
                        allocation.attacks.len()
                    ),
                );
                if ui
                    .add(egui::Button::new("✕").min_size(egui::Vec2::splat(16.0)))
                    .clicked()
                {
                    allocation.panel_open = false;
                }
            });

            ui.add_space(4.0);

            if allocation.attacks.is_empty() {
                ui.colored_label(
                    crate::ui::palette::TEXT_DIM,
                    "No fires allocated yet — select a unit or double-click a hex, then click a red-ringed target.",
                );
            } else {
                egui::ScrollArea::vertical()
                    .max_height(320.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for (i, attack) in allocation.attacks.iter().enumerate() {
                            draw_allocation_row(
                                ui,
                                &gs.0,
                                attack,
                                &mut remove_self,
                                &mut toggle_aim,
                                i,
                            );
                            ui.add_space(4.0);
                        }
                    });
            }

            ui.add_space(6.0);
            // §6.41: the sub-phase's fire resolves together -- a player who
            // resolved one attack to see how it went found the rest refused.
            if !allocation.attacks.is_empty() {
                ui.label(crate::ui::text::note(
                    "All of this sub-phase's fire resolves at once: stage every attack first.",
                ));
            }

            if !allocation.attacks.is_empty()
                && ui
                    .add(
                        egui::Button::new(format!(
                            "Resolve {} attack{}",
                            allocation.attacks.len(),
                            if allocation.attacks.len() == 1 {
                                ""
                            } else {
                                "s"
                            }
                        ))
                        .fill(crate::ui::palette::BTN_GO)
                        .min_size(egui::Vec2::new(120.0, 28.0)),
                    )
                    .on_hover_ui(|ui| {
                        crate::rulebook::refs_label(
                            ui,
                            "Roll and resolve every allocated attack (§6.41)",
                            ui.visuals().text_color(),
                            12.0,
                        );
                    })
                    .clicked()
            {
                allocation.execute_requested = true;
            }
        },
    );

    if let Some(idx) = remove_self {
        allocation.attacks.remove(idx);
    } else if let Some(idx) = toggle_aim
        && let Some(attack) = allocation.attacks.get_mut(idx)
    {
        // §6.54/§6.62: artillery at a garrisoned fort fires at the fort or
        // at the units inside it.
        if attack.at_fort {
            attack.at_fort = false;
        } else if let Some(aimed) = omdurman_rules::effects::aim_at_fort(&gs.0, attack) {
            *attack = aimed;
        }
    }
}

/// One allocation row: firers (with factors) → target, kind, range band, the
/// net die modifier (mandatory set + which sections contribute), and the CRT
/// row. Keep it dense enough to scan several attacks at once.
fn draw_allocation_row(
    ui: &mut egui::Ui,
    gs: &omdurman_rules::effects::GameState,
    attack: &FireAttack,
    remove_self: &mut Option<usize>,
    toggle_aim: &mut Option<usize>,
    index: usize,
) {
    let kind = match attack.kind {
        FireKind::Direct => "Direct",
        FireKind::MaximSecondFire => "Maxim 2nd",
        FireKind::Howitzer => "Howitzer",
    };
    // Each weapon with its printed factor; a named gunboat's Maxims (§2.32)
    // are a weapon of their own.
    let names: Vec<String> = attack
        .shots()
        .into_iter()
        .filter_map(|shot| {
            let factor = gs.find_unit(shot.unit)?.fire_factor(shot.mount)?.value();
            Some(format!(
                "{} ({factor})",
                crate::combat_ui::shot_name(shot, Some(gs))
            ))
        })
        .collect();
    // Every firer's own range: a merged attack can mix range 1 and 3.
    let ranges: Vec<u32> = attack
        .all_firing_units()
        .iter()
        .filter_map(|id| gs.find_unit(*id))
        .map(|u| u.position.distance(attack.target_hex))
        .collect();
    let range_text = match (ranges.iter().min(), ranges.iter().max()) {
        (Some(1), Some(1)) => "1 hex".to_string(),
        (Some(lo), Some(hi)) if lo == hi => format!("{lo} hexes"),
        (Some(lo), Some(hi)) => format!("{lo}\u{2013}{hi} hexes"),
        _ => "?".to_string(),
    };
    // The engine adds the target's terrain and hexside defence (§6.23).
    let (terrain_mod, hexside_mod) = crate::fire::target_defence_modifiers(gs, attack);
    let net = attack.net_modifier() + terrain_mod + hexside_mod;

    let (mod_text, mod_color) = {
        // The canonical modifier wording, shared with the combat card.
        let mut parts: Vec<String> = attack
            .modifiers
            .iter()
            .map(|m| {
                let line = crate::combat_ui::describe_fire_modifier(*m);
                format!("{} (§{})", line.label, line.paragraph)
            })
            .collect();
        if terrain_mod != 0 {
            parts.push(format!("{terrain_mod:+} terrain defence (§6.23)"));
        }
        if hexside_mod != 0 {
            parts.push(format!("{hexside_mod:+} hexside (§6.23)"));
        }
        let text = if parts.is_empty() {
            "no modifiers".to_string()
        } else {
            parts.join("  ·  ")
        };
        let color = if net > 0 {
            crate::ui::palette::FAVOURABLE
        } else if net < 0 {
            crate::ui::palette::UNFAVOURABLE
        } else {
            crate::ui::palette::TEXT_SOFT
        };
        (text, color)
    };

    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.set_min_height(20.0);
            if ui
                .add(
                    egui::Button::new("×")
                        .fill(crate::ui::palette::BTN_DANGER)
                        .min_size(egui::Vec2::splat(18.0)),
                )
                .clicked()
            {
                *remove_self = Some(index);
            }
            ui.colored_label(
                crate::ui::palette::PANEL_TEXT,
                format!(
                    "{names_c} \u{2192} {}",
                    target_label(gs, attack.target_hex),
                    names_c = names.join(" + "),
                ),
            );
        });
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("  {kind}  ·  range {range_text}"))
                    .color(crate::ui::palette::TEXT_SOFT)
                    .size(12.0),
            );
            ui.label(
                egui::RichText::new(format!("net die {net:+}"))
                    .color(mod_color)
                    .size(12.0)
                    .strong(),
            );
        });
        crate::rulebook::refs_rich(ui, &format!("    {mod_text}"), 11.0, |t| {
            t.color(crate::ui::palette::PANEL_DIM).monospace()
        });
        // A garrisoned enemy fort: the artillery chooses its target.
        let garrisoned = gs
            .player_units_in_hex(attack.target_hex, attack.firing_player.opponent())
            .iter()
            .any(|u| !matches!(u.profile.kind, omdurman_types::UnitKind::Fort { .. }));
        if garrisoned && omdurman_rules::effects::aim_at_fort(gs, attack).is_some() {
            let label = if attack.at_fort {
                "Aimed at the fort (§6.62) -- aim at the garrison instead"
            } else {
                "Aimed at the garrison, -3 (§6.54) -- aim at the fort instead"
            };
            if ui.small_button(label).clicked() {
                *toggle_aim = Some(index);
            }
        }
    });
}

/// Consume the [`FireAllocationState`] list: pre-roll dice for every
/// allocated attack and broadcast the corresponding [`GameEffect`].
/// Runs once when the player clicks "Resolve N attacks". Each attack is
/// pre-validated against the engine (in order, over the earlier ones); a
/// refused attack is reported on a slip and not sent.
pub fn execute_fire_allocations(
    mut allocation: ResMut<FireAllocationState>,
    mut rng: Option<ResMut<GameRng>>,
    gs: Option<Res<GameStateResource>>,
    mut submit: crate::submit::CheckedSubmit,
) {
    if !allocation.execute_requested || allocation.committed {
        return;
    }
    let Some(rng) = rng.as_mut() else { return };
    let Some(gs) = gs else { return };

    allocation.committed = true;

    let attacks = std::mem::take(&mut allocation.attacks);
    let mut sent = 0usize;

    for attack in &attacks {
        // Firers that vanished since allocation (eliminated mid-phase) are
        // skipped; the engine re-validates every attack on the echo anyway.
        if attack
            .all_firing_units()
            .first()
            .is_none_or(|id| gs.0.find_unit(*id).is_none())
        {
            continue;
        }
        let disruption = rng.disruption_draw();

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
            sent += usize::from(submit.submit(
                &gs.0,
                GameEvent::Effect(GameEffect::HowitzerFire {
                    attack: attack.clone(),
                    combat_results_table_roll,
                    impact_roll,
                    disruption,
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
            sent += usize::from(submit.submit(
                &gs.0,
                GameEvent::Effect(GameEffect::FireCombat {
                    attack: attack.clone(),
                    roll,
                    disruption,
                }),
            ));
        }
    }

    // The results arrive as Combat Resolution Cards; a refused attack has
    // already posted its reason.
    let _ = sent;

    allocation.execute_requested = false;
}

/// Persistent orange arrows — one per allocated [`FireAttack`], from its
/// firer hex to its target hex — so the player sees *which* shots are pending
/// (§6.41). Same bold-orange look as the melee direction arrows and the hover
/// preview arrow: one visual language for combat targeting. Rebuilt from the
/// allocation list each frame (one arrow mesh per attack at most), so
/// removing/executing an allocation drops its arrow immediately. Runs in the
/// fire `GameSet`; exit the fire phase and they vanish with the rest of the
/// gameplay overlays.
pub fn fire_allocation_arrows(
    mut commands: Commands,
    render: crate::DirectionArrowCtx,
    existing: Query<Entity, With<AllocationArrow>>,
    allocation: Res<FireAllocationState>,
    gs: Option<Res<GameStateResource>>,
) {
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);
    let Some(gs) = gs else { return };
    if allocation.committed || !in_fire_phase(&gs) {
        return;
    }
    for attack in &allocation.attacks {
        let Some(unit) = attack
            .all_firing_units()
            .first()
            .and_then(|id| gs.0.find_unit(*id))
        else {
            continue;
        };
        // The shared arrow mesh (tail at the firer, head at the target) --
        // not the hex-ring mesh, which stretched into a double-pointed
        // hexagon and read as fire in both directions.
        crate::combat_ui::direction_arrow(
            &mut commands,
            &render,
            unit.position,
            attack.target_hex,
            AllocationArrow,
        );
    }
}

/// Reset the allocation state whenever the engine phase changes, so an
/// unexecuted allocation from one fire sub-phase never leaks into the next
/// (§6.41 allocations are per fire sub-phase). Uses a `Local` snapshot of the
/// phase so the reset also fires correctly on replay / snapshot convergence,
/// where no local click precedes the phase advance. Releases the
/// `committed` lock set by [`execute_fire_allocations`].
pub fn reset_fire_allocation_on_phase_change(
    game_state: Option<Res<GameStateResource>>,
    mut allocation: ResMut<FireAllocationState>,
    mut last_phase: Local<Option<Phase>>,
) {
    let Some(gs) = game_state else { return };
    let phase = gs.0.phase;
    if *last_phase != Some(phase) {
        if last_phase.is_some() {
            allocation.attacks.clear();
            allocation.committed = false;
            allocation.execute_requested = false;
        }
        *last_phase = Some(phase);
    }
}
