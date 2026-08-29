//! Compiler-verified traceability anchors.
//!
//! Every Rust item cited by an `[[mapping.impl]]` entry in
//! `docs/traceability.toml` is referenced here in a form the **compiler** must
//! resolve. If a cited type, function, method, field, variant, or const is
//! renamed or removed, this test crate fails to compile -- so the traceability
//! matrix can no longer silently point at a symbol that no longer exists.
//! Every anchor here is a real `use` or item path: there are no string-only
//! "anchors".
//!
//! The companion tests in `traceability.rs` check both directions: every
//! cited symbol must be anchored here (matrix -> paths), and every anchor
//! here must be cited by the matrix (paths -> matrix) -- except owning-type
//! imports that only exist so a `let _ = Type::member;` anchor compiles.
//! The two files cannot drift apart.
//!
//! Reference forms by item kind:
//!   * type / free fn / const / enum variant -> `use path::To::Item;`
//!   * inherent or trait method -> `let _ = Type::method;` (a path to the fn item)
//!   * struct field -> `fn _f(x: Owner) { let _ = x.field; }`
//!
//! Grouped by the source module the symbol lives in.

#![allow(unused_imports, dead_code, path_statements, clippy::no_effect)]

// ===========================================================================
// omdurman-types
// ===========================================================================
mod types_paths {
    // Types / enums.
    use omdurman_types::{
        Faction, HexDirection, HexsideKind, HexsideRef, Location, Scenario, SetupLetter,
        SpriteAnnotation, Terrain, UnitKind,
    };
    // Enum variants (§5.23, §5.44, §9.231 hexside kinds).
    use omdurman_types::HexsideKind::{Breach, Khor, Wall, ZaribaThornHedge, ZaribaTrench};
    // Nile-mouth landmark variant (§9.345).
    use omdurman_types::Location::WhiteNileMouth;

    // Methods.
    #[test]
    fn methods_resolve() {
        let _ = HexsideKind::blocks_los;
        let _ = HexsideKind::blocks_melee;
        let _ = HexsideKind::blocks_movement;
        let _ = HexsideKind::blocks_advance_after_combat;
        let _ = HexsideKind::blocks_zoc;
        let _ = Terrain::blocks_los;
        let _ = Terrain::is_los_trees;
        let _ = Terrain::passable_by_land;
        let _ = Terrain::has_road;
        let _ = Terrain::is_crossroad;
        let _ = UnitKind::is_boat;
        let _ = UnitKind::fires_twice;
        let _ = UnitKind::has_combat_factors;
        // §9.322: the FoK picker allowlist.
        let _ = Scenario::sections_for_picker;
    }
}

// ===========================================================================
// omdurman-hexmap
// ===========================================================================
mod hexmap_paths {
    // Field: GameMap::roads (§6.3 road overlay).
    fn _gamemap_roads(x: omdurman_hexmap::GameMap) {
        let _ = x.roads;
    }
}

// ===========================================================================
// omdurman-rules :: crate root (lib.rs)
// ===========================================================================
mod rules_root_paths {
    use omdurman_rules::{
        BattalionOrdinal, BrigadeIntegrity, BritishLeader, CampaignVictoryLevel, CombatResult,
        DemolitionTarget, FireAttack, FireFactor, FireModifier, FoKVictoryLevel, FriendliesAction,
        GameTurnIndex, GunboatId, GunboatMovement, HexDistance, HistoricalVictoryLevel,
        MeleeAttack, MeleeFactor, MeleeModifier, MineResult, MovementAllowance, MovementPoints,
        NamedGunboat, OldGunboat, OptionalRule, Phase, RangeBand, StackingError, TransportState,
        UnitMovement, UnitProfile, UnitState, VictoryLedger, VictoryPoints, VpEvent, VpSource,
        WeaponClass, ZocReason, brigade_integrity, dervish_leader_for_setup_letter,
        effective_movement_at_night,
    };
    use omdurman_types::{BrigadeId, DayNight, DervishTribe, UnitKind};

