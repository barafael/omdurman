use crate::board_click::FireClick;
use crate::dispatch::Dispatches;
use crate::fire::{fire_group_kinds, fire_selection};
use crate::peers::Peers;
use crate::picker::{PickerState, PlacedUnit};
use crate::{GameRng, GameStateResource};
use bevy::prelude::*;
use bevy_egui::egui;
use omdurman_net::GameEvent;
use omdurman_rules::effects::{FireOrder, GameEffect};
use omdurman_rules::{FireAttack, FireKind, Phase};
use omdurman_types::HexCoord;

/// Tracks fire allocations before batch resolution (§6.41).
/// Resets each fire sub-phase (see [`reset_fire_allocation_on_phase_change`]).
#[derive(Resource, Default)]
pub struct FireAllocationState {
    /// Allocated attacks built when the player clicks valid targets.
    pub attacks: Vec<FireAttack>,
    /// Artillery shots at a wall (§6.63) or the river chain (§10.23 b)
    /// staged from the Artillery Breach card: fire like any other, so they
    /// are allocated and resolved with the attacks (§6.41).
    pub specials: Vec<FireOrder>,
    /// True once "Fire" has been triggered — locks further changes
    /// until the fire phase changes and the state resets.
    pub committed: bool,
    /// This peer resolves the side's allocated fire, one order after the
    /// other ([`continue_fire_resolution`]): a result may wait on its
    /// owner's choice of losses (§7.7), and nothing else is accepted until
    /// it is made.
    pub resolving: bool,
    /// Set by the UI panel; consumed by [`execute_fire_allocations`].
    pub execute_requested: bool,
    /// Whether the resolution overlay is open. Auto-opens on the first
    /// allocation; the player can toggle it with the actions panel's
    /// "Review allocations" button (Esc / right-click closes it).
    pub panel_open: bool,
}

impl FireAllocationState {
    /// How many orders are staged.
    pub fn staged(&self) -> usize {
        self.attacks.len() + self.specials.len()
    }

    /// Every staged order, the attacks first.
    pub fn orders(&self) -> Vec<FireOrder> {
        self.attacks
            .iter()
            .cloned()
            .map(FireOrder::Attack)
            .chain(self.specials.iter().cloned())
            .collect()
    }

    /// Whether `unit` is already staged to fire (§6.13: once a sub-phase).
    pub fn stages_unit(&self, unit: omdurman_rules::UnitId) -> bool {
        self.orders()
            .iter()
            .any(|o| o.shots().iter().any(|s| s.unit == unit))
    }

    /// Drop everything staged.
    fn clear_staged(&mut self) {
        self.attacks.clear();
        self.specials.clear();
    }
}

