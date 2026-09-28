//! Setup / placement validators (rulebook §9.2/§9.3), river obstacles (§10),
//! and reinforcement placement (§9.112/§9.113).

use super::*;

/// A placement entering play (setup deployment §9.2/§9.3, reinforcement
/// §9.112/§9.113) must be the physical counter it names: the canonical
/// profile from `unit_profiles::profile_for_unit` and a fresh
/// [`UnitState`](crate::UnitState). A peer may not invent unit values (a
/// 99-factor infantry, a pre-loaded or engine-less gunboat).
pub(crate) fn require_canonical_placement(p: &UnitPlacement) -> Result<(), RuleError> {
    let canonical = crate::unit_profiles::profile_for_unit(p.id);
    if canonical != Some(p.profile) || p.state != crate::UnitState::default() {
        return Err(RuleError::NonCanonicalPlacement(p.id));
    }
    Ok(())
}

/// Whether a counter of `identity` is in play in the Historical scenario:
/// not GORDON or the "Friendlies" brigade (§9.211), not Isa Zachneih, the
/// gunboats or the forts (§9.212).
pub fn historical_in_play(identity: &crate::UnitIdentity) -> bool {
    use crate::UnitIdentity;
    match identity {
        UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Gordon) => false,
        identity if identity.is_friendlies() => false,
        UnitIdentity::DervishTribal {
            tribe: crate::DervishTribe::IsaZachneih,
        } => false,
        UnitIdentity::DervishGunboat(_)
        | UnitIdentity::DervishFort
        | UnitIdentity::AngloEgyptianFort => false,
        _ => true,
    }
}

/// Whether counter `id` is in play in the Historical scenario: a physical
/// counter of the cut sheet ([`SectionName::SHEET_ORDER`]; the engine-only
/// Kehena, Degheim, Danagla and Mulazmin ids duplicate counters printed in
/// the leaders' blocks) of a type the scenario uses ([`historical_in_play`]).
///
/// [`SectionName::SHEET_ORDER`]: omdurman_types::SectionName::SHEET_ORDER
pub fn historical_counter_in_play(id: UnitId) -> bool {
    omdurman_types::SectionName::SHEET_ORDER.contains(&id.section_pos().0)
        && crate::unit_profiles::profile_for_unit(id)
            .is_some_and(|p| historical_in_play(&p.identity))
}

/// The Historical scenario's Kerreri detachment (§9.211): the two Camel
/// Corps counters, the two Egyptian Cavalry squadrons and the Horse
/// Artillery. (The 21st Lancers share the cavalry identity but set up in the
/// Zariba, so the list is by counter.)
pub const HISTORICAL_KERRERI_UNITS: [UnitId; 5] = [
    UnitId::Kitchener_3_0,
    UnitId::Kitchener_4_0,
    UnitId::EgyptianArmy_0_0,
    UnitId::EgyptianArmy_1_0,
    UnitId::EgyptianArmy_2_0,
];

/// A Fall-of-Khartoum order-of-battle slot group (§9.321/§9.322): the
/// manual counts by type and nationality, not by exact counter -- "two
/// British infantry units" binds across all British battalions whatever
/// their ordinal, "two old style gunboats" across the four old boat
/// counters. Counting exact identities would let one of each variant in.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FokCapGroup {
    Tribe(DervishTribe),
    DervishArtillery,
    DervishFort,
    AeFort,
    OldGunboat,
    AeArtillery,
    Infantry(crate::BrigadeNationality),
    Gordon,
}

/// Which FoK slot group `identity` belongs to (`None`: not in the order of
/// battle at all), and how many counters of that group may deploy.
pub fn fok_cap_group(identity: &crate::UnitIdentity) -> Option<(FokCapGroup, usize)> {
    use crate::UnitIdentity;
    use FokCapGroup::*;
    Some(match identity {
        // §9.322: "32 Mulazmin units ... 2 Hadendowa; 6 Kehena; 5 Degheim
        // ... 3 Dervish artillery units" (the Mulazmin are the two green
        // print runs, 16 + 16). The Kehena and Degheim counters are the
        // "Deghelim" cells of the Ali_Wad_Helu block (see
        // `unit_profiles::ali_wad_helu`): row 1 resolves to Kehena, row 0
        // cols 1-5 to Degheim -- keying this table by *identity* therefore
        // reaches both forces through their cut sprites.
        UnitIdentity::DervishTribal {
            tribe: crate::DervishTribe::Mulazmin,
        } => (Tribe(crate::DervishTribe::Mulazmin), 32),
        UnitIdentity::DervishTribal {
            tribe: crate::DervishTribe::Hadendowa,
        } => (Tribe(crate::DervishTribe::Hadendowa), 2),
        UnitIdentity::DervishTribal {
            tribe: crate::DervishTribe::Kehena,
        } => (Tribe(crate::DervishTribe::Kehena), 6),
        UnitIdentity::DervishTribal {
            tribe: crate::DervishTribe::Degheim,
        } => (Tribe(crate::DervishTribe::Degheim), 5),
        UnitIdentity::DervishArtillery => (DervishArtillery, 3),
        // §9.321: "Two old style (unnamed) gunboats", "one Egyptian
        // Battalion artillery unit", "two British infantry units", "three
        // Egyptian infantry units", "four Sudan infantry units", "four
        // 'Friendlies' units" -- any counter of the group may stand in.
        UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(_)) => (OldGunboat, 2),
        UnitIdentity::AngloEgyptianArtillery => (AeArtillery, 1),
        UnitIdentity::AngloEgyptianInfantry {
            brigade: crate::BrigadeId { nationality, .. },
            ..
        } => (
            Infantry(*nationality),
            match *nationality {
                crate::BrigadeNationality::British => 2,
                crate::BrigadeNationality::Egyptian => 3,
                crate::BrigadeNationality::Sudanese => 4,
                crate::BrigadeNationality::Friendlies => 4,
            },
        ),
        // §9.321: GORDON starts in the palace (the scenario's one leader).
        UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Gordon) => (Gordon, 1),
        // §9.344: the single North Fort is the only Dervish fort in play.
        UnitIdentity::DervishFort => (DervishFort, 1),
        // §9.321: Forts Makran and Buri, the two forts printed on the map.
        UnitIdentity::AngloEgyptianFort => (AeFort, 2),
        _ => return None,
    })
}

