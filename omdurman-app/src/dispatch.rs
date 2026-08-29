//! Period-dispatch system messages (§decision 10). Rejections, combat results,
//! and other terse notices render as a small "field telegraph" slip: a paper
//! card with a 2px ink border and a letter-spaced small-caps header. The flavour
//! lives only in the frame and header line; the body stays dry and factual, with
//! any rulebook `§` reference rendered as a deep link into the Rulebook tab.
//!
//! Two producers feed the queue:
//! * **Rejections / refusals** pushed directly by input systems (e.g. fire
//!   refused for want of line of sight).
//! * **Engine observations** -- [`events::ObservationEvent`]s drained from the
//!   rules engine's side-channel after each `apply_effect` -- translated by
//!   [`format_observation`] into a (header, body) pair so eliminations, leader
//!   deaths, VP awards, demolition results, and so on surface as readable
//!   slips with the rulebook paragraphs that authorise them.

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::events;
use crate::rulebook::{RefTok, split_refs};

/// One queued dispatch slip.
pub struct Dispatch {
    /// Small-caps header line, e.g. "DISPATCH" or "FIELD TELEGRAPH".
    pub header: String,
    /// Dry, factual body. Any `§N` reference in it becomes a rulebook link.
    pub body: String,
    /// Seconds this slip has been shown (for fade-out + expiry).
    pub age: f32,
}

/// The live dispatch queue. Newest slips stack at the bottom-left; each expires
/// after [`DISPATCH_TTL`] seconds.
#[derive(Resource, Default)]
pub struct Dispatches {
    pub slips: Vec<Dispatch>,
}

impl Dispatches {
    /// Queue a dispatch. `header` is the small-caps frame label; `body` is the
    /// factual message (may contain `§N` references).
    pub fn push(&mut self, header: impl Into<String>, body: impl Into<String>) {
        self.slips.push(Dispatch {
            header: header.into(),
            body: body.into(),
            age: 0.0,
        });
        // Cap the backlog so a burst can't pile up indefinitely.
        const MAX: usize = 5;
        let len = self.slips.len();
        if len > MAX {
            self.slips.drain(0..len - MAX);
        }
    }
}

const DISPATCH_TTL: f32 = 6.0;
const DISPATCH_FADE: f32 = 1.0;

pub struct DispatchPlugin;

impl Plugin for DispatchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Dispatches>()
            .add_systems(EguiPrimaryContextPass, draw_dispatches)
            // Translate engine observations into readable dispatch slips.
            // Combat resolutions (FireResolved / MeleeResolved) are surfaced
            // separately by the Combat Resolution Card; this listener handles
            // every other observation kind.
            .add_systems(PreUpdate, queue_observation_dispatches);
        // Dev: keep demo slips on screen for headless verification
        // (OMDURMAN_DISPATCH=1) by re-seeding whenever the queue empties.
        if std::env::var("OMDURMAN_DISPATCH").is_ok() {
            app.add_systems(Update, |mut d: ResMut<Dispatches>| {
                if d.slips.is_empty() {
                    d.push("Field Telegraph", "Fire refused — no line of sight (§6.3).");
                    d.push("Dispatch", "Move rejected — zone of control (§5.41).");
                }
            });
        }
    }
}

/// Listen for engine [`events::ObservationEvent`]s and push a readable dispatch
/// for each variant we know how to caption. Combat resolutions are skipped here
/// -- the Combat Resolution Card owns those -- but every other observation
/// (leader killed, fort destroyed, VP scored, demolition resolved, etc.) lands
/// in the queue with its authorising rulebook paragraphs as deep links.
///
/// Identity lookups (which leader? which fort?) go through the live
/// [`GameState`](omdurman_rules::effects::GameState) so the slip can name the
/// unit rather than printing its raw `UnitId`.
fn queue_observation_dispatches(
    mut reader: MessageReader<events::ObservationEvent>,
    mut dispatches: ResMut<Dispatches>,
    game_state: Option<Res<crate::GameStateResource>>,
) {
    let gs = game_state.as_deref().map(|r| &r.0);
    for ev in reader.read() {
        if matches!(
            ev.observation,
            omdurman_rules::effects::Observation::FireResolved { .. }
                | omdurman_rules::effects::Observation::MeleeResolved { .. }
        ) {
            continue;
        }
        if let Some((header, body)) = format_observation(&ev.observation, gs) {
            dispatches.push(header, body);
        }
    }
}

