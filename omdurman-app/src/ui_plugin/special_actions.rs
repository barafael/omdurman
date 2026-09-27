//! Movement-phase special actions (§5.21 transport, §5.3 zariba,
//! §6.53 demolition), the §6.63 artillery breach card, and the §10
//! optional-rule (mine/chain) setup UI.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn friendlies_transport_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    state: Res<crate::picker::PickerState>,
    placed_units: Query<(Entity, &crate::picker::PlacedUnit)>,
    mut submit: crate::submit::CheckedSubmit,
    peers: crate::peers::Peers,
    net: Res<NetState>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    let Some(gs) = game_state else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let local = peers.local();
    let is_host = net.is_host;

    // Eligibility and effect construction live on the rules engine (§5.21);
    // this system only decides the label and who may act.
    let selected = crate::picker::selected_unit_id(&state, &placed_units).map(|(uid, _)| uid);
    let action = gs.0.friendlies_transport_offer(selected);
    let action_label = match action {
        Some(omdurman_rules::FriendliesAction::Load { .. }) => Some("Load onto Gunboat"),
        // Show "Cross Nile" for the gunboat's owner.
        Some(omdurman_rules::FriendliesAction::Cross { .. }) => {
            (local.is_some() || is_host).then_some("Cross Nile")
        }
        Some(omdurman_rules::FriendliesAction::Disembark { .. }) => Some("Disembark"),
        None => None,
    };
    let Some(label) = action_label else { return };

    crate::ui::stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("friendlies_transport"),
        crate::ui::frames::card(crate::ui::palette::CARD_GOOD),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(13.0));
            crate::rulebook::refs_label(
                ui,
                "\u{1f6a2} Friendlies Transport (§5.21)",
                crate::ui::palette::ATTACKER,
                13.0,
            );
            if ui.button(label).clicked()
                && let Some(action) = action
            {
                submit.submit(
                    &gs.0,
                    omdurman_net::GameEvent::Effect(
                        omdurman_rules::effects::GameEffect::FriendliesTransport(action),
                    ),
                );
            }
        },
    );
}

// -- Special actions UI: Zariba construction + Royal Engineers demolition ------

/// Floating panel for §5.3 Zariba construction and §6.53 Demolition actions.
/// Appears during Movement phase when a relevant unit is selected.
/// Transient selection state for demolition target picking (§6.53).
#[derive(Resource, Default)]
pub(crate) struct DemolitionSelection {
    pub target: Option<omdurman_rules::DemolitionTarget>,
}

/// Placement mode for optional-rule river mines/chains (§10.11, §10.21).
/// Active during Setup phase for the Dervish player.
#[derive(Resource, Default)]
pub(crate) struct OptionalRulePlacement {
    /// `None` = idle. `Some(coord)` = a pending mine placement at that hex
    /// (emitted on next frame via the placement system).
    pub pending_mine: Option<omdurman_types::HexCoord>,
    /// Chain hexes being built up during placement (max 4).
    pub chain_hexes: Vec<omdurman_types::HexCoord>,
    /// Whether we are currently in chain-placement mode.
    pub placing_chain: bool,
}

