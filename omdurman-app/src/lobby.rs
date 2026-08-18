//! Pre-game lobby (§lobby).
//!
//! Once peers are connected the app enters [`AppState::Lobby`]. Each player
//! sees everyone's name + colour (and live cursors, drawn by the existing
//! cursor overlay), and picks a faction -- or chooses to **spectate** (join to
//! watch only, no faction). Picks are broadcast as live previews via
//! [`Ephemeral::FactionChoice`] / [`Ephemeral::SpectatorChoice`] and stored on
//! the peer entities ([`crate::peers::LobbyPick`] / [`crate::peers::Spectator`]).
//! Once both factions are represented among the non-spectating players, the
//! **host** can start the game, which broadcasts the authoritative binding as
//! [`GameEvent::StartGame`] -- recorded and replayed, so late joiners inherit it
//! through the snapshot path. Spectators are never in that binding, so every
//! action gate (`crate::peers::Peers::may_act`) no-ops for them.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use bevy_matchbox::prelude::PeerId;
use omdurman_net::{Ephemeral, GameEvent, NetMsg, NetState, RoomId};
use omdurman_types::{Player, Scenario};

use crate::game_record::{GameRecorder, SavedGamesCache};
use crate::settings::{LocalPlayerSettings, ReconnectRoom};
use crate::timeline::SpectatorTimeline;
use crate::{AppState, PendingEdits};

// -- Lobby resources --------------------------------------------------------

/// Which sub-tab the lobby screen is showing (§lobby). "Setup" is the faction /
/// scenario / start panel; "Saved games" is the review-a-game list (a saved-
/// games browser embedded in the lobby rather than a floating overlay).
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum LobbyTab {
    #[default]
    Setup,
    SavedGames,
}

/// Host's lobby scenario selection (§lobby), committed into
/// [`GameEvent::StartGame`]. Other peers see it as a live preview via
/// [`Ephemeral::ScenarioChoice`].
#[derive(Resource)]
pub struct LobbyScenario(pub Scenario);

impl Default for LobbyScenario {
    fn default() -> Self {
        Self(Scenario::Campaign)
    }
}

/// Latest scenario broadcast by the host's lobby (live preview, §lobby).
/// `None` until the host sends one; the committed value rides in
/// [`GameEvent::StartGame`].
#[derive(Resource, Default)]
pub struct RemoteScenario(pub Option<Scenario>);

/// The local player's current lobby faction pick (pre-commit).
#[derive(Resource, Default)]
pub struct LocalFaction(pub Option<Player>);

/// Whether the local player has chosen to spectate (join to watch, no faction).
/// Kept separate from [`LocalFaction`] so "spectating" is distinct from
/// "undecided". A spectator is never included in the `StartGame` assignments.
#[derive(Resource, Default)]
pub struct LocalSpectator(pub bool);

/// Host's optional-rule selection for a campaign game (§10.11, §10.21).
/// `None` means no optional rule. Only meaningful for the Dervish host in a
/// Campaign scenario.
#[derive(Resource, Default)]
pub struct LocalOptionalRule(pub Option<omdurman_rules::OptionalRule>);

/// Bundles the lobby-specific mutable resources so [`lobby_ui`] stays under
/// Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct LobbyContext<'w, 's> {
    pub local_faction: ResMut<'w, LocalFaction>,
    pub local_spectator: ResMut<'w, LocalSpectator>,
    pub local_optional_rule: ResMut<'w, LocalOptionalRule>,
    pub remote_scenario: Res<'w, RemoteScenario>,
    pub lobby_scenario: ResMut<'w, LobbyScenario>,
    pub pending: ResMut<'w, PendingEdits>,
    pub tab: ResMut<'w, LobbyTab>,
    pub timeline: ResMut<'w, SpectatorTimeline>,
    pub recorder: Res<'w, GameRecorder>,
    pub saved_games: ResMut<'w, SavedGamesCache>,
    pub next_state: ResMut<'w, NextState<AppState>>,
    pub room: Res<'w, RoomId>,
    /// One row per connected peer (remote picks/names live on the peer
    /// entities; the local row is synthesized from the local resources).
    pub peers: crate::peers::RosterQuery<'w, 's>,
}

/// Both selectable factions, with display labels.
const FACTIONS: [(Player, &str); 2] = [
    (Player::AngloEgyptian, "Anglo-Egyptian"),
    (Player::Dervish, "Dervish"),
];

