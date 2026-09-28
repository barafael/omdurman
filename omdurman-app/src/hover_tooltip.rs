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
use omdurman_hexmap::{GameMap, hex_world_pos};
use omdurman_rules::effects::GameState;
use omdurman_rules::{Phase, UnitMovement};
use omdurman_types::{HexCoord, Player, Terrain};

use crate::camera::RtsCamera;
use crate::picker::selected_unit_id;
use crate::rulebook::Rulebook;

pub struct HoverTooltipPlugin;

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
    game_map: Res<GameMap>,
    board: crate::BoardGeometry,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    picker: crate::picker::PickerReadState,
    rulebook: ResMut<Rulebook>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let crate::picker::PickerReadState {
        picker_state: picker,
        movement_path,
        hovered,
        game_state,
        placed_units,
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

    // Anchor the tooltip at the hovered tile's projected screen position so it
    // rides the tile rather than the cursor, and pivot it on its LEFT_CENTER
    // so it expands vertically centred on the tile (instead of dropping down
    // from the cursor). A horizontal nudge to the right of the tile centre
    // keeps the card from covering the hex itself; tiles on the right edge
    // flip to the left side so the tooltip stays on-screen.
    let origin = board.layout.adjusted_origin(&board.overlay.params);
    let world = hex_world_pos(hex, origin, &board.overlay.params);
    let anchor = match cameras.single() {
        Ok((camera, cam_transform)) => match camera.world_to_viewport(cam_transform, world) {
            Ok(vp) => egui::pos2(vp.x, vp.y),
            // Behind the camera or off-screen (e.g. mid fly-to): fall back to
            // the cursor so the tooltip still appears somewhere usable.
            Err(_) => ctx.pointer_latest_pos().unwrap_or(egui::pos2(40.0, 40.0)),
        },
        Err(_) => ctx.pointer_latest_pos().unwrap_or(egui::pos2(40.0, 40.0)),
    };
    let nudge_x = board.overlay.params.hex_size * 0.85;
    let on_right_side = anchor.x + nudge_x + 280.0 <= ctx.viewport_rect().right();
    let pivot = if on_right_side {
        egui::Align2::LEFT_CENTER
    } else {
        egui::Align2::RIGHT_CENTER
    };
    let pivot_pos = if on_right_side {
        egui::pos2(anchor.x + nudge_x, anchor.y)
    } else {
        egui::pos2(anchor.x - nudge_x, anchor.y)
    };

    let shown = egui::Area::new(egui::Id::new("hover_tooltip"))
        .fixed_pos(pivot_pos)
        .pivot(pivot)
        .order(egui::Order::Tooltip)
        // Purely informational: never claim the pointer. An interactable area
        // here feeds `EguiPointerOverUi` (see `omdurman_board_ui::panels`),
        // which nils the board-plane hover -- the hovered stack collapses,
        // the tooltip (anchored to that same hex) disappears, hover returns,
        // and the cycle oscillates: the expand-on-hover "wiggle". With
        // `interactable(false)` the area is click-through and stays out of
        // egui's interactive-rect hit-test, so the pointer remains on the
        // board wherever it sits on the hex -- including over the fanned-out
        // counters the expansion itself pushes toward the tooltip. The §
        // citations are rendered as plain text (see `render_refs_plain`) so
        // no interactive widget re-introduces the loop.
        .interactable(false)
        .show(ctx, |ui| {
            // The rule the tooltip is mainly about: the hint for a selected
            // unit, else the terrain line. The tooltip cannot take a click
            // (see above), so R opens that rule instead.
            let mut main_ref: Option<String> = None;
            crate::ui::frames::paper(egui::Stroke::new(1.0, crate::ui::palette::FAINT_INK))
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
                            .color(crate::ui::palette::INK)
                            .strong()
                            .size(13.0),
                        );
                        if let Some(landmark) = landmark_label(&game_map, hex) {
                            ui.colored_label(crate::ui::palette::FAINT_INK, landmark);
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
                                        crate::ui::palette::INK_DISRUPTED
                                    } else {
                                        crate::ui::palette::INK
                                    };
                                    ui.colored_label(color, label);
                                }
                                // §5.54 brigade integrity: when all four
                                // battalions of an Anglo-Egyptian brigade are
                                // stacked together they fire with a +1 die
                                // modifier.
                                let identities: Vec<_> =
                                    occupants.iter().map(|u| u.profile.identity).collect();
                                if matches!(
                                    omdurman_rules::brigade_integrity(&identities),
                                    omdurman_rules::BrigadeIntegrity::Integrated(_)
                                ) {
                                    ui.colored_label(
                                        crate::ui::palette::INK_BONUS,
                                        "Brigade integrity: +1 fire (§5.54).",
                                    );
                                }
                            }
                        }

                        // Terrain effects card: always show defence modifier
                        // and movement cost so players can read a hex even
                        // without a unit selected (§6.23, §5.11).
                        let terrain_line = gs.map(|_| {
                            let def_mod = omdurman_rules::terrain_chart::defense_modifier(terrain);
                            let move_cost = omdurman_rules::terrain_chart::movement_cost(terrain)
                                .map(|c| c.value())
                                .unwrap_or(0);
                            let mut line = String::new();
                            if def_mod != 0 {
                                line.push_str(&format!("Defence {def_mod:+} (§6.23). "));
                            }
                            if move_cost > 0 {
                                line.push_str(&format!("Move cost {move_cost} MP (§5.11)."));
                            } else if line.is_empty() {
                                line.push_str("Gunboats only (§5.22).");
                            }
                            line
                        });
                        // Legibility hint when a unit is selected.
                        let hint =
                            selected_unit_id(&picker, &placed_units).and_then(|(unit_id, _)| {
                                gs.and_then(|gs| {
                                    movement_hint(gs, unit_id, hex, &game_map, &movement_path)
                                })
                            });
                        main_ref = hint
                            .as_deref()
                            .and_then(first_ref)
                            .or_else(|| terrain_line.as_deref().and_then(first_ref));
                        if let Some(line) = terrain_line.filter(|l| !l.is_empty()) {
                            ui.add_space(2.0);
                            crate::rulebook::render_refs_plain(ui, &line, Some(&rulebook));
                        }
                        if let Some(hint) = hint {
                            ui.add_space(2.0);
                            ui.separator();
                            // Plain-text § citations: the tooltip is
                            // click-through (see `interactable(false)` above);
                            // R opens the main rule instead (below).
                            crate::rulebook::render_refs_plain(ui, &hint, Some(&rulebook));
                        }
                        if let Some(number) = &main_ref {
                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new(format!(
                                    "R \u{2014} open \u{00a7}{number} in the rulebook"
                                ))
                                .color(crate::ui::palette::FAINT_INK)
                                .size(11.0)
                                .italics(),
                            );
                        }
                    });
                });
            main_ref
        });
    // R opens the tooltip's rule in the manual (its § citations can't be
    // clicked: the tooltip must never take the pointer, see above).
    if !ctx.egui_wants_keyboard_input()
        && keys.just_pressed(KeyCode::KeyR)
        && let Some(number) = shown.inner
    {
        crate::rulebook::request_open(ctx, &number);
    }
}

