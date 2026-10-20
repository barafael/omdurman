//! Dervish desertion turn overlay + selection UI (rulebook §8.2).
//!
//! On the first night turn of the Campaign scenario the Dervish player must
//! roll one die and remove 1½ × roll units, rounded up. The Khalifa, gunboats,
//! artillery, and forts are exempt.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::GameEvent;

use crate::{GameStateResource, PendingEdits};
use omdurman_rules::DieRoll;
use omdurman_rules::effects::{GameEffect, desertion_count};
use omdurman_types::Player;

/// Present while the desertion turn is open for the local Dervish seat: the
/// player's selection and the submission in flight. The die itself is not
/// kept here: it is rolled once by its own effect (`RollDesertion`) and read
/// off the engine state (`GameState::desertion_roll`), so it survives a
/// pause, a reload or a rejoin. Removed once the engine records the
/// desertion (`dervish_deserted`) or the turn otherwise ends.
#[derive(Resource, Default)]
pub(crate) struct DesertionTurn {
    /// Which Dervish units the player has selected so far.
    pub selected: Vec<omdurman_rules::UnitId>,
    /// The submission uid of the chosen desertion while its echo is
    /// awaited: the panel then waits instead of offering a second
    /// submission. Cleared again when the echo comes back refused.
    pub submitted: Option<u64>,
}

/// The die and the number of deserters it demands, once rolled (§8.2).
fn rolled_demand(gs: &omdurman_rules::effects::GameState) -> Option<(DieRoll, usize)> {
    gs.desertion_roll
        .map(|roll| (roll, gs.desertion_demand(roll)))
}

/// Whether a desertion die roll of ours is on its way to the engine.
fn roll_in_flight(pending: &PendingEdits) -> bool {
    pending
        .unconfirmed
        .iter()
        .any(|(_, e)| matches!(e, GameEvent::Effect(GameEffect::RollDesertion { .. })))
}

/// Open the desertion turn for the local Dervish seat when the conditions
/// are met, and close it once they no longer hold (the echo of the submitted
/// desertion sets `dervish_deserted`, or the phase moved on).
///
/// The die is rolled once per game: while the engine holds no roll and none
/// of ours is in flight, the local Dervish seat submits `RollDesertion` and
/// the panel shows the die from its echo. Only the seat that may act for the
/// Dervish rolls: every other peer drawing from its `GameRng` here would
/// skew its stream for nothing.
pub(crate) fn detect_desertion_turn(
    game_state: Res<GameStateResource>,
    mut commands: Commands,
    existing: Option<Res<DesertionTurn>>,
    mut game_rng: ResMut<crate::GameRng>,
    mut pending: ResMut<PendingEdits>,
    peers: crate::peers::Peers,
    seats: Res<crate::seats::Seats>,
) {
    // An AI-commanded Dervish deserts through the bot driver instead.
    let local_dervish = peers.may_act(Player::Dervish)
        && !crate::seats::ai_factions(&seats.0).contains(&Player::Dervish);
    if !game_state.0.desertion_due() || !local_dervish {
        if existing.is_some() {
            commands.remove_resource::<DesertionTurn>();
        }
        return;
    }
    // §8.2 "the roll of one die": the game's die is the ten-sided one
    // (§2.4 parts inventory; every other roll in the rules is d10), matching
    // the bot driver and the engine's `desertion_count` domain (1..=10).
    if game_state.0.desertion_roll.is_none() && !roll_in_flight(&pending) {
        pending.submit_game(GameEvent::Effect(GameEffect::RollDesertion {
            roll: game_rng.roll_d10(),
        }));
    }
    if existing.is_none() {
        commands.insert_resource(DesertionTurn::default());
    }
}

