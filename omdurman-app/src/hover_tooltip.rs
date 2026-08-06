//! Hex hover tooltip (§decision: discoverability).
//!
//! When the player hovers a hex during play, a small tooltip appears near the
//! cursor explaining what that hex *is* and (when a unit is selected) why it
//! is or isn't a legal destination -- terrain cost, blocking wall, ZOC,
//! stacking, out of range. Each clause carries its rulebook paragraph so the
//! player can deep-link into the manual for the full rule.
//!
//! The tooltip is informational only: the rules engine remains the authority
//! on legality, the on-map rings still show what's clickable, and combat
//! previews live in [`crate::fire::fire_combat_preview_ui`] (this tooltip
//! deliberately does not duplicate them).
//!
//! Phases without a selected unit show a plain hex card (terrain, coord,
//! landmark, occupants) -- still useful for orientation.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::GameMap;
use omdurman_rules::effects::GameState;
use omdurman_rules::{Phase, UnitMovement, UnitProfile};
use omdurman_types::{HexCoord, Player, Terrain};

use crate::GameStateResource;
use crate::picker::{PickerState, PlacedUnit, selected_unit_id};
use crate::rulebook::Rulebook;

pub struct HoverTooltipPlugin;

/// Bundle of the read-only picker state and the in-progress movement path so
/// [`draw_hover_tooltip`] stays under clippy's argument limit.
#[derive(bevy::ecs::system::SystemParam)]
struct PickerReadState<'w> {
    state: Res<'w, PickerState>,
    movement_path: Res<'w, crate::picker::MovementPath>,
}

impl Plugin for HoverTooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            draw_hover_tooltip.run_if(crate::map_view_active),
        );
    }
}

