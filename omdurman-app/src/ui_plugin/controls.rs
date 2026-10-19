//! The left-rail control sections: game controls, victory scoreboard,
//! setup controls, and the game log.
use super::*;

/// What ending the current phase would do: `Ok(label of the phase that
/// follows)` — with the new turn owner when the turn passes — or the
/// engine's refusal. Dry-runs `AdvancePhase` on a clone of the engine state
/// projected over this peer's unconfirmed submissions (see `submit`).
pub(crate) fn end_phase_preview(
    gs: &omdurman_rules::effects::GameState,
    unconfirmed: &[omdurman_net::GameEvent],
) -> Result<String, omdurman_rules::effects::RuleError> {
    let mut next = crate::submit::projected_state(gs, unconfirmed);
    let owner_before = next.active_player;
    omdurman_rules::effects::apply_effect(
        &mut next,
        &omdurman_rules::effects::GameEffect::AdvancePhase,
    )?;
    let label = crate::ui_phase_state::UiPhaseState::derive(&next).phase_label();
    Ok(if !next.game_over && next.active_player != owner_before {
        format!(
            "{label} ({} turn)",
            crate::ui::faction_name(next.active_player)
        )
    } else {
        label.to_string()
    })
}

/// The hexes ending the phase would close to an advance after combat that a
/// unit of the phasing side could still make (§6.82; §7.6 for the
/// Anglo-Egyptian player, whose advance is optional) -- the End Phase
/// button asks before letting them lapse.
pub(crate) fn advances_lost_by_ending(
    gs: &omdurman_rules::effects::GameState,
    unconfirmed: &[omdurman_net::GameEvent],
) -> Vec<omdurman_types::HexCoord> {
    let now = crate::submit::projected_state(gs, unconfirmed);
    let open: Vec<omdurman_types::HexCoord> = now
        .vacated_by_combat
        .iter()
        .filter(|(hex, eligible)| {
            eligible
                .iter()
                .any(|&id| now.can_advance_after_combat(id, **hex).is_ok())
        })
        .map(|(hex, _)| *hex)
        .collect();
    if open.is_empty() {
        return open;
    }
    let mut next = now.clone();
    if omdurman_rules::effects::apply_effect(
        &mut next,
        &omdurman_rules::effects::GameEffect::AdvancePhase,
    )
    .is_err()
    {
        return Vec::new();
    }
    // The §6.42 bridge keeps the windows open into the second subphase.
    open.into_iter()
        .filter(|hex| !next.vacated_by_combat.contains_key(hex))
        .collect()
}

/// The End Phase request for the phase `gs` is in, stamped with it (§4): the
/// engine refuses it once that phase has ended, so a second request for the
/// same phase -- a double click, a teammate's click within the same round
/// trip -- can never end the *next* phase too.
pub(crate) fn end_phase_effect(
    gs: &omdurman_rules::effects::GameState,
) -> omdurman_rules::effects::GameEffect {
    omdurman_rules::effects::GameEffect::AdvancePhaseFrom {
        turn: gs.current_turn,
        player: gs.active_player,
        phase: gs.phase,
    }
}

/// Whether `event` ends a phase (either End Phase effect).
pub(crate) fn ends_phase(event: &omdurman_net::GameEvent) -> bool {
    use omdurman_rules::effects::GameEffect;
    matches!(
        event,
        omdurman_net::GameEvent::Effect(
            GameEffect::AdvancePhase | GameEffect::AdvancePhaseFrom { .. }
        )
    )
}

/// Submit the End Phase request for the phase this peer's pending
/// submissions lead to (see [`end_phase_effect`]).
fn submit_end_phase(gs: &omdurman_rules::effects::GameState, pending: &mut crate::PendingEdits) {
    let unconfirmed: Vec<omdurman_net::GameEvent> =
        pending.unconfirmed.iter().map(|(_, e)| e.clone()).collect();
    let projected = crate::submit::projected_state(gs, &unconfirmed);
    pending.submit_game(omdurman_net::GameEvent::Effect(end_phase_effect(
        &projected,
    )));
}

/// `E` pressed this frame without modifiers: a fresh press only -- the key
/// repeat of a held `E` must not end phase after phase.
fn end_phase_hotkey(modifiers: egui::Modifiers, events: &[egui::Event]) -> bool {
    modifiers.is_none()
        && events.iter().any(|e| {
            matches!(
                e,
                egui::Event::Key {
                    key: egui::Key::E,
                    pressed: true,
                    repeat: false,
                    ..
                }
            )
        })
}

/// The memoized End-Phase dry run ([`end_phase_button`]): the pending-queue
/// length it was derived against and the label/refusal it produced.
/// Invalidate on engine-state change via the caller's `state_moved` flag.
#[derive(Clone)]
struct EndPhasePreviewCache {
    unconfirmed_len: usize,
    next: Result<String, omdurman_rules::effects::RuleError>,
    /// [`advances_lost_by_ending`].
    lost_advances: Vec<omdurman_types::HexCoord>,
}

