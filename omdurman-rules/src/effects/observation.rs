use super::*;

/// Why a unit was eliminated, surfaced via [`Observation::UnitEliminated`] so
/// the app can render appropriate flavour (dispatch slips, sounds, etc.).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ElimCause {
    /// Eliminated by fire combat (§6) or melee (§7).
    Combat,
    /// Eliminated by a demolition resolution (§6.53 / §6.63).
    Demolition,
    /// A unit loaded on a gunboat that was sunk or eliminated -- the unit is
    /// lost with the ship (§5.21, §10.12).
    LostWithTransport,
    /// GORDON eliminated at the palace (§9.346).
    GordonAtPalace,
    /// Anglo-Egyptian leader eliminated because all combat units in its hex
    /// were eliminated (orphan leader, §5.44).
    OrphanLeader,
}

impl std::fmt::Display for ElimCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElimCause::Combat => write!(f, "eliminated in combat"),
            ElimCause::Demolition => write!(f, "demolition (§6.53)"),
            ElimCause::LostWithTransport => write!(f, "lost with sunk transport"),
            ElimCause::GordonAtPalace => write!(f, "GORDON fallen at the Palace"),
            ElimCause::OrphanLeader => write!(f, "orphan leader eliminated"),
        }
    }
}

/// A side-channel signal emitted by `apply_effect` describing what happened,
/// for the app to translate into Bevy events (dispatch slips, sounds, camera
/// focus, VP animations).  These are *observations of state changes*, not the
/// changes themselves -- `apply_effect` mutates `GameState` synchronously
/// regardless of whether observations are drained.
///
/// Pushed by the engine onto [`GameState::observations`]; the app drains them
/// after each `apply_effect` call.  Serialized so that replay / late-join
/// produces the same observation stream (the user sees the full event flow
/// animate on replay, per project decision).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Observation {
    /// A unit was eliminated.  `vp_source` is `None` when no VP are awarded
    /// (e.g. fort elimination per §9.14: "No pts: eliminating forts").
    UnitEliminated {
        id: UnitId,
        cause: ElimCause,
        vp_source: Option<VpSource>,
    },
    /// A fort was destroyed (§6.53, §6.62, §7.6).
    FortDestroyed { id: UnitId, hex: HexCoord },
    /// A wall hexside was breached, or a breach *attempt* resolved short of
    /// the §6.63 threshold. `breached` distinguishes the two; `row` is the
    /// Combat Results Table row the attempt rolled on, so the log shows what
    /// the roll had to beat. (Demolitions (§6.53) have no CRT row -- they
    /// carry `None`.)
    WallBreached {
        hexside: HexsideRef,
        #[serde(default)]
        breached: bool,
        row: Option<FireFactorRow>,
        adjacent_eliminated: Option<UnitId>,
    },
    /// A named leader was killed in combat.
    LeaderKilled { id: UnitId, by: Player },
    /// GORDON was eliminated at the palace (§9.346).
    GordonEliminated { turn: GameTurnIndex },
    /// A "Friendlies" unit disembarked from its gunboat transport (§5.21).
    FriendliesDisembarked { unit_id: UnitId, at: HexCoord },
    /// A Royal Engineers demolition resolved (§6.53).
    DemolitionResolved {
        engineer_id: UnitId,
        target: DemolitionTarget,
        success: bool,
    },
    /// Victory points were awarded (§9.14).
    VictoryScored {
        source: VpSource,
        points: VictoryPoints,
        for_player: Player,
    },
    /// A fire attack resolved (§6). Carries the full attack, both die rolls
    /// (the raw roll and the modified one used to index the CRT), the
    /// engine-derived terrain defence modifier (§6.23), the resulting Combat
    /// Results Table cell, and the list of units eliminated as a consequence
    /// -- everything a UI needs to show *why* the shot landed the way it did,
    /// each modifier attributable to its rulebook paragraph.
    FireResolved {
        attack: FireAttack,
        roll: DieRoll,
        /// Total modifier applied to `roll` (engine-side terrain modifier
        /// included). Always present so the UI can show "rolled X, +Y = Z"
        /// even when `attack.modifiers` is empty.
        total_modifier: i16,
        modified_roll: DieRoll,
        /// The Combat Results Table factor row the attack was resolved on,
        /// after range-band application. The UI highlights the corresponding
        /// CRT cell.
        factor_row: FireFactorRow,
        /// Sum of post-range-band fire factors -- the number that determined
        /// `factor_row`. Distinct from `attack.factor_row` (which is the
        /// pre-resolution, app-supplied approximation).
        effective_factor: u16,
        result: CombatResult,
        /// Units eliminated by this resolution (empty for NoEffect/Disrupt
        /// unless disruption rounds up to elimination). The UI surfaces each
        /// elimination via [`Observation::UnitEliminated`] too; this list is
        /// for the combat card's "casualties of this shot" line.
        eliminations: Vec<UnitId>,
        /// Range to the impact hex and the range-effects band applied (§6.22,
        /// §8.1 night halving) -- the audit trail for "why was this factor
        /// halved". `None` in records serialized before the field existed.
        #[serde(default)]
        range: Option<u16>,
        #[serde(default)]
        band: Option<String>,
        /// Rulebook paragraphs relevant to this resolution, in citation form
        /// (e.g. `"6.22"`, `"6.24"`), so the UI can deep-link each one.
        /// Populated by the engine to keep the citation authoritative.
        paragraphs: Vec<String>,
    },
    /// A melee resolved (§7). Carries both die rolls and results -- melee is
    /// simultaneous, so each side's roll is applied to the *other*.
    MeleeResolved {
        attack: MeleeAttack,
        attacker_roll: DieRoll,
        attacker_total_modifier: i16,
        attacker_modified_roll: DieRoll,
        attacker_result: CombatResult,
        defender_roll: DieRoll,
        defender_total_modifier: i16,
        defender_modified_roll: DieRoll,
        defender_result: CombatResult,
        /// Melee factors each side rolled on (post-sum, pre-band). The CRT
        /// row is derived from these via [`FireFactorRow::from_total`].
        attacker_factor: u16,
        defender_factor: u16,
        attacker_losses: Vec<UnitId>,
        defender_losses: Vec<UnitId>,
        /// Whether the mandatory Dervish advance-after-melee (§7.6) fired, and
        /// how many units moved into the vacated hex.
        mandatory_advance: Option<u8>,
        paragraphs: Vec<String>,
    },
    /// A hex was vacated by combat, opening the advance-after-combat window
    /// (§6.82 offensive fire, §7.5 retreat, §7.6 melee). `eligible` lists the
    /// surviving participants that may advance into it -- the authoritative
    /// record for the log/UI and the audit trail for §6.82's participation
    /// requirement.
    HexVacatedByCombat {
        hex: HexCoord,
        eligible: Vec<UnitId>,
        /// Rulebook paragraphs that vacated the hex, distinguishing
        /// fire-vacated (§6.82) from melee-vacated (§7.6) and
        /// retreat-vacated (§7.5).
        paragraphs: Vec<String>,
    },
}

// ---------------------------------------------------------------------------
// 3) GameState -- authoritative mutable snapshot
// ---------------------------------------------------------------------------
