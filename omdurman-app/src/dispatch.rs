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
use bevy_egui::egui;

use crate::events;
use crate::rulebook::{RefTok, split_refs};

/// One queued dispatch slip.
pub struct Dispatch {
    /// Small-caps header line, e.g. "DISPATCH" or "FIELD TELEGRAPH".
    pub header: String,
    /// Dry, factual body. Any `§N` reference in it becomes a rulebook link.
    pub body: String,
    /// Seconds this slip has been shown (for fade-out + expiry). Frozen
    /// while the slip is hovered or pinned (see [`crate::ui::CardHold`]).
    pub age: f32,
    /// Hover / click-to-pin state.
    pub hold: crate::ui::CardHold,
    /// Stable per-slip id (egui click target).
    pub serial: u64,
    /// Arrival order across the whole event feed (slips and combat cards
    /// share one column, newest first; see [`feed_seq`]).
    pub seq: u64,
}

/// The next arrival number of the event feed, shared by dispatch slips and
/// combat cards so the one column orders them as they happened.
pub(crate) fn feed_seq() -> u64 {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// How many lines the event log keeps.
const LOG_LINES: usize = 200;

/// The live dispatch queue, shown in the event feed column (top right) with
/// the combat cards; each slip expires after [`DISPATCH_TTL`] seconds unless
/// hovered or pinned.
#[derive(Resource, Default)]
pub struct Dispatches {
    pub slips: Vec<Dispatch>,
    next_serial: u64,
    /// Every message of the game session, oldest first, as one line each --
    /// slips and combat results alike -- for the top bar's Log menu, so a
    /// faded message can still be read.
    pub log: std::collections::VecDeque<String>,
}

impl Dispatches {
    /// Queue a dispatch. `header` is the small-caps frame label; `body` is the
    /// factual message (may contain `§N` references).
    pub fn push(&mut self, header: impl Into<String>, body: impl Into<String>) {
        self.next_serial += 1;
        let (header, body) = (header.into(), body.into());
        self.log_line(format!("{header}: {body}"));
        self.slips.push(Dispatch {
            header,
            body,
            age: 0.0,
            hold: crate::ui::CardHold::default(),
            serial: self.next_serial,
            seq: feed_seq(),
        });
        // Cap the backlog so a burst can't pile up indefinitely: evict the
        // oldest unpinned slip first.
        const MAX: usize = 5;
        while self.slips.len() > MAX {
            let victim = self.slips.iter().position(|s| !s.hold.pinned).unwrap_or(0);
            self.slips.remove(victim);
        }
    }

    /// Append one line to the session log (bounded).
    pub fn log_line(&mut self, line: String) {
        self.log.push_back(line);
        while self.log.len() > LOG_LINES {
            self.log.pop_front();
        }
    }

    /// Age the slips by `dt` and drop expired ones (hovered / pinned slips
    /// hold).
    pub(crate) fn age(&mut self, dt: f32) {
        for slip in &mut self.slips {
            slip.hold
                .age(&mut slip.age, dt, DISPATCH_TTL, DISPATCH_FADE);
        }
        self.slips.retain(|s| s.age < DISPATCH_TTL);
    }
}

pub(crate) const DISPATCH_TTL: f32 = 6.0;
pub(crate) const DISPATCH_FADE: f32 = 1.0;

pub struct DispatchPlugin;

impl Plugin for DispatchPlugin {
    fn build(&self, app: &mut App) {
        // The slips are drawn in the event feed column with the combat
        // cards (`combat_card::combat_card_ui`).
        app.init_resource::<Dispatches>()
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
        // Combat results live on the Combat Resolution Card alone: the
        // resolution itself, its casualties ("lost: ..."), and the advance it
        // opens. No slip repeats them.
        if matches!(
            ev.observation,
            omdurman_rules::effects::Observation::FireResolved { .. }
                | omdurman_rules::effects::Observation::MeleeResolved { .. }
                | omdurman_rules::effects::Observation::HexVacatedByCombat { .. }
                | omdurman_rules::effects::Observation::UnitEliminated {
                    cause: omdurman_rules::effects::ElimCause::Combat
                        // GORDON's fall has its own slip.
                        | omdurman_rules::effects::ElimCause::GordonAtPalace
                        // The "Wall Breached" slip names the breach casualty.
                        | omdurman_rules::effects::ElimCause::WallBreach,
                    ..
                }
        ) {
            continue;
        }
        if let Some((header, body)) = format_observation(&ev.observation, gs) {
            dispatches.push(header, body);
        }
    }
}

/// Draw one slip; returns a section number if the player clicked a `§` link,
/// and the slip's rect.
pub(crate) fn draw_slip(
    ui: &mut egui::Ui,
    slip: &Dispatch,
    fade: f32,
) -> (Option<String>, egui::Rect) {
    let a = |c: egui::Color32| c.gamma_multiply(fade);
    let mut clicked = None;
    let stroke = if slip.hold.pinned { 3.0 } else { 2.0 };

    let frame = crate::ui::frames::paper(egui::Stroke::new(stroke, a(crate::ui::palette::INK)))
        .inner_margin(egui::Margin::symmetric(10, 7))
        .show(ui, |ui| {
            // Text runs left to right: the column aligns its cards to the
            // right, which egui would carry into the wrapped rows.
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.set_max_width(300.0);
                // Header: letter-spaced small caps, faint ink.
                let header: String = slip
                    .header
                    .to_uppercase()
                    .chars()
                    .flat_map(|c| [c, '\u{2009}']) // thin space between glyphs
                    .collect();
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(header)
                            .color(a(crate::ui::palette::FAINT_INK))
                            .size(11.0)
                            .strong(),
                    );
                    if slip.hold.pinned {
                        ui.label(
                            egui::RichText::new("(pinned)")
                                .color(a(crate::ui::palette::FAINT_INK))
                                .size(10.0)
                                .italics(),
                        );
                    }
                });
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
            })
        });
    (clicked, frame.response.rect)
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

    let unit_label = |id: omdurman_rules::UnitId| crate::combat_ui::unit_name(id, gs);

    match obs {
        Observation::UnitEliminated {
            id,
            cause,
            vp_source,
        } => {
            let who = unit_label(*id);
            // Fall of Khartoum has no victory points: it counts Dervish
            // losses against the §9.35 level thresholds instead.
            // Only the Campaign scores victory points (§9.14): Fall of
            // Khartoum counts Dervish losses (§9.35), the Historical
            // scenario units eliminated per side (§9.24).
            let no_vp = gs.is_some_and(|g| !g.scenario.keeps_victory_points());
            let vp_clause = match vp_source {
                _ if no_vp => String::new(),
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
                format!(" {who} at the wall was eliminated.")
            } else {
                String::new()
            };
            Some((
                headline,
                format!(
                    "Wall between {} and {}.{extra} (§6.63)",
                    hexside.a, hexside.b,
                ),
            ))
        }
        // GORDON's fall has its own §9.346 slip (`GordonEliminated`); the
        // casualty report already says how any leader fell.
        Observation::LeaderKilled { id, .. }
            if omdurman_rules::unit_profiles::profile_for_unit(*id)
                .is_some_and(|p| p.identity.is_gordon()) =>
        {
            None
        }
        Observation::LeaderKilled { id, by } => Some((
            "Leader Dispatch".into(),
            format!("{} lost to the {by} (§6.51).", unit_label(*id)),
        )),
        Observation::GordonEliminated { turn } => Some((
            "Fall of Khartoum".into(),
            format!(
                "GORDON has fallen at the Palace on turn {} (§9.346).",
                turn.value()
            ),
        )),
        Observation::MineResolved {
            gunboat,
            hex,
            roll,
            result,
        } => Some((
            "River Mine".into(),
            format!(
                "{} struck a mine at ({},{}): roll {} -- {} (§10.12).",
                unit_label(*gunboat),
                hex.q,
                hex.r,
                roll.value(),
                match result {
                    omdurman_rules::MineResult::NoEffect => "no effect",
                    omdurman_rules::MineResult::EnginesLost => "engines lost, she drifts",
                    omdurman_rules::MineResult::Sunk => "sunk",
                }
            ),
        )),
        Observation::ChainFiredAt { roll, sunk, .. } => Some((
            "River Chain".into(),
            format!(
                "The batteries fire on the chain: roll {} -- {} (§10.23).",
                roll.value(),
                if *sunk {
                    "the chain is sunk"
                } else {
                    "it holds"
                }
            ),
        )),
        Observation::ChainSunkFromTheBank => Some((
            "River Chain".into(),
            "Troops holding the bank have sunk the chain (§10.23).".into(),
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
        // Every VP is scored by an elimination, whose Casualty Report already
        // states it -- a second slip per kill only buries the board (and FoK
        // has no VP at all, §9.35).
        Observation::VictoryScored { .. } => None,
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
        Observation::MeleeLapsed { defender_hex, .. } => Some((
            "Melee".into(),
            format!(
                "The defenders of {defender_hex} withdrew before the blow fell (§7.5): the \
                 melee lapses, with no roll and no forced advance (§7.6)."
            ),
        )),
    }
}