/// Maximum river mines a player may lay (§10.11).
pub const MAX_MINES: usize = 2;

/// Maximum contiguous Nile hexes the river chain may span (§10.21).
pub const MAX_CHAIN_HEXES: usize = 4;

impl GameState {
    /// Whether deployment is finished and the game may leave [`Phase::Setup`]
    /// for the first Movement turn (§9.2/§9.3/§10). Both factions must have at
    /// least one unit on the board. The concrete per-scenario order of battle
    /// (which units, where) is enforced by the app's set-up plan, not here (the
    /// engine's `BoardInfo` carries no OOB); river mines/chain within limits are
    /// enforced at placement time, so they need no re-check here.
    ///
    /// Returns [`RuleError::SetupIncomplete`] naming the first unmet requirement,
    /// so the UI can surface *why* "Begin battle" is disabled. Every scenario
    /// currently shares the same "both sides deployed" gate; when a scenario
    /// needs a different minimum, branch on `self.scenario` here.
    pub fn setup_complete(&self) -> Result<(), RuleError> {
        let has = |player| {
            self.units
                .iter()
                .any(|u| u.profile.identity.owner() == player)
        };
        // §9.113: in the Campaign game the Anglo-Egyptian side starts with
        // *no* units on the map (they arrive as reinforcements from turn 1),
        // so only the Dervish §9.111 initial presence gates leaving Setup.
        if self.scenario != Scenario::Campaign && !has(Player::AngloEgyptian) {
            return Err(RuleError::SetupIncomplete(
                "Anglo-Egyptian forces not yet deployed",
            ));
        }
        if !has(Player::Dervish) {
            return Err(RuleError::SetupIncomplete(
                "Dervish forces not yet deployed",
            ));
        }
        // Fall of Khartoum and the Historical scenario pin both orders of
        // battle (§9.211-9.212, §9.321-9.322), so don't let the game leave
        // Setup until each side has deployed its full contingent. The
        // per-faction Ready button already gates on `setup_target_met`; this
        // is defense-in-depth for the unbound "Begin battle" path and any
        // future caller. The Campaign has no fixed target
        // (`setup_target_met` reduces to "at least one").
        if !self.setup_target_met(Player::AngloEgyptian) {
            return Err(RuleError::SetupIncomplete(
                "Anglo-Egyptian order of battle not fully deployed",
            ));
        }
        if !self.setup_target_met(Player::Dervish) {
            return Err(RuleError::SetupIncomplete(
                "Dervish order of battle not fully deployed",
            ));
        }
        Ok(())
    }

    /// Whether `player` has confirmed it is ready to leave setup (§9.2/§9.3).
    pub fn setup_ready(&self, player: Player) -> bool {
        match player {
            Player::AngloEgyptian => self.setup_ready_ae,
            Player::Dervish => self.setup_ready_dervish,
        }
    }

    /// How many of `player`'s units are currently on the board -- the deployed
    /// count shown during setup and compared against [`Self::setup_target`].
    pub fn setup_deployed_count(&self, player: Player) -> usize {
        self.units
            .iter()
            .filter(|u| u.profile.identity.owner() == player)
            .count()
    }

    /// The number of units `player` must deploy before turn 1, when the scenario
    /// pins it down. **Fall of Khartoum**: British 17, Dervish 48
    /// (§9.321-9.322), plus the scenario-fixed forts (Makran and Buri, the
    /// §9.344 North Fort). **Historical**: every counter in play -- "all
    /// remaining Anglo-Egyptian units set up in the 13 hexes of the Zariba",
    /// "all remaining Dervish units set up within three hexes of their
    /// leader" (§9.211/§9.212). The Campaign is reinforcement-driven (the A-E
    /// player starts with *no* units on the map, §9.113): `None` there means
    /// "no hard count -- just show what's deployed".
    pub fn setup_target(&self, player: Player) -> Option<usize> {
        match (self.scenario, player) {
            // 17 player-deployed garrison + the scenario-fixed Forts Makran
            // and Buri.
            (Scenario::FallOfKhartoum, Player::AngloEgyptian) => Some(19),
            // 48 player-deployed entry force + 1 scenario-fixed North Fort fort.
            (Scenario::FallOfKhartoum, Player::Dervish) => Some(49),
            (Scenario::Historical, player) => Some(
                crate::UnitId::ALL
                    .iter()
                    .filter(|id| {
                        historical_counter_in_play(**id)
                            && crate::unit_profiles::profile_for_unit(**id)
                                .is_some_and(|p| p.identity.owner() == player)
                    })
                    .count(),
            ),
            (Scenario::Campaign, _) => None,
        }
    }