#[allow(clippy::too_many_arguments)]
/// Floating panel for §5.3 Zariba construction and §6.53 Demolition actions.
/// Appears during Movement phase when a relevant unit is selected (the phase
/// gate is the `in_movement_phase` run condition; see `ui_phase_state`).
pub(crate) fn special_actions_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    state: Res<crate::picker::PickerState>,
    placed_units: Query<(Entity, &crate::picker::PlacedUnit)>,
    mut submit: crate::submit::CheckedSubmit,
    peers: crate::peers::Peers,
    net: Res<NetState>,
    mut demolition_sel: ResMut<DemolitionSelection>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    let Some(gs) = game_state else { return };
    let Some((uid, _)) = crate::picker::selected_unit_id(&state, &placed_units) else {
        return;
    };
    let Some(unit) = gs.0.find_unit(uid) else {
        return;
    };
    if unit.state.disrupted {
        return;
    }

    // Only the active player may take special actions.
    let local = peers.local();
    if local.is_none() && !net.is_host {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Zariba construction (§5.3): engineers or adjacent units.
    let can_construct = matches!(
        unit.profile.identity,
        omdurman_rules::UnitIdentity::RoyalEngineers
    );
    // Demolition (§6.53): Royal Engineers adjacent to a zariba hexside.
    let can_demolish = matches!(
        unit.profile.identity,
        omdurman_rules::UnitIdentity::RoyalEngineers
    ) && !unit.state.constructing_zariba;

    if !can_construct && !can_demolish {
        return;
    }

    // Only the sides the engine would accept (§5.3: campaign, A-E movement,
    // unmoved, no authored feature on the hexside).
    let unit_hex = unit.position;
    let legal_sides: Vec<(usize, omdurman_types::HexsideRef)> = if can_construct {
        unit_hex
            .neighbors()
            .into_iter()
            .enumerate()
            .map(|(idx, n)| (idx, omdurman_types::HexsideRef::new(unit_hex, n)))
            .filter(|(_, side)| gs.0.can_construct_zariba(&[uid], *side).is_ok())
            .collect()
    } else {
        Vec::new()
    };
    let has_construct_button = !legal_sides.is_empty();
    let has_demolish_button = can_demolish && gs.0.can_demolition(uid).is_ok();

    // Adjacent demolition targets (§6.53), discovered by the rules engine.
    let targets = gs.0.demolition_targets(uid);
    let adjacent_forts: Vec<omdurman_rules::UnitId> = targets
        .iter()
        .filter_map(|t| match t {
            omdurman_rules::DemolitionTarget::Fort(id) => Some(*id),
            _ => None,
        })
        .collect();
    let adjacent_walls: Vec<omdurman_types::HexsideRef> = targets
        .iter()
        .filter_map(|t| match t {
            omdurman_rules::DemolitionTarget::WallHexside(edge) => Some(*edge),
            _ => None,
        })
        .collect();
    let has_targets = !targets.is_empty();
    let has_demolish_button_full = has_demolish_button && has_targets;

    if !has_construct_button && !has_demolish_button_full {
        // Clear stale demolition selection when no eligible targets
        if !has_targets {
            demolition_sel.target = None;
        }
        return;
    }

    crate::ui::stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("special_actions"),
        crate::ui::frames::card(crate::ui::palette::CARD_ENGINEERING),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(13.0));

            if has_construct_button {
                crate::rulebook::refs_label(
                    ui,
                    "Construct Zariba (§5.3)",
                    crate::ui::palette::CARD_TITLE_TAN,
                    13.0,
                );
                ui.label(crate::ui::text::note(
                    "Place a zariba hexside adjacent to the unit's hex.",
                ));
                // Pick the construction side among the unit hex's six
                // neighbours (canonical `neighbors()` order = compass
                // directions East..NorthEast).
                const DIR_LABELS: [&str; 6] = ["E", "SE", "SW", "W", "NW", "NE"];
                ui.label(crate::ui::text::note("Construct on side:"));
                ui.horizontal(|ui| {
                    for &(idx, hexside) in &legal_sides {
                        if ui.small_button(DIR_LABELS[idx]).clicked() {
                            submit.submit(
                                &gs.0,
                                omdurman_net::GameEvent::Effect(
                                    omdurman_rules::effects::GameEffect::ConstructZariba {
                                        unit_ids: vec![uid],
                                        hexside,
                                    },
                                ),
                            );
                        }
                    }
                });
            }

            if has_demolish_button_full {
                if has_construct_button {
                    ui.add_space(4.0);
                }
                crate::rulebook::refs_label(
                    ui,
                    "Royal Engineers Demolition (§6.53)",
                    crate::ui::palette::UNFAVOURABLE,
                    13.0,
                );
                ui.label(crate::ui::text::note(
                    "Destroy adjacent fort or wall. Resolved at end of turn.",
                ));
                ui.add_space(2.0);

                // Fort targets
                for &fort_id in &adjacent_forts {
                    let label = format!("Fort at {}", {
                        if let Some(f) = gs.0.find_unit(fort_id) {
                            format!("({}, {})", f.position.q, f.position.r)
                        } else {
                            "?".to_string()
                        }
                    });
                    let selected = matches!(demolition_sel.target, Some(omdurman_rules::DemolitionTarget::Fort(id)) if id == fort_id);
                    if ui.selectable_label(selected, label).clicked() {
                        demolition_sel.target =
                            Some(omdurman_rules::DemolitionTarget::Fort(fort_id));
                    }
                }

                // Wall targets
                for &edge in &adjacent_walls {
                    let label = format!(
                        "Wall ({},{})–({},{})",
                        edge.a.q, edge.a.r, edge.b.q, edge.b.r
                    );
                    let selected = matches!(demolition_sel.target, Some(omdurman_rules::DemolitionTarget::WallHexside(e)) if e == edge);
                    if ui.selectable_label(selected, label).clicked() {
                        demolition_sel.target =
                            Some(omdurman_rules::DemolitionTarget::WallHexside(edge));
                    }
                }

                ui.add_space(4.0);
                if demolition_sel.target.is_some() {
                    if ui.button("Commit to Demolition").clicked()
                        && let Some(target) = demolition_sel.target
                    {
                        submit.submit(
                            &gs.0,
                            omdurman_net::GameEvent::Effect(
                                omdurman_rules::effects::GameEffect::Demolition {
                                    unit_id: uid,
                                    target,
                                },
                            ),
                        );
                        demolition_sel.target = None;
                    }
                } else {
                    ui.label(
                        egui::RichText::new("Select a target above.")
                            .size(11.0)
                            .color(crate::ui::palette::AWAITING),
                    );
                }
            }
        },
    );
}

