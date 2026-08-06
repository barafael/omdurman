//! Fall-of-Khartoum-specific status UI (§9.3). Replaces the generic §9.14
//! victory-point scoreboard in the right sidebar with a panel that speaks FoK's
//! vocabulary: GORDON's fate (§9.346), Dervish losses and the §9.35 victory
//! ladder, and an off-board 8-turn track widget (§9.33/§9.341).
//!
//! All quantities are derived live from `GameState` so the panel doubles as a
//! victory-trajectory readout: it projects "the result if the game ended right
//! now" via [`FoKVictoryLevel::resolve`].

use bevy::prelude::Res;
use bevy_egui::{egui, EguiContexts};
use omdurman_rules::turn_track::FALL_OF_KHARTOUM_TURN_TRACK;
use omdurman_rules::{FoKVictoryLevel, GameTurnIndex};
use omdurman_types::{DayNight, Location, Player, Scenario};

use crate::GameStateResource;

/// Whether the FoK status panel should replace the generic VP scoreboard.
pub(crate) fn is_fok(state: &GameStateResource) -> bool {
    state.0.scenario == Scenario::FallOfKhartoum
}

/// Render the FoK status section into the sidebar's "Game control" area:
/// GORDON status, Dervish losses with the next §9.35 penalty tier, the
/// projected victory level, and the 8-turn track widget. Mirrors the structure
/// of the §9.14 VP scoreboard it replaces.
pub(crate) fn fok_status_section(ui: &mut egui::Ui, state: &GameStateResource) {
    let gs = &state.0;
    let gordon_died = gs.gordon_eliminated_turn;

    ui.label(
        egui::RichText::new("Fall of Khartoum (\u{00a7}9.3)")
            .strong()
            .color(egui::Color32::from_rgb(200, 200, 150)),
    );

    // -- GORDON status (§9.346) -------------------------------------------
    let gordon_alive = gordon_died.is_none();
    let gordon_label = if gordon_alive {
        "GORDON: alive at the Palace (\u{00a7}9.346)".to_string()
    } else {
        format!(
            "GORDON: fallen, turn {} (\u{00a7}9.346)",
            gordon_died.map(|t| t.value()).unwrap_or(0)
        )
    };
    let gordon_color = if gordon_alive {
        egui::Color32::from_rgb(120, 200, 120)
    } else {
        egui::Color32::from_rgb(200, 120, 120)
    };
    ui.colored_label(gordon_color, gordon_label);

    // -- Dervish losses + §9.35 penalty tier ------------------------------
    let dervish_lost = gs.victory.units_eliminated_by(Player::AngloEgyptian);
    let penalty = FoKVictoryLevel::loss_penalty(dervish_lost);
    let loss_line = match FoKVictoryLevel::next_loss_threshold(dervish_lost) {
        Some(next) => {
            format!(
                "Dervish losses: {dervish_lost}  (penalty \u{2212}{penalty}, next at {next} \u{2014} \u{00a7}9.35)"
            )
        }
        None => {
            format!(
                "Dervish losses: {dervish_lost}  (max penalty \u{2212}{penalty} \u{2014} \u{00a7}9.35)"
            )
        }
    };
    ui.colored_label(egui::Color32::from_rgb(220, 150, 100), loss_line);

    // -- Projected victory level (§9.35) ----------------------------------
    // "If the game ended right now": feed the current turn as the scenario-end
    // turn. While GORDON lives this tracks how the British level grows; once he
    // falls it freezes on the Dervish base (shifted by losses).
    let projected = FoKVictoryLevel::resolve(
        gordon_died.map(|t| t.value()),
        gs.current_turn.value(),
        dervish_lost,
    );
    let proj_color = match projected {
        l if (l as i16) < 0 => egui::Color32::from_rgb(220, 150, 100),
        l if (l as i16) > 0 => egui::Color32::from_rgb(120, 180, 220),
        _ => egui::Color32::from_gray(170),
    };
    ui.colored_label(proj_color, format!("Projected: {projected} (\u{00a7}9.35)"));

    // -- Off-board turn track widget (§9.33, §9.341) ----------------------
    fok_turn_track_widget(ui, gs.current_turn);

    ui.add_space(4.0);
}