    /// Whether `player` has deployed enough to be allowed to confirm ready: it
    /// meets its `setup_target` when the scenario sets one, else just needs the
    /// board-wide `setup_complete` minimum (at least one unit).
    pub fn setup_target_met(&self, player: Player) -> bool {
        match self.setup_target(player) {
            Some(target) => self.setup_deployed_count(player) >= target,
            // §9.113: the Campaign A-E side deploys nothing at setup.
            None if self.scenario == Scenario::Campaign && player == Player::AngloEgyptian => true,
            None => self.setup_deployed_count(player) >= 1,
        }
    }

    /// Whether `hex` is inside `player`'s deployment zone for this scenario
    /// (§9.211-9.212 Historical, §9.321-9.322 Fall of Khartoum). A hex must first
    /// be on the board (present in `board.terrain`); an empty board (no map facts
    /// attached) is treated as fully permissive so headless tests can deploy
    /// anywhere.
    ///
    /// Zones, from the manual:
    /// - **Fall of Khartoum British** (§9.321): the garrison sets up in building
    ///   or hut hexes, at Fort Makran / Fort Buri / the Palace, or adjacent to a
    ///   wall hexside. (Gordon is pre-placed.) Per §5.22 the split is exclusive
    ///   -- gunboats deploy *only* on Nile hexes, and land units may never
    ///   deploy on the Nile.
    /// - **Fall of Khartoum Dervish** (§9.322): enters from the south or east
    ///   map edge. The FoK board is diamond-shaped: the south edge is the
    ///   bottom row (no hex at `r+1`); the east edge is the diagonal of
    ///   rightmost hexes per row (no hex at `q+1`). Gunboats may also enter
    ///   from the west (Nile) edge (no hex at `q-1`).
    /// - **Historical / Campaign** (§9.211-9.212, §9.11): permissive here.
    ///   The Historical areas depend on the counter and on what is already
    ///   deployed (the Kerreri detachment, a leader's colour, the
    ///   Anglo-Egyptians' line of sight), so [`Self::historical_set_up_area`]
    ///   checks them per placement.
    pub fn in_deployment_zone(&self, player: Player, hex: HexCoord, is_boat: bool) -> bool {
        // No board attached -> permissive (unit tests, unbound session).
        if self.board.terrain.is_empty() {
            return true;
        }
        if self.board.terrain_at(hex).is_none() {
            return false; // off the playable map
        }
        // §5.22 is universal during deployment (all scenarios, both factions):
        // gunboats deploy *only* on the Nile, and land units *never* deploy on
        // the Nile. Previously this was only checked for Fall of Khartoum, so
        // Campaign/Historical set-ups could anchor a gunboat on land or drop
        // an infantry counter in the river (audit §5.22/§9.111).
        let is_nile = matches!(
            self.board.terrain_at(hex),
            Some(omdurman_types::Terrain::Nile { .. })
        );
        if is_boat {
            if !is_nile {
                return false;
            }
        } else if is_nile {
            return false;
        }
        match self.scenario {
            Scenario::Campaign => true,
            // The union of the §9.211/§9.212 set-up areas; which one binds a
            // given counter is [`Self::historical_set_up_area`]'s business.
            Scenario::Historical => match player {
                Player::AngloEgyptian if is_boat => {
                    hex.neighbors().into_iter().any(|n| self.board.is_zariba(n))
                }
                Player::AngloEgyptian => {
                    self.board.is_zariba(hex)
                        || self.board.location_at(hex) == Some(omdurman_types::Location::Kerreri)
                }
                Player::Dervish => {
                    !is_boat
                        && self.units.iter().any(|u| {
                            matches!(u.profile.identity, crate::UnitIdentity::DervishLeader(_))
                                && u.position.distance(hex) <= 3
                        })
                        && self
                            .army_ashore_sighting(
                                hex,
                                self.board
                                    .terrain_at(hex)
                                    .map(crate::los_table::los_level)
                                    .unwrap_or(crate::los_table::LosLevel::Ground),
                            )
                            .is_none()
                }
            },
            Scenario::FallOfKhartoum => {
                // (§5.22 was already applied above.)
                match player {
                    Player::Dervish => {
                        // The North Fort is Dervish-controlled from the start
                        // (§9.344) and is a fixed fortification, not part of the
                        // entry force -- so it's a legal deploy hex for the
                        // Dervish forts regardless of the south/east-edge rule
                        // below.
                        if matches!(
                            self.board.location_at(hex),
                            Some(omdurman_types::Location::NorthFort)
                        ) {
                            return true;
                        }
                        // South or east map edge (§9.322), plus the western Nile
                        // edge for gunboats -- the Nile runs along the west side
                        // of the FoK map and gunboats need water to deploy.
                        //
                        // The FoK board is diamond-shaped: the "east edge" is
                        // the diagonal of rightmost hexes per row (where no
                        // hex exists at q+1), not just q == global max_q.
                        // Similarly the south edge is the bottom row (no hex
                        // at r+1) and the west edge is the leftmost diagonal
                        // (no hex at q-1).
                        let on_south_edge = !self
                            .board
                            .terrain
                            .contains_key(&HexCoord::new(hex.q, hex.r + 1));
                        let on_east_edge = !self
                            .board
                            .terrain
                            .contains_key(&HexCoord::new(hex.q + 1, hex.r));
                        let on_west_edge = !self
                            .board
                            .terrain
                            .contains_key(&HexCoord::new(hex.q - 1, hex.r));
                        on_south_edge || on_east_edge || (is_boat && on_west_edge)
                    }
                    Player::AngloEgyptian => {
                        // The North Fort is Dervish-controlled (§9.344) and must
                        // not appear in the AE deployment zone.
                        if matches!(
                            self.board.location_at(hex),
                            Some(omdurman_types::Location::NorthFort)
                        ) {
                            return false;
                        }
                        // A gunboat was already constrained to a Nile hex by the
                        // §5.22 check above; any Nile hex is a legal anchor for
                        // the two old FoK gunboats (§9.321), with no further
                        // restriction.
                        if is_boat {
                            return true;
                        }
                        // Land units (§9.321): a building or hut hex, a garrison
                        // landmark (Palace / Fort Makran / Fort Buri), or a hex
                        // adjacent to a wall hexside. (Already guaranteed
                        // not-Nile above.)
                        let terrain = self.board.terrain_at(hex);
                        let is_garrison_terrain = matches!(
                            terrain,
                            Some(
                                omdurman_types::Terrain::Building { .. }
                                    | omdurman_types::Terrain::Huts { .. }
                            )
                        );
                        let at_landmark = matches!(
                            self.board.location_at(hex),
                            Some(
                                omdurman_types::Location::Palace
                                    | omdurman_types::Location::FortMakran
                                    | omdurman_types::Location::FortBuri
                            )
                        );
                        // A gate is an opening *in* the wall: the hex behind
                        // the Messalamia or Kalakla gate is "adjacent to a wall
                        // hex" as much as its neighbours along the rampart.
                        let adjacent_to_wall = hex.neighbors().iter().any(|&n| {
                            self.hexside_effective_is(hex, n, |k| {
                                matches!(k, HexsideKind::Wall | HexsideKind::Gate)
                            })
                        });
                        is_garrison_terrain || at_landmark || adjacent_to_wall
                    }
                }
            }
        }
    }

