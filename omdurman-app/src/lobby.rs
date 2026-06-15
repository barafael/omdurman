//! Pre-game lobby (§lobby).
//!
//! Once peers are connected the app enters [`AppState::Lobby`]. Each player
//! sees everyone's name + colour (and live cursors, drawn by the existing
//! cursor overlay), and picks a faction. The pick is broadcast as a live
//! preview via [`Ephemeral::FactionChoice`]. When every connected player has a
//! distinct faction, the **host** can start the game, which broadcasts the
//! authoritative binding as [`GameEvent::StartGame`] -- recorded and replayed,
//! so late joiners inherit it through the snapshot path.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::{Ephemeral, GameEvent, NetMsg, NetState};
use omdurman_rules::Player;

use crate::settings::{LocalPlayerSettings, PlayerInfoMap};
use crate::{AppState, LobbyChoices, LobbyScenario, LocalFaction, PendingEdits};
use omdurman_rules::Scenario;

/// Both selectable factions, with display labels.
const FACTIONS: [(Player, &str); 2] = [
    (Player::AngloEgyptian, "Anglo-Egyptian"),
    (Player::Dervish, "Dervish"),
];

/// Selectable scenarios, with display labels. The Campaign game uses the
/// strategic campaign map; the other two share the Fall-of-Khartoum map
/// (§dual-map).
const SCENARIOS: [(Scenario, &str); 3] = [
    (Scenario::Campaign, "Campaign"),
    (Scenario::Historical, "Historical"),
    (Scenario::FallOfKhartoum, "Fall of Khartoum"),
];

fn faction_label(p: Player) -> &'static str {
    FACTIONS
        .iter()
        .find(|(f, _)| *f == p)
        .map(|(_, l)| *l)
        .unwrap_or("?")
}