/// Render the desertion overlay panel.
pub(crate) fn desertion_panel_ui(
    mut contexts: EguiContexts,
    desertion: Option<ResMut<DesertionTurn>>,
    game_state: Res<GameStateResource>,
    placed_units: Query<(Entity, &super::picker::PlacedUnit)>,
    mut submit: crate::submit::CheckedSubmit,
    layout: Res<crate::ScreenLayout>,
    peers: crate::peers::Peers,
) {
    // The desertion choice is the Dervish player's: other bound seats don't
    // even see the panel (§8.2).
    if !peers.may_act(omdurman_types::Player::Dervish) {
        return;
    }
    let Some(mut desertion) = desertion else {
        return;
    };
    let gs = &game_state;
    if !gs.0.desertion_due() {
        // Applied (or the turn moved on): `detect_desertion_turn` closes it.
        return;
    }
    // The echo of our submission came back while the desertion is still
    // due: the engine refused it. Unlock the panel to choose again.
    if unlock_refused(&mut desertion, &submit.pending)
        && let Some(dispatches) = submit.dispatches.as_deref_mut()
    {
        dispatches.refuse(
            crate::submit::REFUSED_HEADER,
            "The desertion was refused; choose the deserters again (§8.2).".to_string(),
        );
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let Some((die_roll, count)) = rolled_demand(&gs.0) else {
        // The die is on its way to the engine: shown from its echo.
        crate::ui::anchored_card(
            ctx,
            egui::Id::new("desertion_panel"),
            egui::Align2::RIGHT_CENTER,
            egui::vec2(-(layout.right_inset + 10.0), 0.0),
            egui::Frame::popup(&ctx.style_of(ctx.theme())),
            |ui| {
                crate::rulebook::refs_rich(ui, "Dervish Desertion (§8.2)", 18.0, |t| t.strong());
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Rolling the desertion die\u{2026}").size(13.0));
            },
        );
        return;
    };

    crate::ui::anchored_card(
        ctx,
        egui::Id::new("desertion_panel"),
        egui::Align2::RIGHT_CENTER,
        // Clear of the charts sheet / peek tab (see `right_inset`).
        egui::vec2(-(layout.right_inset + 10.0), 0.0),
        egui::Frame::popup(&ctx.style_of(ctx.theme())),
        |ui| {
            crate::rulebook::refs_rich(ui, "Dervish Desertion (§8.2)", 18.0, |t| t.strong());
            ui.add_space(4.0);

            ui.label(
                egui::RichText::new(format!(
                    "Die roll: {} → remove {} unit{}",
                    die_roll.value(),
                    count,
                    if count == 1 { "" } else { "s" }
                ))
                .size(13.0)
                .strong(),
            );
            ui.add_space(4.0);

            // Roll reference table
            ui.collapsing("Roll table", |ui| {
                egui::Grid::new("desertion_table")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Die").strong());
                        ui.label(egui::RichText::new("Remove").strong());
                        ui.end_row();
                        for val in 1..=10u16 {
                            let dr = DieRoll::try_from(val).unwrap();
                            let n = desertion_count(dr);
                            ui.label(format!("{val}"));
                            ui.label(format!("{n}"));
                            ui.end_row();
                        }
                    });
            });
            ui.add_space(4.0);

            // The eligible units first, and always: the status line below
            // changes length (and wraps) as units are picked, so above the
            // list it shifted every row under the pointer mid-selection. The
            // list stays open when the count is reached, so a pick can still
            // be taken back.
            ui.collapsing("Available units", |ui| {
                for (_entity, placed) in placed_units.iter() {
                    let Some(unit_id) = placed.unit_id else {
                        continue;
                    };
                    if let Some(unit) = gs.0.find_unit(unit_id) {
                        if unit.profile.identity.owner() != Player::Dervish {
                            continue;
                        }
                        let label = unit.profile.identity.short_label();
                        // The Khalifa, gunboats, artillery and forts may not
                        // desert (§8.2): listed, greyed out, saying why.
                        if unit.profile.identity.is_desertion_exempt() {
                            ui.add_enabled(false, egui::Button::selectable(false, label))
                                .on_disabled_hover_text(
                                    "Exempt (§8.2): the Khalifa, gunboats, artillery and forts \
                                     never desert.",
                                );
                            continue;
                        }
                        let is_selected = desertion.selected.contains(&unit.id);
                        if ui.selectable_label(is_selected, label).clicked() {
                            if is_selected {
                                desertion.selected.retain(|id| id != &unit.id);
                            } else if desertion.selected.len() < count {
                                desertion.selected.push(unit.id);
                            }
                        }
                    }
                }
            });
            ui.label(crate::ui::text::note(
                "Deserters leave the game: they score no victory points for either side (§8.2).",
            ));
            ui.add_space(4.0);
            let (status, done) = desertion_status(count, desertion.selected.len());
            ui.label(
                egui::RichText::new(status)
                    .color(if done {
                        crate::ui::palette::INK_DONE
                    } else {
                        crate::ui::palette::INK_WARN
                    })
                    .size(12.0),
            );

            ui.add_space(8.0);

            if desertion.submitted.is_some() {
                ui.label(
                    egui::RichText::new("Desertion submitted — awaiting confirmation…").size(12.0),
                );
                return;
            }

            // Confirm button
            let ready = desertion.selected.len() == count;
            if ui
                .add_enabled(
                    ready,
                    egui::Button::new(egui::RichText::new("Confirm desertion").size(13.0).strong()),
                )
                .clicked()
            {
                let effect = GameEffect::DervishDesertion {
                    roll: die_roll,
                    deserters: desertion.selected.clone(),
                };
                // Dry-run first: a refusal explains itself and keeps the
                // panel open. Keep the resource until the echo closes the
                // desertion turn (`detect_desertion_turn`).
                if submit.submit(&gs.0, GameEvent::Effect(effect)) {
                    desertion.submitted = submit.pending.unconfirmed.back().map(|(uid, _)| *uid);
                }
            }
        },
    );
}

