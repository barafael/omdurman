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

/// What the combat waiting on the choice is, for the panel's heading.
fn cause(tail: &CombatTail) -> String {
    match tail {
        CombatTail::Fire { target_hex, .. } => format!("Fire on {target_hex}"),
        CombatTail::Melee { attack, .. } => format!("Melee at {}", attack.defender_hex),
        CombatTail::Eliminate { .. } => "A breach or a demolition (§6.53, §6.62, §6.63)".into(),
    }
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
            crate::rulebook::refs_rich(ui, "Choose your losses (§7.7)", 18.0, |t| t.strong());
            ui.add_space(4.0);
            ui.label(egui::RichText::new(cause(&pending.tail)).size(13.0));
            let s = if count == 1 { "" } else { "s" };
            ui.label(
                egui::RichText::new(format!("Remove {count} unit{s} of these:"))
                    .size(13.0)
                    .strong(),
            );
            ui.add_space(4.0);
            for &id in &choice.candidates {
                let Some(unit) = gs.find_unit(id) else {
                    continue;
                };
                let picked = panel.selected.contains(&id);
                let label = format!("{}  {}", crate::combat_ui::unit_name(id, gs), unit.position);
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
                    egui::Button::new(egui::RichText::new("Take these losses").size(13.0).strong())
                        .fill(crate::ui::palette::BTN_GO),
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