/// Marker for one persistent red arrow drawn from an allocated attack's firer
/// hex to its target hex (§6.41 allocation preview). Rebuilt each frame from
/// [`FireAllocationState`], so removing/executing an allocation clears its
/// arrow immediately.
#[derive(Component)]
pub struct AllocationArrow;

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
    if !gs.0.phase.is_fire() {
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
            dispatches.refuse(
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
        dispatches.refuse("Field Telegraph", format!("Fire refused — {reason}."));
        return;
    }

    for attack in &attacks {
        // §6.13/§6.14: a unit fires once per phase -- refuse a group any of
        // whose units is already allocated.
        // A stack one of whose units is already allocated sends the rest
        // (§6.15: a stack may be divided, so the player may stage one
        // battalion and then decide the others join it).
        let rest;
        let in_special = |unit: &omdurman_rules::UnitId| {
            allocation
                .specials
                .iter()
                .any(|o| o.shots().iter().any(|s| s.unit == *unit))
        };
        let attack = if crate::fire::uses_allocated_weapon(attack, &allocation.attacks)
            || attack.all_firing_units().iter().any(in_special)
        {
            let free: Vec<omdurman_rules::UnitId> = attack
                .firers
                .iter()
                .copied()
                .filter(|f| {
                    !allocation.attacks.iter().any(|a| a.firers.contains(f)) && !in_special(f)
                })
                .collect();
            let rebuilt = free
                .first()
                .and_then(|f| gs.0.find_unit(*f))
                .filter(|_| attack.gunboat_maxims.is_empty())
                .and_then(|u| {
                    omdurman_rules::effects::build_fire_attack_from(
                        &gs.0,
                        u.position,
                        &free,
                        attack.target_hex,
                        attack.kind,
                    )
                });
            let Some(rebuilt) = rebuilt else {
                dispatches.refuse(
                    "Fire Allocation",
                    "These units have already allocated their fire.",
                );
                continue;
            };
            rest = rebuilt;
            &rest
        } else {
            attack
        };
        // §6.14: a hex is fired at once per phase, so fire at an already
        // targeted hex joins that attack instead of opening a second one
        // (which the engine would refuse at resolution) -- Maxim second
        // fire joining a shell makes one howitzer attack (§6.42).
        let joined = allocation.attacks.iter_mut().find_map(|existing| {
            let combined = (existing.target_hex == attack.target_hex)
                .then(|| omdurman_rules::effects::combine_fire_attacks(&gs.0, existing, attack))
                .flatten()?;
            Some((existing, combined))
        });
        if let Some((existing, combined)) = joined {
            *existing = combined;
            continue;
        }
        allocation.attacks.push(attack.clone());
    }
    if allocation.attacks.is_empty() {
        return;
    }
    // The staged list in the rail is the confirmation (opened here); a
    // toast saying the same ("Direct fire allocated to ... 1 attack
    // pending") only covered the board's corner.
    allocation.panel_open = true;
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
    let names = crate::combat_ui::tally_names(names);
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
    format!("{what} {hex}")
}

