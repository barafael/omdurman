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
    mut layout: ResMut<crate::ScreenLayout>,
) {
    let Some(gs) = game_state else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Eligibility and effect construction live on the rules engine (§5.21);
    // this system only labels the offers and lets the Anglo-Egyptian seat
    // pick one.
    if !peers.may_act(omdurman_types::Player::AngloEgyptian) {
        return;
    }
    let selected = crate::picker::selected_unit_id(&state, &placed_units).map(|(uid, _)| uid);
    let offers = gs.0.friendlies_transport_offers(selected);
    if offers.is_empty() {
        return;
    }

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
            for action in offers {
                let label = match action {
                    omdurman_rules::FriendliesAction::Load { .. } => {
                        "Load onto Gunboat".to_string()
                    }
                    omdurman_rules::FriendliesAction::Disembark { to, .. } => {
                        format!("Disembark to ({}, {})", to.q, to.r)
                    }
                };
                if ui.button(label).clicked() {
                    submit.submit(
                        &gs.0,
                        omdurman_net::GameEvent::Effect(
                            omdurman_rules::effects::GameEffect::FriendliesTransport(action),
                        ),
                    );
                }
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
    /// Whether the next Nile-hex click lays a mine.
    pub placing_mine: bool,
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
    // Any selection shape: a single counter or a whole stack (a brigade is
    // usually moved, and so selected, as one). The demolition is the Royal
    // Engineers' alone, so they stand for the selection when they are in it.
    let ids = crate::picker::selected_unit_ids(&state, &placed_units);
    let Some(uid) = ids
        .iter()
        .copied()
        .find(|id| {
            gs.0.find_unit(*id)
                .is_some_and(|u| u.profile.identity == omdurman_rules::UnitIdentity::RoyalEngineers)
        })
        .or(ids.first().copied())
    else {
        return;
    };
    let Some(unit) = gs.0.find_unit(uid) else {
        return;
    };

    // Only the active player may take special actions.
    let local = peers.local();
    if local.is_none() && !net.is_host {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Demolition (§6.53): the Royal Engineers next to an enemy fort or wall.
    let can_demolish = matches!(
        unit.profile.identity,
        omdurman_rules::UnitIdentity::RoyalEngineers
    ) && !unit.state.constructing_zariba;

    // Zariba construction (§5.3): any Anglo-Egyptian infantry inside the
    // printed Zariba, next to printed Zariba hexsides it may build -- the
    // engine's own check decides, unit by unit, so a selected stack sends
    // exactly the battalions that may build.
    let builders: Vec<(omdurman_rules::UnitId, omdurman_types::HexsideRef)> = ids
        .iter()
        .filter_map(|&id| {
            let hex = gs.0.find_unit(id)?.position;
            hex.neighbors()
                .into_iter()
                .map(|n| omdurman_types::HexsideRef::new(hex, n))
                .find(|side| gs.0.can_construct_zariba(&[id], *side).is_ok())
                .map(|side| (id, side))
        })
        .collect();
    let has_construct_button = !builders.is_empty();
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
                ui.label(crate::ui::text::note(if builders.len() == 1 {
                    "Hold this battalion here all turn: at the end of the turn it has built \
                     every printed Zariba hexside it stands beside. It may not move, fire \
                     offensively or melee this turn."
                } else {
                    "Hold these battalions here all turn: at the end of the turn they have \
                     built every printed Zariba hexside they stand beside. They may not \
                     move, fire offensively or melee this turn."
                }));
                let label = if builders.len() == 1 {
                    "Build the Zariba here".to_string()
                } else {
                    format!("Build the Zariba here ({} battalions)", builders.len())
                };
                if ui.small_button(label).clicked() {
                    // One effect per hexside named (a stack shares one).
                    let mut sides: Vec<omdurman_types::HexsideRef> =
                        builders.iter().map(|&(_, side)| side).collect();
                    sides.dedup();
                    for side in sides {
                        submit.submit(
                            &gs.0,
                            omdurman_net::GameEvent::Effect(
                                omdurman_rules::effects::GameEffect::ConstructZariba {
                                    unit_ids: builders
                                        .iter()
                                        .filter(|&&(_, s)| s == side)
                                        .map(|&(id, _)| id)
                                        .collect(),
                                    hexside: side,
                                },
                            ),
                        );
                    }
                }
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
    view: crate::picker::HexMapView,
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
    // §10.23 b: British artillery may fire at the river chain.
    let chain = gs.0.can_fire_at_chain(uid).ok();
    // No card when no wall (or chain) is in range: it would offer nothing
    // (every gunboat and battery far from the walls used to get one).
    if !targets.iter().any(|(_, range)| *range != u16::MAX) && chain.is_none() {
        return;
    }

    let in_reach: Vec<(omdurman_types::HexsideRef, u16)> = targets
        .iter()
        .copied()
        .filter(|(_, range)| *range != u16::MAX)
        .collect();
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
                crate::ui::text::note("Fire at a wall: a combat result of 2 or more breaches it and eliminates one enemy unit beside it."),
            );
            ui.add_space(2.0);
            if let Some((_, range, hex)) = chain {
                let _ = hex;
                let button = ui.small_button(format!(
                    "The river chain \u{00b7} range {} \u{00b7} 3+ sinks it (§10.23)",
                    range.value()
                ));
                if button.clicked() {
                    submit.submit(
                        &gs.0,
                        omdurman_net::GameEvent::Effect(GameEffect::SinkChain {
                            firers: vec![uid],
                            roll: game_rng.roll_d10(),
                        }),
                    );
                }
            }
            let mut fire_at: Option<omdurman_types::HexsideRef> = None;
            for (number, (edge, range)) in in_reach.iter().enumerate() {
                let label = format!(
                    "{} \u{00b7} {} \u{00b7} range {range}",
                    number + 1,
                    compass(unit.position, *edge, &view)
                );
                let button = ui.small_button(label);
                if button.hovered() {
                    aimed.set_if_neq(crate::hexside_layer::HighlightedHexside(Some(*edge)));
                }
                if button.clicked() {
                    fire_at = Some(*edge);
                }
            }
            ui.label(crate::ui::text::note(
                "Or click a numbered wall on the map.",
            ));
            // The numbered badges on the board, clickable like the buttons.
            if let Some(edge) = wall_badges(ui.ctx(), &in_reach, &view, &mut aimed) {
                fire_at = Some(edge);
            }
            if let Some(edge) = fire_at {
                // The "Wall Breached" / "Breach Attempt Failed" slip on the
                // echo reports the outcome; no slip for the click.
                let roll = game_rng.roll_d10();
                submit.submit(
                    &gs.0,
                    omdurman_net::GameEvent::Effect(GameEffect::ArtilleryBreachWall {
                        firers: vec![uid],
                        target: edge,
                        roll,
                    }),
                );
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
        placement.placing_mine = false;
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
                    egui::RichText::new(format!(
                        "Placed: {mines_placed}/{}",
                        omdurman_rules::effects::MAX_MINES
                    ))
                    .size(11.0)
                    .color(crate::ui::palette::TEXT_SOFT),
                );
                if mines_placed < omdurman_rules::effects::MAX_MINES {
                    if placement.placing_mine {
                        if ui
                            .button("Click a Nile hex south of the Khor Shambat")
                            .clicked()
                        {
                            placement.placing_mine = false;
                        }
                    } else if ui.button("Place River Mine").clicked() {
                        placement.placing_mine = true;
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
                            "Selecting hex {}/{}...",
                            placement.chain_hexes.len() + 1,
                            omdurman_rules::effects::MAX_CHAIN_HEXES
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

/// The world-space midpoint of a hexside.
fn hexside_mid(edge: omdurman_types::HexsideRef, view: &crate::picker::HexMapView) -> Vec3 {
    let origin = view.layout.adjusted_origin(&view.overlay.params);
    let a = omdurman_hexmap::hex_world_pos(edge.a, origin, &view.overlay.params);
    let b = omdurman_hexmap::hex_world_pos(edge.b, origin, &view.overlay.params);
    (a + b) / 2.0
}

/// Where the wall lies from the battery, as a player reads the map: one of
/// eight compass points (north is up the board).
fn compass(
    from: omdurman_types::HexCoord,
    edge: omdurman_types::HexsideRef,
    view: &crate::picker::HexMapView,
) -> &'static str {
    let origin = view.layout.adjusted_origin(&view.overlay.params);
    let at = omdurman_hexmap::hex_world_pos(from, origin, &view.overlay.params);
    let mid = hexside_mid(edge, view);
    // Bearing clockwise from north (-z on the board).
    let bearing = (mid.x - at.x)
        .atan2(-(mid.z - at.z))
        .to_degrees()
        .rem_euclid(360.0);
    const POINTS: [&str; 8] = [
        "north",
        "north-east",
        "east",
        "south-east",
        "south",
        "south-west",
        "west",
        "north-west",
    ];
    POINTS[((bearing + 22.5) / 45.0) as usize % 8]
}

/// Draw a numbered, clickable badge on every wall in reach; returns the
/// wall whose badge was clicked. Hovering a badge highlights its wall.
fn wall_badges(
    ctx: &egui::Context,
    in_reach: &[(omdurman_types::HexsideRef, u16)],
    view: &crate::picker::HexMapView,
    aimed: &mut crate::hexside_layer::HighlightedHexside,
) -> Option<omdurman_types::HexsideRef> {
    let Ok((camera, camera_transform)) = view.cameras.single() else {
        return None;
    };
    let mut clicked = None;
    for (number, (edge, _)) in in_reach.iter().enumerate() {
        let mid = hexside_mid(*edge, view);
        let Ok(screen) = camera.world_to_viewport(camera_transform, Vec3::new(mid.x, 2.0, mid.z))
        else {
            continue;
        };
        egui::Area::new(egui::Id::new(("wall_badge", number)))
            .fixed_pos(egui::pos2(screen.x - 10.0, screen.y - 10.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let button = ui.add(
                    egui::Button::new(
                        egui::RichText::new(format!("{}", number + 1))
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(crate::ui::palette::BTN_DANGER)
                    .min_size(egui::vec2(20.0, 20.0)),
                );
                if button.hovered() {
                    aimed.0 = Some(*edge);
                }
                if button.clicked() {
                    clicked = Some(*edge);
                }
            });
    }
    clicked
}
