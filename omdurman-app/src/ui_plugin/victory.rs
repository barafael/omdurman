//! The end-of-game victory modal.
use super::*;

/// Whether the player closed the victory modal. Reset whenever the shown
/// state is not over (a new game, or scrubbing the timeline back), so the
/// next game over opens it again. Reopened from the rail's "Show result"
/// button or the floating chip drawn while it is closed.
#[derive(Resource, Default)]
pub(crate) struct VictoryModalState {
    pub dismissed: bool,
}

/// What a victory-modal button asked for.
enum VictoryAction {
    Close,
    Review,
    Leave,
}

/// Navigation targets the victory modal's buttons drive. Bundled to keep
/// [`victory_modal`] under clippy's argument limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct VictoryNav<'w> {
    app_state: Res<'w, State<AppState>>,
    next_mode: ResMut<'w, NextState<crate::AppMode>>,
    next_app_state: ResMut<'w, NextState<AppState>>,
    timeline: ResMut<'w, crate::timeline::SpectatorTimeline>,
    recorder: Res<'w, crate::game_record::GameRecorder>,
}

pub(crate) fn victory_modal(
    mut contexts: EguiContexts,
    state: Res<crate::GameStateResource>,
    report: Option<Res<crate::newspaper::NewspaperReport>>,
    mut telegrams: Option<ResMut<crate::telegram::TelegramLog>>,
    mut keepsakes: Option<ResMut<crate::keepsakes::Keepsakes>>,
    mut modal: ResMut<VictoryModalState>,
    mut nav: VictoryNav,
) {
    if !state.0.game_over {
        modal.dismissed = false;
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let spectating = *nav.app_state.get() == AppState::Spectating;

    if modal.dismissed {
        // Closed: a small chip to bring the result back (the rail has the
        // same button while in a live game).
        crate::ui::anchored_card(
            ctx,
            egui::Id::new("victory_reopen"),
            egui::Align2::CENTER_BOTTOM,
            egui::vec2(0.0, -16.0),
            crate::ui::frames::chip(),
            |ui| {
                if ui.button("Game over \u{2014} show result").clicked() {
                    modal.dismissed = false;
                }
            },
        );
        return;
    }

    let mut action: Option<VictoryAction> = None;
    let page = report.as_ref().and_then(|r| r.page.clone());
    // A keepsake of the game played (live play only, not a review).
    if page.is_some()
        && !spectating
        && let Some(keepsakes) = keepsakes.as_mut()
    {
        keepsakes.request(&nav.recorder, "gazette");
    }
    let has_record = nav
        .recorder
        .record
        .as_ref()
        .is_some_and(|r| !r.events.is_empty());
    // The Gazette replaces the last telegram: mark them all read.
    if let Some(log) = telegrams.as_mut() {
        let filed = log.entries.len();
        if log.acknowledged < filed {
            log.acknowledged = filed;
        }
    }

    // A dimmed desk behind the paper.
    let screen = ctx.viewport_rect();
    egui::Area::new(egui::Id::new("gazette_backdrop"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            ui.set_clip_rect(screen);
            ui.allocate_rect(screen, egui::Sense::click_and_drag());
            ui.painter()
                .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(170));
        });
    let card = egui::Area::new(egui::Id::new("victory_modal"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            // Room to the screen bottom: the front page scrolls in it (see `with_room`).
            crate::ui::with_room(ui, |ui| {
                let width = (screen.width() - 80.0).clamp(600.0, 1180.0);
                let height = screen.height() - 70.0;
                ui.set_max_width(width);
                match &page {
                    Some(page) => {
                        egui::ScrollArea::vertical()
                            .id_salt("gazette_scroll")
                            .max_height(height - 50.0)
                            .show(ui, |ui| super::gazette::draw_front_page(ui, page, width));
                    }
                    None => {
                        crate::ui::frames::modal().show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("GAME OVER")
                                    .size(28.0)
                                    .strong()
                                    .color(crate::ui::palette::TITLE),
                            );
                        });
                    }
                }
                ui.add_space(8.0);
                // The arithmetic behind the result (§9.14, §9.24, §9.35).
                crate::ui::frames::modal().show(ui, |ui| {
                    ui.set_max_width(width);
                    ui.collapsing("How the result was reached", |ui| {
                        for line in result_arithmetic(&state.0) {
                            crate::rulebook::refs_label(ui, &line, crate::ui::palette::INK, 13.0);
                        }
                    });
                });
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            action = Some(VictoryAction::Close);
                        }
                        if !spectating
                            && ui
                                .add_enabled(has_record, egui::Button::new("Review timeline"))
                                .on_hover_text("Scrub back through the recorded game")
                                .clicked()
                        {
                            action = Some(VictoryAction::Review);
                        }
                        let leave = if spectating {
                            "Back to lobby"
                        } else {
                            "Main menu"
                        };
                        if ui.button(leave).clicked() {
                            action = Some(VictoryAction::Leave);
                        }
                    });
                });
            });
        });
    ctx.move_to_top(card.response.layer_id);

    match action {
        Some(VictoryAction::Close) => modal.dismissed = true,
        Some(VictoryAction::Review) => {
            // Same entry as the lobby's "Review current game".
            if let Some(record) = nav.recorder.record.as_ref() {
                nav.timeline
                    .open(record.clone(), "current game".to_string());
                nav.next_app_state.set(AppState::Spectating);
            }
        }
        Some(VictoryAction::Leave) => {
            if spectating {
                // Same exit as the toolbar's spectator "Back to lobby".
                nav.timeline.record = None;
                nav.next_mode.set(crate::AppMode::Lobby);
                nav.next_app_state.set(AppState::Lobby);
            } else {
                nav.next_mode.set(crate::AppMode::Menu);
            }
        }
        None => {}
    }
}