/// The staged attacks, drawn in the command rail under the "Next step"
/// heading: one row per attack with a remove button, per-firer factors and
/// the specific die modifiers (§6.24, §5.54, §6.23, §9.231/§9.232) it
/// carries, then the "Resolve" button. In the rail, not on a card floating
/// over the board: a card there covered the fight it listed and swallowed
/// the clicks meant for the hexes under it.
pub(crate) fn draw_staged_attacks(
    ui: &mut egui::Ui,
    gs: &omdurman_rules::effects::GameState,
    allocation: &mut FireAllocationState,
) {
    let mut row_action: Option<(usize, RowAction)> = None;
    let mut remove_special: Option<usize> = None;
    let count = allocation.attacks.len();
    ui.scope(|ui| {
        ui.style_mut().override_font_id = Some(egui::FontId::proportional(13.0));
        for (i, attack) in allocation.attacks.iter().enumerate() {
            if let Some(action) = draw_allocation_row(ui, gs, attack, i, count) {
                row_action = Some((i, action));
            }
            ui.add_space(4.0);
        }
        for (i, order) in allocation.specials.iter().enumerate() {
            draw_special_row(ui, gs, order, &mut remove_special, i);
            ui.add_space(4.0);
        }
        // §6.41: the sub-phase's fire resolves together -- a player who
        // resolved one attack to see how it went found the rest refused.
        ui.label(crate::ui::text::note(
            "All of this sub-phase's fire resolves at once: stage every attack first.",
        ));
        if ui
            .add(
                egui::Button::new(format!(
                    "Resolve {} attack{} (Enter)",
                    allocation.staged(),
                    if allocation.staged() == 1 { "" } else { "s" }
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
    });

    if let Some(idx) = remove_special {
        allocation.specials.remove(idx);
    }
    let Some((idx, action)) = row_action else {
        return;
    };
    match action {
        RowAction::Remove => {
            allocation.attacks.remove(idx);
        }
        RowAction::Earlier if idx > 0 => allocation.attacks.swap(idx, idx - 1),
        RowAction::Later if idx + 1 < allocation.attacks.len() => {
            allocation.attacks.swap(idx, idx + 1)
        }
        RowAction::Earlier | RowAction::Later => {}
        RowAction::ToggleAim => {
            let Some(attack) = allocation.attacks.get_mut(idx) else {
                return;
            };
            // §6.54/§6.62: artillery at a garrisoned fort fires at the fort
            // or at the units inside it.
            if attack.at_fort {
                attack.at_fort = false;
            } else if let Some(aimed) = omdurman_rules::effects::aim_at_fort(gs, attack) {
                *attack = aimed;
            }
        }
    }
}

/// One staged artillery shot at a wall (§6.63) or the river chain (§10.23
/// b): its batteries and its target.
fn draw_special_row(
    ui: &mut egui::Ui,
    gs: &omdurman_rules::effects::GameState,
    order: &FireOrder,
    remove_self: &mut Option<usize>,
    index: usize,
) {
    let (firers, target) = match order {
        FireOrder::BreachWall { firers, target } => (
            firers,
            format!("the wall between {} and {} (§6.63)", target.a, target.b),
        ),
        FireOrder::SinkChain { firers } => (firers, "the river chain (§10.23)".to_string()),
        FireOrder::Attack(_) => return,
    };
    let names = crate::combat_ui::tally_names(
        firers
            .iter()
            .map(|&id| crate::combat_ui::unit_name(id, Some(gs)))
            .collect(),
    );
    ui.group(|ui| {
        ui.horizontal(|ui| {
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
            crate::rulebook::refs_rich(
                ui,
                &format!("{} \u{2192} {target}", names.join(" + ")),
                13.0,
                |t| t.color(crate::ui::palette::PANEL_TEXT),
            );
        });
    });
}

/// One allocation row: firers (with factors) → target, kind, range band, the
/// net die modifier (mandatory set + which sections contribute), and the CRT
/// row. Keep it dense enough to scan several attacks at once.
/// What a click in an allocation row asks for.
pub(crate) enum RowAction {
    Remove,
    /// Toggle its aim at a garrisoned fort: the fort or the garrison.
    ToggleAim,
    /// Resolve it one place earlier or later -- the order the attacks
    /// resolve in (§6.41: "in any order he wishes").
    Earlier,
    Later,
}

fn draw_allocation_row(
    ui: &mut egui::Ui,
    gs: &omdurman_rules::effects::GameState,
    attack: &FireAttack,
    index: usize,
    count: usize,
) -> Option<RowAction> {
    let mut action = None;
    let kind = match attack.kind {
        FireKind::Direct => "Direct",
        FireKind::MaximSecondFire => "Maxim 2nd",
        FireKind::Howitzer => "Howitzer",
    };
    // Each weapon with its printed factor; a named gunboat's Maxims (§2.32)
    // are a weapon of their own.
    let names = crate::combat_ui::tally_names(
        attack
            .shots()
            .into_iter()
            .filter_map(|shot| {
                let factor = gs.find_unit(shot.unit)?.fire_factor(shot.mount)?.value();
                Some(format!(
                    "{} ({factor})",
                    crate::combat_ui::shot_name(shot, Some(gs))
                ))
            })
            .collect(),
    );
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
                format!("{} §{}", line.label, line.paragraph)
            })
            .collect();
        if terrain_mod != 0 {
            parts.push(format!("{terrain_mod:+} terrain defence §6.23"));
        }
        if hexside_mod != 0 {
            parts.push(format!("{hexside_mod:+} hexside §6.23"));
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
                action = Some(RowAction::Remove);
            }
            // The order the attacks resolve in (§6.41); a lone attack has
            // none to change.
            if count > 1 {
                if ui
                    .add_enabled(index > 0, egui::Button::new("\u{25b2}").small())
                    .on_hover_text("Resolve earlier")
                    .clicked()
                {
                    action = Some(RowAction::Earlier);
                }
                if ui
                    .add_enabled(index + 1 < count, egui::Button::new("\u{25bc}").small())
                    .on_hover_text("Resolve later")
                    .clicked()
                {
                    action = Some(RowAction::Later);
                }
            }
            // Wraps: a combined fire lists a dozen weapons.
            ui.add(
                egui::Label::new(
                    egui::RichText::new(format!(
                        "{names_c} \u{2192} {}",
                        target_label(gs, attack.target_hex),
                        names_c = names.join(" + "),
                    ))
                    .color(crate::ui::palette::PANEL_TEXT),
                )
                .wrap(),
            );
        });
        ui.horizontal_wrapped(|ui| {
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
                action = Some(RowAction::ToggleAim);
            }
        }
    });
    action
}

