use crate::board_click::FireClick;
use crate::dispatch::Dispatches;
use crate::fire::{fire_group_kinds, fire_selection, group_attacks_for};
use crate::peers::Peers;
use crate::picker::{PickerState, PlacedUnit};
use crate::{GameRng, GameStateResource};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hexmap::hex_world_pos;
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
    if allocation.committed {
        return;
    }
    let firing_player = gs.0.phase_player();
    if !peers.may_act(firing_player) {
        return;
    }
    let Some(group) = fire_selection(&state, &placed_units, &gs.0) else {
        return;
    };
    let kinds = fire_group_kinds(&gs.0, &group);
    if kinds.is_empty() {
        return;
    }

    let attacks = group_attacks_for(&gs.0, &group, &kinds, target);
    if attacks.is_empty() {
        // A release on a friendly or empty hex (e.g. the selecting click
        // itself) is not an attempted shot: stay quiet.
        let firing_player = gs.0.phase_player();
        let enemy_there = gs.0.units.iter().any(|u| {
            u.position == target && u.profile.identity.owner() == firing_player.opponent()
        });
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
        if allocation
            .attacks
            .iter()
            .any(|a| a.firers.iter().any(|f| attack.firers.contains(f)))
        {
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
    let names: Vec<String> = gs
        .units
        .iter()
        .filter(|u| u.position == hex)
        .map(|u| u.profile.identity.short_label())
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
    _placed_units: Query<(Entity, &PlacedUnit)>,
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
                            draw_allocation_row(ui, &gs.0, attack, &mut remove_self, i);
                            ui.add_space(4.0);
                        }
                    });
            }

            ui.add_space(6.0);

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
                    .on_hover_text("Roll and resolve every allocated attack (§6.41)")
                    .clicked()
            {
                allocation.execute_requested = true;
            }
        },
    );

    if let Some(idx) = remove_self {
        allocation.attacks.remove(idx);
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
    index: usize,
) {
    let kind = match attack.kind {
        FireKind::Direct => "Direct",
        FireKind::MaximSecondFire => "Maxim 2nd",
        FireKind::Howitzer => "Howitzer",
    };
    let names: Vec<String> = attack
        .firers
        .iter()
        .filter_map(|id| gs.find_unit(*id))
        .map(|u| {
            let factor = u.profile.fire.map(|f| f.value()).unwrap_or(0);
            format!("{} ({})", u.profile.identity.short_label(), factor)
        })
        .collect();
    let mut range_text = "?".to_string();
    if let Some(firer) = attack.firers.first()
        && let Some(unit) = gs.find_unit(*firer)
    {
        let dist = unit.position.distance(attack.target_hex);
        range_text = format!("{dist} hex{pl}", pl = if dist == 1 { "" } else { "es" });
    }
    let net = attack.net_modifier();

    let (mod_text, mod_color) = {
        // The canonical modifier wording, shared with the combat card.
        let parts: Vec<String> = attack
            .modifiers
            .iter()
            .map(|m| {
                let line = crate::combat_ui::describe_fire_modifier(*m);
                format!("{} (§{})", line.label, line.paragraph)
            })
            .collect();
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
        ui.label(
            egui::RichText::new(format!("    {mod_text}"))
                .color(crate::ui::palette::PANEL_DIM)
                .size(11.0)
                .monospace(),
        );
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
        if gs.0.find_unit(attack.firers[0]).is_none() {
            continue;
        }

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
                }),
            ));
        }
    }

    submit.notify(
        "Fire Allocation",
        format!(
            "Resolving {sent} fire attack{s}.",
            s = if sent == 1 { "" } else { "s" }
        ),
    );

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
    hex: crate::HexRender,
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
    let crate::HexRender {
        assets,
        layout,
        overlay,
    } = hex;
    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;

    for attack in &allocation.attacks {
        let Some(firer) = attack.firers.first() else {
            continue;
        };
        let Some(unit) = gs.0.find_unit(*firer) else {
            continue;
        };
        let from = hex_world_pos(unit.position, origin, &overlay.params);
        let to = hex_world_pos(attack.target_hex, origin, &overlay.params);
        let delta = Vec3::new(to.x - from.x, 0.0, to.z - from.z);
        let len = delta.length();
        if len < f32::EPSILON {
            continue;
        }
        let dir = delta / len;
        let inset = size * 0.18;
        let draw_len = (len - inset).max(len * 0.4);
        let tail = from + dir * ((len - draw_len) * 0.5);
        commands.spawn((
            AllocationArrow,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.orange.clone()),
            Transform::from_xyz(tail.x, 1.6, tail.z)
                .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                .with_scale(Vec3::new(size * 0.5, 1.0, draw_len)),
            Visibility::Visible,
        ));
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
