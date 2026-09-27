//! Pre-game lobby (§lobby).
//!
//! Once peers are connected the app enters [`AppState::Lobby`]. Each player
//! sees everyone's name + colour (and live cursors, drawn by the existing
//! cursor overlay), and picks a faction -- or chooses to **spectate** (join to
//! watch only, no faction). Picks are broadcast as live previews via
//! [`Ephemeral::FactionChoice`] / [`Ephemeral::SpectatorChoice`] and stored on
//! the peer entities ([`crate::peers::LobbyPick`] / [`crate::peers::Spectator`]).
//! Once both factions are represented among the non-spectating players, the
//! **host** can start the game, which commits the seat table as
//! [`GameEvent::StartGame`] -- recorded and replayed, so late joiners inherit it
//! through the snapshot path. Seats name each player's stable
//! [`PlayerKey`] (announced in `Ephemeral::PlayerInfo`), so a reconnecting
//! player keeps their seat. Spectators get no seat, so every action gate
//! (`crate::peers::Peers::may_act`) no-ops for them.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use bevy_matchbox::prelude::PeerId;
use omdurman_net::{Ephemeral, GameEvent, NetMsg, NetState, PlayerKey, RoomId, Seat, SeatHolder};
use omdurman_types::{BrigadeId, BrigadeNationality, DervishTribe, Player, Scenario};
use strum::IntoEnumIterator;

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

/// The local player's current lobby command-scope pick (pre-commit, §1.1
/// multi-player commands). `None` = whole faction: the player acts on every
/// unit of their side and claims no specific tribes/brigades. A scoped pick
/// gates the player to those units plus the faction's communal pool.
#[derive(Resource, Default)]
pub struct LocalCommand(pub Option<omdurman_types::CommandScope>);

/// Whether the local player has chosen to spectate (join to watch, no faction).
/// Kept separate from [`LocalFaction`] so "spectating" is distinct from
/// "undecided". A spectator is never given a seat in `StartGame`.
#[derive(Resource, Default)]
pub struct LocalSpectator(pub bool);

/// Host's optional-rule selection for a campaign game (§10.11, §10.21).
/// Independently checkable — both may be active; empty means none was
/// selected. Only meaningful for the Dervish host in a Campaign scenario.
#[derive(Resource, Default)]
pub struct LocalOptionalRule(pub Vec<omdurman_rules::OptionalRule>);

/// Insert or remove `rule` from `rules`, keeping the vec free of duplicates.
fn set_optional_rule(
    rules: &mut Vec<omdurman_rules::OptionalRule>,
    rule: omdurman_rules::OptionalRule,
    on: bool,
) {
    if on {
        if !rules.contains(&rule) {
            rules.push(rule);
        }
    } else {
        rules.retain(|r| *r != rule);
    }
}

/// Host's pre-commit AI-commander picks: factions no human has chosen that
/// the host hands to the in-game AI (Kitchener for the Anglo-Egyptian,
/// Khalifa for the Dervish — see [`crate::bot_player`]). Committed into
/// [`GameEvent::StartGame`] as AI seats; exclusive with any human pick of
/// the same faction (a human pick overrides the toggle in the UI logic).
#[derive(Resource, Default)]
pub struct LocalAiCommanders(pub Vec<Player>);

/// Bundles the lobby-specific mutable resources so [`lobby_ui`] stays under
/// Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct LobbyContext<'w, 's> {
    pub local_faction: ResMut<'w, LocalFaction>,
    pub local_spectator: ResMut<'w, LocalSpectator>,
    pub local_command: ResMut<'w, LocalCommand>,
    pub local_optional_rule: ResMut<'w, LocalOptionalRule>,
    pub remote_scenario: Res<'w, RemoteScenario>,
    pub lobby_scenario: ResMut<'w, LobbyScenario>,
    pub pending: ResMut<'w, PendingEdits>,
    pub tab: ResMut<'w, LobbyTab>,
    pub timeline: ResMut<'w, SpectatorTimeline>,
    pub recorder: Res<'w, GameRecorder>,
    pub saved_games: ResMut<'w, SavedGamesCache>,
    pub local_ai: ResMut<'w, LocalAiCommanders>,
    pub next_state: ResMut<'w, NextState<AppState>>,
    pub room: Res<'w, RoomId>,
    /// This instance's stable player key (the local roster row's seat key).
    pub local_key: Res<'w, crate::seats::LocalPlayerKey>,
    /// One row per connected peer (remote picks/names live on the peer
    /// entities; the local row is synthesized from the local resources).
    pub peers: crate::peers::RosterQuery<'w, 's>,
}