    /// Where a Historical-scenario counter may set up (§9.211/§9.212). The
    /// Anglo-Egyptians: gunboats in Nile hexes adjacent to the Zariba; the
    /// Camel Corps, Egyptian Cavalry and Horse Artillery in the Kerreri hut
    /// hexes; everything else in the 13 hexes of the Zariba. The Dervishes
    /// (who set up second, seeing the whole Anglo-Egyptian deployment):
    /// within three hexes of the leader of their colour, and out of the line
    /// of sight of the Anglo-Egyptian units ashore. The leaders themselves
    /// are the scenario's fixed placements on the lettered hexes. Permissive
    /// without a board, like [`Self::in_deployment_zone`].
    ///
    /// The gunboats are left out of the line-of-sight test: at rough level
    /// (§6.3 note b) a boat beside the Zariba sees over Jebel Surgham's
    /// slopes to the Y, K, S and O hexes themselves, so counting them would
    /// forbid the very hexes the scenario pins the leaders to -- and most of
    /// the ground around them.
    pub fn historical_set_up_area(&self, placement: &UnitPlacement) -> Result<(), RuleError> {
        use crate::UnitIdentity;
        if self.board.terrain.is_empty() {
            return Ok(());
        }
        let hex = placement.position;
        if placement.profile.identity.owner() == Player::AngloEgyptian {
            let (inside, area) = if placement.profile.kind.is_boat() {
                (
                    hex.neighbors().into_iter().any(|n| self.board.is_zariba(n)),
                    "gunboats start in Nile hexes adjacent to the Zariba (§9.211)",
                )
            } else if HISTORICAL_KERRERI_UNITS.contains(&placement.id) {
                (
                    self.board.location_at(hex) == Some(omdurman_types::Location::Kerreri),
                    "the Camel Corps, Egyptian Cavalry and Horse Artillery start in the \
                     village of Kerreri hut hexes (§9.211)",
                )
            } else {
                (
                    self.board.is_zariba(hex),
                    "the Anglo-Egyptian units set up in the 13 hexes of the Zariba (§9.211)",
                )
            };
            return if inside {
                Ok(())
            } else {
                Err(RuleError::HistoricalSetUpArea { hex, area })
            };
        }
        let leader = match placement.profile.identity {
            UnitIdentity::DervishTribal { tribe } => crate::DervishLeader::of_tribe(tribe),
            // The three guns are the Khalifa's black counters.
            UnitIdentity::DervishArtillery => Some(crate::DervishLeader::KhalifaAbdullah),
            _ => None,
        };
        if let Some(leader) = leader {
            let near = self.units.iter().any(|u| {
                u.profile.identity == UnitIdentity::DervishLeader(leader)
                    && u.position.distance(hex) <= 3
            });
            if !near {
                return Err(RuleError::SetUpFarFromLeader { hex, leader });
            }
        }
        let own_level =
            crate::los_table::los_level_for_unit(placement.profile.kind, hex, &self.board);
        match self.army_ashore_sighting(hex, own_level) {
            Some(seen_from) => Err(RuleError::SetUpInEnemySight { hex, seen_from }),
            None => Ok(()),
        }
    }

