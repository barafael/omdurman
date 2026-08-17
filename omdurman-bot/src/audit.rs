//! Deterministic log scanners: rule-deviation checks over a rendered
//! [`GameLog`](crate::log::GameLog), runnable headless via
//! `omdurman-bot-cli audit <log>`.
//!
//! The LLM observer generates hypotheses; these scanners are the
//! deterministic cross-check (and the standing regression tripwire after any
//! rules-engine change: regenerate the seed matrix, run the audit, diff
//! against the known-clean baseline). Every check cites the rulebook section
//! it audits and classifies findings as:
//!
//! - **Error** — a rule the engine is expected to enforce was demonstrably
//!   violated (nonzero exit code).
//! - **Warning** — a pattern that is *usually* a violation but has legal
//!   explanations the log text alone cannot exclude (§6.14's per-unit
//!   fired-at rule vs stacked forts, occupant turnover mid-phase, the §6.42
//!   subphase reset). Triaged by a human.

use std::collections::HashMap;
use std::fmt;

/// A scanner verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Severity {
    /// A rule violation the engine should have prevented.
    Error,
    /// A suspicious pattern with legal explanations; triage by hand.
    Warning,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

/// One scanner finding.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Finding {
    pub severity: Severity,
    /// Stable check id, e.g. `advance_without_window`.
    pub code: &'static str,
    pub detail: String,
}

/// The result of auditing one log.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AuditReport {
    pub findings: Vec<Finding>,
    pub events_scanned: usize,
    pub scenario: String,
}

impl AuditReport {
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Error)
    }
}

impl fmt::Display for AuditReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# Log audit — {} events scanned", self.events_scanned)?;
        if self.findings.is_empty() {
            writeln!(f, "no findings")?;
            return Ok(());
        }
        for finding in &self.findings {
            writeln!(
                f,
                "[{:<7}] {:24} {}",
                finding.severity.label(),
                finding.code,
                finding.detail
            )?;
        }
        let errors = self
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count();
        writeln!(
            f,
            "{} finding(s), {} error(s)",
            self.findings.len(),
            errors
        )?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Parsed log structures
// ---------------------------------------------------------------------------

/// The phase column of an event line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventLine {
    pub seq: usize,
    pub turn: u8,
    pub phase: String,
    pub actor: String,
    pub text: String,
    /// Phase/subphase index at this point in the log (increments at every
    /// `AdvancePhase`), separating the §6.42 fire subphases.
    pub epoch: usize,
}

/// One `label at (q,r)` pair (deploy lines and reinforcement batches).
#[derive(Debug, Clone)]
pub struct PlacementLine {
    pub label: String,
    pub hex: String,
}

/// Parse the log's per-event lines (`[seq] T<turn> <phase> <actor>  text`).
fn parse_events(text: &str) -> Vec<EventLine> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix('[') else { continue };
        let Some((seq, rest)) = rest.split_once(']') else { continue };
        let Ok(seq) = seq.trim().parse::<usize>() else {
            continue;
        };
        // `T<turn> <phase...> <actor>  text` -- the phase may contain spaces
        // ("Offensive Fire") and is followed by the actor and a double space.
        let rest = rest.trim_start();
        let Some(after_t) = rest.strip_prefix('T') else { continue };
        let Some((turn, rest)) = after_t.split_once(' ') else { continue };
        let Ok(turn) = turn.parse::<u8>() else { continue };
        // Actor is the token before the double space that starts the text.
        let Some((head, text)) = rest.split_once("  ") else { continue };
        let (phase, actor) = match head.rsplit_once(' ') {
            Some((phase, actor)) => (phase.to_string(), actor.to_string()),
            None => (head.to_string(), String::new()),
        };
        out.push(EventLine {
            seq,
            turn,
            phase,
            actor,
            text: text.to_string(),
            epoch: 0,
        });
    }
    // Assign phase epochs: each `AdvancePhase` ends the current phase (or
    // fire subphase -- the §6.42 Direct→Maxim/Howitzer bridge renders as an
    // AdvancePhase within the same top-level "Offensive Fire"/"Defensive
    // Fire" column). Attacks in different epochs are different subphases,
    // where §6.42 explicitly allows re-firing at units fired at in Direct.
    let mut epoch = 0usize;
    for e in out.iter_mut() {
        e.epoch = epoch;
        if e.text.starts_with("AdvancePhase") {
            epoch += 1;
        }
    }
    out
}

