use crate::dispatch::Dispatches;
use crate::fire::{fire_group_kinds, fire_selection, group_attacks_for};
use crate::input::CombatClickCtx;
use crate::peers::Peers;
use crate::picker::{PickerState, PlacedUnit};
use crate::{GameRng, GameStateResource, PendingEdits};
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
    /// True once "Execute All" has been triggered — locks further changes
    /// until the fire phase changes and the state resets.
    pub committed: bool,
    /// Set by the UI panel; consumed by [`execute_fire_allocations`].
    pub execute_requested: bool,
    /// Whether the resolution overlay is open. Auto-opens on the first
    /// allocation; the player can toggle it with the "Fire" button in the
    /// actions panel (or "Review & execute allocations").
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
/// The attack set comes from the fire selection ([`fire_selection`]): a
/// single-unit click allocates exactly that unit's fire (§6.13), a fire-group
/// (double-click) allocates the whole tile, split into one attack per weapon
/// kind (§6.14/§6.42).
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
        // No unit of the group can see this hex. Report the *most likely*
        // cause on the first unit (line of sight vs. any other refusal).
        let Some(&(firer, kind)) = kinds.first() else {
            return;
        };
        if let Err(omdurman_rules::effects::RuleError::LineOfSightBlocked(_, _)) =
            gs.0.can_fire_at(firer, target, kind)
        {
            dispatches.push("Field Telegraph", "Fire refused — no line of sight (§6.3).");
        }
        return;
    }

    for attack in &attacks {
        // Skip if these firers already allocated.
        if allocation.attacks.iter().any(|a| a.firers == attack.firers) {
            dispatches.push(
                "Fire Allocation",
                "These units have already allocated their fire.",
            );
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
            "{kind_str} allocated to ({}, {}). {n} attack{} pending.",
            target.q,
            target.r,
            if n == 1 { "" } else { "s" },
        ),
    );

    *state = PickerState::Idle;
}

/// egui panel showing the current allocation list with remove buttons and
/// an "Execute All" button. Each attack is one row (scrollable when the list
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
        egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 40, 220))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(10, 6)),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(13.0));

            ui.horizontal(|ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(200, 180, 140),
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
                    egui::Color32::from_rgb(150, 150, 150),
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
                        egui::Button::new("Execute All")
                            .fill(egui::Color32::from_rgb(60, 80, 40))
                            .min_size(egui::Vec2::new(120.0, 28.0)),
                    )
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
        let mut parts: Vec<&str> = Vec::new();
        for m in &attack.modifiers {
            parts.push(match m {
                omdurman_rules::FireModifier::AngloEgyptianDirectFire => "A-E +1 (§6.24)",
                omdurman_rules::FireModifier::BrigadeIntegrity => "Brigade +1 (§5.54)",
                omdurman_rules::FireModifier::Terrain(n) => {
                    let _ = n;
                    "terrain mod engine-side (§6.23)"
                }
                omdurman_rules::FireModifier::ZaribaThornHedge => "Zariba hedge −2 (§9.231)",
                omdurman_rules::FireModifier::ZaribaTrenchEntrenched => {
                    "Zariba trench entrenched −4 (§9.232)"
                }
            });
        }
        let text = if parts.is_empty() {
            "no modifiers".to_string()
        } else {
            parts.join("  ·  ")
        };
        let color = if net > 0 {
            egui::Color32::from_rgb(170, 210, 170)
        } else if net < 0 {
            egui::Color32::from_rgb(210, 160, 120)
        } else {
            egui::Color32::from_rgb(180, 180, 180)
        };
        (text, color)
    };

    ui.group(|ui| {
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
                *remove_self = Some(index);
            }
            ui.colored_label(
                egui::Color32::from_rgb(210, 200, 180),
                format!(
                    "{names_c} → ({}, {})",
                    attack.target_hex.q,
                    attack.target_hex.r,
                    names_c = names.join(" + "),
                ),
            );
        });
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("  {kind}  ·  range {range_text}"))
                    .color(egui::Color32::from_rgb(180, 180, 180))
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
                .color(egui::Color32::from_rgb(170, 160, 140))
                .size(11.0)
                .monospace(),
        );
    });
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
            pending.submit_game(GameEvent::Effect(GameEffect::HowitzerFire {
                attack: attack.clone(),
                combat_results_table_roll,
                impact_roll,
            }));
        } else {
            let roll = d10();
            info!(
                target.q = attack.target_hex.q,
                target.r = attack.target_hex.r,
                roll = %roll,
                "fire (batch)",
            );
            pending.submit_game(GameEvent::Effect(GameEffect::FireCombat {
                attack: attack.clone(),
                roll,
            }));
        }
    }

    dispatches.push(
        "Fire Allocation",
        format!(
            "Executed {} fire attack{s}.",
            attacks.len(),
            s = if attacks.len() == 1 { "" } else { "s" }
        ),
    );

    allocation.execute_requested = false;
}

/// Persistent red arrows — one per allocated [`FireAttack`], from its firer
/// hex to its target hex — so the player sees *which* shots are pending
/// (§6.41). Rebuilt from the allocation list each frame (one arrow mesh per
/// attack at most), so removing/executing an allocation drops its arrow
/// immediately. Runs in the fire `GameSet`; exit the fire phase and they
/// vanish with the rest of the gameplay overlays.
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
            MeshMaterial3d(assets.fire_arrow.clone()),
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