    /// The hex of an Anglo-Egyptian land unit with a line of sight to `hex`
    /// (at LOS level `level`), if any -- the §9.212 "out of the line of
    /// sight" test (gunboats excluded, see [`Self::historical_set_up_area`]).
    fn army_ashore_sighting(
        &self,
        hex: HexCoord,
        level: crate::los_table::LosLevel,
    ) -> Option<HexCoord> {
        self.units
            .iter()
            .filter(|u| {
                u.profile.identity.owner() == Player::AngloEgyptian && !u.profile.kind.is_boat()
            })
            .find(|u| {
                crate::los_table::has_los(
                    &self.board,
                    u.position,
                    hex,
                    crate::FireKind::Direct,
                    crate::los_table::los_level_for_unit(u.profile.kind, u.position, &self.board),
                    level,
                    self.los_unit_blocker(),
                    |a, b| self.wall_is_breached(a, b),
                )
            })
            .map(|u| u.position)
    }

    /// Guard shared by every setup placement: the action is legal only during
    /// [`Phase::Setup`] (§9.2/§9.3/§10).
    fn require_setup_phase(&self) -> Result<(), RuleError> {
        if self.phase != Phase::Setup {
            return Err(RuleError::WrongPhase);
        }
        Ok(())
    }

    /// Read-only check of whether `placement` may be deployed in [`Phase::Setup`]
    /// (§9.2/§9.3): right phase, the counter isn't already on the board (each
    /// physical unit deploys once), inside the owner's deployment zone, and legal
    /// stacking. Mirrors the `DeployUnit` effect so the UI can gate input.
    pub fn can_deploy_unit(&self, placement: &UnitPlacement) -> Result<(), RuleError> {
        self.require_setup_phase()?;
        if self.units.iter().any(|u| u.id == placement.id) {
            return Err(RuleError::AlreadyDeployed(placement.id));
        }
        // Scenario-specific "in play at setup" filter: the Campaign's initial
        // force is the §9.111 Dervish set (everything else arrives as a
        // reinforcement, §9.112/§9.113), and the Historical scenario excludes
        // its not-in-play units outright (§9.211/§9.212).
        self.unit_in_play_at_setup(placement)?;
        // The scenario's own fixed counters (GORDON, the North Fort, the
        // Historical leaders) are placed by the scenario at game start.
        let fixed = crate::scenario_setup::is_fixed_placement(self.scenario, placement.id);
        // Per-counter Historical areas first: they name the exact rule.
        if self.scenario == Scenario::Historical && !fixed {
            self.historical_set_up_area(placement)?;
        }
        // (The Historical leaders' letters lie outside the zone union, which
        // is drawn around the leaders themselves.)
        let exempt = fixed && self.scenario == Scenario::Historical;
        let owner = placement.profile.identity.owner();
        if !exempt
            && !self.in_deployment_zone(owner, placement.position, placement.profile.kind.is_boat())
        {
            return Err(RuleError::OutsideDeploymentZone(placement.position));
        }
        self.check_stacking(placement, placement.position)?;
        // Set-up order (§9.111/§9.211/§9.321), checked after the placement's
        // own legality so a misplaced counter reports what is wrong with it.
        if !fixed {
            self.require_setup_turn(placement.profile.identity.owner())?;
        }
        // A peer may not invent unit values: the counter enters with its
        // canonical profile and a fresh state.
        require_canonical_placement(placement)
    }

    /// The FALL OF KHARTOUM orders of battle (§9.321 British, §9.322 Dervish):
    /// the exact number of counters of each type that may deploy at setup, and
    /// `None` for every unit type not in the scenario at all. The single North
    /// Fort (§9.344) and GORDON in the palace (§9.321) are scenario-fixed and
    /// bypass this table.
    ///
    /// §9.321/§9.322: how many more counters of `identity`'s order-of-battle
    /// group may still deploy at setup (`None`: the type is not in the FoK
    /// orders of battle). The bot's setup generator uses this to stop
    /// offering candidates the engine would reject.
    pub fn fok_setup_slots_remaining(&self, identity: &crate::UnitIdentity) -> Option<usize> {
        let (group, cap) = fok_cap_group(identity)?;
        let already = self
            .units
            .iter()
            .filter(|u| fok_cap_group(&u.profile.identity).is_some_and(|(g, _)| g == group))
            .count();
        Some(cap.saturating_sub(already))
    }