/// The victory-point sources each side scores from (§9.14), in the order
/// the rulebook lists them.
pub(crate) const VP_SOURCES: [(omdurman_types::Player, &[omdurman_rules::VpSource]); 2] = {
    use omdurman_rules::VpSource as S;
    [
        (
            omdurman_types::Player::AngloEgyptian,
            &[
                S::MahdisTombTaken,
                S::IsaZachneihEliminated,
                S::KhalifaEliminated,
                S::DervishUnitEliminated,
            ],
        ),
        (
            omdurman_types::Player::Dervish,
            &[
                S::MahdisTombHeld,
                S::BritishLeaderEliminated,
                S::BritishGunboatSunk,
                S::FriendliesEastBankEliminated,
                S::FriendliesWestBankEliminated,
                S::AngloEgyptianLandUnitEliminated,
            ],
        ),
    ]
};

/// How often `source` has scored in `gs`.
pub(crate) fn vp_count(
    gs: &omdurman_rules::effects::GameState,
    source: omdurman_rules::VpSource,
) -> usize {
    gs.victory
        .events
        .iter()
        .filter(|e| e.source == source)
        .count()
}

fn campaign_level_name(level: omdurman_rules::CampaignVictoryLevel) -> String {
    use omdurman_rules::CampaignVictoryLevel as L;
    match level {
        L::Draw => "a draw".to_string(),
        L::Marginal(p) => format!("{p} Marginal Victory"),
        L::Tactical(p) => format!("{p} Tactical Victory"),
        L::Decisive(p) => format!("{p} Decisive Victory"),
    }
}