pub(crate) fn game_control_section(
    ui: &mut egui::Ui,
    state: &crate::GameStateResource,
    peers: &Peers,
    pending: Option<&mut crate::PendingEdits>,
    local_setup_ready: Option<&mut crate::peers::LocalSetupReady>,
    // The staged fire allocations: End Phase discards them.
    allocation: Option<&mut crate::fire_allocation::FireAllocationState>,
    // The victory modal: the rail reopens it.
    victory: &mut VictoryModalState,
    state_moved: bool,
) {
    let Some(pending) = pending else {
        return;
    };
    // The turn, the phase and who acts are in the top bar; this block holds
    // the phase's one control.
    let acting = state.0.player_to_act().unwrap_or(state.0.phase_player());
    let my_turn = peers.may_act(acting);
    let in_setup = matches!(state.0.phase, omdurman_rules::Phase::Setup);
    let game_over = state.0.game_over;
    if game_over {
        let result = state.0.game_result.map(|r| r.display_key());
        ui.colored_label(
            crate::ui::palette::GOLD,
            match result {
                Some(result) => format!("Game over \u{2014} {result}"),
                None => "Game over".to_string(),
            },
        );
        if victory.dismissed && ui.button("Show result").clicked() {
            victory.dismissed = false;
        }
    } else if in_setup {
        // Per-member setup readiness needs the local flag; a None resource
        // just falls back to "ready" semantics for unbound sessions.
        if let Some(local_setup_ready) = local_setup_ready {
            setup_control_section(ui, state, peers, pending, local_setup_ready);
        }
    } else if my_turn {
        // Each player ends their *own* turn: the End Phase button is shown only
        // to whoever controls the active faction.
        end_phase_button(ui, state, pending, allocation, state_moved);
    }
}

/// Clear the per-game state that outlives a game when a new one begins:
///
/// * every teammate's setup-ready flag (`Ephemeral::SetupReady`): the flags
///   live on the peer entities, so a teammate's "ready" from the last game's
///   set-up would otherwise count in the next one -- and the faction's
///   `ConfirmSetupReady` would go out before that teammate had deployed
///   anything;
/// * the dispatch slips and the Log menu's lines, and the combat cards
///   (pinned ones never expire), which belong to the last game.
///
/// A game is told apart by its `StartGame`
/// ([`crate::scenario_setup::start_game_key`]), so a history install or
/// rebuild of the *same* game keeps all of it (a reconnecting teammate
/// re-announces its flag anyway).
pub(crate) fn reset_per_game_ui_on_new_game(
    mut commands: Commands,
    recorder: Res<crate::game_record::GameRecorder>,
    flags: Query<Entity, With<crate::peers::SetupReadyFlag>>,
    mut dispatches: ResMut<crate::dispatch::Dispatches>,
    mut cards: ResMut<crate::combat_card::CombatCardQueue>,
    mut last: Local<Option<crate::scenario_setup::StartGameKey>>,
) {
    if !recorder.is_changed() {
        return;
    }
    let key = recorder
        .record
        .as_ref()
        .and_then(crate::scenario_setup::start_game_key);
    if key.is_none() || key == *last {
        return;
    }
    *last = key;
    for entity in &flags {
        commands
            .entity(entity)
            .remove::<crate::peers::SetupReadyFlag>();
    }
    dispatches.clear();
    cards.clear();
}

/// The score block of the command rail: the scenario's own victory measure
/// (§9.14 VP ladder, §9.24 eliminations, §9.35 FoK), and the §8 night rules
/// while it is night.
pub(crate) fn score_section(ui: &mut egui::Ui, state: &crate::GameStateResource) {
    if matches!(state.0.phase, omdurman_rules::Phase::Setup) {
        return;
    }
    match state.0.scenario {
        omdurman_types::Scenario::FallOfKhartoum => crate::fok_panel::fok_status_section(ui, state),
        omdurman_types::Scenario::Historical => historical_scoreboard(ui, state),
        omdurman_types::Scenario::Campaign => victory_point_scoreboard(ui, state),
    }
    if state.0.day_night == omdurman_types::DayNight::Night {
        ui.add_space(6.0);
        crate::rulebook::refs_rich(ui, "Night rules (§8)", 13.0, |t| {
            t.strong().color(crate::ui::palette::INFO)
        });
        ui.label(
            egui::RichText::new(
                "\u{2022} A-E movement halved\n\
                 \u{2022} Fire ranges halved (min 1)\n\
                 \u{2022} No howitzer fire",
            )
            .small()
            .color(crate::ui::palette::TEXT_SOFT),
        );
    }
}