    // Enum variants.
    use omdurman_rules::{
        FireModifier::{AngloEgyptianDirectFire, ZaribaThornHedge, ZaribaTrenchEntrenched},
        FireSubPhase::{self, DirectFire, MaximSecondAndHowitzer},
        GunboatId::{DervishGunboat, Old},
        MeleeModifier::{AngloEgyptianStandard, DervishStandard, DervishVsTrenchedDefender},
        StackingError::{DervishLeaderCommandMismatch, DervishTribeMix, GunboatStack, OverLimit},
        UnitIdentity::RoyalEngineers,
        UnitMovement::Immobile,
        WeaponClass::Howitzer,
        ZocReason::{Fort, Zariba},
    };
    // §5.23: walled-city entry RuleError variant.
    use omdurman_rules::effects::RuleError::WalledCityEntry;

    #[test]
    fn methods_resolve() {
        let _ = BrigadeIntegrity::None; // marker reference
        let _ = CampaignVictoryLevel::from_superiority;
        let _ = FoKVictoryLevel::resolve;
        let _ = FireAttack::net_modifier;
        let _ = FireModifier::die_modifier;
        let _ = FireModifier::BrigadeIntegrity;
        let _ = MeleeModifier::die_modifier;
        let _ = GameTurnIndex::value;
        let _ = MeleeFactor::sum(std::iter::empty::<&MeleeFactor>());
        let _ = MovementAllowance::halve;
        let _ = UnitKind::may_be_melee_attacked;
        let _ = UnitKind::may_melee_attack;
        let _ = UnitKind::may_retreat_before_melee;
        let _ = VpSource::points;
        let _ = VpSource::who_scores;
        let _ = VictoryLedger::total_for;
        let _ = VictoryLedger::superiority;
        let _ = omdurman_rules::UnitIdentity::is_friendlies;
        let _ = omdurman_rules::UnitIdentity::is_gordon;
        let _ = omdurman_rules::UnitIdentity::may_enter_walled_city;
        let _ = omdurman_rules::DervishLeader::setup_letter;
    }

    // Fields on UnitState (§5.21, §5.3, §6.53).
    fn _unitstate_fields(x: UnitState) {
        let _ = x.loaded_on;
        let _ = x.constructing_zariba;
        let _ = x.demolishing;
    }
}

// ===========================================================================
// omdurman-rules :: effects.rs
// ===========================================================================
mod rules_effects_paths {
    use omdurman_rules::effects::{GameState, MAX_CHAIN_HEXES, PendingMelee};
    // GameEffect variants (§4, §5, §6, §7, §8, §10).
    use omdurman_rules::effects::GameEffect::{
        AdvanceAfterCombat, AdvancePhase, ArtilleryBreachWall, ConstructZariba, Demolition,
        DervishDesertion, FireCombat, FriendliesTransport, HowitzerFire, MeleeCombat,
        PlaceReinforcements, RetreatBeforeMelee, RiverMine,
    };

    #[test]
    fn fns_and_methods_resolve() {
        // Free effect-processing functions.
        let _ = omdurman_rules::effects::advance_phase;
        let _ = omdurman_rules::effects::end_player_turn;
        let _ = omdurman_rules::effects::apply_advance_after_combat;
        let _ = omdurman_rules::effects::apply_construct_zariba;
        let _ = omdurman_rules::effects::apply_demolition;
        let _ = omdurman_rules::effects::apply_friendlies_transport;
        let _ = omdurman_rules::effects::apply_howitzer_fire;
        let _ = omdurman_rules::effects::apply_melee_combat;
        let _ = omdurman_rules::effects::apply_place_reinforcements;
        let _ = omdurman_rules::effects::apply_retreat_before_melee;
        let _ = omdurman_rules::effects::apply_place_mine;
        let _ = omdurman_rules::effects::apply_place_chain;
        let _ = omdurman_rules::effects::apply_river_mine;
        let _ = omdurman_rules::effects::apply_sink_chain;
        let _ = omdurman_rules::effects::apply_artillery_breach_wall;
        let _ = omdurman_rules::effects::score_elimination;
        let _ = omdurman_rules::effects::first_player;
        // Fall of Khartoum special rules (§9.343, §9.345, §9.346).
        let _ = omdurman_rules::effects::range_band_for;
        let _ = omdurman_rules::effects::check_gordon_palace;

        // GameState query/command methods.
        let _ = GameState::new;
        let _ = GameState::setup_complete;
        let _ = GameState::can_move_unit;
        let _ = GameState::can_move_unit_to;
        let _ = GameState::can_move_gunboat;
        let _ = GameState::in_deployment_zone;
        let _ = GameState::can_fire_at;
        let _ = GameState::can_melee;
        let _ = GameState::can_advance_after_combat;
        let _ = GameState::can_retreat_before_melee;
        let _ = GameState::can_place_chain;
        let _ = GameState::hex_in_enemy_zoc;
        let _ = GameState::unit_projects_zoc;
        let _ = GameState::hex_has_enemy_fort;
        let _ = GameState::is_nile_mouth_crossing;
        let _ = GameState::mp_spent;
        let _ = GameState::movement_cost_for;
        let _ = GameState::can_fire_at_wall;
        let _ = GameState::check_stacking;
        let _ = GameState::zoc_hexes;
        let _ = GameState::demolition_targets;
        let _ = GameState::friendlies_transport_offer;
        let _ = omdurman_rules::effects::apply_move_unit;
    }