/// How the result was reached, line by line (§9.14, §9.24, §9.35): the
/// counts and points, the tables they were read on, and any rule that
/// decided the game outright.
pub(crate) fn result_arithmetic(gs: &omdurman_rules::effects::GameState) -> Vec<String> {
    use omdurman_types::{Player, Scenario};
    let mut lines = Vec::new();
    match gs.scenario {
        Scenario::Campaign => {
            if let Some(winner) = omdurman_rules::effects::campaign_decisive_by(gs) {
                lines.push(match winner {
                    Player::AngloEgyptian => {
                        "Every Dervish unit eliminated: an Anglo-Egyptian Decisive Victory, \
                         whatever the points (§9.14)."
                            .to_string()
                    }
                    Player::Dervish => {
                        "Every Anglo-Egyptian unit on the west bank eliminated: a Dervish \
                         Decisive Victory, whatever the points (§9.14)."
                            .to_string()
                    }
                });
            }
            for (player, sources) in VP_SOURCES {
                let total = gs.victory.total_for(player).value();
                lines.push(format!("{player} victory points: {total} (§9.14)"));
                for &source in sources {
                    let n = vp_count(gs, source);
                    let pts = source.points().value();
                    lines.push(format!(
                        "    {source}: {n} \u{d7} {pts} = {}",
                        n as i32 * pts
                    ));
                }
            }
            let ae = gs.victory.total_for(Player::AngloEgyptian).value();
            let d = gs.victory.total_for(Player::Dervish).value();
            let level = omdurman_rules::CampaignVictoryLevel::from_superiority(
                omdurman_rules::VictoryPoints::new(ae - d),
            );
            lines.push(format!(
                "Superiority {ae} \u{2212} {d} = {}: {} (§9.14: the Anglo-Egyptians need \
                 15/30/50 for marginal/tactical/decisive, the Dervish 10/20/30).",
                ae - d,
                campaign_level_name(level)
            ));
        }
        Scenario::Historical => {
            let (ae, d) = gs.victory.historical_levels();
            for (player, level, steps) in [
                (Player::AngloEgyptian, ae, "30/45/60/100"),
                (Player::Dervish, d, "5/10/15/30"),
            ] {
                lines.push(format!(
                    "{player}: {} {} units eliminated \u{2014} {} ({}) (§9.24: {steps} for \
                     marginal/tactical/strategic/decisive).",
                    gs.victory.units_eliminated_by(player),
                    player.opponent(),
                    level.name(),
                    level as i16,
                ));
            }
            let (winner, net) = omdurman_rules::HistoricalVictoryLevel::net(ae, d);
            let higher = (ae as i16).max(d as i16);
            let lower = (ae as i16).min(d as i16);
            lines.push(match winner {
                Some(w) => format!(
                    "The lower level from the higher: {higher} \u{2212} {lower} \u{2192} {w} {} (§9.24).",
                    net.name()
                ),
                None => format!(
                    "The lower level from the higher: {higher} \u{2212} {lower} \u{2192} a draw (§9.24)."
                ),
            });
        }
        Scenario::FallOfKhartoum => {
            let died = gs.gordon_eliminated_turn.map(|t| t.value());
            let end_turn = gs.current_turn.value();
            let base = omdurman_rules::FoKVictoryLevel::base(died, end_turn);
            lines.push(match died {
                Some(t) => format!("GORDON fell on turn {t}: {base} (§9.35)."),
                None => format!("GORDON held out to the end of turn {end_turn}: {base} (§9.35)."),
            });
            let lost = gs.victory.units_eliminated_by(Player::AngloEgyptian);
            let penalty = omdurman_rules::FoKVictoryLevel::loss_penalty(lost);
            let dervish_base = (base as i16) < 0;
            lines.push(if !dervish_base {
                format!(
                    "Dervish losses {lost}: the loss penalty only takes from a Dervish \
                     result (§9.35)."
                )
            } else {
                let s = if penalty == 1 { "" } else { "s" };
                format!(
                    "Dervish losses {lost}: \u{2212}{penalty} level{s} (§9.35: 16/24/32 \
                     losses cost 1/2/3)."
                )
            });
            let level = omdurman_rules::FoKVictoryLevel::resolve(died, end_turn, lost);
            lines.push(format!("Result: {level}."));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use omdurman_rules::effects::GameState;
    use omdurman_types::Scenario;

    /// The arithmetic names every source, zero rows included, and the
    /// superiority the level was read from (§9.14).
    #[test]
    fn the_campaign_result_shows_its_arithmetic() {
        let mut gs = GameState::new(Scenario::Campaign);
        for _ in 0..3 {
            gs.victory.events.push(omdurman_rules::VpEvent {
                turn: omdurman_rules::GameTurnIndex::new(1),
                source: omdurman_rules::VpSource::AngloEgyptianLandUnitEliminated,
            });
        }
        let lines = super::result_arithmetic(&gs);
        assert!(
            lines.iter().any(|l| l.contains("Khalifa eliminated: 0")),
            "{lines:?}"
        );
        assert!(
            lines
                .iter()
                .any(|l| l.contains("Anglo-Egyptian unit eliminated: 3 \u{d7} 3 = 9")),
            "{lines:?}"
        );
        assert!(
            lines
                .iter()
                .any(|l| l.contains("Superiority 0 \u{2212} 9 = -9: a draw")),
            "{lines:?}"
        );
    }

    /// Fall of Khartoum: the base level from GORDON's fate, the loss
    /// penalty, the result (§9.35).
    #[test]
    fn the_fall_of_khartoum_result_shows_its_arithmetic() {
        let mut gs = GameState::new(Scenario::FallOfKhartoum);
        gs.gordon_eliminated_turn = Some(omdurman_rules::GameTurnIndex::new(5));
        let lines = super::result_arithmetic(&gs);
        assert!(lines[0].starts_with("GORDON fell on turn 5"), "{lines:?}");
        assert!(lines.last().unwrap().starts_with("Result:"), "{lines:?}");
    }
}