/// Consume the [`FireAllocationState`] list: pre-roll dice for every
/// allocated attack and broadcast the corresponding [`GameEffect`].
/// Runs once when the player clicks "Resolve N attacks". Each attack is
/// pre-validated against the engine (in order, over the earlier ones); a
/// refused attack is reported on a slip and not sent.
/// Enter resolves the staged attacks, as it confirms a move: the key the
/// player already has under the finger for "go".
pub fn resolve_fire_on_enter(
    mut commands: MessageReader<crate::hotkeys::PickerCommand>,
    game_state: Option<Res<GameStateResource>>,
    mut allocation: ResMut<FireAllocationState>,
    peers: Peers,
) {
    if !commands
        .read()
        .any(|c| *c == crate::hotkeys::PickerCommand::ConfirmMove)
    {
        return;
    }
    let Some(gs) = game_state else { return };
    if allocation.staged() == 0
        || allocation.committed
        || !gs.0.phase.is_fire()
        || !peers.may_act(gs.0.phase_player())
    {
        return;
    }
    allocation.execute_requested = true;
}

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
    allocation.execute_requested = false;

    // An order the board has made impossible since it was staged (its
    // firers or target gone) would refuse the whole allocation: drop it.
    let staged: Vec<FireOrder> = allocation
        .orders()
        .into_iter()
        .filter(|order| gs.0.validate_fire_order(order).is_ok())
        .collect();
    // §6.41: "first allocate all of his fire attacks ... then resolves his
    // attacks": the whole sub-phase's fire goes out as one allocation, then
    // every order is rolled and resolved -- with any order the side
    // allocated earlier and has not resolved (a teammate's, or this peer's
    // own before a reconnect).
    if !staged.is_empty()
        && !submit.submit(
            &gs.0,
            GameEvent::Effect(GameEffect::AllocateFire {
                orders: staged.clone(),
            }),
        )
    {
        // Refused (the refusal is on a slip): keep the tray as it was.
        return;
    }
    let earlier =
        gs.0.fire_allocation
            .iter()
            .filter(|a| !a.resolved && gs.0.validate_fire_order(&a.order).is_ok())
            .map(|a| a.order.clone());
    let orders: Vec<FireOrder> = earlier.chain(staged).collect();
    if orders.is_empty() {
        return;
    }
    allocation.committed = true;
    allocation.clear_staged();
    info!(orders = orders.len(), "fire allocated (§6.41)");
    // The first order now; the rest one at a time as each comes back
    // (`continue_fire_resolution`). The results arrive as Combat Resolution
    // Cards; a refused order has posted its reason.
    if let Some(first) = orders.first() {
        let resolution = first.resolution(&gs.0, rng);
        submit.submit(&gs.0, GameEvent::Effect(resolution));
    }
    allocation.resolving = true;
}