fn faction_label(p: Player) -> &'static str {
    FACTIONS
        .iter()
        .find(|(f, _)| *f == p)
        .map(|(_, l)| *l)
        .unwrap_or("?")
}

/// One row of the lobby roster. Remote fields (name/colour/pick/spectating)
/// come from the peer entity components; the local row is synthesized from the
/// local settings + pick resources.
struct RosterEntry {
    peer: PeerId,
    name: String,
    color: egui::Color32,
    pick: Option<Player>,
    spectating: bool,
    is_host: bool,
}

/// Build the roster (in canonical peer order) from the peer entities, merging
/// the local player's live resources in for its own row.
fn build_roster(
    net: &NetState,
    local: &LocalPlayerSettings,
    local_faction: &LocalFaction,
    local_spectator: &LocalSpectator,
    peers: &crate::peers::RosterQuery<'_, '_>,
) -> Vec<RosterEntry> {
    let host = net.host_id();
    net.sorted_all()
        .iter()
        .map(|peer| {
            if net.my_id == Some(*peer) {
                RosterEntry {
                    peer: *peer,
                    name: local.name.clone(),
                    color: local.color(),
                    pick: local_faction.0,
                    spectating: local_spectator.0,
                    is_host: host == Some(*peer),
                }
            } else {
                let (name, color, pick, spectating) = peers
                    .iter()
                    .find(|(key, ..)| key.0 == *peer)
                    .map(|(_, name, color, pick, spectating)| {
                        let name = name
                            .map(|n| n.0.clone())
                            .unwrap_or_else(|| "(connecting...)".to_string());
                        let color = color.map(|c| c.0).unwrap_or(egui::Color32::GRAY);
                        let pick = pick.and_then(|p| p.0);
                        (name, color, pick, spectating)
                    })
                    .unwrap_or_else(|| {
                        (
                            "(connecting...)".to_string(),
                            egui::Color32::GRAY,
                            None,
                            false,
                        )
                    });
                RosterEntry {
                    peer: *peer,
                    name,
                    color,
                    pick,
                    spectating,
                    is_host: host == Some(*peer),
                }
            }
        })
        .collect()
}

/// The lobby screen. Shown only in [`AppState::Lobby`] (gated at the system
/// registration site).
pub fn lobby_ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    net: Res<NetState>,
    mut local: ResMut<LocalPlayerSettings>,
    mut ctx: LobbyContext,
    mut editing_session: Local<String>,
) {
    let Ok(egui_ctx) = contexts.ctx_mut() else { return };

    let roster = build_roster(&net, &local, &ctx.local_faction, &ctx.local_spectator, &ctx.peers);

    let mut __ui = egui::Ui::new(
        egui_ctx.clone(),
        egui::Id::new("lobby"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(egui_ctx.viewport_rect()),
    );
    let __panel = egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::from_gray(24)))
        .show(&mut __ui, |ui| {
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
                        .add(egui::Button::selectable(
                            *ctx.tab == LobbyTab::Setup,
                            "Setup",
                        ))
                        .clicked()
                    {
                        *ctx.tab = LobbyTab::Setup;
                    }
                    if ui
                        .add(egui::Button::selectable(
                            *ctx.tab == LobbyTab::SavedGames,
                            "Saved games",
                        ))
                        .clicked()
                    {
                        *ctx.tab = LobbyTab::SavedGames;
                    }
                });
                ui.add_space(12.0);

                match *ctx.tab {
                    LobbyTab::Setup => setup_tab(
                        ui,
                        &net,
                        &mut local,
                        &roster,
                        LocalFactionPick {
                            local_faction: &mut ctx.local_faction,
                            local_spectator: &mut ctx.local_spectator,
                        },
                        LobbySetupChoices {
                            remote_scenario: &ctx.remote_scenario,
                            lobby_scenario: &mut ctx.lobby_scenario,
                            optional_rule: &mut ctx.local_optional_rule,
                        },
                        SessionControls {
                            pending: &mut ctx.pending,
                            commands: &mut commands,
                            room: &ctx.room,
                            editing_session: &mut editing_session,
                        },
                        &ctx.recorder,
                    ),
                    LobbyTab::SavedGames => saved_games_tab(
                        ui,
                        &ctx.recorder,
                        &mut ctx.saved_games,
                        &mut ctx.timeline,
                        &mut ctx.next_state,
                    ),
                }
            });
        });
    crate::ui_plugin::register_panel_rect(egui_ctx, __panel.response.rect);
}

