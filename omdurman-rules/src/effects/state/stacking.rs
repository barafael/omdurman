//! Stacking (rulebook §5.5) and zone-of-control (§5.4) validators.

use super::*;

/// The stacking law (§5.51-5.53) evaluated over an explicit list of
/// `occupants` of one hex. Pure and stateless, so [`GameState::check_stacking`]
/// (the prospective move/deploy check) and
/// [`GameState::validate_stacking_invariants`] (the whole-state post-condition)
/// are the same function over different occupant sources -- the two views
/// cannot drift again.
///
/// * §5.51: at most `STACKING_LIMIT` counted units (leaders and gunboats
///   free); gunboats share a hex with nothing (§5.21 transport is modelled
///   separately); and no enemy cohabitation -- engaging the enemy is what
///   melee is for (§7.1). The lone exception is an Anglo-Egyptian leader,
///   who never blocks an enemy stack (§6.51: a Dervish unit occupying his
///   hex eliminates him; §9.346 makes this how GORDON dies).
/// * §5.52: Dervish units of different stacking groups (tribes, plus the
///   artillery as its own group) may not share a hex.
/// * §5.53: a Dervish leader stacks only with units of its command.
pub fn stacking_rule(occupants: &[&UnitPlacement]) -> Result<(), crate::StackingError> {
    use crate::StackingError;

    // §5.51: no enemy cohabitation. Two units of opposite factions may share
    // a hex only while the §6.51 exception applies: an Anglo-Egyptian leader
    // is never a *blocker* (a Dervish unit arriving on his hex eliminates
    // him -- §9.346 is how GORDON dies), so any opposite-owner pair with at
    // least one non-leader on *both* sides is illegal (§7.1).
    for (i, a) in occupants.iter().enumerate() {
        for b in &occupants[i + 1..] {
            let mixed_factions = a.profile.identity.owner() != b.profile.identity.owner();
            let neither_is_ae_leader = !matches!(a.profile.kind, UnitKind::BritishLeader { .. })
                && !matches!(b.profile.kind, UnitKind::BritishLeader { .. });
            if mixed_factions && neither_is_ae_leader {
                return Err(StackingError::EnemyCohabitation);
            }
        }
    }

    // §5.51: gunboats may not stack with anything (Friendlies transport,
    // §5.21, is modelled separately and not via a normal move).
    let gunboats = occupants
        .iter()
        .filter(|u| matches!(u.profile.kind, UnitKind::Gunboat { .. }))
        .count();
    if gunboats > 0 && occupants.len() > 1 {
        return Err(StackingError::GunboatStack);
    }

    // §5.51: the four-unit limit counts neither leaders nor gunboats.
    let counted = occupants
        .iter()
        .filter(|u| {
            !matches!(
                u.profile.kind,
                UnitKind::DervishLeader { .. }
                    | UnitKind::BritishLeader { .. }
                    | UnitKind::Gunboat { .. }
            )
        })
        .count();
    if counted > STACKING_LIMIT {
        return Err(StackingError::OverLimit);
    }

    // §5.52: no two different Dervish stacking groups (tribes, plus the
    // artillery as its own group) in the same hex.
    let mut seen_group: Option<crate::DervishStackingGroup> = None;
    for u in occupants {
        if let Some(group) = u.profile.identity.dervish_stacking_group() {
            match seen_group {
                Some(seen) if seen != group => return Err(StackingError::DervishTribeMix),
                _ => seen_group = Some(group),
            }
        }
    }

    // §5.53: a Dervish leader may only stack with units of its command.
    for u in occupants {
        if let crate::UnitIdentity::DervishLeader(leader) = u.profile.identity {
            let bad = occupants.iter().any(|other| {
                matches!(
                    other.profile.identity,
                    crate::UnitIdentity::DervishTribal { tribe } if !leader.commands(tribe)
                )
            });
            if bad {
                return Err(StackingError::DervishLeaderCommandMismatch);
            }
        }
    }

    Ok(())
}