/// The End Phase control: labelled with the phase that follows, disabled with
/// the engine's reason when it would refuse, and a two-click confirm when
/// fire allocations are staged but unresolved (ending the phase drops them,
/// §6.41 allocations are per fire sub-phase).
fn end_phase_button(
    ui: &mut egui::Ui,
    state: &crate::GameStateResource,
    pending: &mut crate::PendingEdits,
    allocation: Option<&mut crate::fire_allocation::FireAllocationState>,
    state_moved: bool,
) {
    // The preview dry-runs `AdvancePhase` on a full clone of the engine state
    // projected over the pending submissions -- far too much work to repeat
    // every frame for a button label. It is memoized in egui's temp storage:
    // recomputed when the engine state moved (the caller's change flag) or
    // the pending queue changed length, reused otherwise.
    let preview_id = egui::Id::new("end_phase_preview_cache");
    let unconfirmed_len = pending.unconfirmed.len();
    let cached = ui
        .data(|d| d.get_temp::<EndPhasePreviewCache>(preview_id))
        .filter(|c| !state_moved && c.unconfirmed_len == unconfirmed_len);
    let (next, lost_advances) = match cached {
        Some(cached) => (cached.next, cached.lost_advances),
        None => {
            let unconfirmed: Vec<omdurman_net::GameEvent> =
                pending.unconfirmed.iter().map(|(_, e)| e.clone()).collect();
            let next = end_phase_preview(&state.0, &unconfirmed);
            let lost_advances = advances_lost_by_ending(&state.0, &unconfirmed);
            ui.data_mut(|d| {
                d.insert_temp(
                    preview_id,
                    EndPhasePreviewCache {
                        unconfirmed_len,
                        next: next.clone(),
                        lost_advances: lost_advances.clone(),
                    },
                )
            });
            (next, lost_advances)
        }
    };
    let current = crate::ui_phase_state::UiPhaseState::derive(&state.0).phase_label();
    // One End Phase at a time: while ours is unconfirmed the button waits for
    // its echo (the label above already projects over it), so a double
    // click or a held key cannot skip the next phase.
    if pending.unconfirmed.iter().any(|(_, e)| ends_phase(e)) {
        ui.add_enabled(false, egui::Button::new("Ending phase\u{2026}"))
            .on_disabled_hover_text(format!("Ending {current}; waiting for the host."));
        return;
    }
    let next = match next {
        Ok(next) => next,
        Err(reason) => {
            let reason = reason.to_string();
            ui.add_enabled(false, egui::Button::new("End Phase"))
                .on_disabled_hover_text(&reason);
            // What is still to do, as a task rather than a bare refusal.
            crate::rulebook::refs_label(
                ui,
                &format!("Before ending the phase: {reason}."),
                crate::ui::palette::CAUTION,
                13.0,
            );
            return;
        }
    };
    let staged = allocation
        .as_ref()
        .filter(|a| !a.committed)
        .map_or(0, |a| a.staged());
    // Ending the phase would discard staged fire or let an advance lapse:
    // ask first.
    let needs_confirm = staged > 0 || !lost_advances.is_empty();
    // The armed (awaiting-confirm) state is keyed on the phase, so it never
    // survives into the next phase.
    let armed_id = egui::Id::new("end_phase_discard_armed");
    let phase_key = format!(
        "{}:{:?}:{:?}",
        state.0.current_turn.value(),
        state.0.active_player,
        state.0.phase
    );
    let armed = needs_confirm
        && ui.data(|d| d.get_temp::<String>(armed_id)).as_deref() == Some(phase_key.as_str());
    let hover = format!("End {current}; next: {next}");
    if armed {
        let s = if staged == 1 { "" } else { "s" };
        let hexes: Vec<String> = lost_advances.iter().map(|h| h.to_string()).collect();
        let (question, why) = match (staged, hexes.is_empty()) {
            (0, _) => (
                format!("Skip the advance into {} & end phase?", hexes.join(", ")),
                "Units may still advance into the vacated hex (§6.82, §7.6).",
            ),
            (_, true) => (
                format!("Discard {staged} attack{s} & end phase?"),
                "The staged fire allocations have not been resolved.",
            ),
            _ => (
                format!(
                    "Discard {staged} attack{s}, skip the advance into {} & end phase?",
                    hexes.join(", ")
                ),
                "Staged fire is unresolved, and units may still advance (§6.82, §7.6).",
            ),
        };
        let confirm = ui
            .add(egui::Button::new(question).fill(crate::ui::palette::BTN_DANGER))
            .on_hover_text(why);
        let keep = ui.button("Go back");
        if confirm.clicked() {
            ui.data_mut(|d| d.remove::<String>(armed_id));
            if let Some(allocation) = allocation {
                allocation.attacks.clear();
                allocation.specials.clear();
                allocation.panel_open = false;
            }
            crate::ui_trace::button("End Phase (discard allocations)");
            submit_end_phase(&state.0, pending);
        } else if keep.clicked() {
            ui.data_mut(|d| d.remove::<String>(armed_id));
        }
        return;
    }
    // `E` ends the phase like a click (it arms the same discard confirm
    // when attacks are staged); never while a text field has the keyboard.
    let hotkey = !ui.ctx().egui_wants_keyboard_input()
        && ui.input(|i| end_phase_hotkey(i.modifiers, &i.events));
    if ui
        .button(format!("End phase \u{2192} {next}  (E)"))
        .on_hover_text(hover)
        .clicked()
        || hotkey
    {
        if needs_confirm {
            ui.data_mut(|d| d.insert_temp(armed_id, phase_key));
        } else {
            crate::ui_trace::button("End Phase");
            submit_end_phase(&state.0, pending);
        }
    }
}