    // Fields on GameState (§10.11, §10.21).
    fn _gamestate_fields(x: GameState) {
        let _ = x.mines;
    }
}

// ===========================================================================
// omdurman-rules :: other submodules
// ===========================================================================
mod rules_submodule_paths {
    use omdurman_rules::board::{BoardInfo, NileBank, StepDirection};
    use omdurman_rules::board_data::fall_of_khartoum_map_data;
    use omdurman_rules::combat_results_table::{FireFactorRow, combat_results_table};
    use omdurman_rules::howitzer_scatter::{ScatterHexDirection, howitzer_scatter};
    use omdurman_rules::los_table::{
        LosCondition, LosFeature, LosLevel, LosStepResult, blocking_rules, has_los, los_level,
        los_level_for_unit, los_path_analysis,
    };
    use omdurman_rules::range_effects::{ae_range_effects, dervish_range_effects, night_max_range};
    use omdurman_rules::reinforcements::{
        anglo_egyptian_campaign_schedule, dervish_campaign_schedule,
    };
    use omdurman_rules::terrain_chart::{
        defense_modifier, movement_cost, movement_cost_with_road, terrain_effects_chart,
    };
    use omdurman_rules::turn_track::{
        CAMPAIGN_TURN_TRACK, FALL_OF_KHARTOUM_TURN_TRACK, GameTime, HISTORICAL_TURN_TRACK,
        TurnEntry, TurnEvent, TurnLabel,
    };
    // TurnEvent variant (§8.2).
    use omdurman_rules::turn_track::TurnEvent::DervishDesertion;

    #[test]
    fn methods_resolve() {
        let _ = FireFactorRow::from_total;
        let _ = omdurman_rules::FireFactor::sum_to_row(std::iter::empty::<
            &omdurman_rules::FireFactor,
        >());
        let _ = BoardInfo::is_walled_city;
        let _ = BoardInfo::zariba_entry_surcharge;
        let _ = BoardInfo::has_zariba_thorn_hedge;
    }
}

// ===========================================================================
// omdurman-rules :: unit_profiles (cell-by-cell counter classification)
// ===========================================================================
mod rules_unit_profiles_paths {
    // §2.31: Dervish tribe weapon classification (Spears vs Rifles).
    use omdurman_rules::unit_profiles::dervish_tribe;
    // §2.31 / §9.322: cell-by-cell section resolvers.
    use omdurman_rules::unit_profiles::{ali_wad_helu, khalifa_abdullah};

    #[test]
    fn fns_resolve() {
        let _ = (dervish_tribe, khalifa_abdullah, ali_wad_helu);
    }
}

// ===========================================================================
// omdurman-rules :: scenario_setup (fixed-hex scenario placements)
// ===========================================================================
mod rules_scenario_setup_paths {
    // §9.321 / §9.344 / §9.346: the FoK fixed-placement table.
    use omdurman_rules::scenario_setup::FALL_OF_KHARTOUM_SETUP;

    #[test]
    fn consts_resolve() {
        let _ = FALL_OF_KHARTOUM_SETUP;
    }
}