/// Both selectable factions, with display labels.
const FACTIONS: [(Player, &str); 2] = [
    (
        Player::AngloEgyptian,
        crate::ui::faction_name(Player::AngloEgyptian),
    ),
    (Player::Dervish, crate::ui::faction_name(Player::Dervish)),
];

fn faction_label(p: Player) -> &'static str {
    crate::ui::faction_name(p)
}

/// One row of the lobby roster. Remote fields (name/colour/pick/command/
/// spectating) come from the peer entity components; the local row is
/// synthesized from the local settings + pick resources.
struct RosterEntry {
    peer: PeerId,
    /// The peer's stable player key; `None` until its `PlayerInfo` arrived.
    key: Option<PlayerKey>,
    name: String,
    color: egui::Color32,
    pick: Option<Player>,
    command: Option<omdurman_types::CommandScope>,
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
    local_command: &LocalCommand,
    local_key: PlayerKey,
    peers: &crate::peers::RosterQuery<'_, '_>,
) -> Vec<RosterEntry> {
    let host = net.host_id();
    net.sorted_all()
        .iter()
        .map(|peer| {
            if net.my_id == Some(*peer) {
                RosterEntry {
                    peer: *peer,
                    key: Some(local_key),
                    name: local.name.clone(),
                    color: local.color(),
                    pick: local_faction.0,
                    command: local_command.0.clone(),
                    spectating: local_spectator.0,
                    is_host: host == Some(*peer),
                }
            } else {
                let (key, name, color, pick, command, spectating) = peers
                    .iter()
                    .find(|(key, ..)| key.0 == *peer)
                    .map(|(_, player_key, name, color, pick, command, spectating)| {
                        let name = name
                            .map(|n| n.0.clone())
                            .unwrap_or_else(|| "(connecting...)".to_string());
                        let color = color.map(|c| c.0).unwrap_or(egui::Color32::GRAY);
                        let pick = pick.and_then(|p| p.0);
                        let command = command.and_then(|c| c.0.clone());
                        (
                            player_key.map(|k| k.0),
                            name,
                            color,
                            pick,
                            command,
                            spectating,
                        )
                    })
                    .unwrap_or_else(|| {
                        (
                            None,
                            "(connecting...)".to_string(),
                            egui::Color32::GRAY,
                            None,
                            None,
                            false,
                        )
                    });
                RosterEntry {
                    peer: *peer,
                    key,
                    name,
                    color,
                    pick,
                    command,
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
    splash_maps: Res<crate::splash::SplashMaps>,
) {
    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };
    // The background map moves (`splash::map::animate_splash_maps`).
    if splash_maps.is_animating() {
        egui_ctx.request_repaint();
    }

    let roster = build_roster(
        &net,
        &local,
        &ctx.local_faction,
        &ctx.local_spectator,
        &ctx.local_command,
        ctx.local_key.0,
        &ctx.peers,
    );

    let mut __ui = egui::Ui::new(
        egui_ctx.clone(),
        egui::Id::new("lobby"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(egui_ctx.viewport_rect()),
    );
    // Click-sensed full-rect blocker, registered *before* the content so the
    // lobby's own widgets stay above it in egui's hit-test (a blocker drawn
    // last sits on top of every button and swallows all their clicks and
    // hovers). It makes `egui_wants_pointer_input` true over blank panel
    // areas, which is what gates map input. The CentralPanel covers the whole
    // root Ui (no other panels in it), so the blocker rect is the viewport.
    omdurman_board_ui::panels::register_panel_blocker(
        &mut __ui,
        "lobby_panel",
        egui_ctx.viewport_rect(),
    );
    // The period-map background (drawn first, so the lobby sits on it), with
    // the lobby's UI in a floating panel at its centre.
    let screen = egui_ctx.viewport_rect();
    let column_w = lobby_column_width(screen.width());
    let panel = crate::splash::lobby_panel_rect(screen, column_w);
    crate::splash::paint_lobby_backdrop(__ui.painter(), screen, panel, &splash_maps);
    __ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(crate::splash::lobby_panel_content(panel))
            .layout(egui::Layout::top_down(egui::Align::Center)),
        |ui| {
            ui.vertical_centered(|ui| {
                ui.set_max_width(column_w);
                ui.heading(
                    egui::RichText::new("REMEMBER GORDON! -- Lobby")
                        .size(26.0)
                        .color(crate::ui::palette::TEXT_STRONG),
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
                    // The setup tab outgrows short windows: scroll it (the
                    // saved-games tab has its own list scroll area).
                    LobbyTab::Setup => {
                        egui::ScrollArea::vertical()
                            .id_salt("lobby_setup_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                setup_tab(
                                    ui,
                                    &net,
                                    &mut local,
                                    &roster,
                                    LocalFactionPick {
                                        local_faction: &mut ctx.local_faction,
                                        local_spectator: &mut ctx.local_spectator,
                                        local_command: &mut ctx.local_command,
                                    },
                                    LobbySetupChoices {
                                        remote_scenario: &ctx.remote_scenario,
                                        lobby_scenario: &mut ctx.lobby_scenario,
                                        optional_rule: &mut ctx.local_optional_rule,
                                        local_ai: &mut ctx.local_ai,
                                    },
                                    SessionControls {
                                        pending: &mut ctx.pending,
                                        commands: &mut commands,
                                        room: &ctx.room,
                                        editing_session: &mut editing_session,
                                        local_key: ctx.local_key.0,
                                    },
                                    &ctx.recorder,
                                );
                            });
                    }
                    LobbyTab::SavedGames => saved_games_tab(
                        ui,
                        &ctx.recorder,
                        &mut ctx.saved_games,
                        &mut ctx.timeline,
                        &mut ctx.next_state,
                    ),
                }
            });
        },
    );
}

/// The lobby's centred column width for `avail` points: ~55% of the width,
/// clamped so it stays readable on a small window and doesn't sprawl on a
/// wide one. The 460 px floor yields to a narrower window (small / WASM
/// viewports) instead of overflowing it.
fn lobby_column_width(avail: f32) -> f32 {
    (avail * 0.55).clamp(460.0_f32.min(avail), 900.0)
}

/// Mutable session-level state for the lobby "Setup" tab: the pending-edits
/// queue, command buffer, room id, and the in-progress session-id text. Bundled
/// so [`setup_tab`] stays under clippy's argument limit.
struct SessionControls<'a, 'b, 'c> {
    pending: &'a mut PendingEdits,
    commands: &'a mut Commands<'b, 'c>,
    room: &'a RoomId,
    editing_session: &'a mut String,
    local_key: PlayerKey,
}