/// The Historical scenario's progress (§9.24): each side's level comes from
/// the enemy units it has eliminated -- not victory points -- and the net
/// result is the higher level less the lower.
fn historical_scoreboard(ui: &mut egui::Ui, state: &crate::GameStateResource) {
    use omdurman_rules::HistoricalVictoryLevel as Level;
    use omdurman_types::Player;
    let ledger = &state.0.victory;
    let (ae, d) = ledger.historical_levels();
    crate::ui::section_header(ui, "Score");
    for (who, level) in [(Player::AngloEgyptian, ae), (Player::Dervish, d)] {
        let killed = ledger.units_eliminated_by(who);
        let next = level
            .next_threshold(who)
            .map(|n| format!(" (next at {n})"))
            .unwrap_or_default();
        ui.colored_label(
            crate::ui::faction_color(who),
            format!(
                "{who}: destroyed {killed} {} units -- {}{next}",
                who.opponent(),
                level.name()
            ),
        );
    }
    let (text, color) = match Level::net(ae, d) {
        (None, _) => ("Net: Draw".to_string(), crate::ui::palette::TEXT_MUTED),
        (Some(p), level) => (
            format!("Net: {p} {}", level.name()),
            crate::ui::faction_color(p),
        ),
    };
    ui.horizontal(|ui| {
        ui.colored_label(color, text);
        crate::rulebook::ref_link(ui, "9.24", 11.0);
    });
}

/// The Campaign §9.14 victory-point scoreboard. Extracted from
/// `game_control_section` so the FoK scenario can swap in its own
/// victory-progress panel ([`crate::fok_panel::fok_status_section`]) instead.
fn victory_point_scoreboard(ui: &mut egui::Ui, state: &crate::GameStateResource) {
    let ae_vp = state
        .0
        .victory
        .total_for(omdurman_types::Player::AngloEgyptian)
        .value();
    let dv_vp = state
        .0
        .victory
        .total_for(omdurman_types::Player::Dervish)
        .value();
    let net = ae_vp - dv_vp;
    crate::ui::section_header(ui, "Score");
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("A-E {ae_vp}")).color(crate::ui::palette::AE));
        ui.label(egui::RichText::new("\u{2013}").color(crate::ui::palette::TEXT_DIM));
        ui.label(
            egui::RichText::new(format!("{dv_vp} Dervish")).color(crate::ui::palette::DERVISH),
        );
    });
    // "If the game ended now" (§9.14): the Tomb's 25 VP to whoever holds it
    // now (already in the ledger once the game is over), and the level of
    // the difference.
    let tomb = (!state.0.game_over)
        .then(|| omdurman_rules::effects::mahdis_tomb_controller(&state.0))
        .flatten();
    let projected = net
        + match tomb {
            Some(omdurman_types::Player::AngloEgyptian) => 25,
            Some(omdurman_types::Player::Dervish) => -25,
            None => 0,
        };
    if let Some(holder) = tomb {
        ui.label(
            egui::RichText::new(format!(
                "Mahdi's Tomb: held by the {holder} (+25 VP at the end)"
            ))
            .size(12.0)
            .color(crate::ui::palette::TEXT_DIM),
        );
    }
    let level = omdurman_rules::CampaignVictoryLevel::from_superiority(
        omdurman_rules::VictoryPoints::new(projected),
    );
    let level_text = match level {
        omdurman_rules::CampaignVictoryLevel::Draw => "a draw".to_string(),
        omdurman_rules::CampaignVictoryLevel::Marginal(p) => format!("a {p} Marginal victory"),
        omdurman_rules::CampaignVictoryLevel::Tactical(p) => format!("a {p} Tactical victory"),
        omdurman_rules::CampaignVictoryLevel::Decisive(p) => format!("a {p} Decisive victory"),
    };

    // The projected level and the VP by source (§9.14), collapsed: early
    // in the game the projection is only the Tomb's 25 VP.
    ui.collapsing("Details", |ui| {
        ui.style_mut().override_font_id = Some(egui::FontId::proportional(11.0));
        crate::rulebook::refs_label(
            ui,
            &format!("If the game ended now: {level_text} (§9.14)"),
            crate::ui::palette::TEXT_MUTED,
            12.0,
        );
        // Every source, its count and points -- a zero row says what is
        // still to be won (§9.14).
        for (player, sources) in super::victory::VP_SOURCES {
            ui.colored_label(crate::ui::faction_color(player), format!("{player}:"));
            for &src in sources {
                let n = super::victory::vp_count(&state.0, src);
                let pts = src.points().value();
                let line = format!("  {src}: {n} \u{d7} {pts}");
                if n == 0 {
                    ui.colored_label(crate::ui::palette::TEXT_DIM, line);
                } else {
                    ui.label(format!("{line} = {}", n as i32 * pts));
                }
            }
        }
    });

    ui.add_space(4.0);
}

