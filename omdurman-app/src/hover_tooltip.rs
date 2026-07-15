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

impl Plugin for HoverTooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, draw_hover_tooltip);
    }
}

fn draw_hover_tooltip(
    mut contexts: EguiContexts,
    hovered: Res<crate::HoveredHex>,
    game_map: Res<GameMap>,
    game_state: Option<Res<GameStateResource>>,
    picker: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    mut rulebook: ResMut<Rulebook>,
) {
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
                                for u in occupants {
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
                            }
                        }

                        // Legibility hints when a unit is selected.
                        if let Some((unit_id, _)) = selected_unit_id(&picker, &placed_units)
                            && let Some(gs) = gs
                            && let Some(hint) = movement_hint(gs, unit_id, hex, &game_map)
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
/// * **Movement** -- terrain cost, wall/ZOC blocking, out-of-range.
///
/// Fire/melee previewing is handled by the on-map preview panels; this hint
/// only covers the *destination* view.
fn movement_hint(
    gs: &GameState,
    unit_id: omdurman_rules::UnitId,
    hex: HexCoord,
    game_map: &GameMap,
) -> Option<String> {
    let unit = gs.find_unit(unit_id)?;
    let from = unit.position;
    if from == hex {
        return None;
    }
    let is_boat = matches!(unit.profile.movement, UnitMovement::Gunboat(_));

    match gs.phase {
        Phase::Setup => {
            let owner = unit.profile.identity.owner();
            if gs.in_deployment_zone(owner, hex) {
                Some(format!("Inside {owner}'s deployment zone (§9.2)."))
            } else {
                Some(format!("Outside {owner}'s deployment zone (§9.2)."))
            }
        }
        Phase::Movement => {
            // Adjacency first: the picker commits one hex per click, so a
            // non-adjacent destination isn't a candidate this turn.
            if !from.neighbors().contains(&hex) {
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
            if let Some(side) = game_map.hexside_between(from, hex)
                && side.blocks_movement()
            {
                return Some(format!("Blocked by {} hexside (§5.23).", side));
            }
            // Stacking -- count units at the destination, see if one more
            // would violate the rule. The engine's full check (tribe mix,
            // leader command, gunboat isolation) is the authority; here we
            // only flag the common 4-unit case so the player knows the hex
            // is crowded.
            let occupants = gs.units.iter().filter(|u| u.position == hex).count();
            if occupants >= 4 {
                return Some(format!(
                    "Stack full: {occupants} units already here (§5.51)."
                ));
            }
            // Otherwise -- it's a legal adjacent step. Show its cost.
            let has_road = from.neighbors().iter().any(|n| {
                game_map
                    .roads
                    .contains(&omdurman_types::HexsideRef::new(from, *n))
            });
            let cost = omdurman_rules::terrain_chart::movement_cost_with_road(
                tile.unwrap().terrain,
                has_road,
            )
            .map(|c| c.value())
            .unwrap_or(0);
            let remaining = gs.mp_spent(unit_id);
            let total = allowance(&unit.profile);
            let left = total.saturating_sub(remaining);
            if cost == 0 {
                return Some("Impassable terrain (§5.11).".into());
            }
            if (cost as i16) > left {
                Some(format!("Out of MP: costs {cost}, {left} left (§5.11)."))
            } else {
                Some(format!("Move here: costs {cost} MP ({left} left, §5.11)."))
            }
        }
        _ => None,
    }
}

fn allowance(profile: &UnitProfile) -> i16 {
    match profile.movement {
        UnitMovement::Land(a) => a.value() as i16,
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