/// Bundle of the local faction + spectator + command picks so [`setup_tab`]
/// stays under clippy's argument limit.
struct LocalFactionPick<'a> {
    local_faction: &'a mut LocalFaction,
    local_spectator: &'a mut LocalSpectator,
    local_command: &'a mut LocalCommand,
}

/// Bundles the host-broadcast scenario preview, the host's scenario pick, the
/// optional rule, and the AI-commander toggles so [`setup_tab`] stays under
/// clippy's argument limit.
struct LobbySetupChoices<'a> {
    remote_scenario: &'a RemoteScenario,
    lobby_scenario: &'a mut LobbyScenario,
    optional_rule: &'a mut LocalOptionalRule,
    local_ai: &'a mut LocalAiCommanders,
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
        local_command,
    } = faction_pick;
    let LobbySetupChoices {
        remote_scenario,
        lobby_scenario,
        optional_rule,
        local_ai,
    } = lobby;
    let SessionControls {
        pending,
        commands,
        room,
        editing_session,
        local_key,
    } = session;
    ui.label(
        egui::RichText::new("Choose your faction, then the host starts the battle.")
            .color(crate::ui::palette::TEXT_MUTED),
    );
    ui.add_space(16.0);

    {
        // -- Session (room ID + Connect) ----------------------------------
        ui.group(|ui| {
            ui.label(crate::ui::text::subheading("Session"));
            ui.horizontal(|ui| {
                // No smarts: the field holds exactly what was typed. The
                // current room id is a placeholder hint, never a fallback.
                ui.add_sized(
                    egui::vec2(200.0, 22.0),
                    egui::TextEdit::singleline(editing_session).hint_text(room.as_str()),
                );
                // One button: hosting and joining are the same act — the
                // first peer in a room is elected host. An empty field would
                // be a silent no-op (the reconnect ignores an empty room), so
                // the button waits for a room id.
                let target = editing_session.trim().to_string();
                let connect = ui
                    .add_enabled(!target.is_empty(), egui::Button::new("Connect to room"))
                    .on_hover_text(
                        "Join the typed room, creating it if nobody is there yet \
                         (the first player in a room hosts).",
                    )
                    .on_disabled_hover_text("Type a room ID first.");
                if connect.clicked() {
                    commands.insert_resource(ReconnectRoom(target));
                }
            });
            ui.label(
                egui::RichText::new(format!(
                    "Current room: {}. Share a room ID with the other players.",
                    room.as_str()
                ))
                .weak()
                .size(11.0),
            );
        });

        ui.add_space(8.0);

        // -- Player identity (name + color) --------------------------------
        ui.group(|ui| {
            ui.label(crate::ui::text::subheading("Your identity"));
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
                        // A cleared/switched faction invalidates the command
                        // scope (§1.1): tribes/brigades belong to one side.
                        if local_command.0.is_some() {
                            local_command.0 = None;
                            pending
                                .outgoing_broadcast
                                .push(NetMsg::Ephemeral(Ephemeral::CommandChoice(None)));
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
                    if local_spectator.0 && local_command.0.is_some() {
                        local_command.0 = None;
                        pending
                            .outgoing_broadcast
                            .push(NetMsg::Ephemeral(Ephemeral::CommandChoice(None)));
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
            ui.label(crate::ui::text::subheading("Scenario"));
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
                ui.label(crate::ui::text::subheading("Optional Rules (§10)"));
                // §10.11 and §10.21 are independent: the engine gates each
                // placement on its own flag, so both may be active. Nothing
                // ticked = no optional rule.
                let opt_rule = &mut *optional_rule;
                let mut mines = opt_rule
                    .0
                    .contains(&omdurman_rules::OptionalRule::RiverMines);
                if ui.checkbox(&mut mines, "River mines (§10.11)").changed() {
                    set_optional_rule(
                        &mut opt_rule.0,
                        omdurman_rules::OptionalRule::RiverMines,
                        mines,
                    );
                }
                let mut chain = opt_rule
                    .0
                    .contains(&omdurman_rules::OptionalRule::RiverChain);
                if ui.checkbox(&mut chain, "River chain (§10.21)").changed() {
                    set_optional_rule(
                        &mut opt_rule.0,
                        omdurman_rules::OptionalRule::RiverChain,
                        chain,
                    );
                }
            });
        }

        ui.add_space(8.0);
        ui.label(crate::ui::text::subheading("Players"));

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
                    // Command scope (§1.1): which slice of the picked faction
                    // this player commands. The local row gets an editor; the
                    // remote rows show their live pick.
                    if !entry.spectating && entry.pick.is_some() {
                        if is_me {
                            command_scope_widget(
                                ui,
                                local_faction.0,
                                local_command,
                                roster,
                                entry,
                                pending,
                            );
                        } else {
                            let text = match &entry.command {
                                None => "whole faction".to_string(),
                                Some(scope) => scope.to_string(),
                            };
                            ui.label(egui::RichText::new(text).weak().color(egui::Color32::GRAY));
                        }
                    }
                    if entry.spectating {
                        ui.label(
                            egui::RichText::new("spectating").color(crate::ui::palette::BRASS),
                        );
                    } else {
                        match entry.pick {
                            Some(f) => ui.label(
                                egui::RichText::new(faction_label(f))
                                    .color(crate::ui::palette::GOLD),
                            ),
                            None => ui.label(egui::RichText::new("undecided").weak()),
                        };
                    }
                });
            });
        }

        // -- AI commanders (host-only): hand unclaimed factions to the
        //    in-game AI. Kitchener commands the Anglo-Egyptian, Khalifa the
        //    Dervish (`crate::bot_player`); the host plays their turns and
        //    every peer sees ordinary sequenced effects.
        let human_picked = |faction: Player| {
            roster
                .iter()
                .any(|e| !e.spectating && e.pick == Some(faction))
        };
        let effective_ai: Vec<Player> = local_ai
            .0
            .iter()
            .copied()
            .filter(|f| !human_picked(*f))
            .collect();

        if net.is_host {
            ui.add_space(8.0);
            ui.group(|ui| {
                ui.label(crate::ui::text::subheading("AI Commanders"));
                for (faction, commander) in [
                    (
                        Player::AngloEgyptian,
                        crate::bot_player::commander_name_for(Player::AngloEgyptian),
                    ),
                    (
                        Player::Dervish,
                        crate::bot_player::commander_name_for(Player::Dervish),
                    ),
                ] {
                    let claimed = human_picked(faction);
                    let mut toggled = effective_ai.contains(&faction);
                    ui.add_enabled_ui(!claimed, |ui| {
                        let label = format!("AI \u{b7} {commander} ({})", faction_label(faction));
                        if ui.checkbox(&mut toggled, label).changed() {
                            if toggled {
                                if !local_ai.0.contains(&faction) {
                                    local_ai.0.push(faction);
                                }
                            } else {
                                local_ai.0.retain(|f| *f != faction);
                            }
                        }
                    });
                    if claimed {
                        ui.label(
                            egui::RichText::new("(claimed by a player)")
                                .weak()
                                .size(11.0),
                        );
                    }
                }
                ui.label(
                    egui::RichText::new(
                        "The host plays the AI factions' turns; everyone watches the \
                         same sequenced actions.",
                    )
                    .weak()
                    .size(11.0),
                );
            });
        }

        // AI commander rows in the roster (every peer sees the host's
        // committed list via StartGame once started; in the lobby only the
        // host previews its own toggles).
        for faction in &effective_ai {
            ui.horizontal(|ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, 3.0, crate::ui::palette::AI_SWATCH);
                ui.label(
                    egui::RichText::new(format!(
                        "AI \u{b7} {}",
                        crate::bot_player::commander_name_for(*faction)
                    ))
                    .color(crate::ui::palette::INFO),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(faction_label(*faction))
                            .color(crate::ui::palette::GOLD),
                    );
                });
            });
        }

        ui.add_space(16.0);

        // -- Host start control ----------------------------------------
        let ready = all_players_ready_with_ai(roster, &effective_ai);
        let requested_optional_rules = optional_rule.0.clone();
        if net.is_host {
            ui.add_enabled_ui(ready, |ui| {
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("\u{2694}  Start Battle").size(18.0),
                    ))
                    .clicked()
                {
                    let optional_rules = match lobby_scenario.0 {
                        omdurman_types::Scenario::Campaign => requested_optional_rules,
                        _ => Vec::new(),
                    };
                    pending.submit_game(GameEvent::StartGame {
                        seats: collect_seats(roster, &effective_ai),
                        scenario: lobby_scenario.0,
                        optional_rules,
                    });
                }
            });
            if !ready {
                ui.label(
                    egui::RichText::new(
                        "Both factions must be chosen (or handed to an AI commander), \
                         and every player connected, before starting.",
                    )
                    .weak(),
                );
            }
        } else {
            ui.label(
                egui::RichText::new("Waiting for the host to start...")
                    .color(crate::ui::palette::TEXT_MUTED),
            );
        }

        ui.add_space(8.0);

        // -- Preferences ---------------------------------------------------
        ui.group(|ui| {
            ui.label(crate::ui::text::subheading("Preferences"));
            ui.checkbox(&mut local.show_other_cursors, "Show other players' cursors");
            #[cfg(target_arch = "wasm32")]
            if recorder.record.is_some() {
                use ron::ser::PrettyConfig;
                if ui.button("Download game record").clicked()
                    && let Some(ref record) = recorder.record
                    && let Ok(ron_str) = ron::ser::to_string_pretty(record, PrettyConfig::default())
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
                    key: local_key,
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
        ui.label(egui::RichText::new("Saved games").color(crate::ui::palette::TEXT));
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
        .color(crate::ui::palette::TEXT_MUTED)
}

/// Whether the lobby is ready to start. Spectators join to watch and are
/// ignored here; among the *non-spectating* players, everyone must have chosen a
/// faction, and both factions must be represented — by a human pick or by an
/// AI-commander commitment (the host's unclaimed-faction toggles). Multiple
/// players may share a faction (§1.1).
fn all_players_ready_with_ai(roster: &[RosterEntry], ai: &[Player]) -> bool {
    let mut ae = false;
    let mut dervish = false;
    for entry in roster {
        if entry.spectating {
            continue; // spectators don't need a faction
        }
        if entry.key.is_none() {
            return false; // still connecting: no key to seat yet
        }
        match entry.pick {
            Some(Player::AngloEgyptian) => ae = true,
            Some(Player::Dervish) => dervish = true,
            None => return false, // an active player hasn't decided yet
        }
    }
    for faction in ai {
        match faction {
            Player::AngloEgyptian => ae = true,
            Player::Dervish => dervish = true,
        }
    }
    ae && dervish
}

/// Build the seat table for `StartGame`: one seat per non-spectating
/// roster row with a faction pick, keyed by the row's stable player key and
/// carrying its §1.1 command scope (`None` = whole faction), then one AI
/// seat per AI-commanded faction.
fn collect_seats(roster: &[RosterEntry], ai: &[Player]) -> Vec<Seat> {
    roster
        .iter()
        .filter(|e| !e.spectating)
        .filter_map(|e| {
            Some(Seat {
                faction: e.pick?,
                scope: e.command.clone(),
                holder: SeatHolder::Human(e.key?),
            })
        })
        .chain(ai.iter().map(|&faction| Seat {
            faction,
            scope: None,
            holder: SeatHolder::Ai,
        }))
        .collect()
}

/// The per-row command-scope editor (§1.1): a combobox offering
/// "whole faction", "Army" (communal units only), and a checkbox list of the
/// picked faction's tribes (Dervish) or brigades (Anglo-Egyptian). A
/// tribe/brigade another member already claims is shown disabled.
#[allow(clippy::too_many_arguments)]
fn command_scope_widget(
    ui: &mut egui::Ui,
    faction: Option<Player>,
    local_command: &mut LocalCommand,
    roster: &[RosterEntry],
    me: &RosterEntry,
    pending: &mut PendingEdits,
) {
    let selected = match &local_command.0 {
        None => "whole faction".to_string(),
        Some(scope) => scope.to_string(),
    };
    let mut changed = false;
    egui::ComboBox::from_id_salt(("command-scope", me.peer))
        .width(200.0)
        .selected_text(egui::RichText::new(&selected).weak())
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(local_command.0.is_none(), "Whole faction")
                .clicked()
            {
                local_command.0 = None;
                changed = true;
            }
            let Some(faction) = faction else {
                return; // no faction picked: no scope to pick either
            };
            if ui
                .selectable_label(
                    local_command.0.as_ref().is_some_and(|s| s.is_army()),
                    "Army (communal units only)",
                )
                .clicked()
            {
                local_command.0 = Some(omdurman_types::CommandScope::Army);
                changed = true;
            }
            // A unit someone else claimed is not selectable (one commander
            // per tribe/brigade).
            let claimed_by_other = |tribe: DervishTribe| {
                roster.iter().any(|e| {
                    e.peer != me.peer && e.command.as_ref().is_some_and(|s| s.claims_tribe(tribe))
                })
            };
            let brigade_claimed_by_other = |brigade: BrigadeId| {
                roster.iter().any(|e| {
                    e.peer != me.peer
                        && e.command
                            .as_ref()
                            .is_some_and(|s| s.claims_brigade(brigade))
                })
            };
            match faction {
                Player::Dervish => {
                    ui.separator();
                    ui.label(egui::RichText::new("Tribes").weak());
                    for tribe in DervishTribe::iter() {
                        let current = local_command
                            .0
                            .as_ref()
                            .is_some_and(|s| s.claims_tribe(tribe));
                        let claimed = claimed_by_other(tribe);
                        let mut checked = current;
                        if ui
                            .add_enabled(
                                !claimed,
                                egui::Checkbox::new(&mut checked, tribe.to_string()),
                            )
                            .clicked()
                        {
                            toggle_scope_tribe(local_command, tribe, checked);
                            changed = true;
                        }
                    }
                }
                Player::AngloEgyptian => {
                    ui.separator();
                    ui.label(egui::RichText::new("Brigades").weak());
                    for brigade in BrigadeId::ALL {
                        // Friendlies (§6.52) stay communal: the brigade list
                        // covers the twelve integrating brigades only.
                        if brigade.nationality == BrigadeNationality::Friendlies {
                            continue;
                        }
                        let current = local_command
                            .0
                            .as_ref()
                            .is_some_and(|s| s.claims_brigade(brigade));
                        let claimed = brigade_claimed_by_other(brigade);
                        let mut checked = current;
                        if ui
                            .add_enabled(
                                !claimed,
                                egui::Checkbox::new(&mut checked, brigade.to_string()),
                            )
                            .clicked()
                        {
                            toggle_scope_brigade(local_command, brigade, checked);
                            changed = true;
                        }
                    }
                }
            }
        });
    if changed {
        pending
            .outgoing_broadcast
            .push(NetMsg::Ephemeral(Ephemeral::CommandChoice(
                local_command.0.clone(),
            )));
    }
}