/// The set-up rail's one-line brief for the local side: in the Campaign
/// the Anglo-Egyptians set nothing up (§9.113) -- "place your forces" sent
/// a player hunting for counters that were not there.
fn setup_heading(
    gs: &omdurman_rules::effects::GameState,
    local: Option<omdurman_types::Player>,
) -> &'static str {
    match local {
        Some(side) if gs.setup_target(side) == 0 => {
            "Nothing to set up: your forces march on as reinforcements. Press Ready."
        }
        Some(side) if gs.require_setup_turn(side).is_err() => {
            "Deployment -- the other side places its forces first."
        }
        _ => "Deployment -- place your forces, then Ready.",
    }
}

/// The Setup-phase controls: per-faction deployed/target counts and the local
/// player's one-way "Ready" confirmation. Setup is sequential
/// (§9.111/§9.211/§9.321): the first side deploys and confirms Ready, which
/// fixes its deployment, and only then may the second side deploy
/// (`GameState::require_setup_turn`). The engine auto-advances to Movement once
/// both are ready, so there's no explicit "advance" click. An unbound session
/// (no faction binding) confirms the two sides in the same order, and the
/// second confirmation ("Begin battle") starts the game.
fn setup_control_section(
    ui: &mut egui::Ui,
    state: &crate::GameStateResource,
    peers: &Peers,
    pending: &mut crate::PendingEdits,
    local_setup_ready: &mut crate::peers::LocalSetupReady,
) {
    use omdurman_types::Player;

    if !peers.is_spectator() {
        ui.label(
            egui::RichText::new(setup_heading(&state.0, peers.local()))
                .size(12.0)
                .color(crate::ui::palette::TEXT),
        );
    }

    // The local member's §1.1 command scope, when one was assigned.
    if let Some(scope) = peers.local_scope() {
        ui.colored_label(
            crate::ui::palette::HEADING,
            format!("Your command: {scope}"),
        );
    }

    // Per-faction deployed/target + ready status, for both sides.
    for player in [Player::AngloEgyptian, Player::Dervish] {
        let label = crate::ui::faction_abbrev(player);
        let deployed = state.0.setup_deployed_count(player);
        let count = format!("{deployed}/{}", state.0.setup_target(player));
        let ready = state.0.setup_ready(player);
        let mark = if ready { "  \u{2713} ready" } else { "" };
        let color = if ready {
            crate::ui::palette::GOLD
        } else {
            crate::ui::palette::TEXT
        };
        ui.colored_label(color, format!("{label}: {count}{mark}"));
    }

    ui.add_space(2.0);

    let local = peers.local();
    match local {
        // Bound player: confirm ready for *your* faction (one-way). In a
        // commanded session (§1.1) every member of the faction readies their
        // own command first; the faction's engine-level confirm fires once
        // all of them are ready (the last member submits it).
        Some(player) => {
            if state.0.setup_ready(player) {
                ui.colored_label(
                    crate::ui::palette::GOLD,
                    "\u{2713} You are ready -- waiting for the other side.",
                );
            } else if let Err(wait) = state.0.require_setup_turn(player) {
                // Sequential set-up (§9.111/§9.211/§9.321): the other side
                // deploys first; its counters appear as it places them.
                ui.add_enabled(false, egui::Button::new("Ready"))
                    .on_disabled_hover_text(wait.to_string());
                ui.colored_label(
                    crate::ui::palette::CAUTION,
                    omdurman_rules::press::capitalize(&wait.to_string()),
                );
            } else if !state.0.setup_target_met(player) {
                let reason = "Deploy your forces before confirming ready.";
                ui.add_enabled(false, egui::Button::new("Ready"))
                    .on_disabled_hover_text(reason);
                ui.colored_label(crate::ui::palette::CAUTION, reason);
            } else {
                let commanded = peers.any_commands();
                let i_am_ready = local_setup_ready.0;
                let others_ready = peers.faction_others_ready(player);
                let teammates = peers.faction_size(player).saturating_sub(1);
                if commanded && !i_am_ready {
                    if ui.button("Ready").clicked() {
                        local_setup_ready.0 = true;
                        pending
                            .outgoing_broadcast
                            .push(omdurman_net::NetMsg::Ephemeral(
                                omdurman_net::Ephemeral::SetupReady(true),
                            ));
                        if peers.faction_others_ready(player) {
                            pending.submit_game(omdurman_net::GameEvent::Effect(
                                omdurman_rules::effects::GameEffect::ConfirmSetupReady { player },
                            ));
                        }
                    }
                    if teammates > 0 {
                        ui.colored_label(
                            crate::ui::palette::TEXT_MUTED,
                            format!("Waiting for {teammates} other commander(s) of your side."),
                        );
                    }
                } else if commanded && !others_ready {
                    ui.colored_label(
                        crate::ui::palette::TEXT_MUTED,
                        format!(
                            "Your command is ready -- waiting for {teammates} other \
                             commander(s) of your side."
                        ),
                    );
                } else if ui.button("Ready").clicked() {
                    pending.submit_game(omdurman_net::GameEvent::Effect(
                        omdurman_rules::effects::GameEffect::ConfirmSetupReady { player },
                    ));
                }
            }
        }
        // Spectators (a seat table exists, none of it ours) watch only.
        None if peers.is_spectator() => {
            ui.colored_label(
                crate::ui::palette::TEXT_MUTED,
                "Spectating -- the seated commanders deploy.",
            );
        }
        // Unbound session (single seat, no faction binding): one button starts
        // the battle for both sides once deployment is complete.
        // Unbound session (one seat drives both sides): the set-up is still
        // sequential (§9.111/§9.211/§9.321) -- finish the first side, confirm
        // it, then deploy the second; confirming the second begins the battle.
        None => {
            let first = state.0.first_to_set_up();
            let side = if state.0.setup_ready(first) {
                first.opponent()
            } else {
                first
            };
            let label = crate::ui::faction_name(side);
            let button = if side == first {
                format!("{label} deployment done")
            } else {
                "Begin battle".to_string()
            };
            match state.0.can_confirm_setup_ready(side) {
                Ok(()) => {
                    if ui.button(button).clicked() {
                        pending.submit_game(omdurman_net::GameEvent::Effect(
                            omdurman_rules::effects::GameEffect::ConfirmSetupReady { player: side },
                        ));
                    }
                }
                Err(reason) => {
                    let reason = format!("{label}: {reason}");
                    ui.add_enabled(false, egui::Button::new(button))
                        .on_disabled_hover_text(&reason);
                    ui.colored_label(crate::ui::palette::CAUTION, &reason);
                }
            }
        }
    }
}