/// Resolve the side's allocated fire one order at a time (§6.41), once the
/// previous one has come back and no combat waits on a choice of losses
/// (§7.7) -- a result may stop the game for the other side's choice.
pub fn continue_fire_resolution(
    mut allocation: ResMut<FireAllocationState>,
    mut rng: Option<ResMut<GameRng>>,
    gs: Option<Res<GameStateResource>>,
    mut submit: crate::submit::CheckedSubmit,
) {
    if !allocation.resolving {
        return;
    }
    let (Some(rng), Some(gs)) = (rng.as_mut(), gs) else {
        return;
    };
    if !gs.0.phase.is_fire() {
        allocation.resolving = false;
        return;
    }
    if gs.0.pending_losses.is_some() || !submit.pending.unconfirmed.is_empty() {
        return;
    }
    let Some(order) = gs.0.unresolved_fire() else {
        allocation.resolving = false;
        return;
    };
    let resolution = order.resolution(&gs.0, rng);
    if !submit.submit(&gs.0, GameEvent::Effect(resolution)) {
        // Refused (the reason is on a slip): stop rather than retry it.
        allocation.resolving = false;
    }
}

/// What [`fire_allocation_arrows`] draws: one (firer hex, target hex) per
/// allocated attack while the allocations are open in a fire sub-phase --
/// nothing once they are committed, or outside the fire phases.
fn allocation_arrow_key(
    allocation: &FireAllocationState,
    gs: Option<&GameStateResource>,
) -> Vec<(HexCoord, HexCoord)> {
    let Some(gs) = gs else { return Vec::new() };
    if allocation.committed || !gs.0.phase.is_fire() {
        return Vec::new();
    }
    allocation
        .attacks
        .iter()
        .filter_map(|attack| {
            let unit = attack
                .all_firing_units()
                .first()
                .and_then(|id| gs.0.find_unit(*id))?;
            Some((unit.position, attack.target_hex))
        })
        .collect()
}

/// Persistent arrows from each allocated attack's firer hex to its target
/// (§6.41). Same bold-red look as the melee direction arrows and the hover
/// preview arrow: one visual language for combat targeting. One arrow mesh
/// per attack at most, rebuilt only when what they describe changed (the
/// allocation list, the engine state: a firer's hex, the phase) or the
/// overlays were cleared ([`crate::picker::OverlayGeneration`]) -- not every
/// frame, matching the fire-target rings' discipline; removing/executing an
/// allocation still drops its arrow at once, since that changes the list.
/// Runs in the fire `GameSet`; exit the fire phase and they vanish with the
/// rest of the gameplay overlays.
pub fn fire_allocation_arrows(
    mut draw: crate::overlay::Overlay,
    existing: Query<Entity, With<AllocationArrow>>,
    allocation: Res<FireAllocationState>,
    gs: Option<Res<GameStateResource>>,
    mut last: Local<Option<Vec<(HexCoord, HexCoord)>>>,
) {
    let invalidated = draw.stale();
    if invalidated {
        // The overlays were cleared: forget what we last drew, so the arrows
        // come back instead of being taken for still there.
        *last = None;
    }
    let inputs_moved =
        invalidated || allocation.is_changed() || gs.as_ref().is_some_and(|gs| gs.is_changed());
    if !inputs_moved {
        return;
    }
    let key = allocation_arrow_key(&allocation, gs.as_deref());
    if last.as_ref() == Some(&key) {
        return;
    }
    let existing: Vec<Entity> = existing.iter().collect();
    draw.clear(existing.iter().copied());
    for &(from, to) in &key {
        // The shared arrow mesh (tail at the firer, head at the target) --
        // not the hex-ring mesh, which stretched into a double-pointed
        // hexagon and read as fire in both directions.
        draw.arrow(from, to, AllocationArrow);
    }
    *last = Some(key);
}