/// Mutable session-level state for the lobby "Setup" tab: the pending-edits
/// queue, command buffer, room id, and the in-progress session-id text. Bundled
/// so [`setup_tab`] stays under clippy's argument limit.
struct SessionControls<'a, 'b, 'c> {
    pending: &'a mut PendingEdits,
    commands: &'a mut Commands<'b, 'c>,
    room: &'a RoomId,
    editing_session: &'a mut String,
}

/// Bundle of the local faction + spectator picks so [`setup_tab`] stays under
/// clippy's argument limit.
struct LocalFactionPick<'a> {
    local_faction: &'a mut LocalFaction,
    local_spectator: &'a mut LocalSpectator,
}

/// Bundle of the host-broadcast scenario preview, the host's scenario pick, and
/// the optional rule so [`setup_tab`] stays under clippy's argument limit.
struct LobbySetupChoices<'a> {
    remote_scenario: &'a RemoteScenario,
    lobby_scenario: &'a mut LobbyScenario,
    optional_rule: &'a mut LocalOptionalRule,
}

/// The lobby's "Setup" sub-tab: session, identity, faction / scenario picks,
/// the player roster, the host's start control, and preferences.
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
#[allow(clippy::too_many_arguments)]
fn setup_tab(
    ui: &mut egui::Ui,
    net: &NetState,
    local: &mut LocalPlayerSettings,
    roster: &[RosterEntry],
    faction_pick: LocalFactionPick,
    lobby: LobbySetupChoices,
    session: SessionControls,
    recorder: &GameRecorder,
) {
    let LocalFactionPick {
        local_faction,
        local_spectator,
    } = faction_pick;
    let LobbySetupChoices {
        remote_scenario,
        lobby_scenario,
        optional_rule,
    } = lobby;
    let SessionControls {
        pending,
        commands,
        room,
        editing_session,
    } = session;
    ui.label(
        egui::RichText::new("Choose your faction, then the host starts the battle.")
            .color(egui::Color32::from_gray(170)),
    );
    ui.add_space(16.0);

    {
        // -- Session (room ID + Host/Join) --------------------------------
        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Session")
                    .strong()
                    .color(egui::Color32::from_gray(200)),
            );
            ui.horizontal(|ui| {
                if editing_session.is_empty() {
                    *editing_session = room.as_str().to_owned();
                }
                ui.add_sized(
                    egui::vec2(200.0, 22.0),
                    egui::TextEdit::singleline(editing_session),
                );
                let host = ui.button("Host").clicked();
                let join = ui.button("Join").clicked();
                if host || join {
                    let id = if editing_session.is_empty() {
                        room.as_str().to_owned()
                    } else {
                        editing_session.clone()
                    };
                    commands.insert_resource(ReconnectRoom(id));
                }
            });
            ui.label(
                egui::RichText::new("Host creates a room, Join connects to the typed ID.")
                    .weak()
                    .size(11.0),
            );
        });

        ui.add_space(8.0);

        // -- Player identity (name + color) --------------------------------
        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Your identity")
                    .strong()
                    .color(egui::Color32::from_gray(200)),
            );
            ui.horizontal(|ui| {
                ui.label("Name:");
                let name_changed = ui
                    .add_sized(
                        egui::vec2(200.0, 22.0),
                        egui::TextEdit::singleline(&mut local.name),
                    )
                    .changed();
                if name_changed {
                    let n = local.name.clone();
                    local.set_name(n);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Color:");
                let mut c = local.color();
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut c,
                    egui::color_picker::Alpha::Opaque,
                );
                if c != local.color() && !ui.ctx().egui_is_using_pointer() {
                    local.commit_color(c);
                }
            });
        });

        ui.add_space(8.0);

        // -- Local faction picker --------------------------------------
        ui.group(|ui| {
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
                remote_scenario.0.unwrap_or(lobby_scenario.0)
            };
            ui.label(
                egui::RichText::new("Scenario")
                    .strong()
                    .color(egui::Color32::from_gray(200)),
            );
            ui.horizontal(|ui| {
                for scenario in Scenario::ALL {
                    let selected = display == scenario;
                    let button = egui::Button::selectable(selected, scenario.label());
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

        // -- Optional rule picker (host only, campaign only) ------------
        if net.is_host && lobby_scenario.0 == Scenario::Campaign {
            ui.add_space(4.0);
            ui.group(|ui| {
                ui.label(
                    egui::RichText::new("Optional Rule (§10)")
                        .strong()
                        .color(egui::Color32::from_gray(200)),
                );
                ui.horizontal(|ui| {
                    let opt_rule = &mut *optional_rule;
                    let none_sel = opt_rule.0.is_none();
                    if ui.selectable_label(none_sel, "None").clicked() {
                        opt_rule.0 = None;
                    }
                    let mines_sel = opt_rule.0 == Some(omdurman_rules::OptionalRule::RiverMines);
                    if ui.selectable_label(mines_sel, "River Mines").clicked() {
                        opt_rule.0 = Some(omdurman_rules::OptionalRule::RiverMines);
                    }
                    let chain_sel = opt_rule.0 == Some(omdurman_rules::OptionalRule::RiverChain);
                    if ui.selectable_label(chain_sel, "River Chain").clicked() {
                        opt_rule.0 = Some(omdurman_rules::OptionalRule::RiverChain);
                    }
                });
            });
        }

        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Players")
                .strong()
                .color(egui::Color32::from_gray(200)),
        );

        // -- Connected players + their picks ---------------------------
        for entry in roster {
            let is_me = net.my_id == Some(entry.peer);
            ui.horizontal(|ui| {
                // colour swatch
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 3.0, entry.color);
                ui.label(egui::RichText::new(&entry.name).color(entry.color));
                if is_me {
                    ui.label(egui::RichText::new("(you)").weak());
                }
                if entry.is_host {
                    ui.label(egui::RichText::new("[host]").color(egui::Color32::GOLD));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if entry.spectating {
                        ui.label(
                            egui::RichText::new("spectating")
                                .color(egui::Color32::from_rgb(210, 180, 130)),
                        );
                    } else {
                        match entry.pick {
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
        let ready = all_players_ready(roster);
        let requested_optional_rule = optional_rule.0;
        if net.is_host {
            ui.add_enabled_ui(ready, |ui| {
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("\u{2694}  Start Battle").size(18.0),
                    ))
                    .clicked()
                {
                    let assignments = collect_assignments(roster);
                    let optional_rule = match lobby_scenario.0 {
                        omdurman_types::Scenario::Campaign => requested_optional_rule,
                        _ => None,
                    };
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Game(GameEvent::StartGame {
                            assignments,
                            scenario: lobby_scenario.0,
                            optional_rule,
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

        ui.add_space(8.0);

        // -- Preferences ---------------------------------------------------
        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Preferences")
                    .strong()
                    .color(egui::Color32::from_gray(200)),
            );
            ui.checkbox(
                &mut local.show_other_cursors,
                "Show other players' cursors",
            );
            #[cfg(target_arch = "wasm32")]
            if recorder.record.is_some()
            {
                use ron::ser::PrettyConfig;
                if ui.button("Download game record").clicked()
                    && let Some(ref record) = recorder.record
                    && let Ok(ron_str) =
                        ron::ser::to_string_pretty(record, PrettyConfig::default())
                {
                    crate::settings::download_ron_file(&ron_str);
                }
            }
        });

        // -- Sync player info if dirty ------------------------------------
        if local.take_dirty() {
            let (r, g, b) = local.color_u8();
            pending
                .outgoing_broadcast
                .push(NetMsg::Ephemeral(Ephemeral::PlayerInfo {
                    name: local.name.clone(),
                    color: [r, g, b],
                }));
        }
    }
}

/// The lobby's "Saved games" sub-tab: review the in-memory game, or (native)
/// load a finished game from `games/*/events.jsonl`. Replaces the old floating "Review
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

/// Whether the lobby is ready to start. Spectators join to watch and are
/// ignored here; among the *non-spectating* players, everyone must have chosen a
/// faction and **both** factions must be represented (so the battle has two
/// sides). Multiple players may share a faction (§1.1).
fn all_players_ready(roster: &[RosterEntry]) -> bool {
    let mut ae = false;
    let mut dervish = false;
    for entry in roster {
        if entry.spectating {
            continue; // spectators don't need a faction
        }
        match entry.pick {
            Some(Player::AngloEgyptian) => ae = true,
            Some(Player::Dervish) => dervish = true,
            None => return false, // an active player hasn't decided yet
        }
    }
    ae && dervish
}

/// Build the `(peer_id, faction)` assignments for `StartGame`.
fn collect_assignments(roster: &[RosterEntry]) -> Vec<(PeerId, Player)> {
    roster
        .iter()
        .filter_map(|e| e.pick.map(|f| (e.peer, f)))
        .collect()
}