#[allow(clippy::too_many_arguments)]
/// §6.63 Artillery Breach: with an artillery/howitzer unit selected during a
/// fire sub-phase, list every Wall hexside the engine's `can_fire_at_wall`
/// accepts for it; a click pre-rolls the d10 and broadcasts
/// [`GameEffect::ArtilleryBreachWall`]. The CRT cell (Eliminate ≥ 2) decides
/// the breach on the echo — the button is an attempt, not a promise. This is
/// the attacker's standard way into the walled city (the Royal Engineers'
/// §6.53 demolition is the other), so it is the one fire action that targets
/// a *hexside* rather than a hex.
pub(crate) fn artillery_breach_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    state: Res<crate::picker::PickerState>,
    placed_units: Query<(Entity, &crate::picker::PlacedUnit)>,
    mut submit: crate::submit::CheckedSubmit,
    peers: crate::peers::Peers,
    mut game_rng: ResMut<crate::GameRng>,
    mut layout: ResMut<crate::ScreenLayout>,
    mut fire_targets: ResMut<crate::fire::FireTargetCache>,
    mut aimed: ResMut<crate::hexside_layer::HighlightedHexside>,
) {
    use omdurman_rules::WeaponClass;
    use omdurman_rules::effects::GameEffect;

    // Cleared every frame; a hovered wall button below sets it again.
    aimed.set_if_neq(crate::hexside_layer::HighlightedHexside(None));

    let Some(gs) = game_state else { return };
    if !matches!(
        gs.0.phase,
        omdurman_rules::Phase::OffensiveFire(_) | omdurman_rules::Phase::DefensiveFire(_)
    ) {
        return;
    }
    let firing_player = gs.0.phase_player();
    if !peers.may_act(firing_player) {
        return;
    }
    let Some((uid, _)) = crate::picker::selected_unit_id(&state, &placed_units) else {
        return;
    };
    let Some(unit) = gs.0.find_unit(uid) else {
        return;
    };
    if unit.profile.identity.owner() != firing_player || unit.state.disrupted {
        return;
    }
    if !matches!(
        unit.profile.weapon,
        WeaponClass::Artillery | WeaponClass::Howitzer
    ) {
        return;
    }
    if gs.0.units_fired_this_phase.contains(&uid) {
        return;
    }

    // Every wall hexside this battery may currently fire at, nearest first.
    // (Engine re-validates on the echo; this list is just the clickable set.)
    // Cached per (battery, state) — see `FireTargetCache` — since each
    // `can_fire_at_wall` runs a LOS sweep over every wall on the board.
    let targets: Vec<(omdurman_types::HexsideRef, u16)> =
        fire_targets.wall_targets(&gs.0, uid).to_vec();
    // No card when no wall is in range: it would offer nothing (every
    // gunboat and battery far from the walls used to get one).
    if !targets.iter().any(|(_, range)| *range != u16::MAX) {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };
    crate::ui::stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("artillery_breach"),
        crate::ui::frames::card(crate::ui::palette::CARD_ENGINEERING),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(13.0));
            crate::rulebook::refs_label(
                ui,
                "Artillery Breach (§6.63)",
                crate::ui::palette::CARD_TITLE_RUST,
                13.0,
            );
            ui.label(
                crate::ui::text::note("Fire at a wall hexside. A CRT result of Eliminate 2+ breaches it; any enemy adjacent to the wall is eliminated."),
            );
            ui.add_space(2.0);
            for (edge, range) in &targets {
                if *range == u16::MAX {
                    continue;
                }
                let label = format!(
                    "Wall ({},{})–({},{})  [range {}]",
                    edge.a.q, edge.a.r, edge.b.q, edge.b.r, range
                );
                let button = ui.small_button(label);
                if button.hovered() {
                    aimed.set_if_neq(crate::hexside_layer::HighlightedHexside(Some(*edge)));
                }
                if button.clicked() {
                    // The "Wall Breached" / "Breach Attempt Failed" slip on
                    // the echo reports the outcome; no slip for the click.
                    let roll = game_rng.roll_d10();
                    submit.submit(
                        &gs.0,
                        omdurman_net::GameEvent::Effect(GameEffect::ArtilleryBreachWall {
                            firers: vec![uid],
                            target: *edge,
                            roll,
                        }),
                    );
                }
            }
        },
    );
}