/// Reset the allocation state whenever the engine phase changes, so an
/// unexecuted allocation from one fire sub-phase never leaks into the next
/// (§6.41 allocations are per fire sub-phase). Uses a `Local` snapshot of the
/// phase so the reset also fires correctly on replay / snapshot convergence,
/// where no local click precedes the phase advance. Releases the
/// `committed` lock set by [`execute_fire_allocations`] -- and sets it again
/// whenever the engine says the side's fire of the sub-phase has begun
/// resolving ([`omdurman_rules::effects::GameState::fire_resolution_begun`]),
/// so neither a teammate nor a rejoined peer can allocate more.
pub fn reset_fire_allocation_on_phase_change(
    game_state: Option<Res<GameStateResource>>,
    mut allocation: ResMut<FireAllocationState>,
    mut last_phase: Local<Option<Phase>>,
) {
    let Some(gs) = game_state else { return };
    let phase = gs.0.phase;
    if *last_phase != Some(phase) {
        if last_phase.is_some() {
            allocation.clear_staged();
            allocation.committed = false;
            allocation.execute_requested = false;
            allocation.resolving = false;
        }
        *last_phase = Some(phase);
    }
    if gs.is_changed()
        && !allocation.committed
        && gs.0.phase.is_fire()
        && gs.0.fire_resolution_begun()
    {
        allocation.clear_staged();
        allocation.committed = true;
        allocation.execute_requested = false;
    }
}

#[cfg(test)]
mod tray_layout_tests {
    use super::*;
    use omdurman_rules::{FireSubPhase, UnitId, UnitPlacement, UnitState};
    use omdurman_types::{HexCoord, Player, Scenario};

    /// A combined fire of a dozen weapons wraps inside the tray instead of
    /// running past its edge (and off the screen).
    #[test]
    fn a_long_combined_fire_wraps_inside_the_tray() {
        let mut gs = omdurman_rules::effects::GameState::new(Scenario::Campaign);
        gs.phase = Phase::DefensiveFire(FireSubPhase::DirectFire);
        let firers = [
            UnitId::BritishArmy_0_0,
            UnitId::BritishArmy_0_1,
            UnitId::BritishArmy_1_1,
            UnitId::BritishArmy_2_1,
            UnitId::BritishArmy_3_1,
            UnitId::EgyptianArmy_0_0,
            UnitId::EgyptianArmy_1_0,
            UnitId::EgyptianArmy_2_0,
            UnitId::Kitchener_5_0,
            UnitId::Kitchener_6_0,
            UnitId::Kitchener_7_0,
        ];
        let target = HexCoord::new(3, 0);
        for (i, id) in firers.iter().chain([&UnitId::Baggara_0_0]).enumerate() {
            gs.units.push(UnitPlacement {
                id: *id,
                position: if i < firers.len() {
                    HexCoord::new(0, 0)
                } else {
                    target
                },
                profile: omdurman_rules::unit_profiles::profile_for_unit(*id).unwrap(),
                state: UnitState::default(),
            });
        }
        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: gs.phase,
            kind: FireKind::Direct,
            firers: firers.to_vec(),
            target_hex: target,
            at_fort: false,
            factor_row: omdurman_rules::combat_results_table::FireFactorRow::Row41Plus,
            modifiers: vec![omdurman_rules::FireModifier::AngloEgyptianDirectFire],
            gunboat_maxims: Vec::new(),
        };
        let width = 400.0;
        let ctx = crate::ui::headless(4, |ctx, _| {
            crate::ui::anchored_card(
                ctx,
                "tray",
                egui::Align2::CENTER_BOTTOM,
                egui::vec2(0.0, -100.0),
                egui::Frame::NONE,
                |ui| {
                    ui.set_width(width);
                    draw_allocation_row(ui, &gs, &attack, 0, 1);
                },
            );
        });
        let rect = ctx
            .memory(|m| m.area_rect(egui::Id::new("tray")))
            .expect("the tray was shown");
        assert!(
            rect.width() <= width + 1.0,
            "the row widened the tray: {rect:?}"
        );
        assert!(
            rect.min.x >= 0.0 && rect.max.x <= 1200.0,
            "off screen: {rect:?}"
        );
    }
}

