//! The left-rail control sections: game controls, victory scoreboard,
//! setup controls, and the game log.
use super::*;

/// The rail's per-game mutable bits beside the outbound queue: the staged
/// fire allocations (End Phase discards them) and the victory modal's
/// dismissed flag (the rail reopens it). Bundled to keep
/// [`game_control_section`] under clippy's argument limit.
pub(crate) struct GameControlExtras<'a> {
    pub allocation: Option<&'a mut crate::fire_allocation::FireAllocationState>,
    pub victory: Option<&'a mut VictoryModalState>,
}

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

pub(crate) fn game_control_section(
    ui: &mut egui::Ui,
    state: &crate::GameStateResource,
    peers: &Peers,
    pending: Option<&mut crate::PendingEdits>,
    local_setup_ready: Option<&mut crate::peers::LocalSetupReady>,
    extras: GameControlExtras<'_>,
) {
    let turn = state.0.current_turn.value();
    let Some(pending) = pending else {
        return;
    };

    let day_night_str = match state.0.day_night {
        omdurman_types::DayNight::Day => "Day",
        omdurman_types::DayNight::Night => "Night",
    };

    // The player who may act *now*: the turn owner, except during Defensive
    // Fire where control passes to the non-moving side (§6.4/§6.7).
    let acting = state.0.phase_player();
    let acting_str = crate::ui::faction_abbrev(acting);
    let my_turn = peers.may_act(acting);
    let in_setup = matches!(state.0.phase, omdurman_rules::Phase::Setup);

    // At game over the engine's phase is whatever it rolled on to when it
    // found no next turn -- not a phase anyone is in.
    let phase_name = if state.0.game_over {
        "(final)"
    } else {
        state.0.phase.top_level_name()
    };
    ui.colored_label(
        crate::ui::palette::HEADING,
        format!("Turn {turn}  {phase_name}  {day_night_str}"),
    );

    // Turn indicator -- only meaningful once play has begun. Setup is *not* a
    // turn: deployment is sequential (§9.111/§9.211/§9.321), and the
    // deployment status below tells each player whether to deploy or wait, so
    // the "your turn / waiting on" indicator is suppressed during Setup.
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
        if let Some(victory) = extras.victory
            && victory.dismissed
            && ui.button("Show result").clicked()
        {
            victory.dismissed = false;
        }
    } else if !in_setup {
        if my_turn {
            ui.colored_label(
                crate::ui::palette::GOLD,
                format!("\u{25b6} Your turn ({acting_str})"),
            );
        } else {
            ui.colored_label(
                crate::ui::palette::TEXT_DIM,
                format!("Waiting on {acting_str}"),
            );
        }
    }

    ui.add_space(4.0);

    // -- Scoreboard / victory progress --
    // FoK uses a different victory scheme (§9.35: GORDON's fate + Dervish
    // losses) that the §9.14 VP ladder does not model -- the generic scoreboard
    // would read "No scoring yet." for the whole game. Replace it with the
    // FoK-specific panel during a Fall-of-Khartoum session.
    if !in_setup {
        match state.0.scenario {
            omdurman_types::Scenario::FallOfKhartoum => {
                crate::fok_panel::fok_status_section(ui, state)
            }
            omdurman_types::Scenario::Historical => historical_scoreboard(ui, state),
            omdurman_types::Scenario::Campaign => victory_point_scoreboard(ui, state),
        }
    }

    // -- Night-effects reminder (§8) --
    if !in_setup && state.0.day_night == omdurman_types::DayNight::Night {
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
        ui.add_space(4.0);
    }

    if in_setup {
        // Per-member setup readiness needs the local flag; a None resource
        // just falls back to "ready" semantics for unbound sessions.
        if let Some(local_setup_ready) = local_setup_ready {
            setup_control_section(ui, state, peers, pending, local_setup_ready);
        }
    } else if my_turn && !game_over {
        // Each player ends their *own* turn: the End Phase button is shown only
        // to whoever controls the active faction.
        end_phase_button(ui, state, pending, extras.allocation);
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
) {
    let unconfirmed: Vec<omdurman_net::GameEvent> =
        pending.unconfirmed.iter().map(|(_, e)| e.clone()).collect();
    let current = crate::ui_phase_state::UiPhaseState::derive(&state.0).phase_label();
    let next = match end_phase_preview(&state.0, &unconfirmed) {
        Ok(next) => next,
        Err(reason) => {
            let reason = reason.to_string();
            ui.add_enabled(false, egui::Button::new("End Phase"))
                .on_disabled_hover_text(&reason);
            ui.colored_label(crate::ui::palette::CAUTION, &reason);
            return;
        }
    };
    let staged = allocation
        .as_ref()
        .filter(|a| !a.committed)
        .map_or(0, |a| a.attacks.len());
    // The armed (awaiting-confirm) state is keyed on the phase, so it never
    // survives into the next phase.
    let armed_id = egui::Id::new("end_phase_discard_armed");
    let phase_key = format!(
        "{}:{:?}:{:?}",
        state.0.current_turn.value(),
        state.0.active_player,
        state.0.phase
    );
    let armed = staged > 0
        && ui.data(|d| d.get_temp::<String>(armed_id)).as_deref() == Some(phase_key.as_str());
    let hover = format!("End {current}; next: {next}");
    if armed {
        let s = if staged == 1 { "" } else { "s" };
        let confirm = ui
            .add(
                egui::Button::new(format!("Discard {staged} attack{s} & end phase?"))
                    .fill(crate::ui::palette::BTN_DANGER),
            )
            .on_hover_text("The staged fire allocations have not been resolved.");
        let keep = ui.button("Keep them");
        if confirm.clicked() {
            ui.data_mut(|d| d.remove::<String>(armed_id));
            if let Some(allocation) = allocation {
                allocation.attacks.clear();
                allocation.panel_open = false;
            }
            crate::ui_trace::button("End Phase (discard allocations)");
            pending.submit_game(omdurman_net::GameEvent::Effect(
                omdurman_rules::effects::GameEffect::AdvancePhase,
            ));
        } else if keep.clicked() {
            ui.data_mut(|d| d.remove::<String>(armed_id));
        }
        return;
    }
    // `E` ends the phase like a click (it arms the same discard confirm
    // when attacks are staged); never while a text field has the keyboard.
    let hotkey = !ui.ctx().egui_wants_keyboard_input()
        && ui.input(|i| i.modifiers.is_none() && i.key_pressed(egui::Key::E));
    if ui
        .button(format!("End phase \u{2192} {next}  (E)"))
        .on_hover_text(hover)
        .clicked()
        || hotkey
    {
        if staged > 0 {
            ui.data_mut(|d| d.insert_temp(armed_id, phase_key));
        } else {
            crate::ui_trace::button("End Phase");
            pending.submit_game(omdurman_net::GameEvent::Effect(
                omdurman_rules::effects::GameEffect::AdvancePhase,
            ));
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
    ui.label(
        egui::RichText::new("Score")
            .strong()
            .color(crate::ui::palette::HEADING),
    );
    for (who, level) in [(Player::AngloEgyptian, ae), (Player::Dervish, d)] {
        let killed = ledger.units_eliminated_by(who);
        let next = level
            .next_threshold(who)
            .map(|n| format!(" (next at {n})"))
            .unwrap_or_default();
        ui.colored_label(
            crate::ui::faction_color(who),
            format!("{who}: {killed} eliminated, {level:?}{next}"),
        );
    }
    let (text, color) = match Level::net(ae, d) {
        (None, _) => ("Net: Draw".to_string(), crate::ui::palette::TEXT_MUTED),
        (Some(p), level) => (format!("Net: {p} {level:?}"), crate::ui::faction_color(p)),
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
    use omdurman_rules::VpSource;
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
    let net_color = if net > 0 {
        crate::ui::palette::GOOD
    } else if net < 0 {
        crate::ui::palette::BAD
    } else {
        crate::ui::palette::TEXT_MUTED
    };
    ui.label(
        egui::RichText::new("Score")
            .strong()
            .color(crate::ui::palette::HEADING),
    );
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("A-E: {ae_vp}")).color(crate::ui::palette::AE));
        ui.label(
            egui::RichText::new(format!("Dervish: {dv_vp}")).color(crate::ui::palette::DERVISH),
        );
    });
    ui.colored_label(net_color, format!("Net: {net:+}"));

    // VP breakdown by source category (§9.14). Collapsible to keep the
    // sidebar compact; defaults to collapsed.
    ui.collapsing("Breakdown", |ui| {
        ui.style_mut().override_font_id = Some(egui::FontId::proportional(11.0));
        let tally = |src: VpSource| -> i32 {
            state
                .0
                .victory
                .events
                .iter()
                .filter(|e| e.source == src)
                .map(|e| e.source.points().value())
                .sum()
        };
        let ae_sources = [
            VpSource::MahdisTombTaken,
            VpSource::IsaZachneihEliminated,
            VpSource::KhalifaEliminated,
            VpSource::DervishUnitEliminated,
        ];
        let dv_sources = [
            VpSource::MahdisTombHeld,
            VpSource::BritishLeaderEliminated,
            VpSource::BritishGunboatSunk,
            VpSource::FriendliesEastBankEliminated,
            VpSource::FriendliesWestBankEliminated,
            VpSource::AngloEgyptianLandUnitEliminated,
        ];
        let has_ae = ae_sources.iter().any(|s| tally(*s) > 0);
        let has_dv = dv_sources.iter().any(|s| tally(*s) > 0);
        if has_ae {
            ui.colored_label(crate::ui::palette::AE, "Anglo-Egyptian:");
            for src in &ae_sources {
                let pts = tally(*src);
                if pts > 0 {
                    ui.label(format!("  {src}: {pts}"));
                }
            }
        }
        if has_dv {
            ui.colored_label(crate::ui::palette::DERVISH, "Dervish:");
            for src in &dv_sources {
                let pts = tally(*src);
                if pts > 0 {
                    ui.label(format!("  {src}: {pts}"));
                }
            }
        }
        if !has_ae && !has_dv {
            ui.colored_label(crate::ui::palette::TEXT_DIM, "No scoring yet.");
        }

        // Last 5 VP events (most recent last).
        let recent: Vec<&omdurman_rules::VpEvent> =
            state.0.victory.events.iter().rev().take(5).collect();
        if !recent.is_empty() {
            ui.add_space(2.0);
            ui.colored_label(crate::ui::palette::TEXT_MUTED, "Recent:");
            for ev in recent.iter().rev() {
                let who = ev.source.who_scores();
                let pts = ev.source.points().value();
                let color = crate::ui::faction_color(who);
                ui.colored_label(
                    color,
                    format!("  T{}: {} (+{pts})", ev.turn.value(), ev.source),
                );
            }
        }
    });

    ui.add_space(4.0);
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

    ui.label(
        egui::RichText::new("Deployment -- place your forces, then Ready.")
            .size(12.0)
            .color(crate::ui::palette::TEXT),
    );

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
        let count = match state.0.setup_target(player) {
            Some(target) => format!("{deployed}/{target}"),
            None => format!("{deployed}"),
        };
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
                ui.colored_label(crate::ui::palette::CAUTION, capitalize(&wait.to_string()));
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
/// turn's own events), and play waits until the player dismisses it -- a
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
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let mut dismiss = false;
    // Backdrop: dims the board and swallows its clicks (egui owns the
    // pointer everywhere while it is up), dismissing on click.
    let screen = ctx.viewport_rect();
    egui::Area::new(egui::Id::new("telegram_backdrop"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let response = ui.allocate_rect(screen, egui::Sense::click());
            ui.painter()
                .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(110));
            if response.clicked() {
                dismiss = true;
            }
        });
    crate::ui::anchored_card(
        ctx,
        egui::Id::new("telegram_overlay"),
        egui::Align2::CENTER_CENTER,
        egui::Vec2::ZERO,
        crate::ui::frames::modal(),
        |ui| {
            ui.set_max_width(460.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("FIELD TELEGRAM")
                        .size(18.0)
                        .strong()
                        .color(crate::ui::palette::BRASS),
                );
                ui.label(
                    egui::RichText::new(format!("End of turn {turn}"))
                        .size(11.0)
                        .color(crate::ui::palette::TEXT_DIM),
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(text.trim())
                        .monospace()
                        .size(13.0)
                        .color(crate::ui::palette::TEXT),
                );
                ui.add_space(14.0);
                let button = ui.button("Continue");
                button.request_focus();
                if button.clicked() {
                    dismiss = true;
                }
            });
        },
    );
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        dismiss = true;
    }
    if dismiss {
        log.acknowledged += 1;
    }
}

/// `text` with its first letter upper-cased (engine reasons are phrased as
/// clauses).
fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::end_phase_preview;
    use omdurman_net::GameEvent;
    use omdurman_rules::Phase;
    use omdurman_rules::effects::{GameEffect, GameState, apply_effect};
    use omdurman_types::Scenario;

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
