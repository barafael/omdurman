//! Pre-game lobby (§lobby).
//!
//! Once peers are connected the app enters [`AppState::Lobby`]. Each player
//! sees everyone's name + colour (and live cursors, drawn by the existing
//! cursor overlay), and picks a faction -- or chooses to **spectate** (join to
//! watch only, no faction). Picks are broadcast as live previews via
//! [`Ephemeral::FactionChoice`] / [`Ephemeral::SpectatorChoice`]. Once both
//! factions are represented among the non-spectating players, the **host** can
//! start the game, which broadcasts the authoritative binding as
//! [`GameEvent::StartGame`] -- recorded and replayed, so late joiners inherit it
//! through the snapshot path. Spectators are never in that binding, so every
//! action gate (`PlayerFactions::local_may_act`) no-ops for them.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::{Ephemeral, GameEvent, NetMsg, NetState};
use omdurman_rules::Player;

use crate::settings::{LocalPlayerSettings, PlayerInfoMap};
use crate::timeline::SpectatorTimeline;
use crate::{
    AppState, LobbyChoices, LobbyScenario, LobbyTab, LocalFaction, LocalSpectator, PendingEdits,
};
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
#[allow(clippy::too_many_arguments)]
pub fn lobby_ui(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    net: Res<NetState>,
    local: Res<LocalPlayerSettings>,
    player_info: Res<PlayerInfoMap>,
    mut local_faction: ResMut<LocalFaction>,
    mut local_spectator: ResMut<LocalSpectator>,
    choices: Res<LobbyChoices>,
    mut lobby_scenario: ResMut<LobbyScenario>,
    mut pending: ResMut<PendingEdits>,
    mut tab: ResMut<LobbyTab>,
    mut timeline: ResMut<SpectatorTimeline>,
    recorder: Res<crate::game_record::GameRecorder>,
    mut saved_games: ResMut<crate::game_record::SavedGamesCache>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if *state.get() != AppState::Lobby {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::from_gray(24)))
        .show(ctx, |ui| {
            // Center the whole lobby in a column that scales with the window:
            // ~55% of the available width, clamped so it stays readable on a
            // small window and doesn't sprawl on a wide one.
            let column_w = (ui.available_width() * 0.55).clamp(460.0, 900.0);
            let top_pad = (ui.available_height() * 0.06).clamp(16.0, 80.0);
            ui.vertical_centered(|ui| {
                ui.set_max_width(column_w);
                ui.add_space(top_pad);
                ui.heading(
                    egui::RichText::new("REMEMBER GORDON! -- Lobby")
                        .size(26.0)
                        .color(egui::Color32::from_gray(230)),
                );
                ui.add_space(8.0);

                // -- Sub-tabs --------------------------------------------------
                ui.horizontal(|ui| {
                    if ui
                        .add(egui::Button::selectable(*tab == LobbyTab::Setup, "Setup"))
                        .clicked()
                    {
                        *tab = LobbyTab::Setup;
                    }
                    if ui
                        .add(egui::Button::selectable(
                            *tab == LobbyTab::SavedGames,
                            "Saved games",
                        ))
                        .clicked()
                    {
                        *tab = LobbyTab::SavedGames;
                    }
                });
                ui.add_space(12.0);

                match *tab {
                    LobbyTab::Setup => setup_tab(
                        ui,
                        &net,
                        &local,
                        &player_info,
                        &mut local_faction,
                        &mut local_spectator,
                        &choices,
                        &mut lobby_scenario,
                        &mut pending,
                    ),
                    LobbyTab::SavedGames => saved_games_tab(
                        ui,
                        &recorder,
                        &mut saved_games,
                        &mut timeline,
                        &mut next_state,
                    ),
                }
            });
        });
}