    /// Whether `profile` belongs to a unit that may be on the board at setup
    /// in the current scenario (§9.111 Campaign initial force; §9.211/§9.212
    /// Historical not-in-play lists; §9.321/§9.322 Fall of Khartoum orders of
    /// battle, including their exact per-type counts).
    fn unit_in_play_at_setup(&self, placement: &UnitPlacement) -> Result<(), RuleError> {
        use crate::UnitIdentity;
        match self.scenario {
            Scenario::Campaign => match placement.profile.identity {
                // §9.111: the Anglo-Egyptian side starts empty (§9.113).
                UnitIdentity::AngloEgyptianInfantry { .. }
                | UnitIdentity::AngloEgyptianCavalry
                | UnitIdentity::AngloEgyptianCamelCorps
                | UnitIdentity::AngloEgyptianArtillery
                | UnitIdentity::AngloEgyptianMaxim
                | UnitIdentity::AngloEgyptianGunboat(_)
                | UnitIdentity::AngloEgyptianLeader(_)
                | UnitIdentity::RoyalEngineers => Err(RuleError::NotInPlay(placement.id)),
                // §9.111 Dervish initial force: the Khalifa, Isa Zachneih,
                // the three artillery, the Taiasha bodyguard, the forts and
                // the two gunboats. Every other tribe/leader is a §9.112
                // reinforcement wave.
                UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah) => Ok(()),
                UnitIdentity::DervishTribal {
                    tribe: crate::DervishTribe::Taiasha,
                }
                | UnitIdentity::DervishTribal {
                    tribe: crate::DervishTribe::IsaZachneih,
                } => Ok(()),
                UnitIdentity::DervishArtillery
                | UnitIdentity::DervishFort
                | UnitIdentity::DervishGunboat(_) => Ok(()),
                _ => Err(RuleError::NotInPlay(placement.id)),
            },
            Scenario::Historical => {
                if historical_counter_in_play(placement.id) {
                    Ok(())
                } else {
                    Err(RuleError::NotInPlay(placement.id))
                }
            }
            Scenario::FallOfKhartoum => {
                // §9.321/§9.322 orders of battle with their exact per-type
                // counts (grouped: the manual counts "two British infantry
                // units", not per battalion ordinal). The scenario-fixed
                // counters (GORDON in the palace, §9.344's single North
                // Fort) deploy through this same table.
                match self.fok_setup_slots_remaining(&placement.profile.identity) {
                    None => Err(RuleError::NotInPlay(placement.id)),
                    Some(0) => Err(RuleError::FoKOrderOfBattleFull),
                    Some(_) => Ok(()),
                }
            }
        }
    }

    /// Read-only check of whether `player` may pick a deployed unit back up off
    /// the board during [`Phase::Setup`] (§9.2/§9.3): right phase, the unit is on
    /// the board, and it belongs to `player` (you may only re-pick your own
    /// counters). Mirrors the `RemoveDeployedUnit` effect.
    pub fn can_remove_deployed_unit(
        &self,
        unit_id: UnitId,
        player: Player,
    ) -> Result<(), RuleError> {
        self.require_setup_phase()?;
        let unit = self.unit_or_err(unit_id)?;
        if unit.profile.identity.owner() != player {
            return Err(RuleError::NotOwner(unit_id));
        }
        self.require_setup_turn(player)
    }

    /// The side that sets up first: the Dervish in the Campaign (§9.111), the
    /// Anglo-Egyptians in the Historical scenario (§9.211) and in FALL OF
    /// KHARTOUM (§9.321, "The British player sets up first").
    pub fn first_to_set_up(&self) -> Player {
        match self.scenario {
            Scenario::Campaign => Player::Dervish,
            Scenario::Historical | Scenario::FallOfKhartoum => Player::AngloEgyptian,
        }
    }

    /// The side whose move it is now, for "your turn" / "waiting on" text:
    /// during set-up the side deploying (sequential, §9.111/§9.211/§9.321),
    /// otherwise the phase player (defensive fire belongs to the non-moving
    /// side, §6.4/§6.7); `None` once the game is over.
    pub fn player_to_act(&self) -> Option<Player> {
        if self.game_over {
            return None;
        }
        if self.phase == Phase::Setup {
            let first = self.first_to_set_up();
            return Some(if self.setup_ready(first) {
                first.opponent()
            } else {
                first
            });
        }
        Some(self.phase_player())
    }

    /// Whether `player` may change its deployment now (§9.111/§9.211/§9.321):
    /// deployment is sequential -- the first side sets up and confirms Ready
    /// (which fixes its deployment), only then does the second side set up,
    /// seeing where the first stands. A side that has confirmed Ready is done.
    pub fn require_setup_turn(&self, player: Player) -> Result<(), RuleError> {
        if self.setup_ready(player) {
            return Err(RuleError::SetupOrder(
                "your deployment is confirmed and can no longer change",
            ));
        }
        let first = self.first_to_set_up();
        if player != first && !self.setup_ready(first) {
            return Err(RuleError::SetupOrder(match self.scenario {
                Scenario::Campaign => {
                    "the Dervish player sets up first (§9.111) -- wait until they are ready"
                }
                Scenario::Historical => {
                    "the Anglo-Egyptian player sets up first (§9.211) -- wait until they are ready"
                }
                Scenario::FallOfKhartoum => {
                    "the British player sets up first (§9.321) -- wait until they are ready"
                }
            }));
        }
        Ok(())
    }

    /// Read-only check of a river-mine placement in setup (§10.11): Setup phase,
    /// at most [`MAX_MINES`], and no two mines on the same hex.
    pub fn can_place_mine(&self, hex: HexCoord) -> Result<(), RuleError> {
        self.require_setup_phase()?;
        self.require_setup_turn(Player::Dervish)?;
        // Optional-rule gate: mines exist only when the River Mines option was
        // selected at game start (§10.11).
        if !self.optional_rules.contains(&OptionalRule::RiverMines) {
            return Err(RuleError::SetupLimit(
                "the River Mines optional rule is not in play (§10.11)",
            ));
        }
        if self.mines.iter().any(|m| m.hex == hex) {
            return Err(RuleError::SetupLimit("a mine is already laid on that hex"));
        }
        if self.mines.len() >= MAX_MINES {
            return Err(RuleError::SetupLimit("at most two river mines (§10.11)"));
        }
        Ok(())
    }

    /// Read-only check of a river-chain placement in setup (§10.21): Setup phase
    /// and at most [`MAX_CHAIN_HEXES`] hexes.
    pub fn can_place_chain(&self, hexes: &[HexCoord]) -> Result<(), RuleError> {
        self.require_setup_phase()?;
        self.require_setup_turn(Player::Dervish)?;
        // Optional-rule gate: the chain exists only when the River Chain option
        // was selected at game start (§10.21).
        if !self.optional_rules.contains(&OptionalRule::RiverChain) {
            return Err(RuleError::SetupLimit(
                "the River Chain optional rule is not in play (§10.21)",
            ));
        }
        if hexes.is_empty() {
            return Err(RuleError::SetupLimit(
                "the chain must span at least one hex",
            ));
        }
        if hexes.len() > MAX_CHAIN_HEXES {
            return Err(RuleError::SetupLimit(
                "the river chain spans at most four hexes (§10.21)",
            ));
        }
        Ok(())
    }

    /// Read-only check of a pre-placed Zariba hexside in setup (§9.231-9.232):
    /// only during Setup.
    pub fn can_place_zariba(&self) -> Result<(), RuleError> {
        self.require_setup_phase()?;
        // The Zariba is part of the Anglo-Egyptian set-up (§9.211/§9.231).
        self.require_setup_turn(Player::AngloEgyptian)
    }

    /// Read-only check of whether `player` may confirm ready to leave setup
    /// (§9.2/§9.3): must be in Setup and have deployed enough
    /// ([`Self::setup_target_met`]), so a player can't lock in before placing its
    /// order of battle. Re-confirming an already-ready faction is allowed (no-op).
    pub fn can_confirm_setup_ready(&self, player: Player) -> Result<(), RuleError> {
        self.require_setup_phase()?;
        self.require_setup_turn(player)?;
        if !self.setup_target_met(player) {
            return Err(RuleError::SetupIncomplete(
                "deploy your forces before confirming ready",
            ));
        }
        Ok(())
    }

    /// Read-only check of whether a batch of reinforcement placements is legal:
    /// each placement is a canonical counter entering fresh, during its
    /// owner's Movement phase, not already on the board and listed once
    /// (§9.112/§9.113/§9.322); in the Campaign game it follows the order of
    /// appearance. Each destination must satisfy the full stacking rules
    /// (§5.51-5.53), not just the four-unit count, checked *cumulatively* so a
    /// batch that would over-stack a single hex is rejected as a whole.
    pub fn can_place_reinforcements(&self, placements: &[UnitPlacement]) -> Result<(), RuleError> {
        let ids: Vec<UnitId> = placements.iter().map(|p| p.id).collect();
        crate::effects::reject_duplicate_units(&ids)?;
        for p in placements {
            self.reinforcement_preconditions(p)?;
        }
        // §9.112/§9.113: in the Campaign game, off-board arrivals are bound
        // to the order of appearance -- the owning player's wave for the
        // current turn, its quotas, and its leader list.
        if self.scenario == Scenario::Campaign {
            self.validate_campaign_reinforcements(placements)?;
        }
        // Validate each placement against the board *plus* the units placed
        // earlier in this same batch onto the same hex, so two reinforcements
        // landing together can't jointly break stacking.
        for (i, p) in placements.iter().enumerate() {
            // §7.1: a reinforcing unit materialises on its entry hex -- it
            // may not appear on top of enemy units (engaging the enemy is
            // what melee is for). Lone AE leaders do not block a Dervish
            // arrival (§6.51 overrun applies to occupation).
            let enemy = p.profile.identity.owner().opponent();
            if self.units.iter().any(|u| {
                u.position == p.position
                    && u.profile.identity.owner() == enemy
                    && !matches!(u.profile.kind, UnitKind::BritishLeader { .. })
            }) {
                return Err(RuleError::EnemyOccupied(p.position));
            }
            let occupants: Vec<&UnitPlacement> = self
                .units
                .iter()
                .chain(placements[..=i].iter())
                .filter(|u| u.position == p.position)
                .collect();
            stacking_rule(&occupants)?;
        }
        // A peer may not invent unit values (canonical counters only).
        placements.iter().try_for_each(require_canonical_placement)
    }

    /// Per-placement reinforcement preconditions shared by the batch and
    /// single-placement checks: the owner's Movement phase (§9.112/§9.113/§9.322), and a
    /// counter that is not already on the board.
    fn reinforcement_preconditions(&self, p: &UnitPlacement) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        if p.profile.identity.owner() != self.active_player {
            return Err(RuleError::NotYourTurn);
        }
        if self.units.iter().any(|u| u.id == p.id) {
            return Err(RuleError::AlreadyDeployed(p.id));
        }
        if self.eliminated.contains(&p.id) {
            return Err(RuleError::UnitEliminated(p.id));
        }
        Ok(())
    }

    /// Read-only preview of [`Self::can_place_reinforcements`] for a single
    /// placement, for the placing UI's click/preview gate: the campaign
    /// order of appearance (§9.112/§9.113), enemy occupation (§7.1), and
    /// full stacking (§5.51-5.53) — everything of the batch check that one
    /// placement can influence on its own (a lone placement cannot interact
    /// with other batch members). Non-campaign entry (the FoK turn-1 edge,
    /// §9.322) checks board presence instead of the wave schedule.
    pub fn can_place_single_reinforcement(&self, p: &UnitPlacement) -> Result<(), RuleError> {
        self.reinforcement_preconditions(p)?;
        if self.scenario == Scenario::Campaign {
            self.validate_campaign_reinforcements(std::slice::from_ref(p))?;
        }
        let enemy = p.profile.identity.owner().opponent();
        if self.units.iter().any(|u| {
            u.position == p.position
                && u.profile.identity.owner() == enemy
                && !matches!(u.profile.kind, UnitKind::BritishLeader { .. })
        }) {
            return Err(RuleError::EnemyOccupied(p.position));
        }
        self.check_stacking(p, p.position)?;
        require_canonical_placement(p)
    }

    /// Campaign order-of-appearance validation (§9.112 Dervish, §9.113
    /// Anglo-Egyptian). Reinforcements enter during the owning player's
    /// Movement phase; each placement must belong to that side's wave for the
    /// current turn -- by tribe or leader for the Dervish, by the land-unit
    /// cap / three-gunboat quota / free leaders for the Anglo-Egyptian. A
    /// unit may never enter twice, and units that skipped an earlier wave may
    /// still enter in a later one (the schedule gates, it does not expire).
    fn validate_campaign_reinforcements(
        &self,
        placements: &[UnitPlacement],
    ) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        for p in placements {
            let owner = p.profile.identity.owner();
            if owner != self.active_player {
                return Err(RuleError::NotYourTurn);
            }
            if self.units.iter().any(|u| u.id == p.id)
                || self
                    .reinforcements_placed_this_turn
                    .iter()
                    .any(|&(_, id)| id == p.id)
            {
                return Err(RuleError::AlreadyDeployed(p.id));
            }
            let schedule = match owner {
                Player::Dervish => crate::reinforcements::dervish_campaign_schedule(),
                Player::AngloEgyptian => crate::reinforcements::anglo_egyptian_campaign_schedule(),
            };
            let turn = self.current_turn.value();
            let Some(wave) = schedule.wave_for_turn(turn) else {
                return Err(RuleError::NoReinforcementWave { turn });
            };
            // §9.112/§9.113: when the board carries authored entrance-area
            // annotations, arrivals must enter through the annotated hexes
            // (Dervish: west edge south of the Khor Shambat; AE: entrance
            // area / north Nile edge / Abu Alim hut). Boards without the
            // annotation stay permissive (the bot falls back to geometry).
            let entrance_area = match &p.profile.identity {
                crate::UnitIdentity::DervishLeader(_)
                | crate::UnitIdentity::DervishTribal { .. } => {
                    Some(omdurman_types::NamedArea::DervishWestEdge)
                }
                crate::UnitIdentity::AngloEgyptianLeader(_) => {
                    Some(omdurman_types::NamedArea::AngloEgyptianEntrance)
                }
                _ if matches!(p.profile.kind, UnitKind::Gunboat { .. }) => {
                    Some(omdurman_types::NamedArea::GunboatNorthEdge)
                }
                _ if p.profile.identity.is_friendlies() => {
                    Some(omdurman_types::NamedArea::AbuAlimHut)
                }
                _ => Some(omdurman_types::NamedArea::AngloEgyptianEntrance),
            };
            if let Some(area) = entrance_area {
                let annotated = self.board.entrance_hexes(area);
                if !annotated.is_empty() && !annotated.contains(&p.position) {
                    return Err(RuleError::OutsideEntranceArea(p.position));
                }
            }
            match &p.profile.identity {
                crate::UnitIdentity::DervishTribal { tribe } => {
                    if !wave.tribes.contains(tribe) {
                        return Err(RuleError::TribeNotInWave { turn });
                    }
                }
                crate::UnitIdentity::DervishLeader(leader) => {
                    let listed = wave
                        .leaders
                        .iter()
                        .any(|l| matches!(l, crate::reinforcements::CampaignLeader::Dervish(d) if d == leader));
                    if !listed {
                        return Err(RuleError::TribeNotInWave { turn });
                    }
                }
                _ if owner == Player::Dervish => {
                    // Forts, artillery, gunboats: part of the §9.111 initial
                    // force, never reinforcements.
                    return Err(RuleError::TribeNotInWave { turn });
                }
                crate::UnitIdentity::AngloEgyptianLeader(leader) => {
                    let listed = wave.leaders.iter().any(|l| {
                        matches!(l, crate::reinforcements::CampaignLeader::British(d) if d == leader)
                    });
                    if !listed {
                        return Err(RuleError::LeaderNotInWave { turn });
                    }
                }
                _ => {
                    // Non-leader Anglo-Egyptian arrival (§9.113): gunboats
                    // are quota'd three per turn and do not count against
                    // the land-unit cap; land units share the wave's cap
                    // (leaders exempt).
                    let batch_gunboats = placements
                        .iter()
                        .filter(|q| matches!(q.profile.kind, UnitKind::Gunboat { .. }))
                        .count();
                    let batch_land = placements.len() - batch_gunboats;
                    // Count what this side already placed this player-turn,
                    // resolving each recorded id's kind from the board (or
                    // from the current batch for ids placed moments ago).
                    let mut placed_gunboats = 0usize;
                    let mut placed_land = 0usize;
                    for &(player, id) in &self.reinforcements_placed_this_turn {
                        if player != owner {
                            continue;
                        }
                        let is_boat = placements
                            .iter()
                            .find(|q| q.id == id)
                            .or_else(|| self.units.iter().find(|u| u.id == id))
                            .is_some_and(|u| matches!(u.profile.kind, UnitKind::Gunboat { .. }));
                        if is_boat {
                            placed_gunboats += 1;
                        } else {
                            placed_land += 1;
                        }
                    }
                    if matches!(p.profile.kind, UnitKind::Gunboat { .. }) {
                        if placed_gunboats + batch_gunboats > 3 {
                            return Err(RuleError::GunboatQuotaExceeded { turn });
                        }
                    } else if let Some(cap) = wave.unit_cap
                        && placed_land + batch_land > cap
                    {
                        return Err(RuleError::ReinforcementCapExceeded { turn, cap });
                    }
                }
            }
        }
        Ok(())
    }
}