fn draw_hover_tooltip(
    mut contexts: EguiContexts,
    hovered: Res<crate::HoveredHex>,
    game_map: Res<GameMap>,
    game_state: Option<Res<GameStateResource>>,
    picker: PickerReadState,
    placed_units: Query<(Entity, &PlacedUnit)>,
    mut rulebook: ResMut<Rulebook>,
) {
    let PickerReadState {
        state: picker,
        movement_path,
    } = picker;
    let Some(hex) = hovered.0 else {
        return;
    };
    let Some(tile) = game_map.hexes.get(&hex) else {
        return;
    };
    let terrain = tile.terrain;
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let gs = game_state.as_deref().map(|r| &r.0);

    let mut clicked_section: Option<String> = None;

    // Anchor the tooltip near the cursor (eager: render every frame the hex
    // is hovered so the player doesn't have to wait).
    let pos = ctx.pointer_latest_pos().unwrap_or(egui::pos2(40.0, 40.0));
    let tip_rect = egui::Rect::from_center_size(pos, egui::vec2(280.0, 1.0));

    egui::Area::new(egui::Id::new("hover_tooltip"))
        .fixed_pos(tip_rect.min + egui::vec2(14.0, 18.0))
        .order(egui::Order::Tooltip)
        .interactable(true)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(0xF6, 0xED, 0xC5))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x6B, 0x62, 0x50)))
                .inner_margin(egui::Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.set_max_width(280.0);
                    ui.vertical(|ui| {
                        // Header line: coord + terrain name + (optional) landmark.
                        let terrain_str = terrain_label(terrain);
                        ui.label(
                            egui::RichText::new(format!(
                                "({}, {})  ·  {}",
                                hex.q, hex.r, terrain_str
                            ))
                            .color(egui::Color32::from_rgb(0x1A, 0x16, 0x10))
                            .strong()
                            .size(13.0),
                        );
                        if let Some(landmark) = landmark_label(&game_map, hex) {
                            ui.colored_label(egui::Color32::from_rgb(0x6B, 0x62, 0x50), landmark);
                        }

                        // Occupants: which units are in this hex.
                        if let Some(gs) = gs {
                            let occupants: Vec<&omdurman_rules::UnitPlacement> =
                                gs.units.iter().filter(|u| u.position == hex).collect();
                            if !occupants.is_empty() {
                                ui.add_space(2.0);
                                for u in &occupants {
                                    let owner_mark = match u.profile.identity.owner() {
                                        Player::AngloEgyptian => "[AE]",
                                        Player::Dervish => "[D]",
                                    };
                                    let label = format!(
                                        "{owner_mark} {} ({}/{})",
                                        u.profile.identity.short_label(),
                                        u.profile.fire.map(|f| f.value()).unwrap_or(0),
                                        u.profile.melee.map(|m| m.value()).unwrap_or(0),
                                    );
                                    let color = if u.state.disrupted {
                                        egui::Color32::from_rgb(180, 90, 90)
                                    } else {
                                        egui::Color32::from_rgb(0x1A, 0x16, 0x10)
                                    };
                                    ui.colored_label(color, label);
                                }
                                // §5.54 brigade integrity: when all four
                                // battalions of an Anglo-Egyptian brigade are
                                // stacked together they fire with a +1 die
                                // modifier.
                                let identities: Vec<_> = occupants
                                    .iter()
                                    .map(|u| u.profile.identity)
                                    .collect();
                                if matches!(
                                    omdurman_rules::brigade_integrity(&identities),
                                    omdurman_rules::BrigadeIntegrity::Integrated(_)
                                ) {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(0x6B, 0x8B, 0x40),
                                        "Brigade integrity: +1 fire (§5.54).",
                                    );
                                }
                            }
                        }

                        // Terrain effects card: always show defence modifier
                        // and movement cost so players can read a hex even
                        // without a unit selected (§6.23, §5.11).
                        if let Some(_gs) = gs {
                            let def_mod = omdurman_rules::terrain_chart::defense_modifier(terrain);
                            let move_cost = omdurman_rules::terrain_chart::movement_cost(terrain)
                                .map(|c| c.value())
                                .unwrap_or(0);
                            ui.add_space(2.0);
                            let mut line = String::new();
                            if def_mod != 0 {
                                line.push_str(&format!("Defence {def_mod:+} (§6.23). "));
                            }
                            if move_cost > 0 {
                                line.push_str(&format!("Move cost {move_cost} MP (§5.11)."));
                            } else if line.is_empty() {
                                line.push_str("Impassable to land units (§5.11).");
                            }
                            if !line.is_empty() {
                                ui.colored_label(
                                    egui::Color32::from_rgb(0x6B, 0x62, 0x50),
                                    line,
                                );
                            }
                        }

                        // Legibility hints when a unit is selected.
                        if let Some((unit_id, _)) = selected_unit_id(&picker, &placed_units)
                            && let Some(gs) = gs
                            && let Some(hint) = movement_hint(gs, unit_id, hex, &game_map, &movement_path)
                        {
                            ui.add_space(2.0);
                            ui.separator();
                            // Render via the shared rulebook-ref renderer so
                            // the hint's `§N` citations deep-link to the
                            // manual tab (and the citations are annotated
                            // with the section titles).
                            if let Some(sec) = rulebook.render_refs(ui, &hint) {
                                clicked_section = Some(sec);
                            }
                        }
                    });
                });
        });

    if let Some(sec) = clicked_section {
        crate::rulebook::request_section(&mut rulebook, &sec);
    }
}

/// Build the per-hex terrain label. The `Terrain` enum's `strum::Display`
/// already prints a readable form, but a couple of overrides land better on a
/// small card (e.g. "Clear" reads better than "ClearSteppe" if such a variant
/// existed; this is forward-looking).
fn terrain_label(t: Terrain) -> String {
    t.to_string()
}

/// A short landmark line for known named tiles (the Palace, forts, river
/// mouths). Returns `None` for ordinary hexes so the card stays small.
fn landmark_label(game_map: &GameMap, hex: HexCoord) -> Option<String> {
    game_map
        .hexes
        .get(&hex)
        .and_then(|h| h.name.as_deref())
        .filter(|n| !n.is_empty())
        .map(|n| format!("“{n}”"))
}

