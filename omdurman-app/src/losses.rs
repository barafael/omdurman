//! The owner's choice of losses (rulebook §7.7, §CombatResults: "# = That
//! many units in the target hex are eliminated" -- the owner picks which).
//!
//! While a combat waits on this seat's choice the engine accepts nothing
//! else, so the panel stands until the choice is made: the units the losses
//! may be taken from, the count, and a confirm button submitting
//! `TakeLosses`. An AI side chooses through the bot driver instead.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::GameEvent;
use omdurman_rules::UnitId;
use omdurman_rules::effects::{CombatTail, GameEffect, LossChoice};

use crate::GameStateResource;

/// The panel's selection, for the choice it was made for.
#[derive(Default)]
pub(crate) struct LossPanel {
    /// The candidates of the choice the selection belongs to: a new choice
    /// starts afresh.
    choice: Option<Vec<UnitId>>,
    selected: Vec<UnitId>,
    /// The submission uid while its echo is awaited.
    submitted: Option<u64>,
}

/// The rule the losses are taken under, for the panel's title and the
/// slip: a fire result's "# = that many units in the target hex are
/// eliminated" (§6.22 and the Combat Results Table), a melee's (§7.7), or
/// the one unit beside a breached wall (§6.63) or a demolition (§6.53).
pub(crate) fn loss_rule(tail: &CombatTail) -> &'static str {
    match tail {
        CombatTail::Fire { .. } => "§6.22, Combat Results Table",
        CombatTail::Melee { .. } => "§7.7",
        CombatTail::Eliminate {
            cause: omdurman_rules::effects::ElimCause::Demolition,
            ..
        } => "§6.53",
        CombatTail::Eliminate { .. } => "§6.63",
    }
}

/// The rule behind the choice of losses a combat waits on (see
/// [`loss_rule`]), for the rail and the top bar; §7.7 when none is open.
pub(crate) fn pending_loss_rule(gs: &omdurman_rules::effects::GameState) -> &'static str {
    gs.pending_losses
        .as_ref()
        .map_or("§7.7", |pending| loss_rule(&pending.tail))
}

/// What the combat waiting on the choice is, for the panel's heading: who
/// struck and with what roll, so the owner is not asked to pick losses
/// before being told what hit him.
pub(crate) fn cause(tail: &CombatTail, gs: &omdurman_rules::effects::GameState) -> String {
    let names = |ids: &[UnitId]| {
        crate::combat_ui::tally_names(
            ids.iter()
                .map(|id| crate::combat_ui::unit_name(*id, gs))
                .collect(),
        )
        .join(", ")
    };
    match tail {
        CombatTail::Fire {
            attack,
            target_hex,
            roll,
            ..
        } => format!(
            "Fire on {target_hex} by {} (roll {})",
            names(&attack.all_firing_units()),
            roll.value()
        ),
        CombatTail::Melee {
            attack,
            attacker_roll,
            defender_roll,
            ..
        } => format!(
            "Melee at {}: {} (roll {}) against {} (roll {})",
            attack.defender_hex,
            names(&attack.attackers),
            attacker_roll.value(),
            names(&attack.defenders),
            defender_roll.value()
        ),
        CombatTail::Eliminate { cause, .. } => format!("One unit is lost: {cause}"),
    }
}

/// The candidates as the panel lists them: identical counters numbered
/// ("Jaalin #2") and disrupted ones marked, so the owner can give up the
/// disrupted unit knowingly.
pub(crate) fn candidate_labels(
    gs: &omdurman_rules::effects::GameState,
    candidates: &[UnitId],
) -> Vec<String> {
    let names: Vec<String> = candidates
        .iter()
        .map(|id| crate::combat_ui::unit_name(*id, gs))
        .collect();
    let mut seen: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    candidates
        .iter()
        .zip(&names)
        .map(|(id, name)| {
            let twins = names.iter().filter(|n| *n == name).count();
            let mut label = name.clone();
            if twins > 1 {
                let n = seen.entry(name.as_str()).or_insert(0);
                *n += 1;
                label.push_str(&format!(" #{n}"));
            }
            if let Some(unit) = gs.find_unit(*id) {
                if unit.state.disrupted {
                    label.push_str(" (disrupted)");
                }
                label.push_str(&format!("  {}", unit.position));
            }
            label
        })
        .collect()
}

/// The choice this seat has to make, if any.
fn local_choice(
    gs: &omdurman_rules::effects::GameState,
    peers: &crate::peers::Peers,
) -> Option<LossChoice> {
    gs.pending_losses
        .as_ref()?
        .choices
        .iter()
        .find(|c| c.chosen.is_none() && peers.may_act(c.player))
        .cloned()
}

