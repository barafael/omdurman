//! Seat UI: the pause notice, the spectator's join panel, and the vote
//! popup. Everything here only reads the seat state and queues requests /
//! ballots on [`SeatClient`]; `seat_arbiter::seat_control` sends them.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::{PlayerKey, Seat, SeatHolder, SeatRequestKind};
use omdurman_types::{BrigadeId, BrigadeNationality, CommandScope, DervishTribe, Player};
use std::collections::BTreeSet;
use strum::IntoEnumIterator;

use crate::seat_arbiter::{RequestStatus, SeatClient};
use crate::seats::{self, SeatView, human_seats};

/// Top-center card under the top bar while the game is paused: names
/// every absent seat holder and counts down to their seat's abandonment.
/// Once a seat is abandoned, seated players may propose handing it to the AI.
pub(crate) fn pause_card_ui(
    mut contexts: EguiContexts,
    view: SeatView,
    peers: crate::peers::Peers,
    mut client: ResMut<SeatClient>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    if !view.presence.paused() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let seated = peers.local().is_some();
    let absent: Vec<(usize, String, String, Option<f64>)> = view
        .seats
        .0
        .iter()
        .enumerate()
        .filter_map(|(i, seat)| Some((i, seat.holder.human()?, seat)))
        .filter(|(_, key, _)| !view.presence.is_connected(*key))
        .map(|(i, key, seat)| {
            let until = (!view.presence.abandoned(key))
                .then(|| view.presence.secs_until_abandoned(key))
                .flatten();
            (i, view.presence.name(key), seats::seat_label(seat), until)
        })
        .collect();
    crate::ui::stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("seat_pause_card"),
        crate::ui::frames::hud().stroke(egui::Stroke::new(1.0, crate::ui::palette::CAUTION)),
        |ui| {
            ui.label(
                egui::RichText::new("\u{23f8} Game paused")
                    .size(16.0)
                    .strong()
                    .color(crate::ui::palette::CAUTION),
            );
            for (index, name, seat, until) in &absent {
                ui.label(
                    egui::RichText::new(format!("Waiting for {name} ({seat})."))
                        .color(crate::ui::palette::TEXT),
                );
                let status = match until {
                    Some(secs) => {
                        format!("Seat claimable by a newcomer in {}s.", secs.ceil() as u32)
                    }
                    None => "Seat abandoned \u{2014} a newcomer may claim it.".to_string(),
                };
                ui.label(
                    egui::RichText::new(status)
                        .size(12.0)
                        .color(crate::ui::palette::TEXT_MUTED),
                );
                let Ok(seat) = u8::try_from(*index) else {
                    continue;
                };
                if until.is_none() && seated {
                    let kind = SeatRequestKind::HandToAi { seat };
                    if client.is_pending(&kind) {
                        ui.label(
                            egui::RichText::new("Hand-over to the AI: vote pending\u{2026}")
                                .size(12.0)
                                .color(crate::ui::palette::AWAITING),
                        );
                    } else if ui
                        .button("Hand to AI")
                        .on_hover_text("Ask the other commanders to let the AI play this seat.")
                        .clicked()
                    {
                        client.outbox.push(kind);
                    }
                }
            }
        },
    );
}

/// Popup for a seated player asked to vote on a seat request.
pub(crate) fn vote_popup_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    view: SeatView,
    mut client: ResMut<SeatClient>,
) {
    let Some(ballot) = client.ballots.first().cloned() else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let secs_left = (ballot.deadline - time.elapsed_secs_f64()).max(0.0).ceil() as u32;
    let name = view.presence.name(ballot.requester);
    let mut answer: Option<bool> = None;
    crate::ui::anchored_card(
        ctx,
        egui::Id::new("seat_vote_popup"),
        egui::Align2::CENTER_CENTER,
        egui::Vec2::ZERO,
        crate::ui::frames::modal(),
        |ui| {
            ui.set_max_width(420.0);
            ui.label(
                egui::RichText::new("Seat request")
                    .size(18.0)
                    .strong()
                    .color(crate::ui::palette::TITLE),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!("{name} asks to {}.", ballot.summary))
                    .color(crate::ui::palette::TEXT),
            );
            ui.label(
                egui::RichText::new(format!(
                    "Every seated commander must allow it. {secs_left}s left."
                ))
                .size(12.0)
                .color(crate::ui::palette::TEXT_MUTED),
            );
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Allow").clicked() {
                    answer = Some(true);
                }
                if ui.button("Deny").clicked() {
                    answer = Some(false);
                }
            });
        },
    );
    if let Some(approve) = answer {
        client.cast.push((ballot.request_id, approve));
    }
}