/// Add/remove `tribe` from the local command scope. Checking the first tribe
/// converts `None`/`Army` into a tribe scope; unchecking the last reverts to
/// [`omdurman_types::CommandScope::Army`].
fn toggle_scope_tribe(local_command: &mut LocalCommand, tribe: DervishTribe, add: bool) {
    let mut tribes: std::collections::BTreeSet<DervishTribe> = match &local_command.0 {
        Some(omdurman_types::CommandScope::Tribes(t)) => t.clone(),
        _ => Default::default(),
    };
    if add {
        tribes.insert(tribe);
    } else {
        tribes.remove(&tribe);
    }
    local_command.0 = if tribes.is_empty() {
        Some(omdurman_types::CommandScope::Army)
    } else {
        Some(omdurman_types::CommandScope::Tribes(tribes))
    };
}

/// As [`toggle_scope_tribe`], for a brigade scope.
fn toggle_scope_brigade(local_command: &mut LocalCommand, brigade: BrigadeId, add: bool) {
    let mut brigades: std::collections::BTreeSet<BrigadeId> = match &local_command.0 {
        Some(omdurman_types::CommandScope::Brigades(b)) => b.clone(),
        _ => Default::default(),
    };
    if add {
        brigades.insert(brigade);
    } else {
        brigades.remove(&brigade);
    }
    local_command.0 = if brigades.is_empty() {
        Some(omdurman_types::CommandScope::Army)
    } else {
        Some(omdurman_types::CommandScope::Brigades(brigades))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(n: u8, key: Option<u64>, pick: Option<Player>, spectating: bool) -> RosterEntry {
        RosterEntry {
            peer: PeerId(uuid::Uuid::from_u128(u128::from(n))),
            key: key.map(PlayerKey),
            name: format!("p{n}"),
            color: egui::Color32::GRAY,
            pick,
            command: None,
            spectating,
            is_host: n == 0,
        }
    }

    #[test]
    fn collect_seats_keys_humans_and_appends_ai() {
        let mut dervish = entry(1, Some(11), Some(Player::Dervish), false);
        dervish.command = Some(omdurman_types::CommandScope::Army);
        let roster = vec![
            entry(0, Some(10), Some(Player::AngloEgyptian), false),
            dervish,
            entry(2, Some(12), None, true),
        ];
        let seats = collect_seats(&roster, &[Player::Dervish]);
        assert_eq!(
            seats,
            vec![
                Seat {
                    faction: Player::AngloEgyptian,
                    scope: None,
                    holder: SeatHolder::Human(PlayerKey(10)),
                },
                Seat {
                    faction: Player::Dervish,
                    scope: Some(omdurman_types::CommandScope::Army),
                    holder: SeatHolder::Human(PlayerKey(11)),
                },
                Seat {
                    faction: Player::Dervish,
                    scope: None,
                    holder: SeatHolder::Ai,
                },
            ]
        );
    }

    #[test]
    fn start_waits_for_every_player_key() {
        let roster = vec![
            entry(0, Some(10), Some(Player::AngloEgyptian), false),
            entry(1, None, Some(Player::Dervish), false),
        ];
        assert!(!all_players_ready_with_ai(&roster, &[]));
        let roster = vec![
            entry(0, Some(10), Some(Player::AngloEgyptian), false),
            entry(1, Some(11), Some(Player::Dervish), false),
            entry(2, None, None, true),
        ];
        assert!(all_players_ready_with_ai(&roster, &[]));
    }
}