/// Whether `unit` projects a zone of control over a mover of `mover_kind`
/// belonging to `mover_player` (§5.41, §6.51). Pure and stateless: the
/// hexside subtleties (§5.44) need the board and live in
/// [`GameState::hex_in_enemy_zoc`] / [`GameState::zoc_hexes`], which call this
/// as their per-unit core. Extracted as a free function (like
/// [`stacking_rule`]) so the Kani harnesses can verify it without
/// constructing a `GameState`.
///
/// * A disrupted unit projects no ZOC (§5.41).
/// * Friendly units never project ZOC on each other.
/// * Anglo-Egyptian leaders exert no ZOC (§5.41, §6.51).
/// * Gunboats project ZOC *only* against enemy gunboats (§5.41).
/// * A fort projects ZOC out of its hex even when unoccupied (§5.44),
///   modelled by the fort unit projecting normally.
pub fn unit_projects_zoc_rule(
    unit: &UnitPlacement,
    mover_player: Player,
    mover_kind: UnitKind,
) -> Option<ZocReason> {
    if unit.state.disrupted {
        return None;
    }
    if unit.profile.identity.owner() == mover_player {
        return None;
    }
    match unit.profile.kind {
        // §6.51: Anglo-Egyptian leaders exert no ZOC.
        UnitKind::BritishLeader { .. } => None,
        // §5.41: gunboats project ZOC *only* against enemy gunboats.
        UnitKind::Gunboat { .. } => {
            matches!(mover_kind, UnitKind::Gunboat { .. }).then_some(ZocReason::GunboatVsGunboat)
        }
        // §5.44: a fort projects ZOC out of its hex even when unoccupied;
        // that is modelled by the fort *unit* itself projecting normally.
        UnitKind::Fort { .. } => Some(ZocReason::Fort),
        _ => Some(ZocReason::Normal),
    }
}

/// Maximum units per hex (§5.51), excluding free-stacking leaders/gunboats.
pub(crate) const STACKING_LIMIT: usize = 4;

impl GameState {
    /// Whether the unit `mover` may legally end its move stacked in `dest` given
    /// the units already there (§5.51-5.53). Stacking is checked only at the end
    /// of a move (§5.51), so this evaluates the resulting stack: every non-mover
    /// already in `dest` plus `mover`.
    ///
    /// * §5.51 -- at most four units per hex, *excluding* free-stacking leaders
    ///   and gunboats; gunboats may not share a hex with any other unit; and no
    ///   unit may share a hex with *enemy* units (§7.1) except a lone
    ///   Anglo-Egyptian leader (§6.51).
    /// * §5.52 -- units of different Dervish tribes (or stacking groups -- the
    ///   Dervish artillery is its own group) may not stack together.
    /// * §5.53 -- a Dervish leader may stack only with units of its command.
    pub fn check_stacking(
        &self,
        mover: &UnitPlacement,
        dest: HexCoord,
    ) -> Result<(), crate::StackingError> {
        // The prospective occupants: everyone already in `dest` except the
        // mover itself, plus the mover.
        let occupants: Vec<&UnitPlacement> = self
            .units
            .iter()
            .filter(|u| u.position == dest && u.id != mover.id)
            .chain(std::iter::once(mover))
            .collect();
        stacking_rule(&occupants)
    }

    /// Whole-state stacking invariant check (§5.51-5.53): every occupied hex
    /// must satisfy the stacking law ([`stacking_rule`]) on its *actual*
    /// occupants. Unlike [`Self::check_stacking`] this is not a prospective-move
    /// check — it validates the state as it stands, so it can be used as a
    /// post-condition after any mutation (see `apply_effect`) and to audit
    /// replayed records. Delegates to the same [`stacking_rule`] as
    /// [`Self::check_stacking`], so the prospective and whole-state views of
    /// the law cannot drift.
    pub fn validate_stacking_invariants(&self) -> Result<(), String> {
        // Ordered map: this is a grouping helper, and keeping `GameState`'s
        // reachable code free of `hashbrown` keeps it verifiable (see the
        // `verification` module).
        let mut by_hex: BTreeMap<HexCoord, Vec<&UnitPlacement>> = BTreeMap::new();
        for u in &self.units {
            by_hex.entry(u.position).or_default().push(u);
        }
        for (hex, occupants) in by_hex {
            stacking_rule(&occupants).map_err(|e| format!("{hex:?}: {e}"))?;
        }
        Ok(())
    }

    /// Whether `unit` projects a zone of control over a mover of
    /// `mover_kind` belonging to `mover_player` (§5.41, §5.44).
    ///
    /// * A disrupted unit projects no ZOC.
    /// * Anglo-Egyptian leaders project no ZOC.
    /// * Gunboats project ZOC only against enemy gunboats.
    ///
    /// Returns the [`ZocReason`] when ZOC applies, else `None`. The hexside
    /// subtleties (walls/gates/khor/forts/Zariba block or redirect ZOC --
    /// §5.44) need the game map, which the engine does not hold; the app layers
    /// those on top. This is the position/kind/disruption core of the rule.
    pub fn unit_projects_zoc(
        &self,
        unit: &UnitPlacement,
        mover_player: Player,
        mover_kind: UnitKind,
    ) -> Option<ZocReason> {
        unit_projects_zoc_rule(unit, mover_player, mover_kind)
    }