pub(crate) fn loss_choice_ui(
    mut contexts: EguiContexts,
    game_state: Res<GameStateResource>,
    mut submit: crate::submit::CheckedSubmit,
    layout: Res<crate::ScreenLayout>,
    peers: crate::peers::Peers,
    mut panel: Local<LossPanel>,
) {
    let gs = &game_state.0;
    let Some(choice) = local_choice(gs, &peers) else {
        *panel = LossPanel::default();
        return;
    };
    let Some(pending) = gs.pending_losses.as_ref() else {
        return;
    };
    if panel.choice.as_ref() != Some(&choice.candidates) {
        *panel = LossPanel {
            choice: Some(choice.candidates.clone()),
            ..Default::default()
        };
    }
    // Our submission's echo came back with the choice still open: refused.
    if let Some(uid) = panel.submitted
        && !submit.pending.unconfirmed.iter().any(|(u, _)| *u == uid)
    {
        panel.submitted = None;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let count = usize::from(choice.count);
    crate::ui::anchored_card(
        ctx,
        egui::Id::new("loss_choice_panel"),
        egui::Align2::RIGHT_CENTER,
        egui::vec2(-(layout.right_inset + 10.0), 0.0),
        egui::Frame::popup(&ctx.style_of(ctx.theme())),
        |ui| {
            crate::rulebook::refs_rich(
                ui,
                &format!("Choose your losses ({})", loss_rule(&pending.tail)),
                18.0,
                |t| t.strong(),
            );
            ui.add_space(4.0);
            ui.label(egui::RichText::new(cause(&pending.tail, gs)).size(13.0));
            let s = if count == 1 { "" } else { "s" };
            ui.label(
                egui::RichText::new(format!("Remove {count} unit{s} of these:"))
                    .size(13.0)
                    .strong(),
            );
            ui.add_space(4.0);
            let labels = candidate_labels(gs, &choice.candidates);
            for (&id, label) in choice.candidates.iter().zip(labels) {
                if gs.find_unit(id).is_none() {
                    continue;
                }
                let picked = panel.selected.contains(&id);
                if ui.selectable_label(picked, label).clicked() && panel.submitted.is_none() {
                    if picked {
                        panel.selected.retain(|u| *u != id);
                    } else if panel.selected.len() < count {
                        panel.selected.push(id);
                    }
                }
            }
            ui.add_space(6.0);
            if panel.submitted.is_some() {
                ui.label(
                    egui::RichText::new("Losses submitted -- awaiting confirmation…").size(12.0),
                );
                return;
            }
            let ready = panel.selected.len() == count;
            if ui
                .add_enabled(
                    ready,
                    // A full-width, tall target: a click near the small
                    // button's edge was lost.
                    egui::Button::new(egui::RichText::new("Take these losses").size(14.0).strong())
                        .fill(crate::ui::palette::BTN_GO)
                        .min_size(egui::vec2(ui.available_width().max(200.0), 34.0)),
                )
                .clicked()
            {
                let effect = GameEffect::TakeLosses {
                    player: choice.player,
                    units: panel.selected.clone(),
                };
                if submit.submit(gs, GameEvent::Effect(effect)) {
                    panel.submitted = submit.pending.unconfirmed.back().map(|(uid, _)| *uid);
                }
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_types::HexCoord;

    /// Identical counters are numbered and disrupted ones marked, so the
    /// owner can tell which he gives up.
    #[test]
    fn twins_are_numbered_and_the_disrupted_marked() {
        let mut gs = omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign);
        let hex = HexCoord::new(3, 4);
        let ids = [
            UnitId::MulazminI_0_0,
            UnitId::MulazminI_0_1,
            UnitId::BritishArmy_0_0,
        ];
        for (i, id) in ids.into_iter().enumerate() {
            gs.units.push(omdurman_rules::UnitPlacement {
                id,
                position: hex,
                profile: omdurman_rules::unit_profiles::profile_for_unit(id).unwrap(),
                state: omdurman_rules::UnitState {
                    disrupted: i == 1,
                    ..Default::default()
                },
            });
        }
        let labels = candidate_labels(&gs, &ids);
        assert_eq!(labels[0], format!("Mulazmin #1  {hex}"));
        assert_eq!(labels[1], format!("Mulazmin #2 (disrupted)  {hex}"));
        assert_eq!(labels[2], format!("Cavalry  {hex}"));
    }
}