/// The turn's field telegram, as a centered modal over a dimmed board: it
/// arrives when a game turn completes (flavour text from the model, or the
/// turn's own events), and play waits until the player dismisses it (the
/// host's AI holds its moves, `TelegramLog::awaiting_ack`) -- a
/// click anywhere, the Continue button, or Enter / Space (the button takes
/// keyboard focus, which also holds back the game hotkeys) or Esc.
///
/// Only the latest completed turn's telegram is presented; an older backlog
/// (a history replay after joining or relaunching) is acknowledged silently,
/// and at game over the newspaper takes its place.
pub(crate) fn telegram_overlay(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    telegram_log: Option<ResMut<crate::telegram::TelegramLog>>,
    keepsakes: Option<(
        ResMut<crate::keepsakes::Keepsakes>,
        Res<crate::game_record::GameRecorder>,
    )>,
) {
    let (Some(state), Some(mut log)) = (game_state, telegram_log) else {
        return;
    };
    let latest = state.0.turn_summaries.last().map(|s| s.turn.value());
    while log.acknowledged < log.entries.len()
        && (state.0.game_over || Some(log.entries[log.acknowledged].0) != latest)
    {
        log.acknowledged += 1;
    }
    let Some((turn, text)) = log.entries.get(log.acknowledged).cloned() else {
        return;
    };
    // A keepsake of the game played: each turn's telegram, once.
    if let Some((mut keepsakes, recorder)) = keepsakes {
        keepsakes.request(&recorder, &format!("telegram-turn-{turn:02}"));
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let mut dismiss = false;
    // A true modal: the backdrop dims and swallows the *whole* screen (rails
    // and top bar included -- an `Area` is clipped to the space beside the
    // panels), on the topmost order, and only Continue / Enter / Esc close
    // it: a stray click must not dismiss a turn's report unread.
    let screen = ctx.viewport_rect();
    egui::Area::new(egui::Id::new("telegram_backdrop"))
        .order(egui::Order::Tooltip)
        .fixed_pos(screen.min)
        .constrain_to(screen)
        .show(ctx, |ui| {
            ui.set_clip_rect(screen);
            ui.allocate_rect(screen, egui::Sense::click_and_drag());
            ui.painter()
                .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(140));
        });
    let card = egui::Area::new(egui::Id::new("telegram_overlay"))
        .order(egui::Order::Tooltip)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            crate::ui::frames::modal().show(ui, |ui| {
                ui.set_max_width(460.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(
                            omdurman_rules::press::telegram::filed(&state.0, turn).title(),
                        )
                        .size(18.0)
                        .strong()
                        .color(crate::ui::palette::BRASS),
                    );
                    let (date, time) = omdurman_rules::press::turn_date(state.0.scenario, turn);
                    ui.label(
                        egui::RichText::new(format!("{date}, {time} \u{00b7} end of turn {turn}"))
                            .size(11.0)
                            .color(crate::ui::palette::TEXT_DIM),
                    );
                    ui.add_space(10.0);
                    // Keyed as a telegram of the day; records filed
                    // before the style read the same.
                    ui.label(
                        egui::RichText::new(omdurman_rules::press::telegram::telegraphese(&text))
                            .monospace()
                            .size(14.0)
                            .color(crate::ui::palette::TEXT),
                    );
                    ui.add_space(14.0);
                    let button = ui.button("Continue  (Enter)");
                    button.request_focus();
                    if button.clicked() {
                        dismiss = true;
                    }
                });
            });
        });
    // Above the backdrop, which shares its order.
    ctx.move_to_top(card.response.layer_id);
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        dismiss = true;
    }
    if dismiss {
        log.acknowledged += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{end_phase_effect, end_phase_hotkey, end_phase_preview, ends_phase};
    use bevy_egui::egui;
    use omdurman_net::GameEvent;
    use omdurman_rules::Phase;
    use omdurman_rules::effects::{GameEffect, GameState, apply_effect};
    use omdurman_types::Scenario;

    /// A new `StartGame` drops the last game's teammate Ready flags,
    /// dispatches and Log lines; a re-install of the same game keeps them.
    #[test]
    fn a_new_game_resets_the_per_game_ui_and_the_same_game_keeps_it() {
        use bevy::prelude::*;
        use omdurman_net::{GameRecord, InitialGameState, RecordedEvent};

        let record = |uid: u64| GameRecord {
            initial_state: InitialGameState { seed: 1 },
            events: vec![RecordedEvent {
                utc: chrono::DateTime::UNIX_EPOCH,
                sender_idx: Some(0),
                seq: 0,
                uid: Some(uid),
                payload: GameEvent::StartGame {
                    seats: Vec::new(),
                    scenario: Scenario::Campaign,
                    optional_rules: Vec::new(),
                },
            }],
        };
        let mut app = App::new();
        app.init_resource::<crate::game_record::GameRecorder>()
            .init_resource::<crate::dispatch::Dispatches>()
            .init_resource::<crate::combat_card::CombatCardQueue>()
            .add_systems(Update, super::reset_per_game_ui_on_new_game);
        let leftovers = |app: &mut App| {
            let flag = app
                .world_mut()
                .spawn(crate::peers::SetupReadyFlag(true))
                .id();
            app.world_mut()
                .resource_mut::<crate::dispatch::Dispatches>()
                .push("Old", "game");
            flag
        };
        let kept = |app: &mut App, flag| {
            let dispatches = app.world().resource::<crate::dispatch::Dispatches>();
            let any = !dispatches.slips.is_empty() || !dispatches.log.is_empty();
            let flagged = app
                .world()
                .get::<crate::peers::SetupReadyFlag>(flag)
                .is_some();
            assert_eq!(any, flagged, "slips, log and flag go together");
            any
        };

        app.world_mut()
            .resource_mut::<crate::game_record::GameRecorder>()
            .record = Some(record(1));
        app.update();
        let flag = leftovers(&mut app);
        // The same game installed again (a reconnect): nothing is reset.
        app.world_mut()
            .resource_mut::<crate::game_record::GameRecorder>()
            .record = Some(record(1));
        app.update();
        assert!(kept(&mut app, flag));
        // A new game: everything from the last one goes.
        app.world_mut()
            .resource_mut::<crate::game_record::GameRecorder>()
            .record = Some(record(2));
        app.update();
        assert!(!kept(&mut app, flag));
    }

    #[test]
    fn end_phase_refusal_carries_the_engine_reason() {
        // Setup cannot end before both sides deploy (§9.2/§9.3).
        let gs = GameState::new(Scenario::Campaign);
        let mut probe = gs.clone();
        let expected = apply_effect(&mut probe, &GameEffect::AdvancePhase)
            .expect_err("an empty deployment cannot advance");
        let got = end_phase_preview(&gs, &[]).expect_err("preview must refuse too");
        assert_eq!(got.to_string(), expected.to_string());
    }

    /// §6.82/§7.6: ending the phase asks first when it would let an open
    /// advance lapse -- not when the §6.42 bridge keeps it open.
    #[test]
    fn an_open_advance_is_one_ending_the_phase_would_let_lapse() {
        use omdurman_rules::{UnitId, UnitPlacement, unit_profiles::profile_for_unit};
        use omdurman_types::{HexCoord, Player};
        let mut gs = GameState::new(Scenario::Campaign);
        gs.active_player = Player::AngloEgyptian;
        let battalion = UnitId::BritishArmy_0_0;
        gs.units.push(UnitPlacement {
            id: battalion,
            position: HexCoord::new(0, 0),
            profile: profile_for_unit(battalion).unwrap(),
            state: Default::default(),
        });
        let vacated = HexCoord::new(1, 0);
        gs.vacated_by_combat.insert(vacated, vec![battalion]);
        gs.phase = Phase::Melee;
        assert_eq!(super::advances_lost_by_ending(&gs, &[]), vec![vacated]);
        // Direct fire to the Maxim subphase: the window stays open.
        gs.phase = Phase::OffensiveFire(omdurman_rules::FireSubPhase::DirectFire);
        assert!(super::advances_lost_by_ending(&gs, &[]).is_empty());
    }

    #[test]
    fn end_phase_preview_names_the_next_phase() {
        let mut gs = GameState::new(Scenario::Campaign);
        gs.phase = Phase::OffensiveFire(omdurman_rules::FireSubPhase::DirectFire);
        let mut next = gs.clone();
        apply_effect(&mut next, &GameEffect::AdvancePhase).expect("fire phase may end");
        let label = end_phase_preview(&gs, &[]).expect("fire phase may end");
        let expected = crate::ui_phase_state::UiPhaseState::derive(&next).phase_label();
        assert!(label.starts_with(expected), "{label} vs {expected}");
        // The live state is untouched by the dry run.
        assert!(matches!(gs.phase, Phase::OffensiveFire(_)));
    }

    /// End Phase is stamped with the phase it ends: the same request twice
    /// (a double click, a teammate within one round trip) ends one phase,
    /// never the next one too (§4).
    #[test]
    fn a_repeated_end_phase_request_ends_one_phase() {
        let mut gs = GameState::new(Scenario::Campaign);
        gs.phase = Phase::Movement;
        let request = end_phase_effect(&gs);
        assert!(ends_phase(&GameEvent::Effect(request.clone())));
        apply_effect(&mut gs, &request).unwrap();
        let after_one = gs.phase;
        assert!(apply_effect(&mut gs, &request).is_err());
        assert_eq!(gs.phase, after_one, "the next phase is not skipped");
    }

    /// Only a fresh press of `E` ends the phase; the key repeat of a held
    /// key does not.
    #[test]
    fn a_held_e_ends_the_phase_once() {
        let key = |repeat| egui::Event::Key {
            key: egui::Key::E,
            physical_key: None,
            pressed: true,
            repeat,
            modifiers: egui::Modifiers::NONE,
        };
        let none = egui::Modifiers::NONE;
        assert!(end_phase_hotkey(none, &[key(false)]));
        assert!(!end_phase_hotkey(none, &[key(true)]));
        assert!(!end_phase_hotkey(none, &[]));
        assert!(!end_phase_hotkey(egui::Modifiers::CTRL, &[key(false)]));
    }

    #[test]
    fn end_phase_preview_projects_unconfirmed_submissions() {
        // An AdvancePhase already in flight is applied first: the preview
        // shows the phase after *that* one.
        let mut gs = GameState::new(Scenario::Campaign);
        gs.phase = Phase::OffensiveFire(omdurman_rules::FireSubPhase::DirectFire);
        let in_flight = [GameEvent::Effect(GameEffect::AdvancePhase)];
        let mut once = gs.clone();
        apply_effect(&mut once, &GameEffect::AdvancePhase).unwrap();
        let direct = end_phase_preview(&once, &[]);
        let projected = end_phase_preview(&gs, &in_flight);
        assert_eq!(
            direct.map_err(|e| e.to_string()),
            projected.map_err(|e| e.to_string())
        );
    }
}

#[cfg(test)]
mod setup_heading_tests {
    use super::setup_heading;
    use omdurman_types::{Player, Scenario};

    #[test]
    fn a_side_with_nothing_to_set_up_is_told_so() {
        let gs = omdurman_rules::effects::GameState::new(Scenario::Campaign);
        assert!(setup_heading(&gs, Some(Player::AngloEgyptian)).starts_with("Nothing to set up"));
        assert!(setup_heading(&gs, Some(Player::Dervish)).contains("place your forces"));
    }
}