// -- Top bar (cross-cutting) -------------------------------------------------

#[allow(clippy::too_many_arguments)]
/// The full-width top bar: mode-switching controls on the left and
/// phase/turn info beside them. Publishes its measured height to
/// [`crate::ScreenLayout::top_bar_height`] so every band below it (left rail,
/// stacked cards, charts sheet) starts clear of it. Visible in all non-Menu
/// states. While spectating a replay, it also hosts the "Back to lobby" exit
pub(crate) fn optional_rule_setup_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    peers: Peers,
    net: Res<NetState>,
    mut placement: ResMut<OptionalRulePlacement>,
    mut submit: crate::submit::CheckedSubmit,
    layout: Res<crate::ScreenLayout>,
) {
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, omdurman_rules::Phase::Setup) {
        placement.pending_mine = None;
        placement.placing_chain = false;
        placement.chain_hexes.clear();
        return;
    }

    // Only the Dervish player (or unbound host) can place mines/chains.
    let local = peers.local();
    let is_dervish = match local {
        Some(p) => p == omdurman_types::Player::Dervish,
        None => net.is_host,
    };
    if !is_dervish {
        return;
    }

    let has_mines =
        gs.0.optional_rules
            .contains(&omdurman_rules::OptionalRule::RiverMines);
    let has_chain =
        gs.0.optional_rules
            .contains(&omdurman_rules::OptionalRule::RiverChain);
    if !has_mines && !has_chain {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    crate::ui::anchored_card(
        ctx,
        egui::Id::new("optional_rule_setup"),
        egui::Align2::RIGHT_TOP,
        // Clear of the charts sheet / peek tab (see `right_inset`).
        egui::vec2(-(layout.right_inset + 10.0), 380.0),
        crate::ui::frames::card(crate::ui::palette::CARD_SETUP),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(12.0));

            if has_mines {
                crate::rulebook::refs_label(
                    ui,
                    "River Mines (§10.11)",
                    crate::ui::palette::CARD_TITLE_RUST,
                    12.0,
                );
                let mines_placed = gs.0.mines.len();
                ui.label(
                    egui::RichText::new(format!("Placed: {mines_placed}/2"))
                        .size(11.0)
                        .color(crate::ui::palette::TEXT_SOFT),
                );
                if mines_placed < 2 {
                    if placement.pending_mine.is_some() {
                        if ui.button("Click a Nile hex to place").clicked() {
                            placement.pending_mine = None;
                        }
                    } else if ui.button("Place River Mine").clicked() {
                        // Dummy coord; overwritten on hex click.
                        placement.pending_mine = Some(omdurman_types::HexCoord::new(99, 99));
                    }
                }
            }

            if has_chain {
                if has_mines {
                    ui.add_space(4.0);
                }
                crate::rulebook::refs_label(
                    ui,
                    "River Chain (§10.21)",
                    crate::ui::palette::HEADING_DIM,
                    12.0,
                );
                let chain_placed = gs.0.chain.as_ref().map(|c| c.hexes.len()).unwrap_or(0);
                let building = placement.placing_chain;
                if building {
                    ui.label(
                        egui::RichText::new(format!(
                            "Selecting hex {}/4...",
                            placement.chain_hexes.len() + 1
                        ))
                        .size(11.0)
                        .color(crate::ui::palette::TEXT_SOFT),
                    );
                    if ui.button("Finish Chain").clicked() && !placement.chain_hexes.is_empty() {
                        submit.submit(
                            &gs.0,
                            omdurman_net::GameEvent::Effect(
                                omdurman_rules::effects::GameEffect::PlaceChain {
                                    hexes: std::mem::take(&mut placement.chain_hexes),
                                },
                            ),
                        );
                        placement.placing_chain = false;
                    }
                    if ui.button("Cancel").clicked() {
                        placement.chain_hexes.clear();
                        placement.placing_chain = false;
                    }
                } else if chain_placed == 0 {
                    if ui.button("Place River Chain").clicked() {
                        placement.placing_chain = true;
                    }
                } else {
                    ui.label(
                        egui::RichText::new("Chain placed")
                            .size(11.0)
                            .color(crate::ui::palette::GOOD),
                    );
                }
            }
        },
    );
}