/// A unit label, normalized: strips the optional `[Faction]` prefix and
/// ` #n` disambiguator so `"[Dervish] Baggara #3"` and `"Baggara"` compare
/// equal. Older logs render without either suffix.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnitLabel(pub String);

impl UnitLabel {
    fn parse(raw: &str) -> Self {
        let mut s = raw.trim().to_string();
        if s.starts_with('[') {
            if let Some((_, rest)) = s.split_once(']') {
                s = rest.trim().to_string();
            }
        }
        // Strip a trailing ` #<digits>` disambiguator.
        if let Some((head, idx)) = s.rsplit_once(" #") {
            if !idx.is_empty() && idx.chars().all(|c| c.is_ascii_digit()) {
                s = head.trim().to_string();
            }
        }
        UnitLabel(s)
    }
}

fn parse_hex(raw: &str) -> Option<String> {
    let inner = raw.trim().trim_start_matches('(').trim_end_matches(')');
    let (q, r) = inner.split_once(',')?;
    q.trim().parse::<i32>().ok()?;
    r.trim().parse::<i32>().ok()?;
    Some(format!("({},{})", q.trim(), r.trim()))
}

/// `DeployUnit <label> at (q,r)` during Setup.
fn parse_deploy(text: &str) -> Option<PlacementLine> {
    let rest = text.strip_prefix("DeployUnit ")?;
    let (label, hex) = rest.split_once(" at ")?;
    Some(PlacementLine {
        label: UnitLabel::parse(label).0,
        hex: parse_hex(hex)?,
    })
}

/// Entries of a `PlaceReinforcements: A at (q,r), B at (q,r)` line.
fn parse_reinforcements(text: &str) -> Vec<PlacementLine> {
    let Some(rest) = text.strip_prefix("PlaceReinforcements: ") else {
        return Vec::new();
    };
    rest.split(", ")
        .filter_map(|entry| {
            let (label, hex) = entry.split_once(" at ")?;
            Some(PlacementLine {
                label: UnitLabel::parse(label).0,
                hex: parse_hex(hex)?,
            })
        })
        .collect()
}

/// `AdvanceAfterCombat <label>: (a,b) → (c,d)` (current) or
/// `AdvanceAfterCombat <label> → (c,d)` (pre-enrichment logs).
fn parse_advance(text: &str) -> Option<(String, String)> {
    let rest = text.strip_prefix("AdvanceAfterCombat ")?;
    let (head, dest) = rest.rsplit_once(" → ")?;
    let label = head.split_once(':').map(|(name, _)| name).unwrap_or(head);
    Some((UnitLabel::parse(label).0, parse_hex(dest)?))
}

/// The target hex of a fire event: `<verb> <firers> at (q,r) ...`.
/// Firer labels keep their ` #n` disambiguator (stripping it would collapse
/// "1B First Btn #1" and "#2" into one unit) and drop only the faction
/// prefix.
fn parse_fire_target(text: &str) -> Option<(Vec<String>, String)> {
    let rest = text
        .strip_prefix("fire ")
        .or_else(|| text.strip_prefix("howitzer bombardment "))?;
    let (firers, tail) = rest.split_once(" at ")?;
    let hex = tail.split_whitespace().next()?;
    let firers = firers
        .split(", ")
        .map(|f| strip_faction_prefix(f))
        .collect();
    Some((firers, parse_hex(hex)?))
}

/// Remove a leading `[Faction]` prefix, keeping everything else.
fn strip_faction_prefix(raw: &str) -> String {
    let s = raw.trim();
    match s.strip_prefix('[') {
        Some(rest) => rest.split_once(']').map(|(_, r)| r.trim()).unwrap_or(s),
        None => s,
    }
    .to_string()
}

/// `DervishDesertion roll N: A, B, ... desert`.
fn parse_desertion(text: &str) -> Option<(u32, Vec<String>)> {
    let rest = text.strip_prefix("DervishDesertion roll ")?;
    let (roll, names) = rest.split_once(": ")?;
    let roll = roll.trim().parse::<u32>().ok()?;
    let names = names
        .strip_suffix(" desert")?
        .split(", ")
        .map(|n| UnitLabel::parse(n).0)
        .collect();
    Some((roll, names))
}

/// `→ HexVacatedByCombat at (q,r): ... [event N]` observation lines.
fn parse_vacated(line: &str) -> Option<(String, usize)> {
    let rest = line.trim().strip_prefix("→ HexVacatedByCombat at ")?;
    let (hex, tail) = rest.split_once(':')?;
    let event = tail.rsplit_once("[event ")?.1.trim_end_matches(']');
    Some((parse_hex(hex)?, event.parse::<usize>().ok()?))
}

