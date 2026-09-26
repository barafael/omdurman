//! The left-rail control sections: game controls, victory scoreboard,
//! setup controls, and the game log.
use super::*;

pub(crate) fn game_control_section(
    ui: &mut egui::Ui,
    state: &crate::GameStateResource,
    peers: &Peers,
    pending: Option<&mut crate::PendingEdits>,
    local_setup_ready: Option<&mut crate::peers::LocalSetupReady>,
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
    let acting_str = match acting {
        omdurman_types::Player::AngloEgyptian => "A-E",
        omdurman_types::Player::Dervish => "Dervish",
    };
    let my_turn = peers.may_act(acting);
    let in_setup = matches!(state.0.phase, omdurman_rules::Phase::Setup);

    ui.colored_label(
        crate::ui::palette::HEADING,
        format!(
            "Turn {}  {}  {}",
            turn,
            state.0.phase.top_level_name(),
            day_night_str
        ),
    );

    // Turn indicator -- only meaningful once play has begun. Setup is *not* a
    // turn: both players deploy concurrently, so a "your turn / waiting on"
    // indicator would be misleading. It's suppressed during Setup, where the
    // deployment status below tells each player what to do instead.
    if !in_setup {
        if my_turn {
            ui.colored_label(
                crate::ui::palette::GOLD,
                format!("\u{25b6} Your turn ({acting_str})"),
            );
        } else {
            ui.colored_label(
                egui::Color32::from_gray(150),
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
        if crate::fok_panel::is_fok(state) {
            crate::fok_panel::fok_status_section(ui, state);
        } else {
            victory_point_scoreboard(ui, state);
        }
    }

    // -- Night-effects reminder (§8) --
    if !in_setup && state.0.day_night == omdurman_types::DayNight::Night {
        ui.label(
            egui::RichText::new("Night rules (§8)")
                .strong()
                .color(egui::Color32::from_rgb(160, 180, 220)),
        );
        ui.label(
            egui::RichText::new(
                "\u{2022} A-E movement halved\n\
                 \u{2022} Fire ranges halved (min 1)\n\
                 \u{2022} No howitzer fire",
            )
            .small()
            .color(egui::Color32::from_gray(180)),
        );
        ui.add_space(4.0);
    }

    if in_setup {
        // Per-member setup readiness needs the local flag; a None resource
        // just falls back to "ready" semantics for unbound sessions.
        if let Some(local_setup_ready) = local_setup_ready {
            setup_control_section(ui, state, peers, pending, local_setup_ready);
        }
    } else if my_turn && ui.button("End Phase").clicked() {
        // Each player ends their *own* turn: the End Phase button is shown only
        // to whoever controls the active faction.
        pending.submit_game(omdurman_net::GameEvent::Effect(
            omdurman_rules::effects::GameEffect::AdvancePhase,
        ));
    }
}

/// The Campaign/Historical §9.14 victory-point scoreboard. Extracted from
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
        egui::Color32::from_gray(170)
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
            VpSource::MahdisTomb,
            VpSource::IsaZachneihEliminated,
            VpSource::KhalifaEliminated,
            VpSource::DervishUnitEliminated,
        ];
        let dv_sources = [
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
            ui.colored_label(egui::Color32::from_gray(150), "No scoring yet.");
        }

        // Last 5 VP events (most recent last).
        let recent: Vec<&omdurman_rules::VpEvent> =
            state.0.victory.events.iter().rev().take(5).collect();
        if !recent.is_empty() {
            ui.add_space(2.0);
            ui.colored_label(egui::Color32::from_gray(170), "Recent:");
            for ev in recent.iter().rev() {
                let who = ev.source.who_scores();
                let pts = ev.source.points().value();
                let color = match who {
                    omdurman_types::Player::AngloEgyptian => crate::ui::palette::AE,
                    omdurman_types::Player::Dervish => crate::ui::palette::DERVISH,
                };
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
/// player's one-way "Ready" confirmation. Setup is concurrent -- both sides
/// deploy at once and each confirms independently; the engine auto-advances to
/// Movement once both are ready (§9.2/§9.3), so there's no explicit "advance"
/// click. An unbound session (no faction binding) keeps a single "Begin battle"
/// that drives the same `AdvancePhase` for both sides.
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
            .color(egui::Color32::from_gray(190)),
    );

    // The local member's §1.1 command scope, when one was assigned.
    if let Some(scope) = peers.local_scope() {
        ui.colored_label(
            egui::Color32::from_rgb(200, 200, 160),
            format!("Your command: {scope}"),
        );
    }

    // Per-faction deployed/target + ready status, for both sides.
    for (player, label) in [(Player::AngloEgyptian, "A-E"), (Player::Dervish, "Dervish")] {
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
            egui::Color32::from_gray(190)
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
            } else if !state.0.setup_target_met(player) {
                let reason = "Deploy your forces before confirming ready.";
                ui.add_enabled(false, egui::Button::new("Ready"))
                    .on_disabled_hover_text(reason);
                ui.colored_label(egui::Color32::from_rgb(220, 180, 90), reason);
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
                            egui::Color32::from_gray(170),
                            format!("Waiting for {teammates} other commander(s) of your side."),
                        );
                    }
                } else if commanded && !others_ready {
                    ui.colored_label(
                        egui::Color32::from_gray(170),
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
        // Unbound session (single seat, no faction binding): one button starts
        // the battle for both sides once deployment is complete.
        None => match state.0.setup_complete() {
            Ok(()) => {
                if ui.button("Begin battle").clicked() {
                    pending.submit_game(omdurman_net::GameEvent::Effect(
                        omdurman_rules::effects::GameEffect::AdvancePhase,
                    ));
                }
            }
            Err(reason) => {
                let reason = reason.to_string();
                ui.add_enabled(false, egui::Button::new("Begin battle"))
                    .on_disabled_hover_text(&reason);
                ui.colored_label(egui::Color32::from_rgb(220, 180, 90), &reason);
            }
        },
    }
}

/// Combat/event feed: the most recent military telegrams. The structured
/// `turn_events` / `observations` on `GameState` are surfaced elsewhere; this
/// panel now shows only the flavour telegrams.
// TODO(A-rules-4): render the event feed from `turn_events` + `observations`
// now that the human-readable `log` field has been removed.
pub(crate) fn game_log_panel(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    telegram_log: Option<Res<crate::telegram::TelegramLog>>,
    layout: Res<crate::ScreenLayout>,
) {
    let Some(_state) = game_state else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let has_telegrams = telegram_log.as_ref().is_some_and(|t| !t.entries.is_empty());
    if !has_telegrams {
        return;
    }
    crate::ui::anchored_card(
        ctx,
        egui::Id::new("game_log"),
        egui::Align2::LEFT_BOTTOM,
        // Clear of the left rail (see `ScreenLayout::left_inset`).
        egui::Vec2::new(layout.left_inset + 8.0, -8.0),
        egui::Frame::new()
            .fill(egui::Color32::from_black_alpha(180))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(8, 6)),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));
            ui.set_max_width(460.0);
            // Military telegrams — most recent two, newest first.
            if let Some(t) = telegram_log.as_ref()
                && !t.entries.is_empty()
            {
                for (turn, text) in t.entries.iter().rev().take(2) {
                    ui.colored_label(
                        egui::Color32::from_rgb(180, 210, 180),
                        format!("[Turn {}] {}", turn, text.lines().next().unwrap_or("")),
                    );
                }
            }
        },
    );
}