/// Release the panel's lock when our submitted desertion is no longer
/// awaited: its echo came back, and (the caller checks) the desertion is
/// still due, so the engine refused it. Returns whether it unlocked.
fn unlock_refused(desertion: &mut DesertionTurn, pending: &PendingEdits) -> bool {
    match desertion.submitted {
        Some(uid) if !pending.unconfirmed.iter().any(|(u, _)| *u == uid) => {
            desertion.submitted = None;
            true
        }
        _ => false,
    }
}

/// The panel's status line for `selected` of `count` deserters, and whether
/// the selection is complete (§8.2).
fn desertion_status(count: usize, selected: usize) -> (String, bool) {
    let remaining = count.saturating_sub(selected);
    if remaining == 0 {
        ("All units selected.".to_string(), true)
    } else {
        (
            format!(
                "Select {remaining} more Dervish unit{} to desert.",
                if remaining == 1 { "" } else { "s" }
            ),
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::desertion_status;

    #[test]
    fn the_status_counts_down_to_complete() {
        assert_eq!(
            desertion_status(12, 0),
            ("Select 12 more Dervish units to desert.".to_string(), false)
        );
        assert_eq!(
            desertion_status(12, 11),
            ("Select 1 more Dervish unit to desert.".to_string(), false)
        );
        assert_eq!(
            desertion_status(12, 12),
            ("All units selected.".to_string(), true)
        );
    }
}

#[cfg(test)]
mod roll_tests {
    use super::*;
    use omdurman_rules::effects::GameState;
    use omdurman_types::{DayNight, Scenario};

    fn first_night() -> GameState {
        let mut gs = GameState::new(Scenario::Campaign);
        gs.current_turn = omdurman_rules::GameTurnIndex::new(9);
        gs.day_night = DayNight::Night;
        gs.active_player = Player::Dervish;
        gs.phase = omdurman_rules::Phase::Movement;
        assert!(gs.desertion_due());
        gs
    }

    fn app() -> App {
        let mut app = App::new();
        app.insert_resource(GameStateResource(first_night()))
            .insert_resource(PendingEdits::default())
            .insert_resource(crate::GameRng::from_seed(3))
            .insert_resource(crate::seats::Seats::default())
            .insert_resource(crate::seats::SeatPresence::default())
            .insert_resource(crate::seats::LocalPlayerKey(omdurman_net::PlayerKey(1)))
            .add_systems(Update, detect_desertion_turn);
        app
    }

    fn rolls_in_flight(app: &App) -> usize {
        app.world()
            .resource::<PendingEdits>()
            .unconfirmed
            .iter()
            .filter(|(_, e)| matches!(e, GameEvent::Effect(GameEffect::RollDesertion { .. })))
            .count()
    }

    /// The §8.2 die is rolled once, by its own recorded effect: neither a
    /// second frame before the echo nor a re-opened panel afterwards (a
    /// pause, a reload) rolls it again -- the die is the engine's.
    #[test]
    fn the_desertion_die_is_rolled_once_and_kept() {
        let mut app = app();
        app.update();
        app.update();
        assert_eq!(rolls_in_flight(&app), 1, "one roll in flight");
        assert!(app.world().contains_resource::<DesertionTurn>());

        // The echo: the engine keeps the die; the submission is confirmed.
        let (uid, event) = app.world().resource::<PendingEdits>().unconfirmed[0].clone();
        let GameEvent::Effect(effect) = event else {
            unreachable!()
        };
        omdurman_rules::effects::apply_effect(
            &mut app.world_mut().resource_mut::<GameStateResource>().0,
            &effect,
        )
        .unwrap();
        app.world_mut().resource_mut::<PendingEdits>().confirm(uid);
        let rolled = app.world().resource::<GameStateResource>().0.desertion_roll;
        assert!(rolled.is_some());

        // The panel closes (a pause) and opens again: no new die.
        app.world_mut().remove_resource::<DesertionTurn>();
        app.update();
        app.update();
        assert_eq!(rolls_in_flight(&app), 0);
        assert_eq!(
            app.world().resource::<GameStateResource>().0.desertion_roll,
            rolled
        );
        assert!(rolled_demand(&app.world().resource::<GameStateResource>().0).is_some());
    }

    /// A refused desertion echo (no longer awaited, desertion still due)
    /// unlocks the panel; one still in flight keeps it locked.
    #[test]
    fn a_refused_desertion_unlocks_the_panel() {
        let mut pending = PendingEdits::default();
        let uid = pending.submit_game(GameEvent::Effect(GameEffect::DervishDesertion {
            roll: DieRoll::One,
            deserters: vec![],
        }));
        let mut turn = DesertionTurn {
            selected: vec![],
            submitted: Some(uid),
        };
        assert!(!unlock_refused(&mut turn, &pending));
        assert_eq!(turn.submitted, Some(uid));
        pending.confirm(uid);
        assert!(unlock_refused(&mut turn, &pending));
        assert_eq!(turn.submitted, None);
    }
}