/// One-line movement / blocking hint for the currently-selected unit moving
/// to `hex`. Returns `None` when there's nothing useful to say (out of setup,
/// nothing selected, hex not adjacent so no immediate action, etc.).
///
/// Phases covered:
/// * **Setup** -- whether `hex` is in the active player's deployment zone.
/// * **Movement** -- terrain cost, wall/ZOC blocking, out-of-range,
///   accumulated path cost, night label, stacking.
///
/// Fire/melee previewing is handled by the on-map preview panels; this hint
/// only covers the *destination* view.
fn movement_hint(
    gs: &GameState,
    unit_id: omdurman_rules::UnitId,
    hex: HexCoord,
    game_map: &GameMap,
    movement_path: &crate::picker::MovementPath,
) -> Option<String> {
    let unit = gs.find_unit(unit_id)?;
    let from = unit.position;
    if from == hex {
        return None;
    }
    let is_boat = matches!(unit.profile.movement, UnitMovement::Gunboat(_));
    let is_night = gs.day_night == omdurman_types::DayNight::Night;

    match gs.phase {
        Phase::Setup => {
            let owner = unit.profile.identity.owner();
            if gs.in_deployment_zone(owner, hex, is_boat) {
                Some(format!("Inside {owner}'s deployment zone (§9.2)."))
            } else {
                Some(format!("Outside {owner}'s deployment zone (§9.2)."))
            }
        }
        Phase::Movement => {
            // Determine the effective origin for adjacency: if a path is
            // being built, the unit's next move starts from the path's end,
            // not the unit's current board position.
            let effective_from = movement_path.current_end().unwrap_or(from);
            let adjacent = effective_from.neighbors().contains(&hex);
            if !adjacent {
                // Check if the hex is in enemy ZOC to explain blocking.
                if gs.hex_in_enemy_zoc(hex, unit.profile.identity.owner(), unit.profile.kind) {
                    return Some("Blocked by enemy ZOC — may not move beyond (§5.41).".to_string());
                }
                return Some("Not adjacent — step hex-by-hex (§5.12).".to_string());
            }
            // Passability (Nile/water for land, land for gunboats).
            let tile = game_map.hexes.get(&hex);
            let passable = tile
                .map(|t| terrain_passable(t.terrain, is_boat))
                .unwrap_or(false);
            if !passable {
                let reason = if is_boat {
                    "land — gunboats stay on the Nile"
                } else {
                    "Nile hex — land units may not enter"
                };
                return Some(format!("Impassable: {reason} (§5.22)."));
            }
            // Wall hexside blocks movement (§5.23).
            if let Some(side) = game_map.hexside_between(effective_from, hex)
                && side.blocks_movement()
            {
                return Some(format!("Blocked by {} hexside (§5.23).", side));
            }
            // ZOC check: entering a hex in enemy ZOC is allowed but
            // movement may not continue beyond it (§5.41).
            let in_zoc = gs.hex_in_enemy_zoc(hex, unit.profile.identity.owner(), unit.profile.kind);
            // Stacking -- count units at the destination.
            let occupants = gs.units.iter().filter(|u| u.position == hex).count();
            if occupants >= 4 {
                return Some(format!(
                    "Stack full: {occupants} units already here (§5.51)."
                ));
            }
            // Otherwise -- it's a legal adjacent step. Show its cost.
            let has_road = effective_from.neighbors().iter().any(|n| {
                game_map
                    .roads
                    .contains(&omdurman_types::HexsideRef::new(effective_from, *n))
            });
            let cost = tile
                .map(|t| {
                    omdurman_rules::terrain_chart::movement_cost_with_road(t.terrain, has_road)
                        .map(|c| c.value())
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            // Accumulated cost = path cost so far + this step's cost.
            let acc_cost = movement_path.cost_so_far + cost as i16;
            let remaining = gs.mp_spent(unit_id);
            let total = allowance(&unit.profile, gs.day_night);
            let left = total.saturating_sub(remaining);
            // Defence modifier at destination (§6.23).
            let def_mod = tile
                .map(|t| omdurman_rules::terrain_chart::defense_modifier(t.terrain))
                .unwrap_or(0);

            let mut lines = Vec::new();
            if cost == 0 {
                lines.push("Impassable terrain (\u{00a7}5.11).".into());
            } else if (cost as i16) > left {
                lines.push(format!("Out of MP: costs {cost}, {left} left (\u{00a7}5.11)."));
            } else {
                let road_note = if has_road {
                    let base_cost = tile
                        .map(|t| {
                            omdurman_rules::terrain_chart::movement_cost(t.terrain)
                                .map(|c| c.value())
                                .unwrap_or(0)
                        })
                        .unwrap_or(0);
                    if base_cost > 1 {
                        format!(" (road bonus: reduced from {base_cost})")
                    } else {
                        " (road)".into()
                    }
                } else {
                    String::new()
                };
                lines.push(format!(
                    "Move here: costs {cost} MP (accumulated {acc_cost}, {left} left, \u{00a7}5.11){road_note}."
                ));
            }
            if def_mod != 0 {
                lines.push(format!("Defence modifier: {def_mod} (§6.23)."));
            }
            if in_zoc {
                lines.push("Hex is in enemy ZOC \u{2014} movement stops here (\u{00a7}5.41).".into());
                lines.push("May withdraw to adjacent friendly hex next turn (\u{00a7}5.43).".into());
            }
            // §5.52: Dervish tribal units from different tribes may not stack.
            if let omdurman_rules::UnitIdentity::DervishTribal { tribe: my_tribe } =
                unit.profile.identity
            {
                for other in gs.units.iter().filter(|u| u.position == hex) {
                    if let omdurman_rules::UnitIdentity::DervishTribal { tribe: their_tribe } =
                        other.profile.identity
                        && my_tribe != their_tribe
                    {
                        lines.push(format!(
                            "\u{26a0} Would mix {my_tribe} with {their_tribe} \u{2014} different tribes may not stack (\u{00a7}5.52)."
                        ));
                        break;
                    }
                }
            }
            if is_night && !is_boat && unit.profile.identity.owner() == omdurman_types::Player::AngloEgyptian {
                lines.push("Night — AE movement halved (§8.1).".into());
            }
            if is_boat {
                // Annotate upstream/downstream direction and budget (§5.24).
                if let Some(UnitMovement::Gunboat(ga)) = Some(&unit.profile.movement) {
                    let dir_str = gs
                        .board
                        .step_direction(effective_from, hex)
                        .map(|d| match d {
                            omdurman_rules::board::StepDirection::Upstream => "upstream",
                            omdurman_rules::board::StepDirection::Downstream => "downstream",
                        });
                    let spent = gs.mp_spent(unit_id);
                    let up_left = (ga.upstream.value() as i16 - spent).max(0);
                    let down_left = (ga.downstream.value() as i16 - spent).max(0);
                    if let Some(dir) = dir_str {
                        lines.push(format!(
                            "Gunboat stepping {dir} (§5.24) — {up_left}↑ {down_left}↓ MP left."
                        ));
                        if dir == "upstream" {
                            lines.push(
                                "Upstream step caps this turn at the upstream allowance (§5.24)."
                                    .into(),
                            );
                        }
                    } else {
                        lines.push(format!(
                            "Gunboat on the Nile (§5.24) — {up_left}↑ {down_left}↓ MP left."
                        ));
                    }
                } else {
                    lines.push("Gunboat on the Nile (§5.24).".into());
                }
                // FoK: flag the White Nile ↔ Blue Nile off-board crossing
                // (§9.345) -- a flat 6-MP upstream jump unique to this board.
                if gs.scenario == omdurman_types::Scenario::FallOfKhartoum
                    && gs.is_nile_mouth_crossing(effective_from, hex)
                {
                    lines.push(
                        "Nile-mouth crossing \u{2014} 6 MP flat (§9.345).".into(),
                    );
                }
            }
            if occupants > 0 && occupants < 4 {
                lines.push(format!(
                    "{occupants} unit{} here (§5.51).",
                    if occupants == 1 { "" } else { "s" }
                ));
            }
            Some(lines.join(" "))
        }
        _ => None,
    }
}

fn allowance(profile: &UnitProfile, day_night: omdurman_types::DayNight) -> i16 {
    match profile.movement {
        UnitMovement::Land(a) => {
            let effective = omdurman_rules::effective_movement_at_night(
                a,
                profile.identity.owner(),
                day_night,
            );
            effective.value() as i16
        }
        UnitMovement::Gunboat(g) => g.upstream.value().max(g.downstream.value()) as i16,
        UnitMovement::Immobile => 0,
    }
}

fn terrain_passable(t: Terrain, is_boat: bool) -> bool {
    if is_boat {
        t.is_nile()
    } else {
        t.passable_by_land()
    }
}
