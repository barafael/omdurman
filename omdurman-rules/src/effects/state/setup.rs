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

/// Whether counter `id` is in play in the Historical scenario: a real
/// counter ([`profile_for_unit`](crate::unit_profiles::profile_for_unit))
/// of a type the scenario uses ([`historical_in_play`]). The picker, the bot
/// and [`GameState::setup_target`] all ask this.
pub fn historical_counter_in_play(id: UnitId) -> bool {
    crate::unit_profiles::profile_for_unit(id).is_some_and(|p| historical_in_play(&p.identity))
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
    /// for the first Movement turn (§9.2/§9.3/§10): each faction has deployed
    /// its [`Self::setup_target`]. Where each unit may stand is checked as it
    /// deploys (`can_deploy_unit`), and river mines/chain at placement time, so
    /// neither needs a re-check here.
    ///
    /// Returns [`RuleError::SetupIncomplete`] naming the first unmet requirement,
    /// so the UI can surface *why* "Begin battle" is disabled.
    pub fn setup_complete(&self) -> Result<(), RuleError> {
        // Every scenario pins both orders of battle (§9.111/§9.113,
        // §9.211-9.212, §9.321-9.322) -- in the Campaign the A-E side starts
        // with *no* units on the map. The per-faction Ready button already
        // gates on `setup_target_met`; this is defense-in-depth for the
        // unbound "Begin battle" path and any future caller.
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

    /// The number of units `player` must deploy before turn 1. **Fall of
    /// Khartoum**: British 17, Dervish 48
    /// (§9.321-9.322), plus the scenario-fixed forts (Makran and Buri, the
    /// §9.344 North Fort). **Historical**: every counter in play -- "all
    /// remaining Anglo-Egyptian units set up in the 13 hexes of the Zariba",
    /// "all remaining Dervish units set up within three hexes of their
    /// leader" (§9.211/§9.212). **Campaign**: the Dervish initial force
    /// (§9.111); the A-E player starts with *no* units on the map (§9.113).
    pub fn setup_target(&self, player: Player) -> usize {
        match (self.scenario, player) {
            // 17 player-deployed garrison + the scenario-fixed Forts Makran
            // and Buri.
            (Scenario::FallOfKhartoum, Player::AngloEgyptian) => 19,
            // 48 player-deployed entry force + 1 scenario-fixed North Fort fort.
            (Scenario::FallOfKhartoum, Player::Dervish) => 49,
            // Every counter in play (`historical_counter_in_play`), fixed
            // here and pinned to the roster by a test.
            (Scenario::Historical, Player::AngloEgyptian) => 52,
            (Scenario::Historical, Player::Dervish) => 143,
            // §9.111: the whole Dervish initial force -- 1 Isa Zachneih, the
            // Khalifa, 3 guns, 14 Taiasha, 17 forts and 2 gunboats; the
            // Anglo-Egyptians start with nothing on the map (§9.113).
            (Scenario::Campaign, Player::Dervish) => 38,
            (Scenario::Campaign, Player::AngloEgyptian) => 0,
        }
    }

    /// Whether `player` has deployed its `setup_target` and may confirm ready.
    pub fn setup_target_met(&self, player: Player) -> bool {
        self.setup_deployed_count(player) >= self.setup_target(player)
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
    /// - **Historical** (§9.211-9.212): the union of the set-up areas (for
    ///   the rings). Which area binds depends on the counter (the Kerreri
    ///   detachment, a leader's colour), so placements are checked by
    ///   [`Self::historical_set_up_area`] instead.
    /// - **Campaign** (§9.111): the union of the Dervish initial force's
    ///   areas ([`Self::campaign_set_up_area`]); nothing for the
    ///   Anglo-Egyptians.
    pub fn in_deployment_zone(&self, player: Player, hex: HexCoord, is_boat: bool) -> bool {
        // No board attached -> permissive (unit tests, unbound session).
        if self.board.terrain.is_empty() {
            return true;
        }
        if !self.on_deployable_terrain(hex, is_boat) {
            return false;
        }
        match self.scenario {
            // The union of the §9.111 set-up areas, for the rings; which one
            // binds a counter is [`Self::campaign_set_up_area`]'s business.
            // The Anglo-Egyptians set nothing up (§9.113).
            Scenario::Campaign => {
                player == Player::Dervish
                    && if is_boat {
                        self.in_campaign_area(CampaignArea::SouthEdgeNile, hex)
                    } else {
                        [
                            CampaignArea::EastBankFromElDebeba,
                            CampaignArea::Palace,
                            CampaignArea::WalledCity,
                            CampaignArea::FortGround,
                        ]
                        .into_iter()
                        .any(|area| self.in_campaign_area(area, hex))
                    }
            }
            // The union of the §9.211/§9.212 set-up areas, for the rings;
            // which one binds a given counter is
            // [`Self::historical_set_up_area`]'s business.
            Scenario::Historical => match player {
                Player::AngloEgyptian => {
                    self.historical_ae_area(hex, is_boat, false)
                        || self.historical_ae_area(hex, is_boat, true)
                }
                Player::Dervish => {
                    !is_boat
                        && self.near_dervish_leader(hex, None)
                        && self
                            .army_ashore_sighting(
                                hex,
                                UnitKind::Infantry {
                                    fire: 0,
                                    melee: 0,
                                    movement: 0,
                                },
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

    /// Where a counter of the Campaign's Dervish initial force may set up
    /// (§9.111): the Isa Zachneih on the east bank in or south of El Debeba;
    /// the Khalifa in either palace hex; the artillery and the Taiasha in the
    /// walled city; the forts on the west bank south of the Khor Shambat, or
    /// south of all the Halfaya huts on the east bank or a Nile island; the
    /// gunboats on south-edge Nile hexes. Permissive without a board, like
    /// [`Self::in_deployment_zone`].
    pub fn campaign_set_up_area(&self, placement: &UnitPlacement) -> Result<(), RuleError> {
        if self.board.terrain.is_empty() {
            return Ok(());
        }
        let Some(area) = campaign_initial_area(&placement.profile.identity) else {
            return Err(RuleError::NotInPlay(placement.id));
        };
        if self.in_campaign_area(area, placement.position) {
            Ok(())
        } else {
            Err(RuleError::CampaignSetUpArea {
                hex: placement.position,
                area: area.describe(),
            })
        }
    }

    /// Whether `hex` lies in the §9.111 set-up `area`.
    fn in_campaign_area(&self, area: CampaignArea, hex: HexCoord) -> bool {
        use omdurman_types::Location;
        let board = &self.board;
        let rows_of = |loc: Location| {
            board
                .locations
                .iter()
                .filter(move |(_, l)| **l == loc)
                .map(|(h, _)| h.r)
        };
        let east = board.bank_of(hex) == Some(crate::board::NileBank::East);
        match area {
            // "anywhere on the east bank, in or south of El Debeba"
            CampaignArea::EastBankFromElDebeba => {
                east && rows_of(Location::ElDebeba)
                    .min()
                    .is_some_and(|r| hex.r >= r)
            }
            // "in the walled city of Omdurman, in either palace hex"
            CampaignArea::Palace => matches!(
                board.location_at(hex),
                Some(Location::Palace | Location::PalaceGrounds)
            ),
            CampaignArea::WalledCity => board.is_walled_city(hex),
            // "south of the Khor Shambat on the west bank, and/or south of
            // all Halfaya hut hexes on the east bank and Nile River islands"
            CampaignArea::FortGround => {
                let island = !board.is_nile(hex) && board.bank_of(hex).is_none();
                board.south_of_khor_shambat.contains(&hex)
                    || ((east || island)
                        && rows_of(Location::Halfaya).max().is_some_and(|r| hex.r > r))
            }
            // "any south edge Nile River hexes"
            CampaignArea::SouthEdgeNile => {
                board.is_nile(hex) && board.terrain.keys().all(|h| h.r <= hex.r)
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
            let is_boat = placement.profile.kind.is_boat();
            let kerreri = HISTORICAL_KERRERI_UNITS.contains(&placement.id);
            if self.historical_ae_area(hex, is_boat, kerreri) {
                return Ok(());
            }
            let area = if is_boat {
                "gunboats start in Nile hexes adjacent to the Zariba (§9.211)"
            } else if kerreri {
                "the Camel Corps, Egyptian Cavalry and Horse Artillery start in the \
                 village of Kerreri hut hexes (§9.211)"
            } else {
                "the Anglo-Egyptian units set up in the 13 hexes of the Zariba (§9.211)"
            };
            return Err(RuleError::HistoricalSetUpArea { hex, area });
        }
        let leader = match placement.profile.identity {
            UnitIdentity::DervishTribal { tribe } => crate::DervishLeader::of_tribe(tribe),
            // The three guns are the Khalifa's black counters.
            UnitIdentity::DervishArtillery => Some(crate::DervishLeader::KhalifaAbdullah),
            _ => None,
        };
        if let Some(leader) = leader
            && !self.near_dervish_leader(hex, Some(leader))
        {
            return Err(RuleError::SetUpFarFromLeader { hex, leader });
        }
        match self.army_ashore_sighting(hex, placement.profile.kind) {
            Some(seen_from) => Err(RuleError::SetUpInEnemySight { hex, seen_from }),
            None => Ok(()),
        }
    }

    /// The hex of an Anglo-Egyptian land unit with a line of sight to a unit
    /// of `kind` on `hex`, if any -- the §9.212 "out of the line of sight"
    /// test (gunboats excluded, see [`Self::historical_set_up_area`]). Each
    /// occupied hex is one viewpoint, however many units stack there.
    fn army_ashore_sighting(&self, hex: HexCoord, kind: UnitKind) -> Option<HexCoord> {
        let level = crate::los_table::los_level_for_unit(kind, hex, &self.board);
        let mut viewpoints: Vec<HexCoord> = Vec::new();
        self.units
            .iter()
            .filter(|u| {
                u.profile.identity.owner() == Player::AngloEgyptian && !u.profile.kind.is_boat()
            })
            .filter(|u| {
                let fresh = !viewpoints.contains(&u.position);
                if fresh {
                    viewpoints.push(u.position);
                }
                fresh
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

    /// Whether a counter may stand on `hex` at all during set-up: a hex of
    /// the board, and (§5.22, every scenario) the Nile for a gunboat, dry
    /// land for everything else. Permissive without a board.
    pub fn on_deployable_terrain(&self, hex: HexCoord, is_boat: bool) -> bool {
        if self.board.terrain.is_empty() {
            return true;
        }
        match self.board.terrain_at(hex) {
            None => false, // off the playable map
            Some(terrain) => matches!(terrain, omdurman_types::Terrain::Nile { .. }) == is_boat,
        }
    }

    /// An Anglo-Egyptian Historical set-up area (§9.211): Nile hexes beside
    /// the Zariba for a gunboat, the Kerreri huts for the Kerreri
    /// detachment, the Zariba's hexes for everything else.
    fn historical_ae_area(&self, hex: HexCoord, is_boat: bool, kerreri: bool) -> bool {
        if is_boat {
            hex.neighbors().into_iter().any(|n| self.board.is_zariba(n))
        } else if kerreri {
            self.board.location_at(hex) == Some(omdurman_types::Location::Kerreri)
        } else {
            self.board.is_zariba(hex)
        }
    }

    /// Whether a Dervish leader -- `leader`, or any when `None` -- stands
    /// within three hexes of `hex` (§9.212).
    fn near_dervish_leader(&self, hex: HexCoord, leader: Option<crate::DervishLeader>) -> bool {
        self.units.iter().any(|u| match u.profile.identity {
            crate::UnitIdentity::DervishLeader(l) => {
                leader.is_none_or(|want| want == l) && u.position.distance(hex) <= 3
            }
            _ => false,
        })
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
        let (owner, hex) = (placement.profile.identity.owner(), placement.position);
        let is_boat = placement.profile.kind.is_boat();
        if self.scenario == Scenario::Historical {
            // The counter's own area (§9.211/§9.212), after the cheap checks:
            // the Dervish one walks lines of sight. The fixed leaders stand
            // on their lettered hexes.
            if !self.on_deployable_terrain(hex, is_boat) {
                return Err(RuleError::OutsideDeploymentZone(hex));
            }
            self.check_stacking(placement, hex)?;
            if !fixed {
                self.historical_set_up_area(placement)?;
            }
        } else if self.scenario == Scenario::Campaign {
            if !self.board.terrain.is_empty() && !self.on_deployable_terrain(hex, is_boat) {
                return Err(RuleError::OutsideDeploymentZone(hex));
            }
            self.check_stacking(placement, hex)?;
            self.campaign_set_up_area(placement)?;
        } else {
            if !self.in_deployment_zone(owner, hex, is_boat) {
                return Err(RuleError::OutsideDeploymentZone(hex));
            }
            self.check_stacking(placement, hex)?;
        }
        // Set-up order (§9.111/§9.211/§9.321), checked after the placement's
        // own legality so a misplaced counter reports what is wrong with it.
        if !fixed {
            self.require_setup_turn(owner)?;
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
        match self.scenario {
            // §9.111: the Dervish initial force; everyone else arrives as a
            // reinforcement (§9.112/§9.113).
            Scenario::Campaign => match campaign_initial_area(&placement.profile.identity) {
                Some(_) => Ok(()),
                None => Err(RuleError::NotInPlay(placement.id)),
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
        // §10: "Optional Rules (Campaign game only)".
        if self.scenario != Scenario::Campaign {
            return Err(RuleError::SetupLimit(
                "the optional rules are for the campaign game only (§10)",
            ));
        }
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
        self.check_river_obstacle_hex(hex)
    }

    /// §10.11/§10.21: the river obstacles lie in Nile hexes "south of the
    /// E–W hexrow in which the Khor Shambat empties into the Nile" (with no
    /// board loaded, map constraints don't apply).
    fn check_river_obstacle_hex(&self, hex: HexCoord) -> Result<(), RuleError> {
        if self.board.terrain.is_empty() {
            return Ok(());
        }
        if !self.board.is_nile(hex) {
            return Err(RuleError::SetupLimit(
                "mines and the chain lie in Nile hexes (§10.11, §10.21)",
            ));
        }
        if self
            .board
            .khor_shambat_mouth_row()
            .is_some_and(|row| hex.r <= row)
        {
            return Err(RuleError::SetupLimit(
                "south of the hexrow where the Khor Shambat empties into the Nile (§10.11, §10.21)",
            ));
        }
        Ok(())
    }

    /// Read-only check of a river-chain placement in setup (§10.21): Setup phase
    /// and at most [`MAX_CHAIN_HEXES`] hexes.
    pub fn can_place_chain(&self, hexes: &[HexCoord]) -> Result<(), RuleError> {
        self.require_setup_phase()?;
        // §10: "Optional Rules (Campaign game only)".
        if self.scenario != Scenario::Campaign {
            return Err(RuleError::SetupLimit(
                "the optional rules are for the campaign game only (§10)",
            ));
        }
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
        // "a line of river hexes": each hex next to the one before, none
        // twice.
        for (i, hex) in hexes.iter().enumerate() {
            if hexes[..i].contains(hex) || (i > 0 && !hexes[i - 1].is_adjacent_to(*hex)) {
                return Err(RuleError::SetupLimit(
                    "the chain is strung along a line of adjacent river hexes (§10.21)",
                ));
            }
            self.check_river_obstacle_hex(*hex)?;
        }
        Ok(())
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
        // Only the Campaign has reinforcements (§9.112/§9.113); the other
        // scenarios set everything up before play (§9.21, §9.32).
        if self.scenario != Scenario::Campaign {
            return Err(RuleError::NotInPlay(p.id));
        }
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
    /// Movement phase, through their entrance area, on their wave's turn or
    /// later (a unit held back is not lost): the Dervish by tribe and
    /// leader; the Anglo-Egyptians with the three leaders free (turns 1-4),
    /// up to three gunboats a turn until turn 4, on turn 1 only the printed
    /// first wave (the "Friendlies", the Egyptian Cavalry, the Horse
    /// Artillery and two Egyptian Division brigades), on turns 2 and 3 up to
    /// twelve land units, and from turn 4 "all remaining".
    fn validate_campaign_reinforcements(
        &self,
        placements: &[UnitPlacement],
    ) -> Result<(), RuleError> {
        use crate::UnitIdentity;
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        let turn = self.current_turn.value();
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
            let due: Vec<&crate::reinforcements::ReinforcementWave> =
                schedule.waves.iter().filter(|w| w.turn <= turn).collect();
            if due.is_empty() {
                return Err(RuleError::NoReinforcementWave { turn });
            }
            // §9.112/§9.113: through the side's entrance area (Dervish: the
            // west edge south of the Khor Shambat; Anglo-Egyptian: the
            // entrance area, the north-edge Nile for gunboats, the Abu Alim
            // hut for the "Friendlies"), onto ground the unit may stand on.
            let annotated = self.board.entrance_hexes(entrance_area_for(&p.profile));
            if !annotated.is_empty() && !annotated.contains(&p.position) {
                return Err(RuleError::OutsideEntranceArea(p.position));
            }
            if !self.board.terrain.is_empty()
                && !self.on_deployable_terrain(p.position, p.profile.kind.is_boat())
            {
                return Err(RuleError::OutsideEntranceArea(p.position));
            }
            match &p.profile.identity {
                UnitIdentity::DervishTribal { tribe } => {
                    if !due.iter().any(|w| w.tribes.contains(tribe)) {
                        return Err(RuleError::TribeNotInWave { turn });
                    }
                }
                UnitIdentity::DervishLeader(leader) => {
                    let listed = due.iter().flat_map(|w| &w.leaders).any(|l| {
                        matches!(l, crate::reinforcements::CampaignLeader::Dervish(d) if d == leader)
                    });
                    if !listed {
                        return Err(RuleError::TribeNotInWave { turn });
                    }
                }
                _ if owner == Player::Dervish => {
                    // Forts, artillery, gunboats: part of the §9.111 initial
                    // force, never reinforcements.
                    return Err(RuleError::TribeNotInWave { turn });
                }
                UnitIdentity::AngloEgyptianLeader(leader) => {
                    let listed = due.iter().flat_map(|w| &w.leaders).any(|l| {
                        matches!(l, crate::reinforcements::CampaignLeader::British(d) if d == leader)
                    });
                    if !listed {
                        return Err(RuleError::LeaderNotInWave { turn });
                    }
                }
                // Fall of Khartoum's forts are no Campaign counters.
                UnitIdentity::AngloEgyptianFort => return Err(RuleError::NotInPlay(p.id)),
                _ => self.check_ae_arrival_quota(p, placements, turn)?,
            }
        }
        Ok(())
    }

    /// §9.113's per-turn limits for an Anglo-Egyptian arrival other than a
    /// leader, counting what already arrived this turn plus the batch.
    fn check_ae_arrival_quota(
        &self,
        p: &UnitPlacement,
        batch: &[UnitPlacement],
        turn: u8,
    ) -> Result<(), RuleError> {
        // Everything arriving this turn: the recorded arrivals (resolved from
        // the board) and the batch.
        let arrivals: Vec<&UnitPlacement> = self
            .reinforcements_placed_this_turn
            .iter()
            .filter(|&&(player, _)| player == Player::AngloEgyptian)
            .filter_map(|&(_, id)| self.find_unit(id))
            .chain(batch.iter())
            .filter(|u| !matches!(u.profile.kind, UnitKind::BritishLeader { .. }))
            .collect();
        // "Turn 4) All remaining Anglo-Egyptian units."
        if turn >= 4 {
            return Ok(());
        }
        if p.profile.kind.is_boat() {
            // "Any three gunboats" a turn.
            if arrivals.iter().filter(|u| u.profile.kind.is_boat()).count() > 3 {
                return Err(RuleError::GunboatQuotaExceeded { turn });
            }
            return Ok(());
        }
        if turn == 1 {
            return check_first_wave(p, &arrivals);
        }
        // Turns 2 and 3: "any twelve land units".
        let land = arrivals
            .iter()
            .filter(|u| !u.profile.kind.is_boat())
            .count();
        if land > 12 {
            return Err(RuleError::ReinforcementCapExceeded { turn, cap: 12 });
        }
        Ok(())
    }
}

/// §9.113 turn 1: "'Friendlies' brigade; Egyptian Cavalry; Horse Artillery;
/// and two infantry brigades from the Egyptian Division" -- `p` must be one
/// of them, and the turn's arrivals may not span a third Egyptian Division
/// brigade.
fn check_first_wave(p: &UnitPlacement, arrivals: &[&UnitPlacement]) -> Result<(), RuleError> {
    // The Egyptian Cavalry's two counters and the Horse Artillery.
    const MOUNTED: [UnitId; 3] = [
        UnitId::EgyptianArmy_0_0,
        UnitId::EgyptianArmy_1_0,
        UnitId::EgyptianArmy_2_0,
    ];
    let division_brigade = |u: &UnitPlacement| match u.profile.identity {
        crate::UnitIdentity::AngloEgyptianInfantry { .. } => u
            .profile
            .identity
            .brigade()
            .filter(|b| b.nationality == omdurman_types::BrigadeNationality::Egyptian),
        _ => None,
    };
    if p.profile.identity.is_friendlies() || MOUNTED.contains(&p.id) {
        return Ok(());
    }
    if division_brigade(p).is_none() {
        return Err(RuleError::NotInFirstWave(p.id));
    }
    let mut brigades: Vec<omdurman_types::BrigadeId> = arrivals
        .iter()
        .filter_map(|u| division_brigade(u))
        .collect();
    brigades.sort_by_key(|b| b.number);
    brigades.dedup();
    if brigades.len() > 2 {
        return Err(RuleError::NotInFirstWave(p.id));
    }
    Ok(())
}

/// The entrance area a Campaign reinforcement arrives through (§9.112,
/// §9.113): the Dervish at the west edge south of the Khor Shambat; the
/// Anglo-Egyptian gunboats at a north-edge Nile hex, the "Friendlies" at the
/// Abu Alim hut, everyone else at the Anglo-Egyptian Entrance Area.
pub fn entrance_area_for(profile: &crate::UnitProfile) -> omdurman_types::NamedArea {
    use omdurman_types::NamedArea;
    if profile.identity.owner() == Player::Dervish {
        NamedArea::DervishWestEdge
    } else if profile.kind.is_boat() {
        NamedArea::GunboatNorthEdge
    } else if profile.identity.is_friendlies() {
        NamedArea::AbuAlimHut
    } else {
        NamedArea::AngloEgyptianEntrance
    }
}

impl GameState {
    /// Whether the §8.2 Dervish desertion roll is due now: "once each
    /// campaign game, during the first night turn of the game ... during the
    /// movement phase" -- the Dervish player's, since he rolls and chooses.
    pub fn desertion_due(&self) -> bool {
        self.scenario == Scenario::Campaign
            && self.phase == Phase::Movement
            && self.active_player == Player::Dervish
            && self.day_night == DayNight::Night
            && !self.dervish_deserted
            && crate::turn_track::scenario_turn(self.scenario, self.current_turn)
                .is_some_and(|t| t.event == crate::turn_track::TurnEvent::DervishDesertion)
    }

    /// How many Dervish units a desertion `roll` removes (§8.2): 1½ times
    /// the roll, rounded up ([`desertion_count`](super::desertion_count)),
    /// but never more than the units eligible to desert -- a Dervish army
    /// already bled below that deserts everything it can.
    pub fn desertion_demand(&self, roll: DieRoll) -> usize {
        let eligible = self
            .units
            .iter()
            .filter(|u| {
                u.profile.identity.owner() == Player::Dervish
                    && !u.profile.identity.is_desertion_exempt()
            })
            .count();
        super::desertion_count(roll).min(eligible)
    }
}

/// A §9.111 set-up area of the Campaign's Dervish initial force.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CampaignArea {
    /// Isa Zachneih: the east bank, in or south of El Debeba.
    EastBankFromElDebeba,
    /// The Khalifa: either palace hex.
    Palace,
    /// The artillery and the Taiasha: the walled city of Omdurman.
    WalledCity,
    /// The forts: south of the Khor Shambat, or of the Halfaya huts.
    FortGround,
    /// The gunboats: the south-edge Nile hexes.
    SouthEdgeNile,
}

impl CampaignArea {
    fn describe(self) -> &'static str {
        match self {
            CampaignArea::EastBankFromElDebeba => {
                "the Isa Zachneih sets up on the east bank, in or south of El Debeba (§9.111)"
            }
            CampaignArea::Palace => "the Khalifa sets up in either palace hex (§9.111)",
            CampaignArea::WalledCity => {
                "the artillery and the Taiasha set up in the walled city of Omdurman (§9.111)"
            }
            CampaignArea::FortGround => {
                "forts set up south of the Khor Shambat on the west bank, or south of \
                 all the Halfaya huts on the east bank or a Nile island (§9.111)"
            }
            CampaignArea::SouthEdgeNile => "the gunboats set up on south-edge Nile hexes (§9.111)",
        }
    }
}

/// Whether a counter is part of the Campaign's Dervish initial force
/// (§9.111) -- set up before play rather than arriving as a reinforcement.
pub fn in_campaign_initial_force(identity: &crate::UnitIdentity) -> bool {
    campaign_initial_area(identity).is_some()
}

/// The §9.111 set-up area of a counter of the Campaign's Dervish initial
/// force -- `None` for everything that arrives as a reinforcement
/// (§9.112/§9.113).
fn campaign_initial_area(identity: &crate::UnitIdentity) -> Option<CampaignArea> {
    use crate::{DervishTribe, UnitIdentity};
    match identity {
        UnitIdentity::DervishTribal {
            tribe: DervishTribe::IsaZachneih,
        } => Some(CampaignArea::EastBankFromElDebeba),
        UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah) => {
            Some(CampaignArea::Palace)
        }
        UnitIdentity::DervishTribal {
            tribe: DervishTribe::Taiasha,
        }
        | UnitIdentity::DervishArtillery => Some(CampaignArea::WalledCity),
        UnitIdentity::DervishFort => Some(CampaignArea::FortGround),
        UnitIdentity::DervishGunboat(_) => Some(CampaignArea::SouthEdgeNile),
        _ => None,
    }
}
