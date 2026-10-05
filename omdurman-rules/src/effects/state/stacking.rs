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

    // §5.51: gunboats may not stack with any other unit -- "Exception:
    // 5.21", the "Friendlies" unit aboard one (a passenger always shares its
    // gunboat's hex, so with a single gunboat present it is aboard that one).
    let is_gunboat = |u: &UnitPlacement| matches!(u.profile.kind, UnitKind::Gunboat { .. });
    let gunboats = occupants.iter().filter(|u| is_gunboat(u)).count();
    if gunboats > 1
        || (gunboats == 1
            && occupants
                .iter()
                .any(|u| !is_gunboat(u) && u.state.loaded_on.is_none()))
    {
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

    // §5.53: a Dervish leader may only stack with units of its command --
    // its colour's tribes; another leader is of another colour.
    for u in occupants {
        if let crate::UnitIdentity::DervishLeader(leader) = u.profile.identity {
            let bad = occupants.iter().any(|other| match other.profile.identity {
                crate::UnitIdentity::DervishTribal { tribe } => !leader.commands(tribe),
                crate::UnitIdentity::DervishLeader(other_leader) => other_leader != leader,
                _ => false,
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

    /// [`Self::check_stacking`] for a group moving together: whether all of
    /// `movers` may end a move in `dest` with whoever already stands there
    /// (§5.51-5.53). A stack routed as one must fit at its goal as one.
    pub fn check_group_stacking(
        &self,
        movers: &[UnitId],
        dest: HexCoord,
    ) -> Result<(), crate::StackingError> {
        let occupants: Vec<&UnitPlacement> = self
            .units
            .iter()
            .filter(|u| (u.position == dest && !movers.contains(&u.id)) || movers.contains(&u.id))
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
    ///
    /// The §5.44 fort exclusion is unit-independent ("not into a fort"), so it
    /// is computed once for `hex` instead of rescanning the whole board for a
    /// fort counter per adjacent unit.
    pub fn hex_in_enemy_zoc(
        &self,
        hex: HexCoord,
        mover_player: Player,
        mover_kind: UnitKind,
    ) -> bool {
        let fort_hex = self.is_fort_hex(hex);
        self.units.iter().any(|u| {
            u.position.neighbors().contains(&hex)
                && self
                    .unit_projects_zoc(u, mover_player, mover_kind)
                    .is_some()
                // A gunboat's ZOC lives on the water (§5.41) and never
                // reaches the fort exclusion at all (`zoc_extends` returns
                // before it); land units' ZOC does not extend into a fort.
                && (matches!(u.profile.kind, UnitKind::Gunboat { .. }) || !fort_hex)
                && self.zoc_extends_across(u, hex)
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
        let gunboat = matches!(unit.profile.kind, UnitKind::Gunboat { .. });
        // §5.44: no land ZOC reaches *into* a fort. A gunboat's ZOC lives on
        // the water (§5.41) and never reaches this exclusion -- see
        // [`Self::zoc_extends_across`]'s early return.
        if !gunboat && self.is_fort_hex(into) {
            return false;
        }
        self.zoc_extends_across(unit, into)
    }

    /// The §5.44 extent rules *without* the fort exclusion: the hexside
    /// direction rules (khor/wall/gate/Zariba/wall), the Nile exclusion, and
    /// the hut/building one. Private so callers that have already decided the
    /// fort question ([`Self::hex_in_enemy_zoc`] and [`Self::zoc_hexes`], each
    /// of which answers it once for all neighbours instead of per unit) do not
    /// rescan the board for a fort counter per call.
    fn zoc_extends_across(&self, unit: &UnitPlacement, into: HexCoord) -> bool {
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
            Some(side) if side.is_zariba() && zariba_outward => {}
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
    ///
    /// The §5.44 fort exclusion is per *destination* hex and unit-independent,
    /// so the neighbours' fort status is answered in one board scan instead of
    /// one per neighbour.
    pub fn zoc_hexes(
        &self,
        unit: &UnitPlacement,
        mover_player: Player,
        mover_kind: UnitKind,
    ) -> Vec<HexCoord> {
        if self
            .unit_projects_zoc(unit, mover_player, mover_kind)
            .is_none()
        {
            return Vec::new();
        }
        let adjacent = unit.position.neighbors();
        // Which neighbours hold a fort counter (§5.44: no land ZOC into a
        // fort; a gunboat's water ZOC is unaffected -- `zoc_extends_across`
        // returns before the exclusion would apply).
        let mut fort = [false; 6];
        for u in &self.units {
            if matches!(u.profile.kind, UnitKind::Fort { .. })
                && let Some(i) = adjacent.iter().position(|&h| h == u.position)
            {
                fort[i] = true;
            }
        }
        let gunboat = matches!(unit.profile.kind, UnitKind::Gunboat { .. });
        adjacent
            .into_iter()
            .zip(fort)
            .filter(|&(adj, fort)| (gunboat || !fort) && self.zoc_extends_across(unit, adj))
            .map(|(adj, _)| adj)
            .collect()
    }
}