/// The lobby screen. Shown only in [`AppState::Lobby`].
pub fn lobby_ui(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    net: Res<NetState>,
    local: Res<LocalPlayerSettings>,
    player_info: Res<PlayerInfoMap>,
    mut local_faction: ResMut<LocalFaction>,
    choices: Res<LobbyChoices>,
    mut lobby_scenario: ResMut<LobbyScenario>,
    mut pending: ResMut<PendingEdits>,
) {
    if *state.get() != AppState::Lobby {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::from_gray(24)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(24.0);
                ui.heading(
                    egui::RichText::new("REMEMBER GORDON! -- Lobby")
                        .size(26.0)
                        .color(egui::Color32::from_gray(230)),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Choose your faction, then the host starts the battle.")
                        .color(egui::Color32::from_gray(170)),
                );
            });
            ui.add_space(16.0);

            // -- Local faction picker --------------------------------------
            ui.group(|ui| {
                ui.label(
                    egui::RichText::new(format!("You -- {}", local.name))
                        .strong()
                        .color(local.color()),
                );
                ui.horizontal(|ui| {
                    ui.label("Faction:");
                    let mut changed = false;
                    // Multiple players may share a faction (each commands some
                    // of its tribes/brigades -- §1.1), so factions aren't
                    // exclusive; any may be picked.
                    for (faction, label) in FACTIONS {
                        let selected = local_faction.0 == Some(faction);
                        if ui.add(egui::Button::selectable(selected, label)).clicked() {
                            local_faction.0 = if selected { None } else { Some(faction) };
                            changed = true;
                        }
                    }
                    if changed {
                        pending
                            .outgoing_broadcast
                            .push(NetMsg::Ephemeral(Ephemeral::FactionChoice(local_faction.0)));
                    }
                });
            });

            ui.add_space(8.0);

            // -- Scenario picker (host-authoritative) ----------------------
            ui.group(|ui| {
                // Guests preview the host's latest broadcast pick; the host
                // edits its own selection.
                let display = if net.is_host {
                    lobby_scenario.0
                } else {
                    choices.scenario.unwrap_or(lobby_scenario.0)
                };
                ui.label(
                    egui::RichText::new("Scenario")
                        .strong()
                        .color(egui::Color32::from_gray(200)),
                );
                ui.horizontal(|ui| {
                    for (scenario, label) in SCENARIOS {
                        let selected = display == scenario;
                        let button = egui::Button::selectable(selected, label);
                        if net.is_host {
                            if ui.add(button).clicked() && !selected {
                                lobby_scenario.0 = scenario;
                                pending
                                    .outgoing_broadcast
                                    .push(NetMsg::Ephemeral(Ephemeral::ScenarioChoice(scenario)));
                            }
                        } else {
                            // Read-only preview for guests.
                            ui.add_enabled(false, button);
                        }
                    }
                });
                if !net.is_host {
                    ui.label(
                        egui::RichText::new("The host chooses the scenario.")
                            .weak()
                            .size(11.0),
                    );
                }
            });

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Players")
                    .strong()
                    .color(egui::Color32::from_gray(200)),
            );

            // -- Connected players + their picks ---------------------------
            for peer in net.sorted_all() {
                let is_me = net.my_id == Some(*peer);
                let (name, color) = if is_me {
                    (local.name.clone(), local.color())
                } else if let Some(info) = player_info.peers.get(peer) {
                    (info.name.clone(), info.color)
                } else {
                    ("(connecting...)".to_string(), egui::Color32::GRAY)
                };
                let pick = if is_me {
                    local_faction.0
                } else {
                    choices.by_peer.get(peer).copied().flatten()
                };
                ui.horizontal(|ui| {
                    // colour swatch
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 3.0, color);
                    ui.label(egui::RichText::new(&name).color(color));
                    if is_me {
                        ui.label(egui::RichText::new("(you)").weak());
                    }
                    if net.host_id() == Some(*peer) {
                        ui.label(egui::RichText::new("[host]").color(egui::Color32::GOLD));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        match pick {
                            Some(f) => ui.label(
                                egui::RichText::new(faction_label(f))
                                    .color(egui::Color32::LIGHT_GREEN),
                            ),
                            None => ui.label(egui::RichText::new("undecided").weak()),
                        };
                    });
                });
            }

            ui.add_space(16.0);

            // -- Host start control ----------------------------------------
            let ready = all_players_ready(&net, &local_faction, &choices);
            if net.is_host {
                ui.add_enabled_ui(ready, |ui| {
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new("[swords]  Start Battle").size(18.0),
                        ))
                        .clicked()
                    {
                        let assignments = collect_assignments(&net, &local_faction, &choices);
                        pending
                            .outgoing_broadcast
                            .push(NetMsg::Game(GameEvent::StartGame {
                                assignments,
                                scenario: lobby_scenario.0,
                            }));
                    }
                });
                if !ready {
                    ui.label(
                        egui::RichText::new(
                            "Every player needs a distinct faction before starting.",
                        )
                        .weak(),
                    );
                }
            } else {
                ui.label(
                    egui::RichText::new("Waiting for the host to start...")
                        .color(egui::Color32::from_gray(170)),
                );
            }
        });
}

/// The local player's pick keyed by its own peer id, merged with remote picks.
fn local_pick(
    net: &NetState,
    local_faction: &LocalFaction,
) -> Option<(bevy_matchbox::prelude::PeerId, Player)> {
    Some((net.my_id?, local_faction.0?))
}

/// Whether the lobby is ready to start: every connected player has chosen a
/// faction and **both** factions are represented (so the battle has two
/// sides). Multiple players may share a faction (§1.1).
fn all_players_ready(net: &NetState, local_faction: &LocalFaction, choices: &LobbyChoices) -> bool {
    let mut ae = false;
    let mut dervish = false;
    for peer in net.sorted_all() {
        let pick = if net.my_id == Some(*peer) {
            local_faction.0
        } else {
            choices.by_peer.get(peer).copied().flatten()
        };
        match pick {
            Some(Player::AngloEgyptian) => ae = true,
            Some(Player::Dervish) => dervish = true,
            None => return false, // someone hasn't decided yet
        }
    }
    ae && dervish
}

/// Build the `(peer_id_string, faction)` assignments for `StartGame`.
fn collect_assignments(
    net: &NetState,
    local_faction: &LocalFaction,
    choices: &LobbyChoices,
) -> Vec<(String, Player)> {
    let mut out = Vec::new();
    if let Some((id, f)) = local_pick(net, local_faction) {
        out.push((id.0.to_string(), f));
    }
    for peer in net.sorted_all() {
        if net.my_id == Some(*peer) {
            continue;
        }
        if let Some(Some(f)) = choices.by_peer.get(peer).copied() {
            out.push((peer.0.to_string(), f));
        }
    }
    out
}