/// What one seat looks like to a spectator deciding whether to join.
enum SeatOffer {
    /// The holder is here.
    Held(String),
    /// The holder is away; claimable in `Some(secs)`, or now (`None`).
    Away(String, Option<f64>),
    /// Played by the AI.
    Ai,
}

/// The spectator's join panel: claim an abandoned seat, ask for an AI seat,
/// or ask to take over some of a side's tribes / brigades. Folded to a
/// single line by "Keep watching".
pub(crate) fn join_panel_ui(
    mut contexts: EguiContexts,
    view: SeatView,
    peers: crate::peers::Peers,
    game_state: Res<crate::GameStateResource>,
    mut client: ResMut<SeatClient>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    if !peers.is_spectator() || game_state.0.game_over {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let offers: Vec<(u8, String, SeatOffer)> = view
        .seats
        .0
        .iter()
        .enumerate()
        .filter_map(|(i, seat)| {
            let index = u8::try_from(i).ok()?;
            let offer = match seat.holder {
                SeatHolder::Ai => SeatOffer::Ai,
                SeatHolder::Human(key) if view.presence.is_connected(key) => {
                    SeatOffer::Held(view.presence.name(key))
                }
                SeatHolder::Human(key) => SeatOffer::Away(
                    view.presence.name(key),
                    (!view.presence.abandoned(key))
                        .then(|| view.presence.secs_until_abandoned(key))
                        .flatten(),
                ),
            };
            Some((index, seats::seat_label(seat), offer))
        })
        .collect();
    let busy = client.any_pending();
    let folded = client.join_panel_folded;
    let mut outbox: Vec<SeatRequestKind> = Vec::new();
    let mut toggle_fold = false;

    crate::ui::stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("seat_join_panel"),
        crate::ui::frames::hud(),
        |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("You are watching this battle.")
                        .color(crate::ui::palette::TEXT),
                );
                let label = if folded {
                    "Join the game\u{2026}"
                } else {
                    "Keep watching"
                };
                if ui.button(label).clicked() {
                    toggle_fold = true;
                }
            });
            request_status_lines(ui, &client);
            if folded {
                return;
            }
            egui::ScrollArea::vertical()
                .max_height(320.0)
                .id_salt("seat_join_scroll")
                .show(ui, |ui| {
                    ui.label(crate::ui::text::subheading("Seats"));
                    for (index, label, offer) in &offers {
                        ui.horizontal(|ui| match offer {
                            SeatOffer::Held(name) => {
                                ui.label(format!("{label} \u{2014} {name}"));
                            }
                            SeatOffer::Away(name, Some(secs)) => {
                                ui.label(format!("{label} \u{2014} {name} is away"));
                                ui.add_enabled(false, egui::Button::new("Claim"))
                                    .on_disabled_hover_text(format!(
                                        "Claimable once abandoned, in {}s.",
                                        secs.ceil() as u32
                                    ));
                            }
                            SeatOffer::Away(name, None) => {
                                ui.label(format!("{label} \u{2014} abandoned by {name}"));
                                if ui
                                    .add_enabled(!busy, egui::Button::new("Claim"))
                                    .on_hover_text("Take this seat (no vote needed).")
                                    .clicked()
                                {
                                    outbox.push(SeatRequestKind::ClaimAbandoned { seat: *index });
                                }
                            }
                            SeatOffer::Ai => {
                                ui.label(format!("{label} \u{2014} AI"));
                                if ui
                                    .add_enabled(!busy, egui::Button::new("Request"))
                                    .on_hover_text(
                                        "Ask the seated commanders to let you replace the AI.",
                                    )
                                    .clicked()
                                {
                                    outbox.push(SeatRequestKind::ClaimFromAi { seat: *index });
                                }
                            }
                        });
                    }
                    for faction in [Player::Dervish, Player::AngloEgyptian] {
                        if !human_seats(&view.seats.0).any(|(_, s)| s.faction == faction) {
                            continue;
                        }
                        ui.add_space(6.0);
                        ui.label(crate::ui::text::subheading(format!(
                            "Take over part of the {} side",
                            crate::ui::faction_name(faction)
                        )));
                        for (name, scope) in takeover_scopes(faction) {
                            let holders =
                                scope_holders(&view.seats.0, &scope, |k| view.presence.name(k));
                            ui.horizontal(|ui| {
                                ui.label(format!("{name} \u{2014} {holders}"));
                                if ui
                                    .add_enabled(!busy, egui::Button::new("Request takeover"))
                                    .on_hover_text("Every seated commander must allow it.")
                                    .clicked()
                                {
                                    outbox.push(SeatRequestKind::TakeOver { faction, scope });
                                }
                            });
                        }
                    }
                });
        },
    );
    if toggle_fold {
        client.join_panel_folded = !client.join_panel_folded;
    }
    client.outbox.extend(outbox);
}