/// The lobby's "Setup" sub-tab: faction / scenario picks, the player roster, and
/// the host's start control.
#[allow(clippy::too_many_arguments)]
fn setup_tab(
    ui: &mut egui::Ui,
    net: &NetState,
    local: &LocalPlayerSettings,
    player_info: &PlayerInfoMap,
    local_faction: &mut LocalFaction,
    local_spectator: &mut LocalSpectator,
    choices: &LobbyChoices,
    lobby_scenario: &mut LobbyScenario,
    pending: &mut PendingEdits,
) {
    ui.label(
        egui::RichText::new("Choose your faction, then the host starts the battle.")
            .color(egui::Color32::from_gray(170)),
    );
    ui.add_space(16.0);

    {
        // -- Local faction picker --------------------------------------
        ui.group(|ui| {
            ui.label(
                egui::RichText::new(format!("You -- {}", local.name))
                    .strong()
                    .color(local.color()),
            );
            ui.horizontal(|ui| {
                ui.label("Faction:");
                let mut faction_changed = false;
                let mut spectator_changed = false;
                // Multiple players may share a faction (each commands some
                // of its tribes/brigades -- §1.1), so factions aren't
                // exclusive; any may be picked.
                for (faction, label) in FACTIONS {
                    let selected = local_faction.0 == Some(faction);
                    if ui.add(egui::Button::selectable(selected, label)).clicked() {
                        local_faction.0 = if selected { None } else { Some(faction) };
                        faction_changed = true;
                        // Picking a faction cancels spectating.
                        if local_faction.0.is_some() && local_spectator.0 {
                            local_spectator.0 = false;
                            spectator_changed = true;
                        }
                    }
                }
                ui.separator();
                // Spectate: join to watch only, no faction. Mutually
                // exclusive with a faction pick.
                if ui
                    .add(egui::Button::selectable(local_spectator.0, "Spectate"))
                    .clicked()
                {
                    local_spectator.0 = !local_spectator.0;
                    spectator_changed = true;
                    if local_spectator.0 && local_faction.0.is_some() {
                        local_faction.0 = None;
                        faction_changed = true;
                    }
                }
                if faction_changed {
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Ephemeral(Ephemeral::FactionChoice(local_faction.0)));
                }
                if spectator_changed {
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Ephemeral(Ephemeral::SpectatorChoice(
                            local_spectator.0,
                        )));
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
            let spectating = is_spectating(peer, net, local_spectator, choices);
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
                    if spectating {
                        ui.label(
                            egui::RichText::new("spectating")
                                .color(egui::Color32::from_rgb(210, 180, 130)),
                        );
                    } else {
                        match pick {
                            Some(f) => ui.label(
                                egui::RichText::new(faction_label(f))
                                    .color(egui::Color32::from_rgb(230, 200, 120)),
                            ),
                            None => ui.label(egui::RichText::new("undecided").weak()),
                        };
                    }
                });
            });
        }

        ui.add_space(16.0);

        // -- Host start control ----------------------------------------
        let ready = all_players_ready(net, local_faction, local_spectator, choices);
        if net.is_host {
            ui.add_enabled_ui(ready, |ui| {
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("[swords]  Start Battle").size(18.0),
                    ))
                    .clicked()
                {
                    let assignments = collect_assignments(net, local_faction, choices);
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
                        "Both factions must be chosen before starting (spectators excluded).",
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
    }
}