    /// Whether `hex` lies in a zone of control exerted by a unit hostile to a
    /// mover of `mover_kind` belonging to `mover_player` (§5.41, §5.44). A unit
    /// moving into such a hex must stop there and may move no further that turn
    /// (§5.26, §5.43).
    ///
    /// Applies the §5.44 hexside exceptions using the attached board: a ZOC does
    /// not extend across a khor/wall/Zariba hexside, and (except for gunboats)
    /// does not extend into or out of a Nile hex. With no board loaded these
    /// reduce to the plain adjacency rule.
    pub fn hex_in_enemy_zoc(
        &self,
        hex: HexCoord,
        mover_player: Player,
        mover_kind: UnitKind,
    ) -> bool {
        self.units.iter().any(|u| {
            u.position.neighbors().contains(&hex)
                && self
                    .unit_projects_zoc(u, mover_player, mover_kind)
                    .is_some()
                && self.zoc_extends(u, hex)
        })
    }

    /// Whether the ZOC of `unit` (standing next to `into`) reaches into
    /// `into` -- the §5.44 extent rules, in one place:
    /// * not across a khor;
    /// * across a Zariba hexside only out of the Zariba, not into it;
    /// * not into or out of a Nile hex (gunboats excepted, §5.41);
    /// * not into a fort (it does extend *out* of one, even unoccupied);
    /// * "out of, but not into, a hut or building hex";
    /// * across a wall only from a walled-city hex outward, across a gate
    ///   only outward ("out of, but not into, a walled city hex"), across a
    ///   breach both ways.
    pub fn zoc_extends(&self, unit: &UnitPlacement, into: HexCoord) -> bool {
        use omdurman_types::HexsideKind;
        let from = unit.position;
        let gunboat = matches!(unit.profile.kind, UnitKind::Gunboat { .. });
        if !gunboat && (self.board.is_nile(from) || self.board.is_nile(into)) {
            return false;
        }
        let city_outward = self.board.is_walled_city(from) && !self.board.is_walled_city(into);
        let zariba_outward = self.board.is_zariba(from) && !self.board.is_zariba(into);
        match self.hexside_effective(from, into) {
            Some(HexsideKind::Wall | HexsideKind::Gate) if !city_outward => return false,
            // "In the historical scenario ZOCs extend out of, but not into,
            // the Zariba across a Zariba hexside" (also a constructed one).
            Some(
                HexsideKind::ZaribaThornHedge
                | HexsideKind::ZaribaTrench
                | HexsideKind::ZaribaTrenchEndA
                | HexsideKind::ZaribaTrenchEndB,
            ) if zariba_outward => {}
            Some(side)
                if side.blocks_zoc() && !matches!(side, HexsideKind::Wall | HexsideKind::Gate) =>
            {
                return false;
            }
            _ => {}
        }
        if gunboat {
            return true; // gunboat-vs-gunboat ZOC lives on the water
        }
        if self.is_fort_hex(into) {
            return false;
        }
        !matches!(
            self.board.terrain_at(into),
            Some(omdurman_types::Terrain::Huts { .. } | omdurman_types::Terrain::Building { .. })
        )
    }

    /// Whether `hex` is a fort (§5.44, §6.54): a hex holding a fort counter
    /// (FALL OF KHARTOUM's North Fort, §9.344, and Forts Makran and Buri,
    /// §9.321, are counters too).
    pub fn is_fort_hex(&self, hex: HexCoord) -> bool {
        self.units
            .iter()
            .any(|u| u.position == hex && matches!(u.profile.kind, UnitKind::Fort { .. }))
    }

    /// Compute the set of hexes that a given unit projects a zone of control
    /// into (§5.41, §5.44). Returns the 6 adjacent hexes minus exclusions.
    ///
    /// This is a pure function — it computes the ZOC footprint without
    /// side effects. `hex_in_enemy_zoc` checks whether *any* hostile unit's
    /// ZOC covers a given hex; this function returns *which* hexes a
    /// specific unit covers.
    pub fn zoc_hexes(
        &self,
        unit: &UnitPlacement,
        mover_player: Player,
        mover_kind: UnitKind,
    ) -> Vec<HexCoord> {
        let Some(reason) = self.unit_projects_zoc(unit, mover_player, mover_kind) else {
            return Vec::new();
        };
        let _ = reason;
        unit.position
            .neighbors()
            .into_iter()
            .filter(|&adj| self.zoc_extends(unit, adj))
            .collect()
    }
}