/// The single-tribe / single-brigade scopes a spectator may ask for.
fn takeover_scopes(faction: Player) -> Vec<(String, CommandScope)> {
    match faction {
        Player::Dervish => DervishTribe::iter()
            .map(|t| (t.to_string(), CommandScope::Tribes(BTreeSet::from([t]))))
            .collect(),
        Player::AngloEgyptian => BrigadeId::ALL
            .into_iter()
            .filter(|b| b.nationality != BrigadeNationality::Friendlies)
            .map(|b| (b.to_string(), CommandScope::Brigades(BTreeSet::from([b]))))
            .collect(),
    }
}

/// Who commands `scope` today: the named holders of human seats claiming
/// it, or "communal" when none does.
fn scope_holders(
    seats: &[Seat],
    scope: &CommandScope,
    name: impl Fn(PlayerKey) -> String,
) -> String {
    let names: Vec<String> = human_seats(seats)
        .filter(|(_, s)| {
            s.scope
                .as_ref()
                .is_some_and(|own| !own.claims_nothing() && scope.is_subset_of(own))
        })
        .map(|(key, _)| name(key))
        .collect();
    if names.is_empty() {
        "communal".to_string()
    } else {
        format!("held by {}", names.join(", "))
    }
}

/// Status lines for this peer's recent seat requests.
fn request_status_lines(ui: &mut egui::Ui, client: &SeatClient) {
    let Some(last) = client.requests.last() else {
        return;
    };
    let (text, color) = match &last.status {
        RequestStatus::Pending => (
            "Request sent \u{2014} waiting for the host\u{2026}".to_string(),
            crate::ui::palette::AWAITING,
        ),
        RequestStatus::Voting { .. } => (
            "The seated commanders are voting on your request\u{2026}".to_string(),
            crate::ui::palette::AWAITING,
        ),
        RequestStatus::Approved => ("Request approved.".to_string(), crate::ui::palette::SUCCESS),
        RequestStatus::Denied(reason) => (
            format!("Request refused: {reason}"),
            crate::ui::palette::REFUSED,
        ),
    };
    ui.label(egui::RichText::new(text).size(12.0).color(color));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_holders_names_the_claimant_or_communal() {
        let seats = vec![Seat {
            faction: Player::Dervish,
            scope: Some(CommandScope::Tribes(BTreeSet::from([
                DervishTribe::Baggara,
                DervishTribe::Jaalin,
            ]))),
            holder: SeatHolder::Human(PlayerKey(1)),
        }];
        let name = |_| "Ada".to_string();
        let baggara = CommandScope::Tribes(BTreeSet::from([DervishTribe::Baggara]));
        let hadendowa = CommandScope::Tribes(BTreeSet::from([DervishTribe::Hadendowa]));
        assert_eq!(scope_holders(&seats, &baggara, name), "held by Ada");
        assert_eq!(scope_holders(&seats, &hadendowa, name), "communal");
    }

    #[test]
    fn takeover_offers_every_tribe_and_integrating_brigade() {
        assert_eq!(
            takeover_scopes(Player::Dervish).len(),
            DervishTribe::iter().count()
        );
        assert!(
            takeover_scopes(Player::AngloEgyptian)
                .iter()
                .all(|(_, s)| seats::carvable_scope(Player::AngloEgyptian, s))
        );
    }
}
