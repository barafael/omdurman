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
        if s.starts_with('[')
            && let Some((_, rest)) = s.split_once(']') {
                s = rest.trim().to_string();
            }
        // Strip a trailing ` #<digits>` disambiguator.
        if let Some((head, idx)) = s.rsplit_once(" #")
            && !idx.is_empty() && idx.chars().all(|c| c.is_ascii_digit()) {
                s = head.trim().to_string();
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
        .map(strip_faction_prefix)
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
    let mut vacated: HashMap<(u8, String), Vec<usize>> = HashMap::new();
    for line in text.lines() {
        if let Some((hex, event)) = parse_vacated(line)
            && let Some(&turn) = seq_turn.get(&event) {
                vacated.entry((turn, hex)).or_default().push(event);
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
        if e.phase == "Setup"
            && let Some(p) = parse_deploy(&e.text) {
                if p.label == "Gordon" {
                    gordon_deploys.push((e.seq, p.label.clone()));
                }
                setup_deploys.push((e.actor.clone(), p));
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
        // A hex can be vacated several times in one turn (e.g. a retreat
        // window, then a later fire elimination re-emptying it); the advance
        // is legal if ANY window for that hex opened before it.
        let opened_before = vacated
            .get(&(a.turn, a.hex.clone()))
            .is_some_and(|events| events.iter().any(|&opened| opened < a.seq));
        if !opened_before {
            report.findings.push(Finding {
                severity: Severity::Error,
                code: "advance_without_window",
                detail: format!(
                    "seq {}: {} advanced into {} on T{} with no HexVacatedByCombat observation for that hex this turn (§6.82/§7.6)",
                    a.seq, a.unit, a.hex, a.turn
                ),
            });
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
        "fall of khartoum" => {
            // §9.321/§9.322 (+§9.344): replay setup (deploys minus the
            // pull-backs) and validate the order of battle -- which unit
            // types exist and their printed counts.
            use std::collections::BTreeMap;
            let mut on_board: BTreeMap<String, usize> = BTreeMap::new();
            for e in &events {
                if e.phase != "Setup" {
                    break;
                }
                if let Some(p) = parse_deploy(&e.text) {
                    *on_board.entry(p.label).or_default() += 1;
                } else if let Some(rest) = e.text.strip_prefix("RemoveDeployedUnit ") {
                    let label = UnitLabel::parse(rest.split(" (").next().unwrap_or(rest)).0;
                    if let Some(n) = on_board.get_mut(&label) {
                        *n = n.saturating_sub(1);
                    }
                }
            }
            let count = |k: &str| -> usize {
                on_board
                    .iter()
                    .filter(|(label, _)| {
                        label.starts_with(k)
                            && (label.len() == k.len()
                                || label.as_bytes()[k.len()] == b' '
                                || label.as_bytes()[k.len()] == b'#')
                    })
                    .map(|(_, &n)| n)
                    .sum()
            };
            // Dervish (§9.322): exactly these types, at these counts. Forts:
            // §9.344's single North Fort only.
            for (kind, cap) in [
                ("Mulazmin", 32),
                ("Hadendowa", 2),
                ("Kehena", 6),
                ("Degheim", 5),
                ("Dervish Artillery", 3),
                ("Dervish Fort", 1),
            ] {
                let n = count(kind);
                if n > cap {
                    report.findings.push(Finding {
                        severity: Severity::Error,
                        code: "fok_order_of_battle",
                        detail: format!(
                            "setup deployed {n}× {kind} — §9.322/§9.344 allows at most {cap}"
                        ),
                    });
                }
            }
            for forbidden in [
                "Baggara",
                "Jaalin",
                "Danagla",
                "Taiasha",
                "IsaZachneih",
                "Dervish Gunboat",
                "KhalifaAbdullah",
                "OsmanDigna",
                "Yakub",
                "Sherif",
                "AliWadHelu",
                "SheikElDin",
            ] {
                let n = count(forbidden);
                if n > 0 {
                    report.findings.push(Finding {
                        severity: Severity::Error,
                        code: "fok_order_of_battle",
                        detail: format!(
                            "setup deployed {n}× {forbidden} — not in the §9.322 Dervish order of battle"
                        ),
                    });
                }
            }
            // British (§9.321): old gunboats only (≤2), one artillery, the
            // battalion counts per nationality, GORDON fixed.
            if count("Gunboat Named") > 0 {
                report.findings.push(Finding {
                    severity: Severity::Error,
                    code: "fok_order_of_battle",
                    detail: "named gunboats deployed — §9.321 allows only two old-style gunboats".to_string(),
                });
            }
            if count("Gunboat") > 2 {
                report.findings.push(Finding {
                    severity: Severity::Error,
                    code: "fok_order_of_battle",
                    detail: format!("{} gunboats deployed — §9.321 allows two", count("Gunboat")),
                });
            }
            if count("Artillery") > 1 {
                report.findings.push(Finding {
                    severity: Severity::Error,
                    code: "fok_order_of_battle",
                    detail: format!(
                        "{} Anglo-Egyptian artillery deployed — §9.321 allows one",
                        count("Artillery")
                    ),
                });
            }
            for (prefix, cap, what) in [
                ("Cavalry", 0, "cavalry"),
                ("Maxim", 0, "Maxims"),
                ("Royal Engineers", 0, "the Royal Engineers"),
                ("Camel Corps", 0, "the Camel Corps"),
            ] {
                if count(prefix) > cap {
                    report.findings.push(Finding {
                        severity: Severity::Error,
                        code: "fok_order_of_battle",
                        detail: format!("{what} deployed — not in the §9.321 garrison"),
                    });
                }
            }
            let infantry = |nat: u8| -> usize {
                on_board
                    .iter()
                    .filter(|(label, _)| {
                        let b = label.as_bytes();
                        b.len() >= 2 && b[0].is_ascii_digit() && b[1] == nat
                    })
                    .map(|(_, &n)| n)
                    .sum()
            };
            for (nat, cap, name) in [
                (b'B', 2, "British"),
                (b'E', 3, "Egyptian"),
                (b'S', 4, "Sudanese"),
                (b'F', 4, "Friendlies"),
            ] {
                let n = infantry(nat);
                if n > cap {
                    report.findings.push(Finding {
                        severity: Severity::Error,
                        code: "fok_order_of_battle",
                        detail: format!(
                            "{n} {name} battalions deployed — §9.321 allows {cap}"
                        ),
                    });
                }
            }
            for leader in ["Kitchener", "Gatacre", "Hunter", "Wauchope", "Lyttelton", "Collinson"] {
                if count(leader) > 0 {
                    report.findings.push(Finding {
                        severity: Severity::Error,
                        code: "fok_order_of_battle",
                        detail: format!(
                            "{leader} deployed — §9.321 garrison has no leaders but GORDON"
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

    // ---- Board-state reconstruction: §5.51/§5.52 stacking, §7.1 enemy
    // cohabitation, §5.11 MP arithmetic ----
    let lines: Vec<&str> = text.lines().collect();
    audit_occupancy(&events, &lines, &mut report);

    report
}

// ---------------------------------------------------------------------------
// Board-state reconstruction (§5.51-5.53, §7.1, §5.11 checks)
// ---------------------------------------------------------------------------

/// The §5.51 counted/exempt classification from a rendered label.
fn tracked_kind(label: &str) -> &'static str {
    if label.starts_with("Gunboat") || label.starts_with("Dervish Gunboat") {
        "gunboat"
    } else if matches!(
        label,
        "KhalifaAbdullah"
            | "Yakub"
            | "Sherif"
            | "AliWadHelu"
            | "OsmanDigna"
            | "SheikElDin"
            | "Kitchener"
            | "Gatacre"
            | "Hunter"
            | "Gordon"
            | "Wauchope"
            | "Lyttelton"
            | "Collinson"
            | "Cameron"
            | "Broadwood"
            | "Mahdi"
    ) {
        "leader"
    } else {
        "counted"
    }
}

fn tracked_faction(label: &str) -> Option<&'static str> {
    if is_ae_label(label) {
        Some("ae")
    } else if is_dervish_label(label) {
        Some("dervish")
    } else {
        None
    }
}

/// Replay the log's unit events and verify stacking/cohabitation/MP
/// arithmetic. Findings are Errors: each event that places or moves a unit
/// passed the engine's `check_stacking` at apply time, so any violation
/// visible in the reconstructed state is an engine bug.
fn audit_occupancy(events: &[EventLine], lines: &[&str], report: &mut AuditReport) {
    use std::collections::BTreeMap;
    let mut units: BTreeMap<String, (i32, i32)> = BTreeMap::new();
    // Per-(turn, label) cumulative MP spent, verified against the rendered
    // `mp s/t` totals (§5.11 arithmetic).
    let mut mp_spent: BTreeMap<(u8, String), i16> = BTreeMap::new();
    let mut reported: std::collections::BTreeSet<(u8, &'static str, (i32, i32))> =
        std::collections::BTreeSet::new();
    // Dervish reinforcements whose (terrain) entry cost the log does not
    // render: their first MoveUnit cumulative total is ground truth.
    let mut dervish_entry_pending: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();

    // Elimination observations follow their event line; apply them before
    // validating the state by replaying observation lines as they appear.
    // `losses:` inside FireResolved/MeleeResolved and the UnitEliminated /
    // LeaderKilled / GordonEliminated observations all remove units.
    fn remove_units(units: &mut BTreeMap<String, (i32, i32)>, names: &[String]) {
        for name in names {
            if units.remove(name).is_none() {
                // Fall back to a unique label-prefix match (an observation
                // rendering without the `#n` disambiguator).
                let hits: Vec<String> = units
                    .keys()
                    .filter(|k| {
                        k.split_once(" #").map(|(h, _)| h) == Some(name.as_str())
                            || name.starts_with(k.as_str())
                    })
                    .cloned()
                    .collect();
                if hits.len() == 1 {
                    units.remove(&hits[0]);
                }
            }
        }
    }

    fn parse_losses(body: &str) -> Vec<String> {
        body.split("losses: ")
            .nth(1)
            .map(|tail| {
                tail.split(" [")
                    .next()
                    .unwrap_or("")
                    .split(", ")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn check_hexes(
        units: &BTreeMap<String, (i32, i32)>,
        turn: u8,
        report: &mut AuditReport,
        reported: &mut std::collections::BTreeSet<(u8, &'static str, (i32, i32))>,
    ) {
        use std::collections::BTreeMap;
        let mut by_hex: BTreeMap<(i32, i32), Vec<&String>> = BTreeMap::new();
        for (label, hex) in units.iter() {
            by_hex.entry(*hex).or_default().push(label);
        }
        for (hex, labels) in by_hex {
            // §5.51: at most four counted units per hex.
            let counted: Vec<&String> = labels
                .iter()
                .filter(|l| tracked_kind(l) == "counted")
                .copied()
                .collect();
            if counted.len() > 4 && reported.insert((turn, "overstack", hex)) {
                report.findings.push(Finding {
                    severity: Severity::Error,
                    code: "hex_overstack",
                    detail: format!(
                        "T{} hex ({},{}): {} counted units stack ({:?}) — §5.51 caps four",
                        turn,
                        hex.0,
                        hex.1,
                        counted.len(),
                        counted
                    ),
                });
            }
            // §5.52: no two Dervish tribes in one hex.
            let tribes: Vec<&str> = labels
                .iter()
                .filter_map(|l| dervish_tribe_of(l))
                .collect();
            let mut distinct = tribes.clone();
            distinct.sort_unstable();
            distinct.dedup();
            if distinct.len() > 1 && reported.insert((turn, "tribe_mix", hex)) {
                report.findings.push(Finding {
                    severity: Severity::Error,
                    code: "hex_tribe_mix",
                    detail: format!(
                        "T{} hex ({},{}): tribes {:?} stack together — §5.52 forbids mixing",
                        turn,
                        hex.0,
                        hex.1,
                        distinct
                    ),
                });
            }
            // §7.1: friendly and enemy units may never cohabit a hex.
            let factions: Vec<&str> = labels
                .iter()
                .filter_map(|l| tracked_faction(l))
                .collect();
            if factions.contains(&"ae") && factions.contains(&"dervish")
                && reported.insert((turn, "enemy_cohabit", hex))
            {
                report.findings.push(Finding {
                    severity: Severity::Error,
                    code: "hex_enemy_cohabitation",
                    detail: format!(
                        "T{} hex ({},{}): Anglo-Egyptian and Dervish units share the hex ({:?}) — §7.1 (movement may only end adjacent)",
                        turn,
                        hex.0,
                        hex.1,
                        labels
                    ),
                });
            }
        }
    }

    let mut event_iter = events.iter().peekable();
    for line in lines {
        let trimmed = line.trim_start();
        // Observation lines immediately after the current event.
        if let Some(rest) = trimmed.strip_prefix("→ ") {
            if let Some(body) = rest.strip_prefix("FireResolved at ") {
                remove_units(&mut units, &parse_losses(body));
            } else if let Some(body) = rest.strip_prefix("MeleeResolved at ") {
                remove_units(&mut units, &parse_losses(body));
            } else if let Some(body) = rest.strip_prefix("UnitEliminated: ") {
                let name: String = body
                    .split(" eliminated")
                    .next()
                    .unwrap_or(body)
                    .split(" lost with transport")
                    .next()
                    .unwrap_or(body)
                    .trim()
                    .to_string();
                remove_units(&mut units, &[name]);
            } else if let Some(body) = rest.strip_prefix("LeaderKilled: ") {
                let name = body.split(" (killed by").next().unwrap_or(body).trim();
                remove_units(&mut units, &[name.to_string()]);
            } else if trimmed.starts_with("GordonEliminated:") {
                remove_units(&mut units, &["Gordon".to_string(), "Gen. Gordon".to_string()]);
            }
            continue;
        }
        // The next event line (if this line is one).
        let Some(e) = event_iter.peek_if_this_is_event(line) else {
            continue;
        };
        let text = &e.text;
        // Apply the event to the reconstructed board.
        if let Some(rest) = text.strip_prefix("DeployUnit ") {
            if let Some((label, hex)) = rest.split_once(" at ")
                && let Some(h) = parse_hex_pair(hex) {
                    units.insert(strip_faction_prefix(label), h);
                }
        } else if let Some(rest) = text.strip_prefix("PlaceReinforcements: ") {
            for entry in rest.split(", ") {
                if let Some((label, hex)) = entry.split_once(" at ")
                    && let Some(h) = parse_hex_pair(hex) {
                        let label = strip_faction_prefix(label);
                        units.insert(label.clone(), h);
                        // §9.112/§9.113: entering the map costs movement
                        // points, recorded as MP spent by the engine. Seed
                        // the tracker so the unit's first MoveUnit of the
                        // turn validates against the entry cost. AE pays 1
                        // (gunboats' first hex) or 8 (Friendlies via Abu
                        // Alim); the Dervish pay their entry hex's terrain
                        // cost, unknown here -- seed 0 but mark the unit so
                        // its first rendered total is taken as ground truth.
                        let entry_cost: i16 = if is_friendlies_label(&label) {
                            8
                        } else if is_dervish_label(&label) {
                            0
                        } else {
                            1
                        };
                        mp_spent.insert((e.turn, label.clone()), entry_cost);
                        if entry_cost == 0 {
                            // First MoveUnit total becomes authoritative:
                            // record it as pre-spent so the next step's
                            // arithmetic starts from the rendered value.
                            dervish_entry_pending.insert(label);
                        }
                    }
            }
        } else if let Some(rest) = text.strip_prefix("RemoveDeployedUnit ") {
            let label = rest.split(" (").next().unwrap_or(rest);
            remove_units(&mut units, &[label.to_string()]);
        } else if let Some(rest) = text.strip_prefix("DervishDesertion roll ") {
            if let Some((_, names)) = rest.split_once(": ") {
                let names: Vec<String> = names
                    .strip_suffix(" desert")
                    .unwrap_or(names)
                    .split(", ")
                    .map(strip_faction_prefix)
                    .collect();
                remove_units(&mut units, &names);
            }
        } else if let Some(rest) = text.strip_prefix("MoveUnit ") {
            // MoveUnit <label>: (a,b) → (c,d) (N MP) mp s/t [via ...]
            if let Some((label, tail)) = rest.split_once(": ")
                && let Some((from, to)) = parse_move_pair(tail) {
                    let label = strip_faction_prefix(label);
                    if let Some(h) = units.get_mut(&label)
                        && *h == from {
                            *h = to;
                        }
                    // §5.11 arithmetic: rendered cumulative == previous + step.
                    if let Some((cost, shown)) = parse_mp(tail) {
                        let key = (e.turn, label.clone());
                        let prev = mp_spent.get(&key).copied().unwrap_or(0);
                        // A Dervish reinforcement's (terrain) entry cost is
                        // not rendered: its first MoveUnit total of the turn
                        // is ground truth, not an arithmetic error.
                        let first_after_entry = dervish_entry_pending.remove(&label);
                        if !first_after_entry && shown != prev + cost {
                            report.findings.push(Finding {
                                severity: Severity::Error,
                                code: "mp_arithmetic",
                                detail: format!(
                                    "seq {} T{} {}: rendered mp {shown} but {prev} + {cost} were spent (§5.11)",
                                    e.seq, e.turn, label
                                ),
                            });
                        }
                        mp_spent.insert(key, shown);
                    }
                }
        } else if let Some(rest) = text.strip_prefix("AdvanceAfterCombat ") {
            if let Some((from, to)) = parse_advance_pair(rest) {
                let label = strip_faction_prefix(rest.split(':').next().unwrap_or(rest));
                let label = label.trim().to_string();
                if let Some(h) = units.get_mut(&label)
                    && *h == from {
                        *h = to;
                    }
            }
        } else if let Some(rest) = text.strip_prefix("RetreatBeforeMelee ")
            && let Some((from, to)) = parse_advance_pair(rest) {
                let label = strip_faction_prefix(rest.split(':').next().unwrap_or(rest));
                let label = label.trim().to_string();
                if let Some(h) = units.get_mut(&label)
                    && *h == from {
                        *h = to;
                    }
            }
        check_hexes(&units, e.turn, report, &mut reported);
    }
}

fn parse_hex_pair(raw: &str) -> Option<(i32, i32)> {
    let inner = raw.trim().trim_start_matches('(').trim_end_matches(')');
    let (q, r) = inner.split_once(',')?;
    Some((q.trim().parse().ok()?, r.trim().parse().ok()?))
}

/// `(a,b) → (c,d)` prefix of a move/advance/retreat line body.
fn parse_move_pair(tail: &str) -> Option<((i32, i32), (i32, i32))> {
    let (from, rest) = tail.split_once(" → ")?;
    let (to, _) = rest.split_once(" (")?;
    Some((parse_hex_pair(from)?, parse_hex_pair(to)?))
}

/// `<from>: (a,b) → (c,d)` — old-format advance lines have no from prefix.
fn parse_advance_pair(rest: &str) -> Option<((i32, i32), (i32, i32))> {
    let body = rest.split_once(" → ").map(|(_, t)| t).unwrap_or(rest);
    let to = body.split(" (").next()?.trim();
    let to = parse_hex_pair(to)?;
    let from = rest
        .split_once(':')
        .and_then(|(f, _)| f.rsplit_once('('))
        .and_then(|(_, f)| parse_hex_pair(&format!("({f}")));
    Some((from?, to))
}

/// `(N MP) mp s/t` — the step cost and the rendered cumulative spend.
fn parse_mp(tail: &str) -> Option<(i16, i16)> {
    // Anchor on " MP)" (a " (" also matches the arrow's destination hex).
    let cost = tail
        .split_once(" MP)")?
        .0
        .rsplit(' ')
        .next()?
        .trim_start_matches('(')
        .parse()
        .ok()?;
    let mp = tail.split_once(" mp ")?.1.split_whitespace().next()?;
    let (spent, _allowance) = mp.split_once('/')?;
    Some((cost, spent.parse().ok()?))
}

/// Event-iterator helper: advance only when this log line *is* the next
/// event (identified by seq prefix), returning it.
trait PeekIfEvent<'a, I>
where
    I: Iterator<Item = &'a EventLine>,
{
    fn peek_if_this_is_event(&mut self, line: &str) -> Option<&'a EventLine>;
}
impl<'a, I> PeekIfEvent<'a, I> for std::iter::Peekable<I>
where
    I: Iterator<Item = &'a EventLine>,
{
    fn peek_if_this_is_event(&mut self, line: &str) -> Option<&'a EventLine> {
        let next = self.peek()?;
        let prefix = format!("[{}] ", next.seq);
        if line.starts_with(&prefix) {
            let e = self.next();
            return e;
        }
        // Old-format logs render `[seq]` followed by two spaces before T.
        let alt = format!("[{}]", next.seq);
        if line.trim_start().starts_with(&alt) {
            return self.next();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLEAN_LOG: &str = "\
GAME LOG — Remember Gordon! (The Battle of Omdurman)
scenario:        campaign
seed:            0x2a
agents:          ae=random dervish=random

[1] T1 Setup Dervish  DeployUnit [Dervish] KhalifaAbdullah at (1,1)
[2] T1 Setup Dervish  DeployUnit [Dervish] Dervish Fort #1 at (2,2)
[3] T1 Setup Dervish  ConfirmSetupReady (Dervish ready)
[4] T1 Setup AngloEgyptian  AdvancePhase (end Setup)
[5] T1 Movement AngloEgyptian  PlaceReinforcements: Gunboat Old #1 at (5,5), 1B First Btn #1 at (6,6)
[6] T1 Offensive Fire AngloEgyptian  fire 1B First Btn #1 at (2,2) [roll 9]
      → FireResolved at (2,2): Dervish Fort #1 roll 9 (+1) = 10 → Eliminate(2); losses: Dervish Fort #1 [§6.22 §6.24 §6.62]  [event 6]
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
        let report = audit_log(log);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "historical_not_in_play"));
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "gordon_not_in_scenario"));
    }

    // §9.322/§9.344: Dervish fort counters play no role in FoK beyond the
    // single fixed North Fort; no gunboats, no non-entry tribes.
    #[test]
    fn fok_forts_and_tribes_are_checked() {
        let bad = "\
scenario:        fall of khartoum

[1] T1 Setup Dervish  DeployUnit [Dervish] Dervish Fort #1 at (10,10)
[2] T1 Setup Dervish  DeployUnit [Dervish] Dervish Fort #2 at (11,10)
[3] T1 Setup Dervish  DeployUnit [Dervish] Dervish Gunboat #1 at (12,10)
[4] T1 Setup Dervish  DeployUnit [Dervish] Baggara #1 at (13,10)
[5] T1 Setup Dervish  DeployUnit [Dervish] Hadendowa #1 at (14,10)
[6] T1 Setup Dervish  DeployUnit [Dervish] Hadendowa #2 at (14,11)
[7] T1 Setup Dervish  DeployUnit [Dervish] Hadendowa #3 at (14,12)
";
        let report = audit_log(bad);
        let codes: Vec<&str> = report
            .findings
            .iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains(&"fok_order_of_battle"), "{report}");
        assert!(report
            .findings
            .iter()
            .any(|f| f.detail.contains("§9.344") && f.detail.contains("Dervish Fort")));
        assert!(report
            .findings
            .iter()
            .any(|f| f.detail.contains("Dervish Gunboat")));
        assert!(report
            .findings
            .iter()
            .any(|f| f.detail.contains("Baggara")));
        assert!(report
            .findings
            .iter()
            .any(|f| f.detail.contains("Hadendowa")));

        // The legal maximum is silent: one fort, two Hadendowa.
        let good = "\
scenario:        fall of khartoum

[1] T1 Setup Dervish  DeployUnit [Dervish] Dervish Fort #1 at (10,10)
[2] T1 Setup Dervish  DeployUnit [Dervish] Hadendowa #1 at (14,10)
[3] T1 Setup Dervish  DeployUnit [Dervish] Hadendowa #2 at (14,11)
";
        assert!(!audit_log(good)
            .findings
            .iter()
            .any(|f| f.code == "fok_order_of_battle"));
    }

    // §9.321: the British garrison counts.
    #[test]
    fn fok_british_garrison_is_checked() {
        let bad = "\
scenario:        fall of khartoum

[1] T1 Setup AngloEgyptian  DeployUnit [AngloEgyptian] Gunboat Old #1 at (1,1)
[2] T1 Setup AngloEgyptian  DeployUnit [AngloEgyptian] Gunboat Old #2 at (1,2)
[3] T1 Setup AngloEgyptian  DeployUnit [AngloEgyptian] Gunboat Old #3 at (1,3)
[4] T1 Setup AngloEgyptian  DeployUnit [AngloEgyptian] Artillery #1 at (2,1)
[5] T1 Setup AngloEgyptian  DeployUnit [AngloEgyptian] Maxim #1 at (3,1)
[6] T1 Setup AngloEgyptian  DeployUnit [AngloEgyptian] 1B First Btn #1 at (4,1)
[7] T1 Setup AngloEgyptian  DeployUnit [AngloEgyptian] 1B Second Btn #1 at (4,2)
[8] T1 Setup AngloEgyptian  DeployUnit [AngloEgyptian] 1B Third Btn #1 at (4,3)
";
        let report = audit_log(bad);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "fok_order_of_battle" && f.detail.contains("gunboats")));
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "fok_order_of_battle" && f.detail.contains("Maxims")));
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "fok_order_of_battle" && f.detail.contains("British battalions")));
    }

    #[test]
    fn double_gordon_is_an_error() {
        let log = "scenario:        fall of khartoum\n\n[1] T1 Setup AngloEgyptian  DeployUnit Gordon at (1,1)\n[2] T1 Setup AngloEgyptian  DeployUnit Gordon at (2,2)\n";
        let report = audit_log(log);
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

    // ---- Board-state reconstruction (§5.51/§5.52/§7.1/§5.11) ----

    #[test]
    fn overstack_is_an_error() {
        let log = "\
scenario:        campaign

[1] T1 Setup Dervish  DeployUnit [Dervish] Baggara #1 at (1,1)
[2] T1 Setup Dervish  DeployUnit [Dervish] Baggara #2 at (1,1)
[3] T1 Setup Dervish  DeployUnit [Dervish] Baggara #3 at (1,1)
[4] T1 Setup Dervish  DeployUnit [Dervish] Baggara #4 at (1,1)
[5] T1 Setup Dervish  DeployUnit [Dervish] Baggara #5 at (1,1)
";
        let report = audit_log(log);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "hex_overstack" && f.severity == Severity::Error));
    }

    #[test]
    fn leaders_and_gunboats_do_not_count_toward_the_limit() {
        let log = "\
scenario:        campaign

[1] T1 Setup Dervish  DeployUnit [Dervish] Baggara #1 at (1,1)
[2] T1 Setup Dervish  DeployUnit [Dervish] Baggara #2 at (1,1)
[3] T1 Setup Dervish  DeployUnit [Dervish] Baggara #3 at (1,1)
[4] T1 Setup Dervish  DeployUnit [Dervish] Baggara #4 at (1,1)
[5] T1 Setup Dervish  DeployUnit [Dervish] Yakub at (1,1)
";
        let report = audit_log(log);
        assert!(!report
            .findings
            .iter()
            .any(|f| f.code == "hex_overstack"));
    }

    #[test]
    fn tribe_mix_is_an_error() {
        let log = "\
scenario:        campaign

[1] T1 Setup Dervish  DeployUnit [Dervish] Baggara #1 at (1,1)
[2] T1 Setup Dervish  DeployUnit [Dervish] Mulazmin #1 at (1,1)
";
        let report = audit_log(log);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "hex_tribe_mix" && f.severity == Severity::Error));
    }

    #[test]
    fn enemy_cohabitation_is_an_error() {
        // A move into an enemy-held hex: the engine must have rejected it
        // (§7.1); if the log shows it applied, that is a violation.
        let log = "\
scenario:        campaign

[1] T1 Setup Dervish  DeployUnit [Dervish] Baggara #1 at (1,1)
[2] T1 Setup Dervish  DeployUnit [AngloEgyptian] 1B First Btn #1 at (2,1)
[3] T1 Movement AngloEgyptian  MoveUnit 1B First Btn #1: (2,1) → (1,1) (1 MP) mp 1/8 via [(1,1)]
";
        let report = audit_log(log);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "hex_enemy_cohabitation" && f.severity == Severity::Error));
    }

    #[test]
    fn mp_arithmetic_is_checked() {
        // The second step renders cumulative 3 but 1 + 2 = 3... use a wrong
        // render: cumulative 5 after 1 + 2.
        let log = "\
scenario:        campaign

[1] T1 Setup Dervish  DeployUnit [Dervish] Baggara #1 at (1,1)
[2] T1 Movement Dervish  MoveUnit Baggara #1: (1,1) → (2,1) (1 MP) mp 1/9 via [(2,1)]
[3] T1 Movement Dervish  MoveUnit Baggara #1: (2,1) → (3,1) (2 MP) mp 5/9 via [(3,1)]
";
        let report = audit_log(log);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == "mp_arithmetic" && f.severity == Severity::Error));
        // And the consistent version is silent.
        let ok = log.replace("mp 5/9", "mp 3/9");
        assert!(!audit_log(&ok)
            .findings
            .iter()
            .any(|f| f.code == "mp_arithmetic"));
    }

    #[test]
    fn desertion_and_eliminations_clear_units() {
        // A hex holding five counted units drops to four when one is
        // eliminated by desertion -- no residual overstack finding.
        let log = "\
scenario:        campaign

[1] T1 Setup Dervish  DeployUnit [Dervish] Baggara #1 at (1,1)
[2] T1 Setup Dervish  DeployUnit [Dervish] Baggara #2 at (1,1)
[3] T1 Setup Dervish  DeployUnit [Dervish] Baggara #3 at (1,1)
[4] T1 Setup Dervish  DeployUnit [Dervish] Baggara #4 at (1,1)
[5] T1 Setup Dervish  DeployUnit [Dervish] Baggara #5 at (1,1)
[6] T9 Movement Dervish  DervishDesertion roll 1: Baggara #5 desert
";
        // Five stacked at [5] is still flagged (the stack existed before the
        // desertion); after the desertion the hex must not be re-flagged for
        // the *eliminating* event -- verify exactly one finding, not two.
        let report = audit_log(log);
        assert_eq!(
            report
                .findings
                .iter()
                .filter(|f| f.code == "hex_overstack")
                .count(),
            1,
            "deduped per (turn, hex)"
        );
    }

    #[test]
    fn parses_both_label_generations() {
        assert_eq!(UnitLabel::parse("[Dervish] Baggara #3").0, "Baggara");
        assert_eq!(UnitLabel::parse("Baggara").0, "Baggara");
        assert_eq!(UnitLabel::parse("[AngloEgyptian] 1B First Btn #2").0, "1B First Btn");
        assert_eq!(UnitLabel::parse("Dervish Fort").0, "Dervish Fort");
    }
}