fn draw_dispatches(
    mut contexts: EguiContexts,
    mut dispatches: ResMut<Dispatches>,
    time: Res<Time>,
    mut rulebook: ResMut<crate::rulebook::Rulebook>,
) {
    // Age and expire.
    let dt = time.delta_secs();
    for slip in &mut dispatches.slips {
        slip.age += dt;
    }
    dispatches.slips.retain(|s| s.age < DISPATCH_TTL);
    if dispatches.slips.is_empty() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let mut clicked_section: Option<String> = None;

    crate::ui::anchored_card(
        ctx,
        egui::Id::new("dispatch_slips"),
        egui::Align2::LEFT_BOTTOM,
        egui::vec2(14.0, -48.0),
        egui::Frame::NONE,
        |ui| {
            ui.set_max_width(320.0);
            // Oldest on top, newest at the bottom (nearest the corner).
            for slip in &dispatches.slips {
                let fade = ((DISPATCH_TTL - slip.age) / DISPATCH_FADE).clamp(0.0, 1.0);
                if let Some(sec) = draw_slip(ui, slip, fade) {
                    clicked_section = Some(sec);
                }
                ui.add_space(6.0);
            }
        },
    );

    if let Some(sec) = clicked_section {
        crate::rulebook::request_section(&mut rulebook, &sec);
    }
    ctx.request_repaint(); // keep the fade animating
}

/// Draw one slip; returns a section number if the player clicked a `§` link.
fn draw_slip(ui: &mut egui::Ui, slip: &Dispatch, fade: f32) -> Option<String> {
    let a = |c: egui::Color32| c.gamma_multiply(fade);
    let mut clicked = None;

    crate::ui::paper_frame(egui::Stroke::new(2.0, a(crate::ui::palette::INK)))
        .inner_margin(egui::Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.set_max_width(300.0);
            // Header: letter-spaced small caps, faint ink.
            let header: String = slip
                .header
                .to_uppercase()
                .chars()
                .flat_map(|c| [c, '\u{2009}']) // thin space between glyphs
                .collect();
            ui.label(
                egui::RichText::new(header)
                    .color(a(crate::ui::palette::FAINT_INK))
                    .size(11.0)
                    .strong(),
            );
            ui.add_space(2.0);
            // Body: dry text with §N references as rulebook links. References
            // are annotated with their section title via `Rulebook::title_of`
            // so a reader sees the rule's name, not just its number.
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for tok in split_refs(&slip.body) {
                    match tok {
                        RefTok::Text(t) => {
                            ui.label(
                                egui::RichText::new(t)
                                    .color(a(crate::ui::palette::INK))
                                    .size(14.0),
                            );
                        }
                        RefTok::Ref(n) => {
                            let label = format!("§{n}");
                            if ui
                                .add(
                                    egui::Label::new(
                                        egui::RichText::new(label)
                                            .color(a(crate::ui::palette::INK))
                                            .size(14.0)
                                            .underline(),
                                    )
                                    .sense(egui::Sense::click()),
                                )
                                .clicked()
                            {
                                clicked = Some(n.to_string());
                            }
                        }
                    }
                }
            });
        });
    clicked
}

// -- Observation -> dispatch formatting ---------------------------------------