/// The lobby's "Saved games" sub-tab: review the in-memory game, or (native)
/// load a finished game from `games/*.jsonl`. Replaces the old floating "Review
/// a game" overlay; the list is served from [`SavedGamesCache`] (refreshed on
/// entering the lobby) and shows minimal per-game metadata.
fn saved_games_tab(
    ui: &mut egui::Ui,
    recorder: &crate::game_record::GameRecorder,
    saved_games: &mut crate::game_record::SavedGamesCache,
    timeline: &mut SpectatorTimeline,
    next_state: &mut NextState<AppState>,
) {
    // Review whatever this peer has recorded in memory so far.
    if let Some(record) = recorder.record.as_ref()
        && !record.events.is_empty()
    {
        if ui
            .button(format!(
                "Review current game ({} events)",
                record.events.len()
            ))
            .clicked()
        {
            timeline.open(record.clone(), "current game".to_string());
            next_state.set(AppState::Spectating);
        }
        ui.add_space(8.0);
    }

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Saved games").color(egui::Color32::from_gray(190)));
        if ui.small_button("Refresh").clicked() {
            saved_games.refresh();
        }
    });

    if saved_games.games.is_empty() {
        ui.label(egui::RichText::new("(none found)").weak());
        return;
    }

    // Let the list use most of the remaining vertical space (still scrolls if
    // it overflows), rather than a fixed 280px that wasted a tall window.
    let list_h = (ui.available_height() - 8.0).max(200.0);
    egui::ScrollArea::vertical()
        .max_height(list_h)
        .id_salt("saved_games_scroll")
        .show(ui, |ui| {
            for game in &saved_games.games {
                let review = ui
                    .group(|ui| {
                        ui.horizontal(|ui| {
                            let clicked = ui
                                .button(egui::RichText::new(&game.name).monospace())
                                .clicked();
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| ui.label(game_meta_label(game)),
                            );
                            clicked
                        })
                        .inner
                    })
                    .inner;
                if review {
                    // Loading from disk is native-only; on wasm the list is
                    // always empty, so this branch never fires there.
                    #[cfg(not(target_arch = "wasm32"))]
                    match crate::game_record::load_record_from_jsonl(&game.path) {
                        Ok(record) => {
                            timeline.open(record, game.name.clone());
                            next_state.set(AppState::Spectating);
                        }
                        Err(error) => {
                            warn!(%error, path = %game.path, "failed to load saved game");
                        }
                    }
                }
            }
        });
}

/// One-line metadata summary for a saved game: scenario + event count (+ last-
/// played time when known). Reads only the cheap fields extracted at scan time.
fn game_meta_label(game: &crate::game_record::SavedGame) -> egui::RichText {
    let text = match &game.meta {
        Some(meta) => {
            let scenario = meta
                .scenario
                .map_or_else(|| "not started".to_string(), |s| s.to_string());
            let mut s = format!("{scenario} \u{2022} {} events", meta.events);
            if let Some(ts) = meta.last_played {
                s.push_str(&format!(" \u{2022} {}", ts.format("%Y-%m-%d %H:%M")));
            }
            s
        }
        None => "unreadable".to_string(),
    };
    egui::RichText::new(text)
        .weak()
        .size(11.0)
        .color(egui::Color32::from_gray(160))
}

/// Whether `peer` is spectating, reading the local toggle for our own peer and
/// the broadcast preview for everyone else.
fn is_spectating(
    peer: &bevy_matchbox::prelude::PeerId,
    net: &NetState,
    local_spectator: &LocalSpectator,
    choices: &LobbyChoices,
) -> bool {
    if net.my_id == Some(*peer) {
        local_spectator.0
    } else {
        choices.spectators.contains(peer)
    }
}

/// The local player's pick keyed by its own peer id, merged with remote picks.
fn local_pick(
    net: &NetState,
    local_faction: &LocalFaction,
) -> Option<(bevy_matchbox::prelude::PeerId, Player)> {
    Some((net.my_id?, local_faction.0?))
}

/// Whether the lobby is ready to start. Spectators join to watch and are
/// ignored here; among the *non-spectating* players, everyone must have chosen a
/// faction and **both** factions must be represented (so the battle has two
/// sides). Multiple players may share a faction (§1.1).
fn all_players_ready(
    net: &NetState,
    local_faction: &LocalFaction,
    local_spectator: &LocalSpectator,
    choices: &LobbyChoices,
) -> bool {
    let mut ae = false;
    let mut dervish = false;
    for peer in net.sorted_all() {
        if is_spectating(peer, net, local_spectator, choices) {
            continue; // spectators don't need a faction
        }
        let pick = if net.my_id == Some(*peer) {
            local_faction.0
        } else {
            choices.by_peer.get(peer).copied().flatten()
        };
        match pick {
            Some(Player::AngloEgyptian) => ae = true,
            Some(Player::Dervish) => dervish = true,
            None => return false, // an active player hasn't decided yet
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