#[cfg(test)]
mod arrow_key_tests {
    use super::*;
    use omdurman_rules::{FireSubPhase, UnitId, UnitPlacement, UnitState};
    use omdurman_types::{Player, Scenario};

    fn fire_state(phase: Phase) -> GameStateResource {
        let mut gs = omdurman_rules::effects::GameState::new(Scenario::Campaign);
        gs.phase = phase;
        for (id, hex) in [
            (UnitId::BritishArmy_0_0, HexCoord::new(0, 0)),
            (UnitId::Baggara_0_0, HexCoord::new(3, 0)),
        ] {
            gs.units.push(UnitPlacement {
                id,
                position: hex,
                profile: omdurman_rules::unit_profiles::profile_for_unit(id).unwrap(),
                state: UnitState::default(),
            });
        }
        GameStateResource(gs)
    }

    fn attack(phase: Phase) -> FireAttack {
        FireAttack {
            firing_player: Player::AngloEgyptian,
            phase,
            kind: FireKind::Direct,
            firers: vec![UnitId::BritishArmy_0_0],
            target_hex: HexCoord::new(3, 0),
            at_fort: false,
            factor_row: omdurman_rules::combat_results_table::FireFactorRow::Row41Plus,
            modifiers: Vec::new(),
            gunboat_maxims: Vec::new(),
        }
    }

    /// One arrow per open allocation, from the firer's hex to the target;
    /// none once committed, outside the fire phases, or without a state.
    #[test]
    fn arrows_follow_the_open_allocations() {
        let phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let gs = fire_state(phase);
        let mut allocation = FireAllocationState {
            attacks: vec![attack(phase)],
            ..Default::default()
        };
        assert_eq!(
            allocation_arrow_key(&allocation, Some(&gs)),
            vec![(HexCoord::new(0, 0), HexCoord::new(3, 0))]
        );
        assert!(allocation_arrow_key(&allocation, None).is_empty());
        assert!(allocation_arrow_key(&allocation, Some(&fire_state(Phase::Movement))).is_empty());
        allocation.committed = true;
        assert!(allocation_arrow_key(&allocation, Some(&gs)).is_empty());
    }

    fn lock_app(gs: GameStateResource) -> App {
        let mut app = App::new();
        app.insert_resource(gs)
            .insert_resource(FireAllocationState {
                attacks: vec![attack(Phase::OffensiveFire(FireSubPhase::DirectFire))],
                ..Default::default()
            })
            .add_systems(Update, reset_fire_allocation_on_phase_change);
        app
    }

    /// §6.41: once the side's fire of a sub-phase has begun resolving -- this
    /// peer's before a rejoin, or a teammate's -- the tray locks and drops
    /// what it staged: nothing more may be allocated. An allocation not yet
    /// resolving leaves it open; the next sub-phase reopens it.
    #[test]
    fn the_tray_locks_once_the_sides_fire_resolves() {
        let phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let mut gs = fire_state(phase);
        gs.0.active_player = Player::AngloEgyptian;
        gs.0.fire_allocation
            .push(omdurman_rules::effects::AllocatedFire {
                order: FireOrder::Attack(attack(phase)),
                resolved: false,
            });
        let mut app = lock_app(gs);
        app.update();
        assert!(!app.world().resource::<FireAllocationState>().committed);

        app.world_mut()
            .resource_mut::<GameStateResource>()
            .0
            .fire_allocation[0]
            .resolved = true;
        app.update();
        let allocation = app.world().resource::<FireAllocationState>();
        assert!(allocation.committed);
        assert_eq!(allocation.staged(), 0);

        // The next sub-phase opens fire again (the phase advance clears the
        // allocation).
        {
            let mut gs = app.world_mut().resource_mut::<GameStateResource>();
            gs.0.phase = Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer);
            gs.0.fire_allocation.clear();
        }
        app.update();
        assert!(!app.world().resource::<FireAllocationState>().committed);
    }
}