/// Translate a non-combat [`Observation`](omdurman_rules::effects::Observation)
/// into a `(header, body)` dispatch pair, looking up unit identities through
/// the live game state so slips read "Khalifa eliminated" rather than
/// "Unit(72b3..) eliminated".
///
/// Each body cites the rulebook paragraphs that authorise the event as `§N`
/// references; the slip renderer turns those into deep links into the Rulebook
/// tab (annotated with the section title).
///
/// Returns `None` for observation kinds with no meaningful player-readable
/// summary yet (combat resolutions are handled by the Combat Resolution Card
/// and never reach here).
fn format_observation(
    obs: &omdurman_rules::effects::Observation,
    gs: Option<&omdurman_rules::effects::GameState>,
) -> Option<(String, String)> {
    use omdurman_rules::effects::Observation;

    let unit_label = |id: omdurman_rules::UnitId| -> String {
        gs.and_then(|s| s.find_unit(id))
            .map(|u| u.profile.identity.short_label())
            .unwrap_or_else(|| format!("unit {id:?}"))
    };

    match obs {
        Observation::UnitEliminated {
            id,
            cause,
            vp_source,
        } => {
            let who = unit_label(*id);
            let vp_clause = match vp_source {
                Some(src) => {
                    let pts = src.points();
                    let scorer = src.who_scores();
                    if pts.value() > 0 {
                        format!(" {scorer} scores {} VP (§9.14).", pts.value())
                    } else {
                        " No VP awarded (§9.14: forts are worth 0).".to_string()
                    }
                }
                None => String::new(),
            };
            Some((
                "Casualty Report".into(),
                format!("{who} eliminated ({cause}).{vp_clause}"),
            ))
        }
        Observation::FortDestroyed { hex, .. } => Some((
            "Engineer Dispatch".into(),
            format!("Fort at ({},{}) destroyed (§6.53, §6.62).", hex.q, hex.r,),
        )),
        Observation::WallBreached {
            hexside,
            breached,
            adjacent_eliminated,
            ..
        } => {
            let headline = if *breached {
                "Wall Breached".into()
            } else {
                "Breach Attempt Failed".into()
            };
            let extra = if let Some(victim) = adjacent_eliminated {
                let who = unit_label(*victim);
                format!(" Adjacent {who} eliminated in the breach (§6.63).")
            } else {
                String::new()
            };
            Some((
                headline,
                format!(
                    "Wall between ({},{}) and ({},{}).{extra} (§6.63)",
                    hexside.a.q, hexside.a.r, hexside.b.q, hexside.b.r,
                ),
            ))
        }
        Observation::LeaderKilled { id, by } => Some((
            "Leader Dispatch".into(),
            format!("{} killed in combat by {} (§9.14).", unit_label(*id), by),
        )),
        Observation::GordonEliminated { turn } => Some((
            "Fall of Khartoum".into(),
            format!(
                "GORDON has fallen at the Palace on turn {} (§9.346).",
                turn.value()
            ),
        )),
        Observation::FriendliesDisembarked { unit_id, at } => Some((
            "Disembarkation".into(),
            format!(
                "{} disembarked at ({},{}) (§5.21).",
                unit_label(*unit_id),
                at.q,
                at.r,
            ),
        )),
        Observation::DemolitionResolved {
            engineer_id,
            target,
            success,
        } => {
            let target_str = match target {
                omdurman_rules::DemolitionTarget::Fort(fid) => {
                    let pos = gs.and_then(|s| s.find_unit(*fid)).map(|u| u.position);
                    match pos {
                        Some(p) => format!("fort at ({},{})", p.q, p.r),
                        None => "fort".to_string(),
                    }
                }
                omdurman_rules::DemolitionTarget::WallHexside(h) => {
                    format!(
                        "wall between ({},{}) and ({},{})",
                        h.a.q, h.a.r, h.b.q, h.b.r
                    )
                }
            };
            let outcome = if *success { "succeeded" } else { "failed" };
            Some((
                "Royal Engineers".into(),
                format!(
                    "{} demolition of {} {} (§6.53).",
                    unit_label(*engineer_id),
                    target_str,
                    outcome,
                ),
            ))
        }
        Observation::VictoryScored {
            source,
            points,
            for_player,
        } => Some((
            "Victory Points".into(),
            format!(
                "{for_player} scores {} VP: {source} (§9.14).",
                points.value(),
            ),
        )),
        // Combat resolutions are surfaced by the Combat Resolution Card and
        // intentionally not duplicated here.
        Observation::FireResolved { .. } | Observation::MeleeResolved { .. } => None,
        Observation::HexVacatedByCombat {
            hex,
            eligible,
            paragraphs,
        } => {
            let who = eligible
                .iter()
                .map(|id| unit_label(*id))
                .collect::<Vec<_>>()
                .join(", ");
            Some((
                "Advance After Combat".into(),
                format!(
                    "Hex ({},{}) vacated; {who} may advance (§{}).",
                    hex.q,
                    hex.r,
                    paragraphs.join(", §"),
                ),
            ))
        }
    }
}
