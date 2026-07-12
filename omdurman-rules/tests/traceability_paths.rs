//! Compiler-verified traceability anchors.
//!
//! Every Rust item cited by an `[[mapping.impl]]` entry in
//! `docs/traceability.toml` is referenced here in a form the **compiler** must
//! resolve. If a cited type, function, method, field, variant, or const is
//! renamed or removed, this test crate fails to compile -- so the traceability
//! matrix can no longer silently point at a symbol that no longer exists.
//!
//! The companion test in `traceability.rs` checks the *other* direction (every
//! mapping section exists, every `§` citation is mapped) and that every symbol
//! named here also appears in the TOML, so the two files cannot drift apart.
//!
//! Reference forms by item kind:
//!   * type / free fn / const / enum variant -> `use path as _;`
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
        Brigade, Faction, HexDirection, HexsideKind, HexsideRef, Location, NileFlow, SetupLetter,
        SpriteAnnotation, Terrain, UnitFormKind,
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
        let _ = Terrain::overlay_color;
        let _ = UnitFormKind::is_boat;
        let _ = UnitFormKind::fires_twice;
        let _ = UnitFormKind::has_combat_factors;
    }

    // Fields (§5.11, §5.24 crossroad annotation).
    fn _hexdata_is_crossroad(x: omdurman_types::HexData) {
        let _ = x.is_crossroad;
    }
    fn _tileinfo_is_crossroad(x: omdurman_types::TileInfo) {
        let _ = x.is_crossroad;
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
        BattalionOrdinal, BrigadeId, BrigadeIntegrity, BritishLeader, CampaignVictoryLevel,
        CombatResult, DayNight, DemolitionTarget, DervishTribe, FireAttack, FireFactor,
        FireModifier, FoKVictoryLevel, FriendliesTransport, GameTurnIndex, GunboatId,
        GunboatMovement, HexDistance, HistoricalVictoryLevel, HowitzerResolution, MeleeAttack,
        MeleeFactor, MeleeModifier, MineResult, MovementAllowance, MovementPoints, NamedGunboat,
        OldGunboat, OptionalRule, Phase, Range, RangeBand, StackingError, UnitKind, UnitMovement,
        UnitProfile, UnitState, VictoryLedger, VictoryPoints, VpEvent, VpSource, WeaponClass,
        ZocReason, brigade_integrity, effective_movement_at_night, effective_range_at_night,
    };

    // Enum variants.
    use omdurman_rules::{
        DieModifier,
        FireModifier::{AngloEgyptianDirectFire, ZaribaThornHedge, ZaribaTrenchEntrenched},
        FireSubPhase::{self, DirectFire, MaximSecondAndHowitzer},
        GunboatId::{DervishGunboat, Old},
        MeleeModifier::{AngloEgyptianStandard, DervishStandard, DervishVsTrenchedDefender},
        StackingError::{DervishLeaderCommandMismatch, DervishTribeMix, GunboatStack, OverLimit},
        UnitIdentity::RoyalEngineers,
        UnitKind::{BritishLeaderUnit, Fort},
        UnitMovement::Immobile,
        WeaponClass::Howitzer,
        ZocReason::Zariba,
    };

    #[test]
    fn methods_resolve() {
        let _ = BrigadeIntegrity::None; // marker reference
        let _ = CampaignVictoryLevel::from_superiority;
        let _ = FoKVictoryLevel::resolve;
        let _ = FireAttack::net_modifier;
        let _ = FireModifier::die_modifier;
        let _ = MeleeModifier::die_modifier;
        let _ = GameTurnIndex::value;
        let _ = HowitzerResolution::hit_target_hex;
        let _ = MeleeFactor::sum(std::iter::empty::<&MeleeFactor>());
        let _ = MovementAllowance::halve;
        let _ = UnitKind::may_be_melee_attacked;
        let _ = UnitKind::may_melee_attack;
        let _ = UnitKind::may_retreat_before_melee;
        let _ = UnitState::may_act;
        let _ = UnitState::may_attack_this_turn;
        let _ = VpSource::points;
        let _ = VpSource::who_scores;
        let _ = VictoryLedger::total_for;
        let _ = VictoryLedger::superiority;
        let _ = omdurman_rules::UnitIdentity::is_friendlies;
        let _ = omdurman_rules::UnitIdentity::is_gordon;
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
    use omdurman_rules::effects::{GameState, PendingMelee};
    // GameEffect variants (§4, §5, §6, §7, §8, §10).
    use omdurman_rules::effects::GameEffect::{
        AdvanceAfterCombat, AdvancePhase, ConstructZariba, Demolition, DervishDesertion,
        FriendliesTransport, HowitzerFire, MeleeCombat, PlaceReinforcements, RetreatBeforeMelee,
        RiverMine,
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
        let _ = omdurman_rules::effects::apply_river_mine;
        let _ = omdurman_rules::effects::apply_sink_chain;
        let _ = omdurman_rules::effects::score_elimination;
        let _ = omdurman_rules::effects::first_player;
        // Fall of Khartoum special rules (§9.343, §9.345, §9.346).
        let _ = omdurman_rules::effects::range_band_for;
        let _ = omdurman_rules::effects::check_gordon_palace;

        // GameState query/command methods.
        let _ = GameState::new;
        let _ = GameState::can_move_unit;
        let _ = GameState::can_move_unit_to;
        let _ = GameState::can_move_gunboat;
        let _ = GameState::in_deployment_zone;
        let _ = GameState::can_fire_at;
        let _ = GameState::can_melee;
        let _ = GameState::can_advance_after_combat;
        let _ = GameState::can_retreat_before_melee;
        let _ = GameState::hex_in_enemy_zoc;
        let _ = GameState::unit_projects_zoc;
        let _ = GameState::hex_has_enemy_fort;
        let _ = GameState::is_nile_mouth_crossing;
        let _ = GameState::mp_spent;
    }
}

// ===========================================================================
// omdurman-rules :: other submodules
// ===========================================================================
mod rules_submodule_paths {
    use omdurman_rules::combat_results_table::{FireFactorRow, combat_results_table};
    use omdurman_rules::howitzer_scatter::{ScatterDirection, howitzer_scatter};
    use omdurman_rules::los_table::{
        LosFirerTerrain, LosResult, LosSpecialNote, LosTargetTerrain, has_los, los_table,
    };
    use omdurman_rules::range_effects::{ae_range_effects, dervish_range_effects};
    use omdurman_rules::terrain_chart::{
        defense_modifier, movement_cost, movement_cost_with_road, terrain_effects_chart,
    };
    use omdurman_rules::turn_track::{
        CAMPAIGN_TURN_TRACK, FALL_OF_KHARTOUM_TURN_TRACK, GameTime, HISTORICAL_TURN_TRACK,
        TurnEntry, TurnEvent, TurnLabel, turn_marker_pixel,
    };
    // TurnEvent variant (§8.2).
    use omdurman_rules::turn_track::TurnEvent::DervishDesertion;

    #[test]
    fn methods_resolve() {
        let _ = FireFactorRow::from_total;
        let _ = omdurman_rules::FireFactor::sum_to_row(std::iter::empty::<
            &omdurman_rules::FireFactor,
        >());
    }
}