/// The first `§N` citation in `text`, if any.
fn first_ref(text: &str) -> Option<String> {
    crate::rulebook::split_refs(text)
        .into_iter()
        .find_map(|tok| match tok {
            crate::rulebook::RefTok::Ref(n) => Some(n.to_string()),
            crate::rulebook::RefTok::Text(_) => None,
        })
}

/// Build the per-hex terrain label. The `Terrain` enum's `Display` impl
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
        // The engine's own set-up check for this counter on this hex.
        Phase::Setup => {
            let placement = omdurman_rules::UnitPlacement {
                position: hex,
                ..*unit
            };
            let mut probe = gs.clone();
            probe.units.retain(|u| u.id != unit_id);
            Some(match probe.can_deploy_unit(&placement) {
                Ok(()) => "May set up here.".to_string(),
                Err(reason) => format!("Cannot set up here: {reason}."),
            })
        }
        Phase::Movement => {
            // Determine the effective origin for adjacency: if a path is
            // being built, the unit's next move starts from the path's end,
            // not the unit's current board position.
            let effective_from = movement_path.current_end().unwrap_or(from);
            let adjacent = effective_from.neighbors().contains(&hex);
            let in_zoc = gs.hex_in_enemy_zoc(hex, unit.profile.identity.owner(), unit.profile.kind);
            if !adjacent {
                if in_zoc {
                    return Some(
                        "In enemy ZOC \u{2014} a route may end here, not pass through (§5.43)."
                            .to_string(),
                    );
                }
                return Some("Click to plot the cheapest route here (§5.11).".to_string());
            }
            let tile = game_map.hexes.get(&hex);
            // The step itself, as the engine judges it: for a land unit the
            // Nile, the board edge, walls and the walled city (§5.22, §5.23),
            // the Zariba (§9.233), enemy units and forts (§6.54).
            if is_boat {
                if !tile.is_some_and(|t| t.terrain.is_nile()) {
                    return Some(
                        "Impassable: land \u{2014} gunboats stay on the Nile (§5.22).".into(),
                    );
                }
            } else if let Err(reason) = gs.check_land_step(unit, effective_from, hex) {
                return Some(match reason {
                    omdurman_rules::effects::RuleError::MoveBlockedByHexside(..) => {
                        "Behind a wall or the Zariba \u{2014} a click plots a route through a gate, breach or Zariba end (§5.23, §9.233).".to_string()
                    }
                    other => format!("Cannot step here: {other}."),
                });
            }
            // The step price the plot and the engine use (§5.11 Terrain
            // Effects Chart: terrain, road link, crossed hexside).
            let along_road = !is_boat
                && game_map
                    .roads
                    .contains(&omdurman_types::HexsideRef::new(effective_from, hex));
            let cost = i32::from(crate::picker::floor_movement_cost(
                game_map,
                effective_from,
                hex,
                is_boat,
                Some(gs),
            ));
            // Accumulated cost = path cost so far + this step's cost; what is
            // left is the engine's remaining allowance (night halving §8.1,
            // the sticky upstream cap §5.24) less the plotted path.
            let acc_cost = movement_path.cost_so_far + cost as i16;
            let left = gs
                .remaining_movement(unit_id)
                .saturating_sub(movement_path.cost_so_far);
            // Defence modifier at destination (§6.23).
            let def_mod = tile
                .map(|t| omdurman_rules::terrain_chart::defense_modifier(t.terrain))
                .unwrap_or(0);

            let mut lines = Vec::new();
            if cost == 0 {
                lines.push("Impassable terrain (\u{00a7}5.11).".into());
            } else if (cost as i16) > left {
                lines.push(format!(
                    "Out of MP: costs {cost}, {left} left (\u{00a7}5.11)."
                ));
            } else {
                let road_note = if along_road {
                    let base_cost = tile
                        .and_then(|t| omdurman_rules::terrain_chart::movement_cost(t.terrain))
                        .map_or(0, |c| c.value());
                    if base_cost > 1 {
                        format!(" (along the road instead of {base_cost})")
                    } else {
                        " (along the road)".into()
                    }
                } else {
                    String::new()
                };
                lines.push(format!(
                    "Move here: costs {cost} MP (accumulated {acc_cost}, {left} left, \u{00a7}5.11){road_note}."
                ));
            }
            // Stacking binds where a move ends, not on the way (§5.51-§5.53).
            if let Err(reason) = gs.check_stacking(unit, hex) {
                lines.push(format!("May pass through, not stop here: {reason}."));
            }
            if def_mod != 0 {
                lines.push(format!("Defence modifier: {def_mod} (§6.23)."));
            }
            if in_zoc {
                lines.push(
                    "Hex is in enemy ZOC \u{2014} movement stops here (\u{00a7}5.43).".into(),
                );
            }
            if is_night && unit.profile.identity.owner() == omdurman_types::Player::AngloEgyptian {
                lines.push("Night — Anglo-Egyptian movement halved (§8.1).".into());
            }
            if is_boat {
                // Annotate upstream/downstream direction and budget (§5.24).
                if let Some(ga) = gs.gunboat_allowances(unit) {
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
                }
                // FoK: flag the White Nile ↔ Blue Nile off-board crossing
                // (§9.345) -- a flat 6-MP upstream jump unique to this board.
                if gs.scenario == omdurman_types::Scenario::FallOfKhartoum
                    && gs.is_nile_mouth_crossing(effective_from, hex)
                {
                    lines.push("Nile-mouth crossing \u{2014} 6 MP flat (§9.345).".into());
                }
            }
            let occupants = gs.units.iter().filter(|u| u.position == hex).count();
            if occupants > 0 {
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