/// Compact horizontal 8-cell strip: each cell is the turn number, tinted for
/// night turns (§9.341), with the current turn outlined. FoK has no printed
/// track on its map (unlike the Campaign board), so this off-board widget is
/// the only turn-arc visualisation the player gets.
fn fok_turn_track_widget(ui: &mut egui::Ui, current: GameTurnIndex) {
    ui.label(
        egui::RichText::new("Turn track (\u{00a7}9.33)")
            .strong()
            .color(egui::Color32::from_rgb(200, 200, 150)),
    );
    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing = egui::vec2(2.0, 0.0);
        for entry in FALL_OF_KHARTOUM_TURN_TRACK {
            let is_current = entry.turn == current.value();
            let is_night = entry.day_night == DayNight::Night;
            let is_past = entry.turn < current.value();

            let fill = if is_night {
                egui::Color32::from_rgb(40, 45, 70)
            } else {
                egui::Color32::from_rgb(70, 65, 45)
            };
            let stroke = if is_current {
                egui::Stroke::new(2.0, egui::Color32::from_rgb(235, 200, 110))
            } else {
                egui::Stroke::new(1.0, egui::Color32::from_gray(90))
            };
            let text_color = if is_past {
                egui::Color32::from_gray(110)
            } else if is_current {
                egui::Color32::from_rgb(235, 210, 130)
            } else {
                egui::Color32::from_gray(210)
            };

            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(20.0, 20.0),
                egui::Sense::hover(),
            );
            let painter = ui.painter();
            painter.rect_filled(rect, 2.0, fill);
            painter.rect_stroke(rect, 2.0, stroke, egui::StrokeKind::Inside);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                entry.turn,
                egui::FontId::proportional(11.0),
                text_color,
            );
            if is_night {
                painter.text(
                    rect.center() + egui::vec2(0.0, 10.0),
                    egui::Align2::CENTER_TOP,
                    "\u{1f319}",
                    egui::FontId::proportional(7.0),
                    egui::Color32::from_rgb(160, 170, 210),
                );
            }
        }
    });
}

/// Floating GORDON badge anchored top-centre below the phase banner: always
/// visible during a FoK game so both players can see GORDON's fate at a glance
/// (§9.346). Suppressed during setup (GORDON is auto-placed, not yet at risk)
/// and once the victory modal takes over.
pub(crate) fn gordon_badge_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<GameStateResource>>,
) {
    let Some(state) = game_state.as_deref() else {
        return;
    };
    if !is_fok(state) {
        return;
    }
    if state.0.game_over {
        return;
    }
    if matches!(state.0.phase, omdurman_rules::Phase::Setup) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let gs = &state.0;
    let (label, fg, bg) = if gs.gordon_eliminated_turn.is_none() {
        let palace = gs
            .board
            .hex_of_location(Location::Palace)
            .map(|h| format!("({}, {})", h.q, h.r))
            .unwrap_or_else(|| "the Palace".into());
        (
            format!("GORDON holds the Palace {palace}"),
            egui::Color32::from_rgb(150, 220, 150),
            egui::Color32::from_rgba_unmultiplied(30, 50, 30, 210),
        )
    } else {
        let turn = gs.gordon_eliminated_turn.map(|t| t.value()).unwrap_or(0);
        (
            format!("GORDON fallen, turn {turn} (\u{00a7}9.346)"),
            egui::Color32::from_rgb(220, 140, 140),
            egui::Color32::from_rgba_unmultiplied(60, 25, 25, 210),
        )
    };

    egui::Area::new(egui::Id::new("gordon_badge"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 74.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(bg)
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(10, 4))
                .show(ui, |ui| {
                    ui.colored_label(fg, egui::RichText::new(label).size(13.0));
                });
        });
}