/// `→ FireResolved at (q,r): ... [§6.22 §6.24 §6.61]  [event N]` observation
/// lines: whether the attack engaged a *special target* (§6.61 gunboat /
/// §6.62 fort). Those targets are exempt from §6.14's fired-at-once rule, so
/// duplicate attacks on such hexes are legal.
fn parse_fire_special_target(line: &str) -> Option<(String, bool, usize)> {
    let rest = line.trim().strip_prefix("→ FireResolved at ")?;
    let (hex, tail) = rest.split_once(':')?;
    let event = tail.rsplit_once("[event ")?.1.trim_end_matches(']');
    let special = tail.contains("§6.61") || tail.contains("§6.62");
    Some((parse_hex(hex)?, special, event.parse::<usize>().ok()?))
}

// ---------------------------------------------------------------------------
// Rule data (mirrors omdurman-rules/src/reinforcements.rs + the manual)
// ---------------------------------------------------------------------------

/// §9.112 Dervish wave tribes per turn.
fn dervish_wave_tribes(turn: u8) -> Option<&'static [&'static str]> {
    Some(match turn {
        1 => &["Baggara", "Jaalin", "Danagla", "Kehena", "Degheim"],
        2 => &["Hadendowa"],
        3 => &["Mulazmin", "Jehadia"],
        _ => return None,
    })
}

/// §9.112 Dervish wave leaders per turn.
fn dervish_wave_leaders(turn: u8) -> Option<&'static [&'static str]> {
    Some(match turn {
        1 => &["Yakub", "Sherif", "AliWadHelu"],
        2 => &["OsmanDigna"],
        3 => &["SheikElDin"],
        _ => return None,
    })
}

/// §9.111: the Dervish initial force deployable at Campaign setup.
const CAMPAIGN_INITIAL_DERVISH: &[&str] = &[
    "KhalifaAbdullah",
    "Taiasha",
    "IsaZachneih",
    "Dervish Fort",
    "Dervish Artillery",
    "Dervish Gunboat DervishGunboat",
];

/// §8.2: units that may never desert.
const DESERTION_EXEMPT: &[&str] = &[
    "KhalifaAbdullah",
    "Dervish Gunboat DervishGunboat",
    "Dervish Artillery",
    "Dervish Fort",
];

fn is_ae_label(label: &str) -> bool {
    // nB/nE/nS/nF battalion labels, gunboats, Maxims, artillery, cavalry,
    // camel corps, Royal Engineers, British leaders.
    label.starts_with("Gunboat")
        || label.starts_with("Maxim")
        || label.starts_with("Artillery")
        || label == "Cavalry"
        || label == "Camel Corps"
        || label == "Royal Engineers"
        || matches!(
            label,
            "Kitchener" | "Gatacre" | "Hunter" | "Gordon" | "Wauchope" | "Lyttelton" | "Collinson"
        )
        || is_ae_infantry_label(label)
}

fn is_ae_infantry_label(label: &str) -> bool {
    // "1B First Btn", "3E Second Btn", "4F First Btn" (Friendlies)...
    let mut chars = label.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_digit())
        && matches!(chars.next(), Some('B' | 'E' | 'S' | 'F'))
        && matches!(chars.next(), Some(' '))
}

fn is_friendlies_label(label: &str) -> bool {
    is_ae_infantry_label(label)
        && label
            .chars()
            .nth(1)
            .is_some_and(|c| c == 'F')
}

/// Dervish tribe names (any unit whose label starts with one).
const DERVISH_TRIBES: &[&str] = &[
    "Baggara",
    "Jaalin",
    "Danagla",
    "Kehena",
    "Degheim",
    "Hadendowa",
    "Mulazmin",
    "Jehadia",
    "Taiasha",
    "IsaZachneih",
];

fn dervish_tribe_of(label: &str) -> Option<&'static str> {
    DERVISH_TRIBES.iter().copied().find(|t| label.starts_with(t))
}

fn is_dervish_label(label: &str) -> bool {
    label.starts_with("Dervish")
        || matches!(label, "KhalifaAbdullah" | "Yakub" | "Sherif" | "AliWadHelu" | "OsmanDigna" | "SheikElDin" | "Mahdi")
        || dervish_tribe_of(label).is_some()
}

// ---------------------------------------------------------------------------
// The audit
// ---------------------------------------------------------------------------

/// Run all scanners over a rendered game log.
pub fn audit_log(text: &str) -> AuditReport {
    let mut report = AuditReport::default();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("scenario:") {
            report.scenario = rest.trim().to_string();
        }
    }

    let events = parse_events(text);
    report.events_scanned = events.len();
    let seq_turn: HashMap<usize, u8> = events
        .iter()
        .map(|e| (e.seq, e.turn))
        .collect();

    // ---- Vacated-hex windows (§6.82/§7.6) ----
    let mut vacated: HashMap<(u8, String), usize> = HashMap::new();
    for line in text.lines() {
        if let Some((hex, event)) = parse_vacated(line) {
            if let Some(&turn) = seq_turn.get(&event) {
                vacated.insert((turn, hex), event);
            }
        }
    }

    struct Advance {
        seq: usize,
        turn: u8,
        unit: String,
        hex: String,
    }
    struct FireAttackLine {
        seq: usize,
        turn: u8,
        epoch: usize,
        phase: String,
        actor: String,
        firers: Vec<String>,
        hex: String,
    }
    let mut advances: Vec<Advance> = Vec::new();
    let mut fires: Vec<FireAttackLine> = Vec::new();
    let mut setup_deploys: Vec<(String, PlacementLine)> = Vec::new(); // (actor, placement)
    let mut gordon_deploys: Vec<(usize, String)> = Vec::new(); // (seq, label)
    let mut desertions: Vec<(usize, u8, u32, Vec<String>)> = Vec::new();
    let mut reinforcement_batches: Vec<(usize, u8, String, Vec<PlacementLine>)> =
        Vec::new(); // (seq, turn, actor, entries)

    for e in &events {
        if e.phase == "Setup" {
            if let Some(p) = parse_deploy(&e.text) {
                if p.label == "Gordon" {
                    gordon_deploys.push((e.seq, p.label.clone()));
                }
                setup_deploys.push((e.actor.clone(), p));
            }
        }
        if let Some((unit, hex)) = parse_advance(&e.text) {
            advances.push(Advance {
                seq: e.seq,
                turn: e.turn,
                unit,
                hex,
            });
        }
        if let Some((firers, hex)) = parse_fire_target(&e.text) {
            fires.push(FireAttackLine {
                seq: e.seq,
                turn: e.turn,
                epoch: e.epoch,
                phase: e.phase.clone(),
                actor: e.actor.clone(),
                firers,
                hex,
            });
        }
        if let Some((roll, names)) = parse_desertion(&e.text) {
            desertions.push((e.seq, e.turn, roll, names));
        }
        if e.text.starts_with("PlaceReinforcements: ") {
            reinforcement_batches.push((e.seq, e.turn, e.actor.clone(), parse_reinforcements(&e.text)));
        }
    }

    // ---- §6.82/§7.6: every advance needs a combat-opened window this turn ----
    for a in &advances {
        match vacated.get(&(a.turn, a.hex.clone())) {
            Some(&opened) if opened < a.seq => {}
            _ => report.findings.push(Finding {
                severity: Severity::Error,
                code: "advance_without_window",
                detail: format!(
                    "seq {}: {} advanced into {} on T{} with no HexVacatedByCombat observation for that hex this turn (§6.82/§7.6)",
                    a.seq, a.unit, a.hex, a.turn
                ),
            }),
        }
    }

    // ---- §6.14: fired-at-once (per unit, per subphase) ----
    // The log cannot show hex occupancy per attack, so flag duplicate
    // attacks on the same hex in one (turn, subphase-epoch, actor) group and
    // duplicate firings by the same unit. Two legal patterns are recognised
    // from the log itself: attacks in different epochs are different fire
    // subphases (§6.42 allows re-firing there), and a hex whose attack
    // resolution cites §6.61/§6.62 holds a gunboat/fort — the §6.14
    // fired-at exemption. The rest (stacked occupants, turnover mid-phase)
    // stays a Warning for human triage.
    {
        use std::collections::BTreeMap;
        // Events whose fire resolution engaged a gunboat/fort (§6.61/§6.62).
        let mut special_target: std::collections::HashSet<usize> =
            std::collections::HashSet::new();
        for line in text.lines() {
            if let Some((_, true, event)) = parse_fire_special_target(line) {
                special_target.insert(event);
            }
        }
        let mut by_target: BTreeMap<(u8, usize, String, String), Vec<usize>> = BTreeMap::new();
        let mut by_firer: BTreeMap<(u8, usize, String, String), usize> = BTreeMap::new();
        for f in &fires {
            if !f.phase.contains("Fire") {
                continue;
            }
            by_target
                .entry((f.turn, f.epoch, f.actor.clone(), f.hex.clone()))
                .or_default()
                .push(f.seq);
            for firer in &f.firers {
                // §6.14 parenthetical: Maxim guns and gunboats fire (and are
                // fired at) more than once per phase.
                if firer.starts_with("Gunboat") || firer.starts_with("Maxim") {
                    continue;
                }
                let key = (f.turn, f.epoch, f.actor.clone(), firer.clone());
                match by_firer.get(&key) {
                    Some(&first) => report.findings.push(Finding {
                        severity: Severity::Warning,
                        code: "unit_fired_twice_per_phase",
                        detail: format!(
                            "T{} {}: {} fires again at seq {} after seq {} (§6.14: once per subphase)",
                            f.turn, f.phase, firer, f.seq, first
                        ),
                    }),
                    None => {
                        by_firer.insert(key, f.seq);
                    }
                }
            }
        }
        for ((turn, epoch, actor, hex), seqs) in by_target {
            if seqs.len() > 1 && !seqs.iter().any(|s| special_target.contains(s)) {
                report.findings.push(Finding {
                    severity: Severity::Warning,
                    code: "hex_targeted_twice_per_phase",
                    detail: format!(
                        "T{} {} (epoch {}): hex {} attacked {}× (seq {:?}) — §6.14 fired-at-once unless distinct occupants / occupant turnover",
                        turn, actor, epoch, hex, seqs.len(), seqs
                    ),
                });
            }
        }
    }

    // ---- §9.111 / §9.211 / §9.212: setup force composition ----
    match report.scenario.as_str() {
        "campaign" => {
            for (_, p) in &setup_deploys {
                if is_ae_label(&p.label) {
                    report.findings.push(Finding {
                        severity: Severity::Error,
                        code: "campaign_ae_setup_deploy",
                        detail: format!(
                            "{} at {} deployed at Campaign setup — the A-E side deploys nothing (§9.113); all units arrive as reinforcements",
                            p.label, p.hex
                        ),
                    });
                } else if !CAMPAIGN_INITIAL_DERVISH.iter().any(|k| p.label.starts_with(k)) {
                    report.findings.push(Finding {
                        severity: Severity::Error,
                        code: "campaign_noninitial_dervish_setup",
                        detail: format!(
                            "{} at {} deployed at Campaign setup — not in the §9.111 initial force (wave reinforcement per §9.112)",
                            p.label, p.hex
                        ),
                    });
                }
            }
        }
        "historical" => {
            for (_, p) in &setup_deploys {
                let bad = if is_ae_label(&p.label) {
                    p.label == "Gordon" || is_friendlies_label(&p.label)
                } else {
                    p.label.starts_with("Dervish Gunboat")
                        || p.label.starts_with("Dervish Fort")
                        || p.label.starts_with("IsaZachneih")
                };
                if bad {
                    report.findings.push(Finding {
                        severity: Severity::Error,
                        code: "historical_not_in_play",
                        detail: format!(
                            "{} at {} deployed at Historical setup — §9.211/§9.212 not-in-play unit",
                            p.label, p.hex
                        ),
                    });
                }
            }
        }
        _ => {}
    }

    // ---- Gordon uniqueness (one physical counter, one id) ----
    if gordon_deploys.len() > 1 {
        report.findings.push(Finding {
            severity: Severity::Error,
            code: "gordon_deployed_twice",
            detail: format!(
                "GORDON deployed {}× (seq {:?}) — one physical counter (the old UnitId::Gordon/BritishBoats_3_1 alias regression)",
                gordon_deploys.len(),
                gordon_deploys.iter().map(|(s, _)| *s).collect::<Vec<_>>()
            ),
        });
    }
    if !gordon_deploys.is_empty()
        && matches!(report.scenario.as_str(), "campaign" | "historical")
    {
        report.findings.push(Finding {
            severity: Severity::Error,
            code: "gordon_not_in_scenario",
            detail: format!(
                "GORDON deployed in {} — not in play (§9.113 Campaign / §9.211 Historical)",
                report.scenario
            ),
        });
    }

    // ---- §8.2: desertion math and exemptions ----
    if desertions.len() > 1 {
        report.findings.push(Finding {
            severity: Severity::Error,
            code: "desertion_rolled_twice",
            detail: format!(
                "{} desertion rolls in one game (§8.2: once per campaign)",
                desertions.len()
            ),
        });
    }
    for (seq, turn, roll, names) in &desertions {
        let expected = (*roll as usize * 3) / 2; // floor(1.5 × roll)
        if names.len() != expected {
            report.findings.push(Finding {
                severity: Severity::Error,
                code: "desertion_count_mismatch",
                detail: format!(
                    "seq {} T{}: roll {} → {} deserters, expected {} (§8.2: 1.5 × roll)",
                    seq, turn, roll, names.len(), expected
                ),
            });
        }
        for name in names {
            if DESERTION_EXEMPT.iter().any(|k| name.starts_with(k)) {
                report.findings.push(Finding {
                    severity: Severity::Error,
                    code: "desertion_exempt_unit",
                    detail: format!(
                        "seq {} T{}: {} may not desert (§8.2 exemption)",
                        seq, turn, name
                    ),
                });
            }
        }
    }

    // ---- §9.112/§9.113: reinforcement schedule ----
    if report.scenario == "campaign" {
        for (seq, turn, actor, entries) in &reinforcement_batches {
            if *turn > 4 {
                report.findings.push(Finding {
                    severity: Severity::Error,
                    code: "reinforcement_off_schedule",
                    detail: format!(
                        "seq {} T{}: {} reinforcement batch after turn 4 (§9.112/§9.113 waves are turns 1-4)",
                        seq, turn, actor
                    ),
                });
            }
            if actor == "Dervish" {
                if let Some(tribes) = dervish_wave_tribes(*turn) {
                    for p in entries {
                        if let Some(tribe) = dervish_tribe_of(&p.label) {
                            if !tribes.contains(&tribe) {
                                report.findings.push(Finding {
                                    severity: Severity::Error,
                                    code: "reinforcement_wrong_wave",
                                    detail: format!(
                                        "seq {} T{}: {} enters, but the §9.112 wave is {:?}",
                                        seq, turn, tribe, tribes
                                    ),
                                });
                            }
                        } else if !dervish_wave_leaders(*turn).is_some_and(|ls| {
                            ls.iter().any(|l| p.label.starts_with(l))
                        }) {
                            report.findings.push(Finding {
                                severity: Severity::Warning,
                                code: "reinforcement_unclassified",
                                detail: format!(
                                    "seq {} T{}: {} is neither a wave tribe nor a wave leader",
                                    seq, turn, p.label
                                ),
                            });
                        }
                    }
                }
            } else {
                // §9.113: at most three gunboats per turn, across batches.
                let gunboats: usize = reinforcement_batches
                    .iter()
                    .filter(|(_, t, a, _)| *t == *turn && a == actor)
                    .flat_map(|(_, _, _, entries)| entries.iter())
                    .filter(|p| p.label.starts_with("Gunboat"))
                    .count();
                if gunboats > 3 {
                    report.findings.push(Finding {
                        severity: Severity::Error,
                        code: "reinforcement_gunboat_quota",
                        detail: format!(
                            "T{}: {} gunboats entered — §9.113 caps three per turn",
                            turn, gunboats
                        ),
                    });
                }
            }
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLEAN_LOG: &str = "\
GAME LOG — Remember Gordon! (The Battle of Omdurman)
scenario:        campaign
seed:            0x2a
agents:          ae=random dervish=random
rules_version:   Manual §1–§10

[1] T1 Setup Dervish  DeployUnit [Dervish] KhalifaAbdullah at (1,1)
[2] T1 Setup Dervish  DeployUnit [Dervish] Dervish Fort #1 at (2,2)
[3] T1 Setup Dervish  ConfirmSetupReady (Dervish ready)
[4] T1 Setup AngloEgyptian  AdvancePhase (end Setup)
[5] T1 Movement AngloEgyptian  PlaceReinforcements: Gunboat Old #1 at (5,5), 1B First Btn #1 at (6,6)
[6] T1 Offensive Fire AngloEgyptian  fire 1B First Btn #1 at (2,2) [roll 9]
      → FireResolved at (2,2): Dervish Fort #1 roll 9 (+1) = 10 → Eliminate(2) [§6.22 §6.24 §6.62]  [event 6]
      → HexVacatedByCombat at (2,2): 1B First Btn #1 may advance [§6.82 §6.62]  [event 6]
[7] T1 Offensive Fire AngloEgyptian  AdvanceAfterCombat 1B First Btn #1: (6,6) → (2,2)
[8] T1 Melee AngloEgyptian  AdvancePhase (end Melee)
=== Turn 1 complete (6:00 am, Day) — 1 fire, 0 melee, 1 eliminations, 1 advances; VP AE 2 / Dervish 0 ===
=== GAME OVER ===  result: —
victory: AE 2 / Dervish 0
";

    #[test]
    fn clean_log_has_no_findings() {
        let report = audit_log(CLEAN_LOG);
        assert!(!report.has_errors(), "{report}");
        assert_eq!(report.events_scanned, 8);
    }

    #[test]
    fn advance_without_window_is_an_error() {
        let log = CLEAN_LOG.replace(
            "[7] T1 Offensive Fire AngloEgyptian  AdvanceAfterCombat 1B First Btn #1: (6,6) → (2,2)",
            "[7] T1 Offensive Fire AngloEgyptian  AdvanceAfterCombat 1B First Btn #1: (6,6) → (9,9)",
        );
        let report = audit_log(&log);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "advance_without_window" && f.severity == Severity::Error));
    }

    #[test]
    fn same_hex_targeted_twice_is_a_warning() {
        // Two attacks on the same tribal-held hex within one subphase —
        // no legal exemption visible in the log (unlike a §6.61/§6.62
        // gunboat/fort target or a §6.42 subphase split).
        let log = "\
scenario:        campaign

[1] T1 Offensive Fire AngloEgyptian  fire 1B First Btn #1 at (8,8) [roll 3]
      → FireResolved at (8,8): Baggara #1 roll 3 (+1) = 4 → Disrupt [§6.22 §6.24 §6.23]  [event 1]
[2] T1 Offensive Fire AngloEgyptian  fire 1B Second Btn #1 at (8,8) [roll 4]
      → FireResolved at (8,8): Baggara #1 roll 4 (+1) = 5 → Disrupt [§6.22 §6.24 §6.23]  [event 2]
[3] T1 Offensive Fire AngloEgyptian  AdvancePhase (end Offensive Fire)
";
        let report = audit_log(log);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "hex_targeted_twice_per_phase" && f.severity == Severity::Warning));
    }

    #[test]
    fn gunboat_target_hex_may_be_attacked_repeatedly() {
        // §6.61: a gunboat target is exempt from §6.14's fired-at-once rule —
        // repeated attacks on the hex are legal, the scanner must not warn.
        let log = "\
scenario:        campaign

[1] T1 Offensive Fire Dervish  fire Dervish Artillery #1 at (6,11) [roll 7]
      → FireResolved at (6,11): Gunboat Old #1 roll 7 (+0) = 7 → NoEffect [§6.22 §6.24 §6.61]  [event 1]
[2] T1 Offensive Fire Dervish  fire Dervish Fort #5 at (6,11) [roll 3]
      → FireResolved at (6,11): Gunboat Old #1 roll 3 (+0) = 3 → NoEffect [§6.22 §6.24 §6.61]  [event 2]
[3] T1 Offensive Fire Dervish  AdvancePhase (end Offensive Fire)
";
        let report = audit_log(log);
        assert!(!report
            .findings
            .iter()
            .any(|f| f.code == "hex_targeted_twice_per_phase"));
    }

    #[test]
    fn refire_across_maxim_subphase_bridge_is_legal() {
        // §6.42: the Direct→Maxim/Howitzer subphase transition (an
        // AdvancePhase within "Offensive Fire") allows firing at units
        // already fired at in Direct Fire.
        let log = "\
scenario:        campaign

[1] T1 Offensive Fire AngloEgyptian  fire 1B First Btn #1 at (8,8) [roll 3]
      → FireResolved at (8,8): Baggara #1 roll 3 (+1) = 4 → Disrupt [§6.22 §6.24 §6.23]  [event 1]
[2] T1 Offensive Fire AngloEgyptian  AdvancePhase (end Offensive Fire)
[3] T1 Offensive Fire AngloEgyptian  fire Maxim #1 at (8,8) [roll 6]
      → FireResolved at (8,8): Baggara #1 roll 6 (+1) = 7 → NoEffect [§6.22 §6.42 §6.23]  [event 3]
[4] T1 Offensive Fire AngloEgyptian  AdvancePhase (end Offensive Fire)
";
        let report = audit_log(log);
        assert!(!report
            .findings
            .iter()
            .any(|f| f.code == "hex_targeted_twice_per_phase"));
    }

    #[test]
    fn campaign_ae_setup_deploy_is_an_error() {
        let log = CLEAN_LOG.replace(
            "[2] T1 Setup Dervish  DeployUnit [Dervish] Dervish Fort #1 at (2,2)",
            "[2] T1 Setup Dervish  DeployUnit [AngloEgyptian] 1B First Btn #1 at (9,9)\n[3] T1 Setup Dervish  DeployUnit [Dervish] Dervish Fort #1 at (2,2)",
        );
        let report = audit_log(&log);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "campaign_ae_setup_deploy" && f.severity == Severity::Error));
    }

    #[test]
    fn campaign_noninitial_dervish_is_an_error() {
        let log = CLEAN_LOG.replace(
            "DeployUnit [Dervish] KhalifaAbdullah at (1,1)",
            "DeployUnit [Dervish] Baggara #1 at (1,1)",
        );
        let report = audit_log(&log);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "campaign_noninitial_dervish_setup"));
    }

    #[test]
    fn historical_not_in_play_is_an_error() {
        let log = "scenario:        historical\n\n[1] T1 Setup AngloEgyptian  DeployUnit [AngloEgyptian] Gordon at (3,3)\n";
        let report = audit_log(&log);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "historical_not_in_play"));
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "gordon_not_in_scenario"));
    }

    #[test]
    fn double_gordon_is_an_error() {
        let log = "scenario:        fall of khartoum\n\n[1] T1 Setup AngloEgyptian  DeployUnit Gordon at (1,1)\n[2] T1 Setup AngloEgyptian  DeployUnit Gordon at (2,2)\n";
        let report = audit_log(&log);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "gordon_deployed_twice"));
    }

    #[test]
    fn desertion_math_is_checked() {
        let good = "scenario: campaign\n\n[9] T7 Movement Dervish  DervishDesertion roll 4: Baggara #1, Baggara #2, Kehena #1, Kehena #2, Taiasha #1, Taiasha #2 desert\n";
        assert!(!audit_log(good).has_errors());
        let short = good.replace(
            "Baggara #1, Baggara #2, Kehena #1, Kehena #2, Taiasha #1, Taiasha #2",
            "Baggara #1",
        );
        assert!(audit_log(&short)
            .findings
            .iter()
            .any(|f| f.code == "desertion_count_mismatch"));
        let exempt = good.replace("Baggara #1,", "KhalifaAbdullah,");
        assert!(audit_log(&exempt)
            .findings
            .iter()
            .any(|f| f.code == "desertion_exempt_unit"));
    }

    #[test]
    fn reinforcement_schedule_is_checked() {
        // Hadendowa (turn-2 wave) entering on turn 1 -> wrong wave.
        let wrong_wave = CLEAN_LOG.replace(
            "PlaceReinforcements: Gunboat Old #1 at (5,5), 1B First Btn #1 at (6,6)",
            "PlaceReinforcements: Hadendowa #1 at (5,5)",
        );
        // The actor is AngloEgyptian; swap it to Dervish for the wave check.
        let wrong_wave = wrong_wave.replace(
            "[5] T1 Movement AngloEgyptian  PlaceReinforcements",
            "[5] T1 Movement Dervish  PlaceReinforcements",
        );
        assert!(audit_log(&wrong_wave)
            .findings
            .iter()
            .any(|f| f.code == "reinforcement_wrong_wave"));

        // Turn-5 batch -> off schedule.
        let late = CLEAN_LOG.replace(
            "[5] T1 Movement AngloEgyptian  PlaceReinforcements",
            "[5] T5 Movement AngloEgyptian  PlaceReinforcements",
        );
        assert!(audit_log(&late)
            .findings
            .iter()
            .any(|f| f.code == "reinforcement_off_schedule"));

        // Four gunboats in one turn -> quota error.
        let quota = CLEAN_LOG.replace(
            "PlaceReinforcements: Gunboat Old #1 at (5,5), 1B First Btn #1 at (6,6)",
            "PlaceReinforcements: Gunboat Old #1 at (5,5), Gunboat Old #2 at (5,6), Gunboat Old #3 at (5,7), Gunboat Old #4 at (5,8)",
        );
        assert!(audit_log(&quota)
            .findings
            .iter()
            .any(|f| f.code == "reinforcement_gunboat_quota"));
    }

    #[test]
    fn parses_both_label_generations() {
        assert_eq!(UnitLabel::parse("[Dervish] Baggara #3").0, "Baggara");
        assert_eq!(UnitLabel::parse("Baggara").0, "Baggara");
        assert_eq!(UnitLabel::parse("[AngloEgyptian] 1B First Btn #2").0, "1B First Btn");
        assert_eq!(UnitLabel::parse("Dervish Fort").0, "Dervish Fort");
    }
}
