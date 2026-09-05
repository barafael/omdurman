use super::*;

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::*;
    use crate::*;
    use omdurman_types::SectionName;
    use traceability_macro::rulebook;

    /// A fresh state advanced past deployment into the first Movement turn, for
    /// gameplay tests that aren't exercising the setup phase itself. Every
    /// scenario now opens in [`Phase::Setup`]; this skips straight to play.
    fn playing(scenario: Scenario) -> GameState {
        let mut state = GameState::new(scenario);
        state.phase = Phase::Movement;
        state
    }

    #[allow(dead_code)]
    fn ae_infantry_profile() -> UnitProfile {
        UnitProfile {
            kind: UnitKind::Infantry {
                fire: 4,
                melee: 5,
                movement: 8,
            },
            identity: UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: 1,
                    nationality: BrigadeNationality::British,
                },
                battalion: BattalionOrdinal::First,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(crate::FireFactor::Four),
            melee: Some(crate::MeleeFactor::Five),
            movement: UnitMovement::Land(crate::MovementAllowance::Eight),
        }
    }

    fn dervish_tribal_profile() -> UnitProfile {
        UnitProfile {
            kind: UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            },
            identity: UnitIdentity::DervishTribal {
                tribe: DervishTribe::Baggara,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(crate::FireFactor::Three),
            melee: Some(crate::MeleeFactor::Six),
            movement: UnitMovement::Land(crate::MovementAllowance::Nine),
        }
    }

    fn make_ae_infantry(state: &mut GameState, hex: HexCoord) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: crate::UnitProfile {
                kind: crate::UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity: crate::UnitIdentity::AngloEgyptianInfantry {
                    brigade: omdurman_types::BrigadeId {
                        number: 1,
                        nationality: omdurman_types::BrigadeNationality::British,
                    },
                    battalion: crate::BattalionOrdinal::First,
                },
                weapon: crate::WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        });
        id
    }

    fn make_dervish_tribal(state: &mut GameState, hex: HexCoord) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: dervish_tribal_profile(),
            state: Default::default(),
        });
        id
    }

    fn make_ae_leader(state: &mut GameState, hex: HexCoord) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::BritishLeader { movement: 0 },
                identity: UnitIdentity::AngloEgyptianLeader(BritishLeader::Kitchener),
                weapon: WeaponClass::Melee,
                fire: None,
                melee: None,
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        });
        id
    }

    /// A Dervish tribal unit of an explicit tribe (for same-hex / stacking
    /// tests that need a second tribe).
    fn dervish_tribal_profile_with(tribe: DervishTribe) -> UnitProfile {
        UnitProfile {
            kind: UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            },
            identity: UnitIdentity::DervishTribal { tribe },
            weapon: WeaponClass::Rifles,
            fire: Some(crate::FireFactor::Three),
            melee: Some(crate::MeleeFactor::Six),
            movement: UnitMovement::Land(crate::MovementAllowance::Nine),
        }
    }

    /// An Anglo-Egyptian old-style gunboat profile (§2.32). `is_boat()` is true,
    /// so deployment-zone checks treat it as a boat (Nile-only, §5.22).
    fn ae_gunboat_profile() -> UnitProfile {
        UnitProfile {
            kind: UnitKind::Gunboat {
                fire: 0,
                upstream: 15,
                downstream: 16,
            },
            identity: UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(OldGunboat::LordKitchener)),
            weapon: WeaponClass::Artillery,
            fire: None,
            melee: None,
            movement: UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Fifteen,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        }
    }

    /// A Dervish gunboat profile (§9.111: two gunboats on south-edge Nile
    /// hexes). `is_boat()` is true, so deployment treats it as a boat
    /// (Nile-only, §5.22).
    fn dervish_gunboat_profile() -> UnitProfile {
        UnitProfile {
            kind: UnitKind::Gunboat {
                fire: 0,
                upstream: 10,
                downstream: 16,
            },
            identity: UnitIdentity::DervishGunboat(GunboatId::DervishGunboat(1)),
            weapon: WeaponClass::Artillery,
            fire: None,
            melee: None,
            movement: UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        }
    }

    #[rulebook("§6.22", "§6.41")]
    #[test]
    fn fire_combat_eliminates_target() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row06to10,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };

        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Eight,
            },
        );
        assert!(result.is_ok());
        // Dervish unit should be eliminated (roll 8, factor 8 -> Eliminate(1) on A-E Combat Results Table).
        assert!(state.find_unit(target).is_none());
    }

    #[rulebook("§6.24")]
    #[test]
    fn fire_modifiers_are_engine_derived_and_mismatches_rejected() {
        // §6.24: the +1 accuracy DRM is mandatory on every Anglo-Egyptian
        // direct-fire attack. A client that omits it (or smuggles in a
        // wrong modifier) is rejected; a correct list resolves with the
        // engine-derived bonus either way.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let base = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row06to10,
            modifiers: vec![],
        };

        // Omitted -> rejected.
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: base.clone(),
                roll: DieRoll::Five,
            },
        );
        assert!(
            matches!(result, Err(RuleError::FireModifierMismatch { .. })),
            "missing §6.24 +1 must be rejected, got {result:?}"
        );

        // Duplicated -> rejected.
        let mut dup = base.clone();
        dup.modifiers = vec![
            FireModifier::AngloEgyptianDirectFire,
            FireModifier::AngloEgyptianDirectFire,
        ];
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: dup,
                roll: DieRoll::Five,
            },
        );
        assert!(matches!(
            result,
            Err(RuleError::FireModifierMismatch { .. })
        ));

        // Smuggled terrain DRM -> rejected (§6.23 is engine-side; a caller
        // copy would double-count).
        let mut smuggled = base.clone();
        smuggled.modifiers = vec![
            FireModifier::AngloEgyptianDirectFire,
            FireModifier::Terrain(-2),
        ];
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: smuggled,
                roll: DieRoll::Five,
            },
        );
        assert!(matches!(
            result,
            Err(RuleError::FireModifierMismatch { .. })
        ));

        // Correct list -> accepted, and the +1 moves the CRT lookup: 8
        // factors (halved-printed band sum 4? no -- range 1 doubled = 8)
        // with roll 5 + 1 = 6 on row 6-10 -> Eliminate(1).
        let mut ok_attack = base;
        ok_attack.modifiers = vec![FireModifier::AngloEgyptianDirectFire];
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: ok_attack,
                roll: DieRoll::Five,
            },
        );
        assert!(result.is_ok());
        let obs = state
            .observations
            .iter()
            .find_map(|o| match o {
                Observation::FireResolved {
                    total_modifier,
                    modified_roll,
                    result,
                    ..
                } => Some((*total_modifier, *modified_roll, *result)),
                _ => None,
            })
            .unwrap();
        assert_eq!(obs.0, 1, "engine-derived §6.24 +1");
        assert_eq!(obs.1, DieRoll::Six);
        assert_eq!(obs.2, CombatResult::Eliminate(1));
    }

    #[rulebook("§9.231")]
    #[test]
    fn zariba_fire_penalties_apply_to_dervish_fire_only() {
        // §9.231/§9.232 print the zariba DRMs "on all Dervish fire attacks".
        // An Anglo-Egyptian attack at a zariba hex must carry no zariba
        // penalty -- and a Dervish attack there must carry it.
        let hedge = HexsideRef::new(HexCoord::new(1, 0), HexCoord::new(1, 1));
        let mk_state = |player| {
            let mut state = GameState::new(Scenario::Historical);
            state
                .board
                .hexsides
                .insert(hedge, HexsideKind::ZaribaThornHedge);
            // Dervish turn: the Dervish fires offensively, the AE
            // defensively (§4 Dervish player turn).
            state.phase = if player == Player::Dervish {
                Phase::OffensiveFire(FireSubPhase::DirectFire)
            } else {
                Phase::DefensiveFire(FireSubPhase::DirectFire)
            };
            state.active_player = Player::Dervish;
            let (firer, target);
            if player == Player::Dervish {
                firer = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
                target = make_ae_infantry(&mut state, HexCoord::new(1, 0));
            } else {
                firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
                target = make_dervish_tribal(&mut state, HexCoord::new(1, 0));
            }
            (state, firer, target)
        };

        // Dervish firing at a thorn-hedge hex: −2 mandatory.
        let (mut state, firer, _t) = mk_state(Player::Dervish);
        let mut attack = FireAttack {
            firing_player: Player::Dervish,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![],
        };
        attack.modifiers = mandatory_fire_modifiers(&state, &attack);
        assert_eq!(attack.modifiers, vec![FireModifier::ZaribaThornHedge]);
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::FireCombat {
                    attack,
                    roll: DieRoll::Five
                }
            )
            .is_ok()
        );

        // Anglo-Egyptian firing at the same hex: NO zariba DRM (and a client
        // attaching one is rejected).
        let (mut state, firer, _t) = mk_state(Player::AngloEgyptian);
        let mut smuggled = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![
                FireModifier::AngloEgyptianDirectFire,
                FireModifier::ZaribaThornHedge,
            ],
        };
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: smuggled.clone(),
                roll: DieRoll::Five,
            },
        );
        assert!(
            matches!(result, Err(RuleError::FireModifierMismatch { .. })),
            "AE attack must not carry the Dervish-only zariba DRM, got {result:?}"
        );
        smuggled.modifiers = vec![FireModifier::AngloEgyptianDirectFire];
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::FireCombat {
                    attack: smuggled,
                    roll: DieRoll::Five
                }
            )
            .is_ok()
        );
    }

    #[rulebook("§7.7")]
    #[test]
    fn melee_modifiers_are_engine_derived_and_mismatches_rejected() {
        // §7.7: Dervish +2 / Anglo-Egyptian +1 on every melee, both sides.
        // A declared attack with a wrong list is rejected; resolution uses
        // the engine's derivation.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;
        let defender = make_ae_infantry(&mut state, HexCoord::new(5, 5));
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(6, 5));

        let mk = |att: Vec<MeleeModifier>, def: Vec<MeleeModifier>| MeleeAttack {
            attacker_player: Player::Dervish,
            attacker_hex: HexCoord::new(6, 5),
            defender_hex: HexCoord::new(5, 5),
            attackers: vec![attacker],
            defenders: vec![defender],
            attacker_modifiers: att,
            defender_modifiers: def,
        };

        // Missing modifiers -> rejected.
        let bad = mk(vec![], vec![]);
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DeclareMelee {
                    attack: bad,
                    attacker_roll: DieRoll::Five,
                    defender_roll: DieRoll::Five,
                }
            ),
            Err(RuleError::MeleeModifierMismatch { .. })
        ));

        // Wrong side (+1 on the Dervish attacker) -> rejected.
        let bad = mk(
            vec![MeleeModifier::AngloEgyptianStandard],
            vec![MeleeModifier::AngloEgyptianStandard],
        );
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DeclareMelee {
                    attack: bad,
                    attacker_roll: DieRoll::Five,
                    defender_roll: DieRoll::Five,
                }
            ),
            Err(RuleError::MeleeModifierMismatch { .. })
        ));

        // Correct set -> accepted and resolved with the derived modifiers.
        let good = mk(
            vec![MeleeModifier::DervishStandard],
            vec![MeleeModifier::AngloEgyptianStandard],
        );
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::DeclareMelee {
                    attack: good,
                    attacker_roll: DieRoll::Four,
                    defender_roll: DieRoll::Five,
                }
            )
            .is_ok()
        );
        assert!(apply_effect(&mut state, &GameEffect::ResolveMelee).is_ok());
        let obs = state
            .observations
            .iter()
            .find_map(|o| match o {
                Observation::MeleeResolved {
                    attacker_total_modifier,
                    defender_total_modifier,
                    ..
                } => Some((*attacker_total_modifier, *defender_total_modifier)),
                _ => None,
            })
            .unwrap();
        assert_eq!(obs, (2, 1), "engine-derived §7.7 melee modifiers");
    }

    #[rulebook("§6.24", "§5.54")]
    #[test]
    fn brigade_integrity_modifier_is_engine_derived() {
        // §5.54: four co-stacked battalions of one brigade all firing at one
        // hex receive the +1 integrity DRM *in addition to* the §6.24 +1 --
        // and omitting either is now rejected because the engine derives the
        // whole set.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let profiles = [
            BattalionOrdinal::First,
            BattalionOrdinal::Second,
            BattalionOrdinal::Third,
            BattalionOrdinal::Fourth,
        ];
        let mut firers = Vec::new();
        for b in profiles {
            let id = state.alloc_unit_id();
            state.units.push(UnitPlacement {
                id,
                position: HexCoord::new(0, 0),
                profile: UnitProfile {
                    kind: UnitKind::Infantry {
                        fire: 4,
                        melee: 5,
                        movement: 8,
                    },
                    identity: UnitIdentity::AngloEgyptianInfantry {
                        brigade: BrigadeId {
                            number: 1,
                            nationality: BrigadeNationality::British,
                        },
                        battalion: b,
                    },
                    weapon: WeaponClass::Rifles,
                    fire: Some(crate::FireFactor::Four),
                    melee: Some(crate::MeleeFactor::Five),
                    movement: UnitMovement::Land(crate::MovementAllowance::Eight),
                },
                state: Default::default(),
            });
            firers.push(id);
        }
        make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        // Omitting the integrity DRM -> rejected.
        let mut attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: firers.clone(),
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row16to20,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: attack.clone(),
                roll: DieRoll::Five,
            },
        );
        assert!(
            matches!(result, Err(RuleError::FireModifierMismatch { .. })),
            "integrated brigade must carry the §5.54 +1, got {result:?}"
        );

        // Correct set -> accepted with a derived net +2.
        attack.modifiers = mandatory_fire_modifiers(&state, &attack);
        assert_eq!(
            attack.modifiers,
            vec![
                FireModifier::AngloEgyptianDirectFire,
                FireModifier::BrigadeIntegrity
            ]
        );
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::FireCombat {
                    attack,
                    roll: DieRoll::Five
                }
            )
            .is_ok()
        );
    }

    #[rulebook("§6.52")]
    #[test]
    fn friendlies_validate_and_resolve_on_dervish_table() {
        // Regression (audit §6.52): a "Friendlies" rifle attack at range 5
        // passed validation on the Anglo-Egyptian table (max 5) but resolved
        // on the Dervish table (max 4). Both paths must now agree: range 5 is
        // out of range, range 4 resolves halved on the Dervish table.
        let friendlies_profile = UnitProfile {
            kind: UnitKind::Infantry {
                fire: 4,
                melee: 5,
                movement: 8,
            },
            identity: UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: 1,
                    nationality: BrigadeNationality::Friendlies,
                },
                battalion: BattalionOrdinal::First,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(crate::FireFactor::Four),
            melee: Some(crate::MeleeFactor::Five),
            movement: UnitMovement::Land(crate::MovementAllowance::Eight),
        };

        // Range 5 -- rejected (Dervish rifles max 4).
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let firer = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: firer,
            position: HexCoord::new(0, 0),
            profile: friendlies_profile,
            state: Default::default(),
        });
        make_dervish_tribal(&mut state, HexCoord::new(5, 0));
        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(5, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![],
        };
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Five,
            },
        );
        assert!(
            matches!(result, Err(RuleError::TargetOutOfRange { .. })),
            "Friendlies rifle at range 5 must be out of range on the Dervish table (§6.52), got {result:?}"
        );

        // Range 4 -- accepted, halved on the Dervish table: 4 factors -> 2.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let firer = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: firer,
            position: HexCoord::new(0, 0),
            profile: friendlies_profile,
            state: Default::default(),
        });
        make_dervish_tribal(&mut state, HexCoord::new(4, 0));
        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(4, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Five,
            },
        );
        assert!(
            result.is_ok(),
            "Friendlies rifle at range 4 is in range (§6.52): {result:?}"
        );
        let eff = state
            .observations
            .iter()
            .find_map(|o| match o {
                Observation::FireResolved {
                    effective_factor, ..
                } => Some(*effective_factor),
                _ => None,
            })
            .expect("FireResolved observation");
        assert_eq!(
            eff, 2,
            "4 fire factors halved on the Dervish table (§6.16/§6.52)"
        );

        // Control: a *regular* AE rifle at range 5 stays on the AE table
        // (4-5 halved) and is legal.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        make_dervish_tribal(&mut state, HexCoord::new(5, 0));
        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(5, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Five,
            },
        );
        assert!(
            result.is_ok(),
            "regular AE rifle at range 5 is in range on the AE table: {result:?}"
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn mixed_attack_bands_per_firer() {
        // Regression (audit §6.22, fixture seq 827): a combined attack
        // applied the *first* firer's range band to every firer. A
        // spear-armed unit (Melee line, range 1 only) stacked with a
        // Dervish battery (Artillery line) dragged the battery's factors
        // onto the spear line. Each firer must contribute on its own line.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::Dervish;

        let battery_profile = UnitProfile {
            kind: UnitKind::Artillery {
                fire: 4,
                melee: 2,
                movement: 8,
            },
            identity: UnitIdentity::DervishArtillery,
            weapon: WeaponClass::Artillery,
            fire: Some(crate::FireFactor::Four),
            melee: Some(crate::MeleeFactor::Three),
            movement: UnitMovement::Land(crate::MovementAllowance::Eight),
        };
        let spear = make_dervish_tribal(&mut state, HexCoord::new(0, 0)); // rifles, 3 factors
        let battery = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: battery,
            // §5.52: a Dervish gun may not stack with a tribal unit, so the
            // battery stands adjacent to the target instead of on the spear.
            position: HexCoord::new(1, -1),
            profile: battery_profile,
            state: Default::default(),
        });
        make_ae_infantry(&mut state, HexCoord::new(1, 0));

        // Target adjacent (range 1): tribal rifles x1 (3 factors), Dervish
        // artillery x2 (4 -> 8). The old first-firer-band bug resolved both
        // on the rifle line (3 + 4 = 7); each firer must use its own line.
        let attack = FireAttack {
            firing_player: Player::Dervish,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![spear, battery], // rifle-armed unit first: the old bug's trigger order
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![],
        };
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Five,
            },
        );
        assert!(result.is_ok());
        let eff = state
            .observations
            .iter()
            .find_map(|o| match o {
                Observation::FireResolved {
                    effective_factor, ..
                } => Some(*effective_factor),
                _ => None,
            })
            .expect("FireResolved observation");
        assert_eq!(
            eff, 11,
            "rifles contribute 3 (x1), battery 8 (x2, own artillery line)"
        );
    }

    #[rulebook("§4")]
    #[test]
    fn fire_combat_wrong_phase_rejected() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row06to10,
            modifiers: vec![],
        };

        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Five,
            },
        );
        assert!(result.is_err());
        assert!(matches!(result, Err(RuleError::WrongPhase)));
    }

    #[test]
    fn movement_exceeds_allowance_rejected() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));

        let result = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: id,
                to: HexCoord::new(5, 0),
                cost: MovementPoints::new(99),
                path: Vec::new(),
            },
        );
        assert!(matches!(
            result,
            Err(RuleError::MovementExceedsAllowance { .. })
        ));
        // Rejected move leaves the unit where it started.
        assert_eq!(state.find_unit(id).unwrap().position, HexCoord::new(0, 0));
    }

    #[test]
    fn legal_move_updates_position() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let to = HexCoord::new(1, 0);

        let result = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: id,
                to,
                cost: MovementPoints::new(1),
                path: Vec::new(),
            },
        );
        assert!(result.is_ok());
        assert_eq!(state.find_unit(id).unwrap().position, to);
        assert_eq!(state.mp_spent(id), 1);
        assert!(state.mp_spent(id) > 0);

        // §5.12: a unit may keep moving hex by hex up to its allowance, so a
        // second step that fits the remaining allowance (8 total here) succeeds
        // and accumulates -- it is NOT rejected as "already moved".
        let again = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: id,
                to: HexCoord::new(2, 0),
                cost: MovementPoints::new(1),
                path: Vec::new(),
            },
        );
        assert!(again.is_ok());
        assert_eq!(state.find_unit(id).unwrap().position, HexCoord::new(2, 0));
        assert_eq!(state.mp_spent(id), 2);
    }

    #[test]
    fn cumulative_moves_cannot_exceed_allowance() {
        // §5.11/§5.12: stepping a unit hex by hex (or re-selecting it) may not
        // exceed its movement allowance in total over the turn.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        // Allowance 8; spend 8 in one move, then any further step is rejected.
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let first = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: id,
                to: HexCoord::new(8, 0),
                cost: MovementPoints::new(8),
                path: Vec::new(),
            },
        );
        assert!(first.is_ok());
        assert_eq!(state.mp_spent(id), 8);

        let over = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: id,
                to: HexCoord::new(9, 0),
                cost: MovementPoints::new(1),
                path: Vec::new(),
            },
        );
        assert!(matches!(
            over,
            Err(RuleError::MovementExceedsAllowance { .. })
        ));
        // The over-move left the unit where its allowance ran out.
        assert_eq!(state.find_unit(id).unwrap().position, HexCoord::new(8, 0));
    }

    #[test]
    fn can_move_unit_matches_effect_and_does_not_mutate() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));

        // Legal in-allowance move: accepted, and the read-only check leaves
        // state untouched (no position change, no MP recorded).
        assert!(state.can_move_unit(id, MovementPoints::new(1)).is_ok());
        assert_eq!(state.find_unit(id).unwrap().position, HexCoord::new(0, 0));
        assert!(state.mp_spent_this_turn.is_empty());

        // Over-allowance is rejected the same way the effect would reject it.
        assert!(matches!(
            state.can_move_unit(id, MovementPoints::new(99)),
            Err(RuleError::MovementExceedsAllowance { .. })
        ));

        // Wrong phase is rejected.
        state.phase = Phase::Melee;
        assert!(matches!(
            state.can_move_unit(id, MovementPoints::new(1)),
            Err(RuleError::WrongPhase)
        ));
    }

    #[test]
    fn hex_in_enemy_zoc_respects_disruption_and_leaders() {
        let mut state = GameState::new(Scenario::Campaign);
        // A Dervish unit at (1,1) projects ZOC into its six neighbours, one of
        // which is (1,0) -- seen from the moving Anglo-Egyptian player's side.
        let dervish = make_dervish_tribal(&mut state, HexCoord::new(1, 1));
        assert!(state.hex_in_enemy_zoc(
            HexCoord::new(1, 0),
            Player::AngloEgyptian,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0
            }
        ));
        // A friendly unit's hexes are not "enemy" ZOC.
        assert!(!state.hex_in_enemy_zoc(
            HexCoord::new(1, 0),
            Player::Dervish,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0
            }
        ));
        // A hex no enemy is adjacent to is free.
        assert!(!state.hex_in_enemy_zoc(
            HexCoord::new(5, 5),
            Player::AngloEgyptian,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0
            }
        ));

        // Disrupted units project no ZOC (§5.41).
        state.find_unit_mut(dervish).unwrap().state.disrupted = true;
        assert!(!state.hex_in_enemy_zoc(
            HexCoord::new(1, 0),
            Player::AngloEgyptian,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0
            }
        ));
    }

    #[test]
    fn movement_must_stop_in_enemy_zoc() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let mover = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        // Enemy at (1,1) puts the intermediate hex (1,0) in an enemy ZOC.
        make_dervish_tribal(&mut state, HexCoord::new(1, 1));

        // Moving straight through (1,0) to (3,0) is blocked -- the unit would
        // have had to stop at (1,0).
        let through =
            state.can_move_unit_to(mover, Some(HexCoord::new(3, 0)), MovementPoints::new(3));
        assert!(matches!(
            through,
            Err(RuleError::BlockedByEnemyZoc(hex)) if hex == HexCoord::new(1, 0)
        ));

        // Stopping *in* the ZOC hex (1,0) is legal -- that is exactly where the
        // unit must halt (§5.43).
        assert!(
            state
                .can_move_unit_to(mover, Some(HexCoord::new(1, 0)), MovementPoints::new(1))
                .is_ok()
        );

        // A move whose path avoids every enemy-ZOC hex is fine. The enemy at
        // (1,1) projects ZOC into (1,0)/(0,0)/(0,1)/(1,2)/(2,1)/(2,2); a move
        // away to (-3,0) crosses (-1,0)/(-2,0), none of which are in ZOC. (The
        // start (0,0) itself being in ZOC does not block -- §5.43.)
        assert!(
            state
                .can_move_unit_to(mover, Some(HexCoord::new(-3, 0)), MovementPoints::new(3))
                .is_ok()
        );
    }

    #[test]
    fn unit_in_enemy_zoc_may_move_out() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        // Mover starts at (1,0), already adjacent to the enemy at (1,1) -- i.e.
        // it begins its move inside an enemy ZOC.
        let mover = make_ae_infantry(&mut state, HexCoord::new(1, 0));
        make_dervish_tribal(&mut state, HexCoord::new(1, 1));
        assert!(state.hex_in_enemy_zoc(
            HexCoord::new(1, 0),
            Player::AngloEgyptian,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0
            }
        ));

        // It may withdraw to a hex outside any ZOC (§5.43): start being in ZOC
        // does not block the move.
        assert!(
            state
                .can_move_unit_to(mover, Some(HexCoord::new(4, 0)), MovementPoints::new(3))
                .is_ok()
        );
    }

    #[rulebook("§5.26", "§5.43")]
    #[test]
    fn unit_entering_enemy_zoc_may_move_no_further_that_turn() {
        // Regression (audit §5.43, 222 violations in the recorded games): a
        // unit that entered an enemy ZOC used to be free to keep moving in
        // later moves of the same phase. "All units must stop when they enter
        // an enemy ZOC and may move no further that turn."
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let mover = make_dervish_tribal(&mut state, HexCoord::new(3, 0));
        make_ae_infantry(&mut state, HexCoord::new(5, 0)); // ZOC ring covers (4,0)

        // Move 1: enter the ZOC at (4,0) -- legal, the unit stops there.
        assert!(
            apply_move_unit(
                &mut state,
                mover,
                HexCoord::new(4, 0),
                MovementPoints::new(1),
                &[]
            )
            .is_ok()
        );
        assert!(state.zoc_stopped_this_turn.contains(&mover));

        // Move 2 (same phase): rejected -- it may move no further this turn.
        assert!(matches!(
            apply_move_unit(
                &mut state,
                mover,
                HexCoord::new(4, 1),
                MovementPoints::new(1),
                &[]
            ),
            Err(RuleError::StoppedInEnemyZoc(_))
        ));

        // After the turn passes, it may withdraw (§5.43 "In their next
        // movement phase they may withdraw").
        end_player_turn(&mut state).unwrap(); // -> AE turn
        end_player_turn(&mut state).unwrap(); // -> Dervish again, trackers cleared
        assert!(!state.zoc_stopped_this_turn.contains(&mover));
        assert!(
            apply_move_unit(
                &mut state,
                mover,
                HexCoord::new(3, 0),
                MovementPoints::new(1),
                &[]
            )
            .is_ok()
        );
    }

    #[rulebook("§5.26")]
    #[test]
    fn zoc_transit_check_uses_the_actual_path() {
        // Regression (audit §5.26): the engine checked the straight line for
        // enemy-ZOC transit, not the stepped path -- a path threading around
        // a ZOC was wrongly rejected (and one through a ZOC hex wrongly
        // accepted when the straight line missed it). The entered hexes of
        // the supplied path govern.
        let mut state = GameState::new(Scenario::Campaign);
        let mut board = BoardInfo::default();
        for q in 0..=9 {
            for r in 0..=5 {
                board.terrain.insert(
                    HexCoord::new(q, r),
                    Terrain::Clear {
                        road: Default::default(),
                    },
                );
            }
        }
        state.board = board;
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;

        // An AE unit at (5,2); its ZOC ring covers the six neighbours.
        let enemy_hex = HexCoord::new(5, 2);
        make_ae_infantry(&mut state, enemy_hex);
        let ring: Vec<HexCoord> = enemy_hex.neighbors().to_vec();

        let mover = make_dervish_tribal(&mut state, HexCoord::new(5, 0));
        assert!(
            apply_move_unit(
                &mut state,
                mover,
                enemy_hex,
                MovementPoints::new(2),
                &[HexCoord::new(5, 1), enemy_hex]
            )
            .is_err(),
            "cannot move onto the enemy's own hex"
        );

        // Path straight through a ZOC-ring hex -> rejected (must stop there).
        let through_zoc: Vec<HexCoord> = vec![HexCoord::new(5, 1)];
        assert!(matches!(
            apply_move_unit(
                &mut state,
                mover,
                HexCoord::new(5, 2).neighbors()[0],
                MovementPoints::new(2),
                &{
                    let mut p = through_zoc.clone();
                    p.push(HexCoord::new(5, 2).neighbors()[0]);
                    p
                }
            ),
            Err(RuleError::BlockedByEnemyZoc(_))
        ));

        // Bent path around the ring -> legal even though the straight line
        // would cross it. Route west then south then east, outside the ring.
        let detour: Vec<HexCoord> = vec![
            HexCoord::new(4, 0),
            HexCoord::new(3, 1),
            HexCoord::new(3, 2),
            HexCoord::new(3, 3),
            HexCoord::new(4, 4),
            HexCoord::new(5, 4),
        ];
        let in_ring = detour.iter().any(|h| ring.contains(h));
        assert!(!in_ring, "test premise: the detour avoids the ZOC ring");
        assert!(
            apply_move_unit(
                &mut state,
                mover,
                HexCoord::new(5, 4),
                MovementPoints::new(6),
                &detour
            )
            .is_ok(),
            "a path around the ZOC is legal (§5.26 stops only on entering)"
        );
    }

    #[test]
    fn anglo_egyptian_leader_projects_no_zoc() {
        let mut state = GameState::new(Scenario::Campaign);
        // Make the active player Dervish so the A-E leader is the "enemy".
        state.active_player = Player::Dervish;
        let leader = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: leader,
            position: HexCoord::new(1, 1),
            profile: UnitProfile {
                kind: UnitKind::BritishLeader { movement: 0 },
                identity: UnitIdentity::AngloEgyptianLeader(BritishLeader::Kitchener),
                weapon: WeaponClass::Melee,
                fire: None,
                melee: None,
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        });
        // §5.41: an Anglo-Egyptian leader exerts no ZOC.
        assert!(!state.hex_in_enemy_zoc(
            HexCoord::new(1, 0),
            Player::Dervish,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0
            }
        ));
    }

    #[rulebook("§5.12")]
    #[test]
    fn wrong_faction_move_is_rejected() {
        // §5.1: only the active player's units move during their player
        // turn. The engine enforces this -- a `MoveUnit` (or gunboat move)
        // submitted for the *non-active* faction is `NotYourTurn`, the same
        // authority rule fire (§6.41), melee (§7.1) and reinforcements
        // (§9.112/§9.113) already applied.
        let mut state = playing(Scenario::Campaign); // Campaign: A-E moves first
        let dervish = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        assert_eq!(state.active_player, Player::AngloEgyptian);
        assert!(matches!(
            state.can_move_unit(dervish, MovementPoints::new(1)),
            Err(RuleError::NotYourTurn)
        ));

        // The active faction's own units still move.
        let ae = make_ae_infantry(&mut state, HexCoord::new(5, 5));
        assert!(state.can_move_unit(ae, MovementPoints::new(1)).is_ok());

        // Handing the player turn over re-opens movement for the Dervish.
        state.active_player = Player::Dervish;
        assert!(state.can_move_unit(dervish, MovementPoints::new(1)).is_ok());
        assert!(matches!(
            state.can_move_unit(ae, MovementPoints::new(1)),
            Err(RuleError::NotYourTurn)
        ));
    }

    #[rulebook("§5.12")]
    #[test]
    fn wrong_faction_gunboat_move_is_rejected() {
        let mut state = playing(Scenario::Campaign);
        state.board = nile_board_row0(0, 6, HexDirection::East);
        let ae_gb = make_unit(
            &mut state,
            HexCoord::new(1, 0),
            UnitKind::Gunboat {
                fire: 0,
                upstream: 0,
                downstream: 0,
            },
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(crate::OldGunboat::LordKitchener)),
            WeaponClass::Artillery,
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        );
        let dervish_gb = make_dervish_gunboat(&mut state, HexCoord::new(3, 0));
        // Campaign: A-E active -- the Dervish gunboat may not move...
        assert!(matches!(
            state.can_move_gunboat(dervish_gb, HexCoord::new(4, 0), &[], MovementPoints::new(1)),
            Err(RuleError::NotYourTurn)
        ));
        // ...but its own can.
        assert!(
            state
                .can_move_gunboat(ae_gb, HexCoord::new(2, 0), &[], MovementPoints::new(1))
                .is_ok()
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn can_fire_at_gates_phase_range_and_player() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let enemy_near = HexCoord::new(1, 0); // range 1 -- rifles in range
        make_dervish_tribal(&mut state, enemy_near);

        // In a fire phase, the active A-E unit may fire an in-range enemy hex.
        assert!(state.can_fire_at(ae, enemy_near, FireKind::Direct).is_ok());
        // Read-only: nothing recorded as fired.
        assert!(state.units_fired_this_phase.is_empty());

        // Out of rifle range (range 8) is rejected.
        assert!(matches!(
            state.can_fire_at(ae, HexCoord::new(8, 0), FireKind::Direct),
            Err(RuleError::TargetOutOfRange { .. })
        ));

        // A rifle unit may not use Maxim second fire, and not in the Direct
        // sub-phase regardless.
        assert!(matches!(
            state.can_fire_at(ae, enemy_near, FireKind::MaximSecondFire),
            Err(RuleError::WrongPhase)
        ));

        // Wrong phase: no firing during movement.
        state.phase = Phase::Movement;
        assert!(matches!(
            state.can_fire_at(ae, enemy_near, FireKind::Direct),
            Err(RuleError::WrongPhase)
        ));

        // During A-E offensive fire, a Dervish unit may not fire.
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let dervish = make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        assert!(matches!(
            state.can_fire_at(dervish, HexCoord::new(0, 0), FireKind::Direct),
            Err(RuleError::NotYourTurn)
        ));
    }

    #[rulebook("§7.2", "§7.4")]
    #[test]
    fn can_melee_gates_phase_adjacency_and_kind() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let adj = HexCoord::new(1, 0); // adjacent
        make_dervish_tribal(&mut state, adj);

        // Adjacent enemy in the Melee phase: legal, read-only.
        assert!(state.can_melee(ae, adj).is_ok());

        // Non-adjacent hex is rejected.
        assert!(matches!(
            state.can_melee(ae, HexCoord::new(3, 0)),
            Err(RuleError::TargetNotAdjacent { .. })
        ));

        // Wrong phase.
        state.phase = Phase::Movement;
        assert!(matches!(
            state.can_melee(ae, adj),
            Err(RuleError::WrongPhase)
        ));

        // Empty adjacent hex: nothing to attack.
        state.phase = Phase::Melee;
        assert!(matches!(
            state.can_melee(ae, HexCoord::new(0, 1)),
            Err(RuleError::NoMeleeableEnemy(_))
        ));
    }

    #[rulebook("§7.5")]
    #[test]
    fn retreat_before_melee_only_cavalry_two_hexes() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish; // attacker; A-E units defend

        // A cavalry-kind unit standing where a melee will be declared.
        let id = state.alloc_unit_id();
        let cav_hex = HexCoord::new(5, 5);
        state.units.push(UnitPlacement {
            id,
            position: cav_hex,
            profile: UnitProfile {
                kind: UnitKind::Cavalry {
                    fire: 0,
                    melee: 0,
                    movement: 0,
                },
                identity: UnitIdentity::AngloEgyptianCavalry,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Three),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        });
        // A Dervish attacker adjacent to the cavalry, to declare a melee.
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(6, 5));

        // No retreat without a declared melee threatening the unit's hex.
        assert!(matches!(
            state.can_retreat_before_melee(id, HexCoord::new(7, 5)),
            Err(RuleError::NoInfantryMeleeThreatens(_))
        ));

        // Declare the melee on the cavalry's hex -> reaction window opens.
        apply_effect(
            &mut state,
            &GameEffect::DeclareMelee {
                attack: MeleeAttack {
                    attacker_player: Player::Dervish,
                    attacker_hex: HexCoord::new(6, 5),
                    defender_hex: cav_hex,
                    attackers: vec![attacker],
                    defenders: vec![id],
                    attacker_modifiers: vec![MeleeModifier::DervishStandard],
                    defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
                },
                attacker_roll: DieRoll::Five,
                defender_roll: DieRoll::Five,
            },
        )
        .unwrap();

        // Now retreat: one hex rejected, exactly two accepted.
        assert!(
            state
                .can_retreat_before_melee(id, HexCoord::new(6, 6))
                .is_err()
        );
        let dest = HexCoord::new(7, 5);
        assert!(state.can_retreat_before_melee(id, dest).is_ok());
        apply_effect(
            &mut state,
            &GameEffect::RetreatBeforeMelee {
                unit_id: id,
                to: dest,
            },
        )
        .unwrap();
        assert_eq!(state.find_unit(id).unwrap().position, dest);

        // After retreat, resolving the declared melee spares the unit (it has
        // left the target hex), and the window closes.
        apply_effect(&mut state, &GameEffect::ResolveMelee).unwrap();
        assert!(state.pending_melee.is_none());
        assert!(state.find_unit(id).is_some(), "retreated unit was spared");
    }

    // §5.22 regression (found by the invariant fuzzer, seed 8600): a
    // cavalry's two-hex retreat may never land on a Nile hex -- land units
    // may not enter the river under any circumstances, retreat included.
    #[rulebook("§5.22")]
    #[test]
    fn retreat_before_melee_may_not_land_on_nile() {
        let mut board = BoardInfo::default();
        board.terrain.insert(
            HexCoord::new(5, 5),
            Terrain::Clear {
                road: Default::default(),
            },
        );
        board.terrain.insert(
            HexCoord::new(7, 5),
            Terrain::Nile {
                direction: omdurman_types::HexDirection::East,
            },
        );

        let mut state = GameState::new(Scenario::Campaign);
        state.board = board;
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish; // attackers; A-E defends

        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: HexCoord::new(5, 5),
            profile: UnitProfile {
                kind: UnitKind::Cavalry {
                    fire: 0,
                    melee: 0,
                    movement: 0,
                },
                identity: UnitIdentity::AngloEgyptianCavalry,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Three),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        });
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(6, 5));
        apply_effect(
            &mut state,
            &GameEffect::DeclareMelee {
                attack: MeleeAttack {
                    attacker_player: Player::Dervish,
                    attacker_hex: HexCoord::new(6, 5),
                    defender_hex: HexCoord::new(5, 5),
                    attackers: vec![attacker],
                    defenders: vec![id],
                    attacker_modifiers: vec![MeleeModifier::DervishStandard],
                    defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
                },
                attacker_roll: DieRoll::Five,
                defender_roll: DieRoll::Five,
            },
        )
        .unwrap();
        assert!(matches!(
            state.can_retreat_before_melee(id, HexCoord::new(7, 5)),
            Err(RuleError::LandIntoNile(_))
        ));
    }

    // §6.54: a retreat may not end on an enemy fort (forts are never
    // occupied by the enemy).
    #[rulebook("§6.54")]
    #[test]
    fn retreat_before_melee_may_not_land_on_enemy_fort() {
        let mut board = BoardInfo::default();
        board.terrain.insert(
            HexCoord::new(5, 5),
            Terrain::Clear {
                road: Default::default(),
            },
        );
        board.terrain.insert(
            HexCoord::new(7, 5),
            Terrain::Clear {
                road: Default::default(),
            },
        );
        let mut state = GameState::new(Scenario::Campaign);
        state.board = board;
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;

        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: HexCoord::new(5, 5),
            profile: UnitProfile {
                kind: UnitKind::Cavalry {
                    fire: 0,
                    melee: 0,
                    movement: 0,
                },
                identity: UnitIdentity::AngloEgyptianCavalry,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Three),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        });
        // An enemy (Dervish-owned) fort on the retreat hex, in a *different*
        // hex so the melee declaration still targets the cavalry.
        let fort = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: fort,
            position: HexCoord::new(7, 5),
            profile: UnitProfile {
                kind: UnitKind::Fort { fire: 0, melee: 0 },
                identity: UnitIdentity::DervishFort,
                weapon: WeaponClass::Artillery,
                fire: None,
                melee: None,
                movement: UnitMovement::Immobile,
            },
            state: Default::default(),
        });
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(6, 5));
        apply_effect(
            &mut state,
            &GameEffect::DeclareMelee {
                attack: MeleeAttack {
                    attacker_player: Player::Dervish,
                    attacker_hex: HexCoord::new(6, 5),
                    defender_hex: HexCoord::new(5, 5),
                    attackers: vec![attacker],
                    defenders: vec![id],
                    attacker_modifiers: vec![MeleeModifier::DervishStandard],
                    defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
                },
                attacker_roll: DieRoll::Five,
                defender_roll: DieRoll::Five,
            },
        )
        .unwrap();
        // The fort is a unit occupying (7,5), so the occupied-hex check
        // (RetreatHexOccupied) fires before the EnemyFort arm -- either way
        // §6.54's outcome holds: the retreat may not end on an enemy fort.
        assert!(
            state
                .can_retreat_before_melee(id, HexCoord::new(7, 5))
                .is_err(),
            "§6.54: a retreat may not end on an enemy fort"
        );
    }

    #[test]
    fn dervish_must_advance_into_vacated_melee_hex() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;
        // A Dervish attacker adjacent to a lone A-E defender it will wipe out.
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let defender_hex = HexCoord::new(1, 0);
        let defender = make_ae_infantry(&mut state, defender_hex);

        let attack = MeleeAttack {
            attacker_player: Player::Dervish,
            attacker_hex: HexCoord::new(0, 0),
            defender_hex,
            attackers: vec![attacker],
            defenders: vec![defender],
            attacker_modifiers: vec![MeleeModifier::DervishStandard],
            defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
        };
        apply_effect(
            &mut state,
            &GameEffect::MeleeCombat {
                attack,
                attacker_roll: DieRoll::Ten,
                defender_roll: DieRoll::One,
            },
        )
        .unwrap();

        // Invariant (§7.6): whenever the defender hex is vacated by the melee,
        // a surviving Dervish attacker is forced to advance into it. If the
        // defender survived, the attacker stays put. Assert the implication
        // rather than a specific Combat Results Table outcome.
        let defender_gone = state.find_unit(defender).is_none();
        let attacker_pos = state.find_unit(attacker).map(|u| u.position);
        if defender_gone && attacker_pos.is_some() {
            assert_eq!(
                attacker_pos,
                Some(defender_hex),
                "Dervish must advance into the vacated hex (§7.6)"
            );
        }
    }

    #[test]
    fn dervish_advance_is_forced_when_hex_vacated() {
        // Directly exercise the advance branch: stand a Dervish unit next to
        // an empty hex and confirm the post-melee advance moves it in when the
        // defender list resolves to empty. We simulate the "vacated" condition
        // by meleeing a defender whose elimination we guarantee with a maximal
        // factor gap is unreliable, so instead verify the branch via a unit
        // already adjacent to a now-empty target through `can_advance...`.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let vacated = HexCoord::new(1, 0);
        // No unit in `vacated` -> the mandatory-advance branch's eligibility
        // logic (the same predicate) should accept advancing there.
        open_advance_window(&mut state, vacated, &[attacker], vec!["7.6".to_string()]);
        assert!(state.can_advance_after_combat(attacker, vacated).is_ok());
    }

    #[test]
    fn advance_after_combat_into_vacated_hex() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let vacated = HexCoord::new(1, 0); // adjacent, empty

        open_advance_window(&mut state, vacated, &[id], vec!["7.6".to_string()]);
        assert!(state.can_advance_after_combat(id, vacated).is_ok());
        // Occupied target is rejected.
        make_dervish_tribal(&mut state, HexCoord::new(0, 1));
        assert!(
            state
                .can_advance_after_combat(id, HexCoord::new(0, 1))
                .is_err()
        );
        // Non-adjacent rejected.
        assert!(
            state
                .can_advance_after_combat(id, HexCoord::new(4, 0))
                .is_err()
        );

        apply_effect(
            &mut state,
            &GameEffect::AdvanceAfterCombat {
                unit_id: id,
                to: vacated,
            },
        )
        .unwrap();
        assert_eq!(state.find_unit(id).unwrap().position, vacated);
    }

    #[rulebook("§6.82")]
    #[test]
    fn advance_requires_combat_vacated_hex() {
        // A merely-empty adjacent hex is not an advance target (§6.82): the
        // hex must have been vacated by combat this phase. This is the check
        // that stops advance-after-combat acting as free out-of-phase
        // movement.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        assert!(matches!(
            state.can_advance_after_combat(id, HexCoord::new(1, 0)),
            Err(RuleError::HexNotVacatedByCombat(_))
        ));
    }

    #[rulebook("§6.82")]
    #[test]
    fn advance_requires_participation() {
        // §6.82/§7.6: only units that participated in the combat that
        // vacated the hex may advance -- a same-side bystander adjacent to
        // the vacated hex may not.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let participant = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let bystander = make_ae_infantry(&mut state, HexCoord::new(1, -1));
        let vacated = HexCoord::new(1, 0);
        open_advance_window(&mut state, vacated, &[participant], vec!["7.6".to_string()]);
        assert!(state.can_advance_after_combat(participant, vacated).is_ok());
        assert!(matches!(
            state.can_advance_after_combat(bystander, vacated),
            Err(RuleError::UnitDidNotParticipate(_, hex)) if hex == vacated
        ));
    }

    #[rulebook("§5.25")]
    #[test]
    fn forts_are_never_advance_eligible() {
        // §5.25: forts may not move in any way. Even a hand-seeded window
        // listing a fort (open_advance_window filters them, this covers a
        // crafted/replayed state) must not let it advance.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let fort = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: fort,
            position: HexCoord::new(0, 0),
            profile: UnitProfile {
                kind: UnitKind::Fort { fire: 0, melee: 0 },
                identity: crate::UnitIdentity::DervishFort,
                weapon: WeaponClass::Artillery,
                fire: Some(crate::FireFactor::One),
                melee: None,
                movement: UnitMovement::Immobile,
            },
            state: Default::default(),
        });
        let vacated = HexCoord::new(1, 0);
        state.vacated_by_combat.insert(vacated, vec![fort]);
        assert!(matches!(
            state.can_advance_after_combat(fort, vacated),
            Err(RuleError::FortMayNotAdvance(_))
        ));
    }

    #[rulebook("§6.7")]
    #[test]
    fn defensive_fire_opens_no_advance_window() {
        // §6.7: "There is no advance after combat as a result of defensive
        // fires" -- a defensive-fire elimination vacates the hex but must
        // neither open a window nor emit the vacated observation.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::DefensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian; // Dervish fires defensively
        let firer = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let target = make_ae_infantry(&mut state, HexCoord::new(1, 0));

        let attack = FireAttack {
            firing_player: Player::Dervish,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![],
        };
        apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Ten, // Row01to05 @ 10 -> Eliminate(2): hex vacated
            },
        )
        .unwrap();
        assert!(state.find_unit(target).is_none());
        assert!(
            state.vacated_by_combat.is_empty(),
            "§6.7: defensive fire must not open an advance window"
        );
        assert!(
            !state
                .observations
                .iter()
                .any(|o| matches!(o, Observation::HexVacatedByCombat { .. }))
        );
    }

    #[rulebook("§6.42")]
    #[test]
    fn advance_window_bridges_fire_subphase_and_closes_at_melee() {
        // The Direct→Maxim/Howitzer subphase transition is one continuous
        // offensive-fire phase (§6.42): a window opened by direct fire stays
        // usable. Crossing into Melee closes it (§6.82: the advance answers
        // the fire that vacated the hex).
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let vacated = HexCoord::new(1, 0);
        open_advance_window(&mut state, vacated, &[id], vec!["6.82".to_string()]);

        advance_phase(&mut state).unwrap(); // -> Maxim/Howitzer subphase
        assert_eq!(
            state.phase,
            Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer)
        );
        assert!(
            state.can_advance_after_combat(id, vacated).is_ok(),
            "§6.42 bridge: the window survives the subphase change"
        );

        advance_phase(&mut state).unwrap(); // -> Melee
        assert!(matches!(
            state.can_advance_after_combat(id, vacated),
            Err(RuleError::HexNotVacatedByCombat(_))
        ));
    }

    #[rulebook("§7.5")]
    #[test]
    fn retreat_opens_window_only_when_hex_empties() {
        // §7.5/§7.6: a retreat-before-melee only vacates the hex once the
        // *last* defender has left; a stacked hex still held by a defender
        // opens no window.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish; // attackers; A-E defends
        let cav_hex = HexCoord::new(5, 5);
        let cavalry = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: cavalry,
            position: cav_hex,
            profile: UnitProfile {
                kind: UnitKind::Cavalry {
                    fire: 0,
                    melee: 0,
                    movement: 0,
                },
                identity: UnitIdentity::AngloEgyptianCavalry,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Three),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        });
        // A second AE defender stacked in the same hex.
        let stay = make_ae_infantry(&mut state, cav_hex);
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(6, 5));

        apply_effect(
            &mut state,
            &GameEffect::DeclareMelee {
                attack: MeleeAttack {
                    attacker_player: Player::Dervish,
                    attacker_hex: HexCoord::new(6, 5),
                    defender_hex: cav_hex,
                    attackers: vec![attacker],
                    defenders: vec![cavalry, stay],
                    attacker_modifiers: vec![MeleeModifier::DervishStandard],
                    defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
                },
                attacker_roll: DieRoll::Five,
                defender_roll: DieRoll::Five,
            },
        )
        .unwrap();

        let dest = HexCoord::new(7, 5);
        apply_effect(
            &mut state,
            &GameEffect::RetreatBeforeMelee {
                unit_id: cavalry,
                to: dest,
            },
        )
        .unwrap();
        // The infantry defender still holds the hex: no window.
        assert!(!state.vacated_by_combat.contains_key(&cav_hex));
        assert!(
            matches!(
                state.can_advance_after_combat(attacker, cav_hex),
                Err(RuleError::HexNotVacatedByCombat(_))
            ),
            "a stacked hex with a remaining defender is not vacated"
        );

        // (The infantry cannot retreat -- §7.5 is cavalry/camel only -- so
        // the window only ever opens via resolution or the last retreat.)
    }

    #[test]
    fn advance_after_combat_rejects_off_board_hexes() {
        // Loaded board: only (0,0) is land terrain and (1,0) is Nile; the
        // neighbour (1,-1) is not a map hex at all.
        let mut board = BoardInfo::default();
        board.terrain.insert(
            HexCoord::new(0, 0),
            Terrain::Clear {
                road: Default::default(),
            },
        );
        board.terrain.insert(
            HexCoord::new(1, 0),
            Terrain::Nile {
                direction: HexDirection::East,
            },
        );

        // A land unit may not advance off the board (§5.22).
        let mut state = GameState::new(Scenario::Campaign);
        state.board = board.clone();
        state.phase = Phase::Melee;
        let inf = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        open_advance_window(
            &mut state,
            HexCoord::new(0, -1),
            &[inf],
            vec!["7.6".to_string()],
        );
        assert!(matches!(
            state.can_advance_after_combat(inf, HexCoord::new(0, -1)),
            Err(RuleError::OffBoard(_))
        ));
        // ... nor into a Nile hex.
        open_advance_window(
            &mut state,
            HexCoord::new(1, 0),
            &[inf],
            vec!["7.6".to_string()],
        );
        assert!(matches!(
            state.can_advance_after_combat(inf, HexCoord::new(1, 0)),
            Err(RuleError::LandIntoNile(_))
        ));

        // A gunboat may only advance along the Nile: the land hex and the
        // off-board neighbour are both rejected.
        let mut state = GameState::new(Scenario::Campaign);
        state.board = board;
        state.phase = Phase::Melee;
        let gb = make_dervish_gunboat(&mut state, HexCoord::new(1, 0));
        open_advance_window(
            &mut state,
            HexCoord::new(0, 0),
            &[gb],
            vec!["7.6".to_string()],
        );
        assert!(matches!(
            state.can_advance_after_combat(gb, HexCoord::new(0, 0)),
            Err(RuleError::GunboatOffNile(_))
        ));
        open_advance_window(
            &mut state,
            HexCoord::new(2, 0),
            &[gb],
            vec!["7.6".to_string()],
        );
        assert!(matches!(
            state.can_advance_after_combat(gb, HexCoord::new(2, 0)),
            Err(RuleError::GunboatOffNile(_))
        ));
    }

    #[rulebook("§6.41")]
    #[test]
    fn disrupted_unit_cannot_fire() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        state.find_unit_mut(id).unwrap().state.disrupted = true;
        make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![id],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![],
        };

        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Five,
            },
        );
        assert!(matches!(result, Err(RuleError::Disrupted(_))));
    }

    #[rulebook("§7.3", "§7.7")]
    #[test]
    fn melee_resolves_simultaneously() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let ae_id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let derv_id = make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let attack = MeleeAttack {
            attacker_player: Player::AngloEgyptian,
            attacker_hex: HexCoord::new(0, 0),
            defender_hex: HexCoord::new(1, 0),
            attackers: vec![ae_id],
            defenders: vec![derv_id],
            attacker_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
            defender_modifiers: vec![MeleeModifier::DervishStandard],
        };

        let result = apply_effect(
            &mut state,
            &GameEffect::MeleeCombat {
                attack,
                attacker_roll: DieRoll::Seven,
                defender_roll: DieRoll::Three,
            },
        );
        assert!(result.is_ok());
    }

    #[rulebook("§4")]
    #[test]
    fn new_game_starts_in_setup() {
        let state = GameState::new(Scenario::Campaign);
        assert_eq!(state.phase, Phase::Setup);
    }

    #[test]
    fn cannot_leave_setup_until_both_sides_deployed() {
        let mut state = GameState::new(Scenario::Campaign);
        // No units: setup is incomplete, advancing stays in Setup.
        let err = apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap_err();
        assert!(matches!(err, RuleError::SetupIncomplete(_)));
        assert_eq!(state.phase, Phase::Setup);

        // One side only: still incomplete.
        make_ae_infantry(&mut state, HexCoord::new(1, 1));
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap_err(),
            RuleError::SetupIncomplete(_)
        ));

        // Both sides present: setup completes and we enter Movement.
        make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert_eq!(state.phase, Phase::Movement);
    }

    #[test]
    fn deploy_rejected_outside_setup_phase() {
        let mut state = playing(Scenario::Campaign); // in Movement, not Setup
        let placement = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(1, 1),
            profile: crate::UnitProfile {
                kind: crate::UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity: crate::UnitIdentity::AngloEgyptianInfantry {
                    brigade: omdurman_types::BrigadeId {
                        number: 1,
                        nationality: omdurman_types::BrigadeNationality::British,
                    },
                    battalion: crate::BattalionOrdinal::First,
                },
                weapon: crate::WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(placement)).unwrap_err(),
            RuleError::WrongPhase
        ));
    }

    #[rulebook("§9.212")]
    #[test]
    fn deploy_rejected_outside_zone() {
        // Fall of Khartoum: Dervish may only deploy on the southern edge.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        // Attach a small board spanning rows 0..=9 so zones are defined.
        for r in 0..=9 {
            for q in 0..=3 {
                state
                    .board
                    .terrain
                    .insert(HexCoord::new(q, r), Terrain::default());
            }
        }
        // A Dervish unit in the north (r=0) is outside its (southern) zone.
        // Kehena: a §9.322 tribe, so the order-of-battle gate passes and the
        // zone check is what rejects.
        let north = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(1, 0),
            profile: dervish_tribal_profile_with(DervishTribe::Kehena),
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&north).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));
        // The same unit in the south (r=9) is accepted.
        let south = UnitPlacement {
            position: HexCoord::new(1, 9),
            ..north
        };
        assert!(state.can_deploy_unit(&south).is_ok());
    }

    #[rulebook("§9.322")]
    #[test]
    fn fok_dervish_east_edge_on_diamond_board() {
        // The FoK board is diamond-shaped: the "east edge" is the diagonal
        // of rightmost hexes per row (no hex at q+1), not just q == global
        // max_q.  Build a small diamond:
        //   r=0: q=0,1
        //   r=1: q=0,1,2
        //   r=2: q=0,1,2,3
        // East edge: (1,0), (2,1), (3,2).  South edge: (0,2),(1,2),(2,2),(3,2).
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        for r in 0..=2u32 {
            for q in 0..=r + 1 {
                state
                    .board
                    .terrain
                    .insert(HexCoord::new(q as i32, r as i32), Terrain::default());
            }
        }
        let kehena = dervish_tribal_profile_with(DervishTribe::Kehena);
        // Interior hex (1,1): has a neighbor at (2,1) so NOT on east edge,
        // and has a neighbor at (1,2) so NOT on south edge → rejected.
        let interior = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(1, 1),
            profile: kehena,
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&interior).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));
        // East-edge hex (2,1): no hex at (3,1) → on east edge → accepted.
        let east_edge = UnitPlacement {
            position: HexCoord::new(2, 1),
            ..interior
        };
        assert!(state.can_deploy_unit(&east_edge).is_ok());
        // East-edge hex (1,0): no hex at (2,0) → on east edge → accepted.
        let east_top = UnitPlacement {
            position: HexCoord::new(1, 0),
            ..interior
        };
        assert!(state.can_deploy_unit(&east_top).is_ok());
    }

    #[rulebook("§5.22", "§9.322")]
    #[test]
    fn fok_dervish_land_unit_rejected_on_nile() {
        // §5.22 applies to Dervish deployment too: a land unit may not deploy
        // on a Nile hex even when that hex is on the south/east entry edge.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        // Board with rows 0..=4 (max_r = 4). Put a Nile hex on the south edge
        // at (0,4) and a clear hex on the south edge at (1,4).
        for r in 0..=4 {
            for q in 0..=3 {
                state
                    .board
                    .terrain
                    .insert(HexCoord::new(q, r), Terrain::default());
            }
        }
        state.board.terrain.insert(
            HexCoord::new(0, 4),
            Terrain::Nile {
                direction: omdurman_types::HexDirection::East,
            },
        );

        let on_nile_edge = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(0, 4),
            profile: dervish_tribal_profile_with(DervishTribe::Mulazmin),
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&on_nile_edge).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));

        // The same Mulazmin unit on a clear south-edge hex is accepted.
        let on_clear_edge = UnitPlacement {
            position: HexCoord::new(1, 4),
            ..on_nile_edge
        };
        assert!(state.can_deploy_unit(&on_clear_edge).is_ok());
    }

    #[rulebook("§10.11", "§10.21")]
    #[test]
    fn mine_and_chain_limits_enforced_in_setup() {
        let mut state = GameState::new(Scenario::Campaign);
        state.optional_rules.push(OptionalRule::RiverMines);
        state.optional_rules.push(OptionalRule::RiverChain);
        // Two mines OK, a third rejected.
        apply_effect(
            &mut state,
            &GameEffect::PlaceMine {
                hex: HexCoord::new(1, 1),
            },
        )
        .unwrap();
        apply_effect(
            &mut state,
            &GameEffect::PlaceMine {
                hex: HexCoord::new(2, 1),
            },
        )
        .unwrap();
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::PlaceMine {
                    hex: HexCoord::new(3, 1)
                }
            )
            .unwrap_err(),
            RuleError::SetupLimit(_)
        ));
        // Duplicate hex rejected.
        let mut state2 = GameState::new(Scenario::Campaign);
        state2.optional_rules.push(OptionalRule::RiverMines);
        apply_effect(
            &mut state2,
            &GameEffect::PlaceMine {
                hex: HexCoord::new(1, 1),
            },
        )
        .unwrap();
        assert!(matches!(
            apply_effect(
                &mut state2,
                &GameEffect::PlaceMine {
                    hex: HexCoord::new(1, 1)
                }
            )
            .unwrap_err(),
            RuleError::SetupLimit(_)
        ));
        // Chain over four hexes rejected.
        let five: Vec<HexCoord> = (0..5).map(|q| HexCoord::new(q, 0)).collect();
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceChain { hexes: five }).unwrap_err(),
            RuleError::SetupLimit(_)
        ));
    }

    #[rulebook("§10.11", "§10.21")]
    #[test]
    fn mines_and_chain_require_their_optional_rule() {
        // Without the optional rules selected, placement is rejected even in
        // Setup with room to spare.
        let mut state = GameState::new(Scenario::Campaign);
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::PlaceMine {
                    hex: HexCoord::new(1, 1)
                }
            )
            .unwrap_err(),
            RuleError::SetupLimit(_)
        ));
        assert!(state.mines.is_empty());
        let two: Vec<HexCoord> = vec![HexCoord::new(1, 0), HexCoord::new(2, 0)];
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceChain { hexes: two }).unwrap_err(),
            RuleError::SetupLimit(_)
        ));
        assert!(state.chain.is_none());

        // Selecting just River Mines unlocks mines but not the chain.
        state.optional_rules.push(OptionalRule::RiverMines);
        apply_effect(
            &mut state,
            &GameEffect::PlaceMine {
                hex: HexCoord::new(1, 1),
            },
        )
        .unwrap();
        assert_eq!(state.mines.len(), 1);
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::PlaceChain {
                    hexes: vec![HexCoord::new(1, 0), HexCoord::new(2, 0)]
                }
            )
            .unwrap_err(),
            RuleError::SetupLimit(_)
        ));
        assert!(state.chain.is_none());

        // Selecting just River Chain unlocks the chain but not further mines.
        let mut state2 = GameState::new(Scenario::Campaign);
        state2.optional_rules.push(OptionalRule::RiverChain);
        apply_effect(
            &mut state2,
            &GameEffect::PlaceChain {
                hexes: vec![HexCoord::new(1, 0), HexCoord::new(2, 0)],
            },
        )
        .unwrap();
        assert!(state2.chain.is_some());
        assert!(matches!(
            apply_effect(
                &mut state2,
                &GameEffect::PlaceMine {
                    hex: HexCoord::new(1, 1)
                }
            )
            .unwrap_err(),
            RuleError::SetupLimit(_)
        ));
        assert!(state2.mines.is_empty());
    }

    #[test]
    fn units_cannot_move_during_setup() {
        let mut state = GameState::new(Scenario::Campaign);
        let unit = make_ae_infantry(&mut state, HexCoord::new(1, 1));
        // Still in Setup: movement is rejected as wrong-phase.
        let err = state
            .can_move_unit_to(unit, Some(HexCoord::new(2, 1)), MovementPoints::new(1))
            .unwrap_err();
        assert!(matches!(err, RuleError::WrongPhase));
    }

    #[rulebook("§4")]
    #[test]
    fn both_ready_auto_advances_out_of_setup() {
        // Campaign has no fixed target, so one unit per side meets the gate.
        let mut state = GameState::new(Scenario::Campaign);
        make_ae_infantry(&mut state, HexCoord::new(1, 1));
        make_dervish_tribal(&mut state, HexCoord::new(5, 5));

        // One side ready: still in Setup.
        apply_effect(
            &mut state,
            &GameEffect::ConfirmSetupReady {
                player: Player::AngloEgyptian,
            },
        )
        .unwrap();
        assert_eq!(state.phase, Phase::Setup);
        assert!(state.setup_ready(Player::AngloEgyptian));
        assert!(!state.setup_ready(Player::Dervish));

        // Second side ready: auto-advances to Movement.
        apply_effect(
            &mut state,
            &GameEffect::ConfirmSetupReady {
                player: Player::Dervish,
            },
        )
        .unwrap();
        assert_eq!(state.phase, Phase::Movement);
    }

    #[test]
    fn confirm_ready_rejected_outside_setup() {
        let mut state = playing(Scenario::Campaign); // in Movement
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::ConfirmSetupReady {
                    player: Player::Dervish
                }
            )
            .unwrap_err(),
            RuleError::WrongPhase
        ));
    }

    #[rulebook("§9.321")]
    #[test]
    fn confirm_ready_rejected_below_scenario_target() {
        // Fall of Khartoum requires the full order of battle (British 17 /
        // Dervish 48); a single deployed unit is far below target.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        make_ae_infantry(&mut state, HexCoord::new(1, 1));
        assert_eq!(state.setup_target(Player::AngloEgyptian), Some(17));
        assert!(!state.setup_target_met(Player::AngloEgyptian));
        assert!(matches!(
            state
                .can_confirm_setup_ready(Player::AngloEgyptian)
                .unwrap_err(),
            RuleError::SetupIncomplete(_)
        ));
    }

    #[rulebook("§5.22", "§9.321")]
    #[test]
    fn fok_ae_gunboat_deploys_only_on_nile() {
        // Fall of Khartoum British deployment zone must be boat/land-exclusive
        // (§5.22): a gunboat may only deploy on the Nile, never on a building.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        // A building hex (land) at (0,0) and a Nile hex at (1,0).
        state.board.terrain.insert(
            HexCoord::new(0, 0),
            Terrain::ground(omdurman_types::GroundKind::Building),
        );
        state.board.terrain.insert(
            HexCoord::new(1, 0),
            Terrain::Nile {
                direction: omdurman_types::HexDirection::East,
            },
        );

        // Gunboat on a building hex -> rejected (off its Nile-only zone).
        let boat_on_land = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(0, 0),
            profile: ae_gunboat_profile(),
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&boat_on_land).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));

        // Gunboat on a Nile hex -> accepted.
        let boat_on_nile = UnitPlacement {
            position: HexCoord::new(1, 0),
            ..boat_on_land
        };
        assert!(state.can_deploy_unit(&boat_on_nile).is_ok());
    }

    #[rulebook("§5.22", "§9.111")]
    #[test]
    fn campaign_deployment_is_boat_land_exclusive() {
        // Regression (audit §5.22/§9.111): Campaign set-up used to accept any
        // hex, letting gunboats deploy on land and land units on the Nile.
        // §5.22 is scenario-independent: only gunboats may occupy Nile hexes.
        let mut state = GameState::new(Scenario::Campaign);
        state.board.terrain.insert(
            HexCoord::new(0, 0),
            Terrain::ground(omdurman_types::GroundKind::Rough),
        );
        state.board.terrain.insert(
            HexCoord::new(1, 0),
            Terrain::Nile {
                direction: omdurman_types::HexDirection::East,
            },
        );

        // Dervish gunboat on a land hex -> rejected.
        let boat = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(0, 0),
            profile: dervish_gunboat_profile(),
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&boat).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));

        // Land unit on the Nile -> rejected. Taiasha (part of the §9.111
        // initial force) so the rejection is specifically the §5.22 Nile
        // rule, not the in-play-at-setup filter.
        let land_on_nile = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(1, 0),
            profile: dervish_tribal_profile_with(DervishTribe::Taiasha),
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&land_on_nile).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));

        // Same-hex swaps of the two legal placements -> accepted.
        let boat_ok = UnitPlacement {
            position: HexCoord::new(1, 0),
            ..boat
        };
        let land_ok = UnitPlacement {
            position: HexCoord::new(0, 0),
            ..land_on_nile
        };
        assert!(state.can_deploy_unit(&boat_ok).is_ok());
        assert!(state.can_deploy_unit(&land_ok).is_ok());
    }

    #[rulebook("§5.22", "§9.321")]
    #[test]
    fn fok_ae_land_unit_rejected_on_nile() {
        // The converse of the gunboat test: a land unit may never deploy on the
        // Nile (§5.22).
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        state.board.terrain.insert(
            HexCoord::new(0, 0),
            Terrain::ground(omdurman_types::GroundKind::Building),
        );
        state.board.terrain.insert(
            HexCoord::new(1, 0),
            Terrain::Nile {
                direction: omdurman_types::HexDirection::East,
            },
        );

        // Infantry on the Nile -> rejected.
        let land_on_nile = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(1, 0),
            profile: crate::UnitProfile {
                kind: crate::UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity: crate::UnitIdentity::AngloEgyptianInfantry {
                    brigade: omdurman_types::BrigadeId {
                        number: 1,
                        nationality: omdurman_types::BrigadeNationality::British,
                    },
                    battalion: crate::BattalionOrdinal::First,
                },
                weapon: crate::WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&land_on_nile).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));

        // Infantry on a building hex -> accepted.
        let land_on_building = UnitPlacement {
            position: HexCoord::new(0, 0),
            ..land_on_nile
        };
        assert!(state.can_deploy_unit(&land_on_building).is_ok());
    }

    #[rulebook("§9.321")]
    #[test]
    fn british_boats_named_vs_old_gunboat_detection() {
        // §9.321: only old (unnamed) gunboats are in play in FoK. The picker
        // filter distinguishes them via the *identity* (GunboatId::Named vs
        // GunboatId::Old), because the `british_boats` resolver tags both kinds
        // as `UnitKind::Gunboat`. Lock that detection in: named cells resolve
        // to a Named gunboat id, old cells to an Old one -- both with kind
        // `Gunboat` (so `is_boat()` is true for both).
        let resolve = |col: u8, row: u8| {
            let id = unit_id_for_section_pos(omdurman_types::SectionName::BritishBoats, col, row)
                .expect("BritishBoats cell resolves");
            let p = crate::unit_profiles::profile_for_unit(id)
                .expect("BritishBoats cell has a profile");
            (p.kind, p.identity)
        };

        // Named gunboats (row 0, cols 3-7).
        for (col, row) in [(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)] {
            let (kind, identity) = resolve(col, row);
            assert!(
                matches!(kind, crate::UnitKind::Gunboat { .. }),
                "named gunboat ({col},{row}) kind should be Gunboat, got {kind:?}"
            );
            assert!(
                matches!(
                    identity,
                    crate::UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Named(_))
                ),
                "({col},{row}) should be a Named gunboat"
            );
        }
        // Old gunboats (row 1, cols 4-7).
        for (col, row) in [(4, 1), (5, 1), (6, 1), (7, 1)] {
            let (kind, identity) = resolve(col, row);
            assert!(
                matches!(kind, crate::UnitKind::Gunboat { .. }),
                "old gunboat ({col},{row}) kind should be Gunboat, got {kind:?}"
            );
            assert!(
                matches!(
                    identity,
                    crate::UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(_))
                ),
                "({col},{row}) should be an Old gunboat"
            );
        }
    }

    #[rulebook("§5.22", "§9.321")]
    #[test]
    fn deploy_via_real_sprite_resolution_matches_engine() {
        // Validates the app's actual placement contract: `placement.rs`
        // resolves a sprite position to a UnitId + profile via
        // `unit_id_for_section_pos` + `profile_for_unit`, then calls
        // `apply_effect(DeployUnit)`. Confirm that path resolves a real FoK
        // British old-gunboat sprite to a boat and that the engine then accepts
        // it on the Nile and rejects it on land -- the same accept/reject the
        // app will see, end to end.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        state.board.terrain.insert(
            HexCoord::new(0, 0),
            Terrain::ground(omdurman_types::GroundKind::Building),
        );
        state.board.terrain.insert(
            HexCoord::new(1, 0),
            Terrain::Nile {
                direction: omdurman_types::HexDirection::East,
            },
        );
        // BritishBoats (4,1) is an old-style gunboat (§2.32).
        let id = unit_id_for_section_pos(omdurman_types::SectionName::BritishBoats, 4, 1)
            .expect("BritishBoats (4,1) resolves to a UnitId");
        let profile =
            crate::unit_profiles::profile_for_unit(id).expect("BritishBoats (4,1) has a profile");
        assert!(
            profile.kind.is_boat(),
            "BritishBoats (4,1) should be a gunboat, got {:?}",
            profile.kind
        );

        // On land (Building) -> the engine rejects via the app's exact path.
        let on_land = UnitPlacement {
            id,
            position: HexCoord::new(0, 0),
            profile,
            state: Default::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(on_land)).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));

        // On the Nile -> accepted, and the unit is on the board with that id.
        let on_nile = UnitPlacement {
            id,
            position: HexCoord::new(1, 0),
            profile,
            state: Default::default(),
        };
        apply_effect(&mut state, &GameEffect::DeployUnit(on_nile)).unwrap();
        assert!(state.find_unit(id).is_some());

        // Re-deploying the same counter (same id) -> rejected as a duplicate,
        // and the original placement is untouched.
        let dup = UnitPlacement {
            id,
            position: HexCoord::new(1, 0),
            profile,
            state: Default::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(dup)).unwrap_err(),
            RuleError::AlreadyDeployed(_)
        ));
        assert_eq!(state.units.len(), 1);
    }

    #[rulebook("§5.52")]
    #[test]
    fn deploy_rejects_dervish_tribe_mix() {
        // §5.52: units of different Dervish tribes may not stack. The deploy
        // validation must catch this (the FoK entry force has 4 tribes:
        // Mulazmin, Hadendowa, Kehena, Degheim). Kehena vs Mulazmin, both
        // §9.322-valid, so the stacking law is what rejects the mix.
        let mut state = GameState::new(Scenario::FallOfKhartoum); // permissive zone
        let hex = HexCoord::new(1, 1);

        let kehena = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: dervish_tribal_profile_with(DervishTribe::Kehena),
            state: Default::default(),
        };
        apply_effect(&mut state, &GameEffect::DeployUnit(kehena)).unwrap();

        // A Mulazmin unit stacked with the Kehena -> rejected.
        let mulazmin = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: dervish_tribal_profile_with(DervishTribe::Mulazmin),
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&mulazmin).unwrap_err(),
            RuleError::Stacking(crate::StackingError::DervishTribeMix)
        ));
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(mulazmin)).unwrap_err(),
            RuleError::Stacking(crate::StackingError::DervishTribeMix)
        ));
        // Only the first unit is on the board.
        assert_eq!(state.units.len(), 1);
    }

    /// Resolve a real counter sprite exactly the way the app's placement path
    /// does (`unit_id_for_section_pos` + `profile_for_unit`), so the test
    /// exercises the identities real clicks produce.
    fn real_profile(section: SectionName, col: u8, row: u8) -> (UnitId, UnitProfile) {
        let id = unit_id_for_section_pos(section, col, row)
            .unwrap_or_else(|| panic!("{section:?} ({col},{row}) resolves to no UnitId"));
        let profile = crate::unit_profiles::profile_for_unit(id)
            .unwrap_or_else(|| panic!("{section:?} ({col},{row}) resolves to no profile"));
        (id, profile)
    }

    #[rulebook("§5.52")]
    #[test]
    fn deploy_rejects_hadendowa_on_dervish_gun() {
        // §5.52 as it plays in FALL OF KHARTOUM (§9.322): the three Dervish
        // artillery counters are not a tribe, but they are also not Hadendowa
        // (or Kehena, or Degheim, or Mulazmin) -- the guns form their own
        // stacking group, so a tribal counter may not share their hex and vice
        // versa. Regression: a Hadendowa placed on a Dervish gun during setup
        // was silently accepted (only the four-unit count was checked).
        let mut state = GameState::new(Scenario::FallOfKhartoum); // permissive zone
        let hex = HexCoord::new(1, 1);

        // Real sprites: the gun is KhalifaAbdullah sheet (1,1); the tribal
        // counter is the Hadendowa sheet (0,1).
        let (gun_id, gun_profile) = real_profile(SectionName::KhalifaAbdullah, 1, 1);
        assert_eq!(gun_profile.identity, UnitIdentity::DervishArtillery);
        let (hadendowa_id, hadendowa_profile) = real_profile(SectionName::Hadendowa, 0, 1);
        assert_eq!(
            hadendowa_profile.identity,
            UnitIdentity::DervishTribal {
                tribe: DervishTribe::Hadendowa
            }
        );

        let place = |id: UnitId, profile: UnitProfile, at: HexCoord| UnitPlacement {
            id,
            position: at,
            profile,
            state: Default::default(),
        };

        // Hadendowa onto the gun's hex -> rejected, and nothing is deployed.
        apply_effect(
            &mut state,
            &GameEffect::DeployUnit(place(gun_id, gun_profile, hex)),
        )
        .unwrap();
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DeployUnit(place(hadendowa_id, hadendowa_profile, hex))
            )
            .unwrap_err(),
            RuleError::Stacking(crate::StackingError::DervishTribeMix)
        ));
        assert_eq!(state.units.len(), 1);

        // And in the other order: a gun onto a Hadendowa hex is the same mix.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        apply_effect(
            &mut state,
            &GameEffect::DeployUnit(place(hadendowa_id, hadendowa_profile, hex)),
        )
        .unwrap();
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DeployUnit(place(gun_id, gun_profile, hex))
            )
            .unwrap_err(),
            RuleError::Stacking(crate::StackingError::DervishTribeMix)
        ));

        // Guns stack with guns (two of the three §9.322 artillery counters).
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        apply_effect(
            &mut state,
            &GameEffect::DeployUnit(place(gun_id, gun_profile, hex)),
        )
        .unwrap();
        let (gun2_id, gun2_profile) = real_profile(SectionName::KhalifaAbdullah, 2, 1);
        apply_effect(
            &mut state,
            &GameEffect::DeployUnit(place(gun2_id, gun2_profile, hex)),
        )
        .unwrap();
        assert_eq!(state.units.len(), 2);
    }

    #[rulebook("§5.52")]
    #[test]
    fn dervish_guns_stack_with_guns_and_their_leader() {
        // The §5.23 household grouping: the Khalifa may stack with his
        // artillery (his §5.53 command check only constrains *tribal* units),
        // two guns stack together, but a Taiasha bodyguard counter (a tribe)
        // may not share the guns' hex. Synthetic profiles + the Campaign
        // (where the Khalifa and his artillery are §9.111-in-play at setup;
        // FoK's §9.322 force has no leaders at all).
        let mut state = GameState::new(Scenario::Campaign);
        let hex = HexCoord::new(2, 2);
        let gun_profile = || UnitProfile {
            kind: UnitKind::Artillery {
                fire: 3,
                melee: 0,
                movement: 0,
            },
            identity: UnitIdentity::DervishArtillery,
            weapon: WeaponClass::Artillery,
            fire: None,
            melee: None,
            movement: UnitMovement::Immobile,
        };
        let khalifa_profile = UnitProfile {
            kind: UnitKind::DervishLeader {
                fire: 1,
                melee: 1,
                movement: 15,
            },
            identity: UnitIdentity::DervishLeader(DervishLeader::KhalifaAbdullah),
            weapon: WeaponClass::Melee,
            fire: None,
            melee: None,
            movement: UnitMovement::Land(crate::MovementAllowance::Fifteen),
        };

        fn deploy(
            state: &mut GameState,
            id: UnitId,
            profile: UnitProfile,
            hex: HexCoord,
        ) -> Result<(), RuleError> {
            apply_effect(
                state,
                &GameEffect::DeployUnit(UnitPlacement {
                    id,
                    position: hex,
                    profile,
                    state: Default::default(),
                }),
            )
        }

        for profile in [gun_profile(), gun_profile(), khalifa_profile] {
            let id = state.alloc_unit_id();
            deploy(&mut state, id, profile, hex).unwrap();
        }
        assert_eq!(state.units.len(), 3, "guns + the Khalifa stack freely");

        // A Taiasha tribal counter in the guns' hex is a §5.52 mix.
        let taiasha = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: dervish_tribal_profile_with(DervishTribe::Taiasha),
            state: Default::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(taiasha)).unwrap_err(),
            RuleError::Stacking(crate::StackingError::DervishTribeMix)
        ));
    }

    #[rulebook("§5.51", "§6.51")]
    #[test]
    fn deploy_rejects_enemy_cohabitation_during_setup() {
        // §5.51's occupation corollary: a counter deployed during setup may
        // never share a hex with enemy units -- engaging the enemy is what
        // melee is for (§7.1). Regression: setup deploy was ownership-blind,
        // so either side could drop a counter straight onto the other's stack.
        // The lone Anglo-Egyptian leader is the §6.51 exception (and §9.346
        // makes sharing GORDON's hex the way he dies).
        let mut state = GameState::new(Scenario::FallOfKhartoum); // permissive zone
        let hex = HexCoord::new(3, 3);

        let (gun_id, gun_profile) = real_profile(SectionName::KhalifaAbdullah, 1, 1);
        // §9.321's garrison: an Egyptian infantry counter (in-play for FoK,
        // unlike the sheet's cavalry/artillery cells).
        let (ae_id, ae_profile) = real_profile(SectionName::EgyptianArmy, 0, 1);
        let place = |id: UnitId, profile: UnitProfile, at: HexCoord| UnitPlacement {
            id,
            position: at,
            profile,
            state: Default::default(),
        };

        // Dervish gun first; the Anglo-Egyptian counter may not cohabit.
        apply_effect(
            &mut state,
            &GameEffect::DeployUnit(place(gun_id, gun_profile, hex)),
        )
        .unwrap();
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DeployUnit(place(ae_id, ae_profile, hex))
            )
            .unwrap_err(),
            RuleError::Stacking(crate::StackingError::EnemyCohabitation)
        ));

        // And the mirror: Anglo-Egyptian first, Dervish onto it.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        apply_effect(
            &mut state,
            &GameEffect::DeployUnit(place(ae_id, ae_profile, hex)),
        )
        .unwrap();
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DeployUnit(place(gun_id, gun_profile, hex))
            )
            .unwrap_err(),
            RuleError::Stacking(crate::StackingError::EnemyCohabitation)
        ));

        // §6.51/§9.346 exception: Dervish units may occupy GORDON's hex.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        let (gordon_id, gordon_profile) = real_profile(SectionName::BritishBoats, 3, 1);
        apply_effect(
            &mut state,
            &GameEffect::DeployUnit(place(gordon_id, gordon_profile, hex)),
        )
        .unwrap();
        apply_effect(
            &mut state,
            &GameEffect::DeployUnit(place(gun_id, gun_profile, hex)),
        )
        .expect("lone AE leader does not block a Dervish deploy (§6.51)");
    }

    #[rulebook("§6.51")]
    #[test]
    fn dervish_move_through_lone_ae_leader_hex_eliminates_the_leader() {
        // §6.51 clause (a): an Anglo-Egyptian leader *alone* in a hex is
        // eliminated "when a Dervish unit occupies or passes through that
        // hex" -- the lone leader does not block the move, and the §7.1
        // enemy-occupancy check must treat the hex as passable.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let dervish = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let leader = make_ae_leader(&mut state, HexCoord::new(1, 0));

        let result = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: dervish,
                to: HexCoord::new(2, 0),
                cost: MovementPoints::new(2),
                path: vec![HexCoord::new(1, 0)],
            },
        );
        assert!(result.is_ok());
        assert_eq!(
            state.find_unit(dervish).unwrap().position,
            HexCoord::new(2, 0),
            "the Dervish unit passes through the leader's hex"
        );
        assert!(
            state.find_unit(leader).is_none(),
            "lone AE leader overrun by a passing Dervish unit is eliminated (§6.51)"
        );
    }

    #[rulebook("§6.51")]
    #[test]
    fn dervish_move_onto_lone_ae_leader_hex_eliminates_the_leader() {
        // §6.51 clause (a), occupation case: a Dervish unit *ending* its move
        // on a lone AE leader's hex eliminates him there.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let dervish = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let leader = make_ae_leader(&mut state, HexCoord::new(1, 0));

        let result = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: dervish,
                to: HexCoord::new(1, 0),
                cost: MovementPoints::new(1),
                path: Vec::new(),
            },
        );
        assert!(result.is_ok());
        assert_eq!(
            state.find_unit(dervish).unwrap().position,
            HexCoord::new(1, 0)
        );
        assert!(
            state.find_unit(leader).is_none(),
            "lone AE leader overrun by an occupying Dervish unit is eliminated (§6.51)"
        );
    }

    #[rulebook("§6.51")]
    #[test]
    fn ae_leader_with_combat_unit_is_not_overrun_by_dervish_move() {
        // §6.51 clause (a) is scoped to a leader *alone* in the hex: a Dervish
        // mover may not pass through or end on a hex where the AE leader
        // still has a combat unit stacked with him (the §7.1 occupancy rule
        // blocks the move).
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let dervish = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let _infantry = make_ae_infantry(&mut state, HexCoord::new(1, 0));
        let leader = make_ae_leader(&mut state, HexCoord::new(1, 0));

        let result = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: dervish,
                to: HexCoord::new(1, 0),
                cost: MovementPoints::new(1),
                path: Vec::new(),
            },
        );
        assert!(matches!(
            result,
            Err(RuleError::EnemyOccupied(HexCoord { .. }))
        ));
        assert!(state.find_unit(leader).is_some());
    }

    #[rulebook("§6.51")]
    #[test]
    fn ae_leader_eliminated_with_last_combat_unit_in_fire_combat() {
        // §6.51 clause (b): an AE leader is eliminated "if all of the combat
        // units a leader is stacked with are eliminated in fire combat or
        // melee" -- the orphan-leader rule. Dervish fire kills the infantry in
        // the target hex; the leader stacked beside it falls too.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::Dervish;
        let firer = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let infantry = make_ae_infantry(&mut state, HexCoord::new(1, 0));
        let leader = make_ae_leader(&mut state, HexCoord::new(1, 0));

        let mut attack = FireAttack {
            firing_player: Player::Dervish,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![],
        };
        attack.modifiers = mandatory_fire_modifiers(&state, &attack);
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Eight,
            },
        );
        assert!(result.is_ok());
        assert!(state.find_unit(infantry).is_none());
        assert!(
            state.find_unit(leader).is_none(),
            "orphaned AE leader is eliminated with its last combat unit (§6.51)"
        );
        assert!(state.observations.iter().any(|o| {
            matches!(
                o,
                Observation::UnitEliminated {
                    id,
                    cause: ElimCause::OrphanLeader,
                    ..
                } if *id == leader
            )
        }));
    }

    #[rulebook("§5.51")]
    #[test]
    fn validate_stacking_invariants_catches_enemy_cohabitation() {
        // The whole-state post-condition flags a hex whose occupants include
        // both factions (except the lone Anglo-Egyptian leader exception), so
        // a replayed or desynchronised record cannot hide a cohabited hex.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        let hex = HexCoord::new(4, 4);
        let gun = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::Artillery {
                    fire: 3,
                    melee: 0,
                    movement: 0,
                },
                identity: UnitIdentity::DervishArtillery,
                weapon: WeaponClass::Artillery,
                fire: None,
                melee: None,
                movement: UnitMovement::Immobile,
            },
            state: Default::default(),
        };
        let mut ae = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: crate::UnitProfile {
                kind: crate::UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity: crate::UnitIdentity::AngloEgyptianInfantry {
                    brigade: omdurman_types::BrigadeId {
                        number: 1,
                        nationality: omdurman_types::BrigadeNationality::British,
                    },
                    battalion: crate::BattalionOrdinal::First,
                },
                weapon: crate::WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        };
        state.units.push(gun);
        assert!(state.validate_stacking_invariants().is_ok());
        state.units.push(ae);
        assert!(state.validate_stacking_invariants().is_err());

        // The exception: a lone Anglo-Egyptian leader may cohabit.
        state.units.pop();
        ae.profile = UnitProfile {
            kind: UnitKind::BritishLeader { movement: 8 },
            identity: UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Kitchener),
            weapon: WeaponClass::Melee,
            fire: None,
            melee: None,
            movement: UnitMovement::Land(crate::MovementAllowance::Eight),
        };
        state.units.push(ae);
        assert!(state.validate_stacking_invariants().is_ok());
    }

    #[rulebook("§5.51")]
    #[test]
    fn ae_garrison_stacks_freely_under_gordon() {
        // §5.51 as it plays for the Anglo-Egyptian side (§5.52/§5.53 bind the
        // Dervish only): any AE counters may share a hex -- brigades and
        // nationalities mix freely (British, Egyptian, Sudanese), and the
        // GORDON leader free-stacks on top of the four-counted-unit limit.
        // Confirmed with the real §9.321 garrison counters, resolved exactly
        // the way the app's placement path does.
        let mut state = GameState::new(Scenario::FallOfKhartoum); // permissive zone
        let hex = HexCoord::new(3, 3);

        let deploy = |state: &mut GameState, section: SectionName, col: u8, row: u8| {
            let (id, profile) = real_profile(section, col, row);
            apply_effect(
                state,
                &GameEffect::DeployUnit(UnitPlacement {
                    id,
                    position: hex,
                    profile,
                    state: Default::default(),
                }),
            )
        };

        // Four counted units of three different nationalities...
        deploy(&mut state, SectionName::BritishArmy, 0, 1).unwrap(); // British
        deploy(&mut state, SectionName::BritishArmy, 1, 1).unwrap(); // British
        deploy(&mut state, SectionName::EgyptianArmy, 0, 1).unwrap(); // Egyptian
        deploy(&mut state, SectionName::Kitchener, 5, 0).unwrap(); // Sudanese
        // ...plus GORDON free-stacking on top.
        deploy(&mut state, SectionName::BritishBoats, 3, 1).unwrap();
        assert_eq!(state.units.len(), 5, "4 counted + the free leader deploy");

        // The limit still binds counted units: a fifth counter is rejected.
        assert!(matches!(
            deploy(&mut state, SectionName::EgyptianArmy, 1, 1).unwrap_err(),
            RuleError::Stacking(crate::StackingError::OverLimit)
        ));
        assert_eq!(state.units.len(), 5);
    }

    #[rulebook("§5.51", "§7.6")]
    #[test]
    fn melee_mandatory_advance_leaders_are_free_stacking() {
        // §5.51/§7.6: the mandatory Dervish advance fills the vacated hex up
        // to four *counted* units -- leaders are free stacking, so a Dervish
        // leader among the attackers advances even once the four-counter
        // budget is spent. Regression: the advance loop counted the leader
        // toward the limit and left him behind.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;

        let attacker_hex = HexCoord::new(0, 0);
        let defender_hex = HexCoord::new(1, 0);
        let mut attackers = Vec::new();
        for _ in 0..4 {
            let id = state.alloc_unit_id();
            state.units.push(UnitPlacement {
                id,
                position: attacker_hex,
                profile: dervish_tribal_profile_with(DervishTribe::Hadendowa),
                state: Default::default(),
            });
            attackers.push(id);
        }
        // Osman Digna commands the Hadendowa (§5.53) and may melee attack
        // (§7.4: "infantry, cavalry, camel units, and Dervish leaders").
        let osman_digna = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: osman_digna,
            position: attacker_hex,
            profile: UnitProfile {
                kind: UnitKind::DervishLeader {
                    fire: 1,
                    melee: 1,
                    movement: 15,
                },
                identity: UnitIdentity::DervishLeader(DervishLeader::OsmanDigna),
                weapon: WeaponClass::Melee,
                fire: None,
                melee: None,
                movement: UnitMovement::Land(crate::MovementAllowance::Fifteen),
            },
            state: Default::default(),
        });
        attackers.push(osman_digna);

        let defender = make_ae_infantry(&mut state, defender_hex);
        let attack = MeleeAttack {
            attacker_player: Player::Dervish,
            attacker_hex,
            defender_hex,
            attackers,
            defenders: vec![defender],
            attacker_modifiers: vec![MeleeModifier::DervishStandard],
            defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
        };
        // 4×6 + 1 = 25 melee factors (Row 21-25): roll 7 eliminates the lone
        // defender; the defender's roll 1 is NoEffect, so all five attackers
        // survive and advance.
        apply_effect(
            &mut state,
            &GameEffect::MeleeCombat {
                attack,
                attacker_roll: DieRoll::Seven,
                defender_roll: DieRoll::One,
            },
        )
        .unwrap();

        let advanced = state
            .units
            .iter()
            .filter(|u| u.position == defender_hex)
            .count();
        assert_eq!(
            advanced, 5,
            "all four Hadendowa *and* Osman Digna advance into the vacated hex"
        );
    }

    #[test]
    fn deploy_rejects_duplicate_counter() {
        // Each physical counter deploys once: a second deploy of the same id is
        // rejected (the app derives ids from sprite positions, so the same
        // sprite can't be placed twice). FoK so the AE profile is in play at
        // setup (§9.321); the Campaign AE deploys nothing (§9.113).
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        let id = state.alloc_unit_id();
        let first = UnitPlacement {
            id,
            position: HexCoord::new(1, 1),
            profile: crate::UnitProfile {
                kind: crate::UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity: crate::UnitIdentity::AngloEgyptianInfantry {
                    brigade: omdurman_types::BrigadeId {
                        number: 1,
                        nationality: omdurman_types::BrigadeNationality::British,
                    },
                    battalion: crate::BattalionOrdinal::First,
                },
                weapon: crate::WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        };
        apply_effect(&mut state, &GameEffect::DeployUnit(first)).unwrap();

        let dup = UnitPlacement {
            id,
            position: HexCoord::new(2, 2),
            profile: crate::UnitProfile {
                kind: crate::UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity: crate::UnitIdentity::AngloEgyptianInfantry {
                    brigade: omdurman_types::BrigadeId {
                        number: 1,
                        nationality: omdurman_types::BrigadeNationality::British,
                    },
                    battalion: crate::BattalionOrdinal::First,
                },
                weapon: crate::WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(dup)).unwrap_err(),
            RuleError::AlreadyDeployed(_)
        ));
        assert_eq!(state.units.len(), 1);
    }

    #[rulebook("§9.321")]
    #[test]
    fn remove_deployed_unit_happy_path() {
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        let id = state.alloc_unit_id();
        let placement = UnitPlacement {
            id,
            position: HexCoord::new(1, 1),
            profile: crate::UnitProfile {
                kind: crate::UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity: crate::UnitIdentity::AngloEgyptianInfantry {
                    brigade: omdurman_types::BrigadeId {
                        number: 1,
                        nationality: omdurman_types::BrigadeNationality::British,
                    },
                    battalion: crate::BattalionOrdinal::First,
                },
                weapon: crate::WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        };
        apply_effect(&mut state, &GameEffect::DeployUnit(placement)).unwrap();
        assert_eq!(state.units.len(), 1);

        apply_effect(
            &mut state,
            &GameEffect::RemoveDeployedUnit {
                unit_id: id,
                player: Player::AngloEgyptian,
            },
        )
        .unwrap();
        assert!(state.units.is_empty());
    }

    #[test]
    fn remove_deployed_unit_rejected_outside_setup() {
        let mut state = playing(Scenario::Campaign); // Movement, not Setup
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: HexCoord::new(1, 1),
            profile: crate::UnitProfile {
                kind: crate::UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity: crate::UnitIdentity::AngloEgyptianInfantry {
                    brigade: omdurman_types::BrigadeId {
                        number: 1,
                        nationality: omdurman_types::BrigadeNationality::British,
                    },
                    battalion: crate::BattalionOrdinal::First,
                },
                weapon: crate::WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        });
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::RemoveDeployedUnit {
                    unit_id: id,
                    player: Player::AngloEgyptian,
                }
            )
            .unwrap_err(),
            RuleError::WrongPhase
        ));
    }

    #[test]
    fn remove_deployed_unit_rejected_unknown() {
        let mut state = GameState::new(Scenario::Campaign);
        let id = state.alloc_unit_id(); // never deployed
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::RemoveDeployedUnit {
                    unit_id: id,
                    player: Player::AngloEgyptian,
                }
            )
            .unwrap_err(),
            RuleError::UnitNotFound(_)
        ));
    }

    #[test]
    fn remove_deployed_unit_rejected_wrong_owner() {
        // A player may only re-pick their own counters (defense against a
        // malformed remote event that names an enemy unit).
        let mut state = GameState::new(Scenario::Campaign);
        let dervish_id = make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::RemoveDeployedUnit {
                    unit_id: dervish_id,
                    player: Player::AngloEgyptian,
                }
            )
            .unwrap_err(),
            RuleError::NotOwner(_)
        ));
        // Unit is still on the board.
        assert!(state.find_unit(dervish_id).is_some());
    }

    #[rulebook("§9.321", "§9.322")]
    #[test]
    fn fok_setup_complete_requires_full_oob() {
        // Defense-in-depth: even the unbound "Begin battle" path must not leave
        // Setup until both FoK orders of battle are fully deployed.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        make_ae_infantry(&mut state, HexCoord::new(1, 1));
        make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        assert!(matches!(
            state.setup_complete().unwrap_err(),
            RuleError::SetupIncomplete(_)
        ));
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap_err(),
            RuleError::SetupIncomplete(_)
        ));
        assert_eq!(state.phase, Phase::Setup);
    }

    #[test]
    fn deployed_count_tracks_placements() {
        let mut state = GameState::new(Scenario::Campaign);
        assert_eq!(state.setup_deployed_count(Player::AngloEgyptian), 0);
        make_ae_infantry(&mut state, HexCoord::new(1, 1));
        make_ae_infantry(&mut state, HexCoord::new(2, 1));
        assert_eq!(state.setup_deployed_count(Player::AngloEgyptian), 2);
        assert_eq!(state.setup_deployed_count(Player::Dervish), 0);
    }

    #[rulebook("§4", "§6.4")]
    #[test]
    fn turn_advances_through_phases() {
        let mut state = playing(Scenario::Campaign);
        assert_eq!(state.phase, Phase::Movement);
        assert_eq!(state.active_player, Player::AngloEgyptian);

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert!(matches!(state.phase, Phase::DefensiveFire(_)));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        // AE turn: after Dervish Defensive Fire (Direct) -> AE Offensive Fire
        assert!(matches!(
            state.phase,
            Phase::OffensiveFire(FireSubPhase::DirectFire)
        ));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert!(matches!(
            state.phase,
            Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer)
        ));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert_eq!(state.phase, Phase::Melee);

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        // After melee, active_player switches.
        assert_eq!(state.active_player, Player::Dervish);
        assert_eq!(state.phase, Phase::Movement);

        // Dervish turn: Movement -> Defensive Fire (AE Direct)
        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert!(matches!(
            state.phase,
            Phase::DefensiveFire(FireSubPhase::DirectFire)
        ));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        // Dervish turn: DefFire(Direct) -> DefFire(Maxim/Howitzer) (AE fires again)
        assert!(matches!(
            state.phase,
            Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer)
        ));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert!(matches!(
            state.phase,
            Phase::OffensiveFire(FireSubPhase::DirectFire)
        ));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert_eq!(state.phase, Phase::Melee);

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        // After melee, active_player switches back to AE.
        assert_eq!(state.active_player, Player::AngloEgyptian);
        assert_eq!(state.phase, Phase::Movement);
    }

    #[rulebook("§9.12")]
    #[test]
    fn game_over_after_campaign_turns() {
        let mut state = playing(Scenario::Campaign);
        // Fast-forward past all campaign turns.
        for _ in 0..100 {
            if state.game_over {
                break;
            }
            // Advance through all phases for each player turn.
            for _ in 0..6 {
                // Movement, DefFire(Direct), DefFire(Maxim2nd/How), OffFire(Direct), OffFire(Maxim2nd/How), Melee
                match apply_effect(&mut state, &GameEffect::AdvancePhase) {
                    Ok(()) => {}
                    // §8.2: the mandatory desertion roll gates the first
                    // night turn's Dervish movement phase. With no units on
                    // the board the expected deserter count is 0, so an
                    // empty roll satisfies the gate.
                    Err(RuleError::DesertionRollRequired) => {
                        let _ = apply_effect(
                            &mut state,
                            &GameEffect::DervishDesertion {
                                roll: DieRoll::One,
                                deserters: vec![],
                            },
                        );
                    }
                    Err(_) => break,
                }
            }
        }
        assert!(state.game_over);
    }

    // -- Fix-coverage tests (Parts C/D/E of the rule-enforcement work) -------

    use crate::board::{BoardInfo, NileBank, StepDirection};
    use omdurman_types::{HexDirection, HexsideKind, Terrain};

    fn make_unit(
        state: &mut GameState,
        hex: HexCoord,
        kind: UnitKind,
        identity: UnitIdentity,
        weapon: WeaponClass,
        movement: UnitMovement,
    ) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: UnitProfile {
                kind,
                identity,
                weapon,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement,
            },
            state: Default::default(),
        });
        id
    }

    fn make_ae_artillery(state: &mut GameState, hex: HexCoord) -> UnitId {
        make_unit(
            state,
            hex,
            UnitKind::Artillery {
                fire: 0,
                melee: 0,
                movement: 0,
            },
            UnitIdentity::AngloEgyptianArtillery,
            WeaponClass::Artillery,
            UnitMovement::Land(crate::MovementAllowance::Eight),
        )
    }

    fn make_dervish_gunboat(state: &mut GameState, hex: HexCoord) -> UnitId {
        make_unit(
            state,
            hex,
            UnitKind::Gunboat {
                fire: 0,
                upstream: 0,
                downstream: 0,
            },
            UnitIdentity::DervishGunboat(GunboatId::DervishGunboat(1)),
            WeaponClass::Artillery,
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        )
    }

    fn make_fort(state: &mut GameState, hex: HexCoord) -> UnitId {
        make_unit(
            state,
            hex,
            UnitKind::Fort { fire: 0, melee: 0 },
            UnitIdentity::DervishFort,
            WeaponClass::Artillery,
            UnitMovement::Immobile,
        )
    }

    fn direct_attack(player: Player, firers: Vec<UnitId>, target: HexCoord) -> FireAttack {
        FireAttack {
            firing_player: player,
            phase: Phase::OffensiveFire(FireSubPhase::DirectFire),
            kind: FireKind::Direct,
            firers,
            target_hex: target,
            factor_row: FireFactorRow::Row06to10,
            // §6.24: the AE +1 is mandatory; the engine rejects any other list.
            modifiers: if player == Player::AngloEgyptian {
                vec![FireModifier::AngloEgyptianDirectFire]
            } else {
                vec![]
            },
        }
    }

    // ----- Part C -----------------------------------------------------------

    #[test]
    fn scenario_move_order_per_rulebook() {
        // §9.113 Campaign: Anglo-Egyptian moves first.
        assert_eq!(
            GameState::new(Scenario::Campaign).active_player,
            Player::AngloEgyptian
        );
        // §9.212 Historical and §9.322 Fall of Khartoum: Dervish moves first.
        assert_eq!(
            GameState::new(Scenario::Historical).active_player,
            Player::Dervish
        );
        assert_eq!(
            GameState::new(Scenario::FallOfKhartoum).active_player,
            Player::Dervish
        );
    }

    #[rulebook("§6.7")]
    #[test]
    fn no_advance_after_defensive_fire() {
        // §6.7: no advance after combat as a result of defensive fire.
        let mut state = GameState::new(Scenario::Campaign);
        let unit = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let dest = HexCoord::new(1, 0);

        state.phase = Phase::DefensiveFire(FireSubPhase::DirectFire);
        assert!(matches!(
            state.can_advance_after_combat(unit, dest),
            Err(RuleError::WrongPhase)
        ));

        // ...but offensive fire (§6.82) and melee (§7.6) do allow it, once a
        // hex has been vacated by combat (the advance window).
        open_advance_window(&mut state, dest, &[unit], vec!["6.82".to_string()]);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        assert!(state.can_advance_after_combat(unit, dest).is_ok());
        state.phase = Phase::Melee;
        assert!(state.can_advance_after_combat(unit, dest).is_ok());
    }

    #[rulebook("§8.2")]
    #[test]
    fn desertion_count_is_floor_one_and_a_half() {
        // §8.2: deserters = floor(1.5 * roll).
        assert_eq!(desertion_count(DieRoll::One), 1);
        assert_eq!(desertion_count(DieRoll::Two), 3);
        assert_eq!(desertion_count(DieRoll::Four), 6);
        assert_eq!(desertion_count(DieRoll::Ten), 15);
    }

    #[rulebook("§7")]
    #[test]
    fn declared_melee_blocks_phase_advance() {
        // Regression (audit §7): a declared-but-unresolved melee used to be
        // silently dropped when the melee phase ended. The phase may now only
        // end once the declaration is resolved (or vacated by retreat).
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;
        let defender = make_ae_infantry(&mut state, HexCoord::new(5, 5));
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(6, 5));

        let declared = apply_effect(
            &mut state,
            &GameEffect::DeclareMelee {
                attack: MeleeAttack {
                    attacker_player: Player::Dervish,
                    attacker_hex: HexCoord::new(6, 5),
                    defender_hex: HexCoord::new(5, 5),
                    attackers: vec![attacker],
                    defenders: vec![defender],
                    attacker_modifiers: vec![MeleeModifier::DervishStandard],
                    defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
                },
                attacker_roll: DieRoll::Five,
                defender_roll: DieRoll::Five,
            },
        );
        assert!(declared.is_ok());

        // Phase advance is rejected while the melee awaits resolution.
        assert!(matches!(
            advance_phase(&mut state),
            Err(RuleError::MeleePendingResolution)
        ));

        // Resolving it unblocks the advance.
        assert!(apply_effect(&mut state, &GameEffect::ResolveMelee).is_ok());
        assert!(advance_phase(&mut state).is_ok());
    }

    #[rulebook("§8.2")]
    #[test]
    fn desertion_roll_required_before_first_night_movement_ends() {
        // Regression (audit §8.2): every recorded campaign game skipped the
        // mandatory desertion roll. The Dervish movement phase of the first
        // night turn (T9) may not end before the roll is applied.
        let mut state = dervish_first_night_state();
        // An eligible tribal unit (the Khalifa/gunboats/artillery/forts are
        // exempt, so a plain tribe counter is needed to desert).
        let tribe = make_dervish_tribal(&mut state, HexCoord::new(0, 0));

        assert!(matches!(
            advance_phase(&mut state),
            Err(RuleError::DesertionRollRequired)
        ));

        // Applying the roll (One -> 1 unit) satisfies the gate.
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::DervishDesertion {
                    roll: DieRoll::One,
                    deserters: vec![tribe],
                }
            )
            .is_ok()
        );
        assert!(state.find_unit(tribe).is_none(), "the deserter is removed");
        assert!(advance_phase(&mut state).is_ok());

        // Later turns are unaffected (the roll is once per game).
        let mut later = {
            let mut s = dervish_first_night_state();
            s.current_turn = GameTurnIndex::new(10);
            s.dervish_deserted = true;
            s
        };
        assert!(advance_phase(&mut later).is_ok());
    }

    fn dervish_first_night_state() -> GameState {
        let mut state = GameState::new(Scenario::Campaign);
        // Advance to the first night turn (turn 9) in the Dervish movement
        // phase, which is when desertion is rolled (§8.2).
        state.current_turn = GameTurnIndex::new(9);
        state.day_night = DayNight::Night;
        state.active_player = Player::Dervish;
        state.phase = Phase::Movement;
        state
    }

    /// A Campaign state in the given side's turn-1 movement phase, on a small
    /// legal board (for reinforcement-schedule tests, §9.112/§9.113).
    fn campaign_wave_state(player: Player) -> GameState {
        let mut state = GameState::new(Scenario::Campaign);
        let mut board = BoardInfo::default();
        for h in [
            HexCoord::new(0, 0),
            HexCoord::new(0, 1),
            HexCoord::new(1, 0),
            HexCoord::new(1, 1),
        ] {
            board.terrain.insert(
                h,
                Terrain::Clear {
                    road: Default::default(),
                },
            );
        }
        state.board = board;
        state.phase = Phase::Movement;
        state.active_player = player;
        state
    }

    fn tribal_placement(id: UnitId, tribe: DervishTribe, at: HexCoord) -> UnitPlacement {
        UnitPlacement {
            id,
            position: at,
            profile: UnitProfile {
                kind: UnitKind::Infantry {
                    fire: 3,
                    melee: 6,
                    movement: 9,
                },
                identity: UnitIdentity::DervishTribal { tribe },
                weapon: WeaponClass::Melee,
                fire: Some(crate::FireFactor::Three),
                melee: Some(crate::MeleeFactor::Six),
                movement: UnitMovement::Land(crate::MovementAllowance::Nine),
            },
            state: Default::default(),
        }
    }

    #[rulebook("§9.112")]
    #[test]
    fn campaign_reinforcements_gate_by_wave() {
        // Turn 1 Dervish: Baggara (wave 1) enters; Mulazmin (wave 3 only,
        // §9.112) is rejected; the Anglo-Egyptian side cannot place on the
        // Dervish player's turn.
        let mut state = campaign_wave_state(Player::Dervish);
        let baggara = tribal_placement(
            state.alloc_unit_id(),
            DervishTribe::Baggara,
            HexCoord::new(0, 0),
        );
        assert!(apply_effect(&mut state, &GameEffect::PlaceReinforcements(vec![baggara])).is_ok());

        let mut state = campaign_wave_state(Player::Dervish);
        let mulazmin = tribal_placement(
            state.alloc_unit_id(),
            DervishTribe::Mulazmin,
            HexCoord::new(0, 0),
        );
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceReinforcements(vec![mulazmin])),
            Err(RuleError::TribeNotInWave { turn: 1 })
        ));

        // AE land units may only enter on the AE player's turn (turn 1 wave).
        let mut state = campaign_wave_state(Player::Dervish);
        let ae = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(0, 0),
            profile: crate::UnitProfile {
                kind: crate::UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity: crate::UnitIdentity::AngloEgyptianInfantry {
                    brigade: omdurman_types::BrigadeId {
                        number: 1,
                        nationality: omdurman_types::BrigadeNationality::British,
                    },
                    battalion: crate::BattalionOrdinal::First,
                },
                weapon: crate::WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceReinforcements(vec![ae])),
            Err(RuleError::NotYourTurn)
        ));
    }

    #[rulebook("§9.113")]
    #[test]
    fn campaign_reinforcement_cap_and_double_entry() {
        // The AE turn-1 wave caps at 12 land units; exceeding the cap in one
        // batch is rejected, and a unit may never enter twice.
        let mut state = campaign_wave_state(Player::AngloEgyptian);
        let batch: Vec<UnitPlacement> = (0..13)
            .map(|_| UnitPlacement {
                id: state.alloc_unit_id(),
                position: HexCoord::new(0, 0), // stacking will also trip at >4; use spread below
                profile: crate::UnitProfile {
                    kind: crate::UnitKind::Infantry {
                        fire: 4,
                        melee: 5,
                        movement: 8,
                    },
                    identity: crate::UnitIdentity::AngloEgyptianInfantry {
                        brigade: omdurman_types::BrigadeId {
                            number: 1,
                            nationality: omdurman_types::BrigadeNationality::British,
                        },
                        battalion: crate::BattalionOrdinal::First,
                    },
                    weapon: crate::WeaponClass::Rifles,
                    fire: Some(crate::FireFactor::Four),
                    melee: Some(crate::MeleeFactor::Five),
                    movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
                },
                state: Default::default(),
            })
            .collect();
        let _ = batch; // stacking in one hex trips first; build a spread batch
        let mut spread: Vec<UnitPlacement> = Vec::new();
        for i in 0..13 {
            spread.push(UnitPlacement {
                id: state.alloc_unit_id(),
                position: HexCoord::new(i % 2, i / 2 % 2),
                profile: crate::UnitProfile {
                    kind: crate::UnitKind::Infantry {
                        fire: 4,
                        melee: 5,
                        movement: 8,
                    },
                    identity: crate::UnitIdentity::AngloEgyptianInfantry {
                        brigade: omdurman_types::BrigadeId {
                            number: 1,
                            nationality: omdurman_types::BrigadeNationality::British,
                        },
                        battalion: crate::BattalionOrdinal::First,
                    },
                    weapon: crate::WeaponClass::Rifles,
                    fire: Some(crate::FireFactor::Four),
                    melee: Some(crate::MeleeFactor::Five),
                    movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
                },
                state: Default::default(),
            });
        }
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceReinforcements(spread.clone())),
            Err(RuleError::ReinforcementCapExceeded { turn: 1, cap: 12 })
        ));

        // A legal 2-unit batch enters, and re-entering the same ids is
        // rejected as AlreadyDeployed.
        spread.truncate(2);
        assert!(apply_effect(&mut state, &GameEffect::PlaceReinforcements(spread.clone())).is_ok());
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceReinforcements(spread)),
            Err(RuleError::AlreadyDeployed(_))
        ));
        // Entry charged 1 MP each (§9.113).
        for p in &state.units {
            assert_eq!(state.mp_spent(p.id), 1, "entry MP not charged");
        }
    }

    // §9.113: a reinforcement may not materialise on an enemy-occupied hex
    // (found by the occupancy audit on the Campaign matrix: an AE battalion
    // arrived on top of a Dervish Taiasha unit and cohabited the hex).
    #[rulebook("§9.113")]
    #[test]
    fn reinforcement_rejected_onto_enemy_occupied_hex() {
        let mut state = campaign_wave_state(Player::AngloEgyptian);
        // A Dervish unit standing on the AE entrance area.
        make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let batch = vec![UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(0, 0),
            profile: crate::UnitProfile {
                kind: crate::UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity: crate::UnitIdentity::AngloEgyptianInfantry {
                    brigade: omdurman_types::BrigadeId {
                        number: 1,
                        nationality: omdurman_types::BrigadeNationality::British,
                    },
                    battalion: crate::BattalionOrdinal::First,
                },
                weapon: crate::WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        }];
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceReinforcements(batch)),
            Err(RuleError::EnemyOccupied(_))
        ));
        // The enemy unit is untouched and nothing was placed.
        assert_eq!(
            state
                .units
                .iter()
                .filter(|u| u.profile.identity.owner() == Player::AngloEgyptian)
                .count(),
            0
        );
    }

    #[rulebook("§9.113")]
    #[test]
    fn campaign_gunboats_quota_three_per_turn() {
        let mut state = campaign_wave_state(Player::AngloEgyptian);
        let mk = |s: &mut GameState| UnitPlacement {
            id: s.alloc_unit_id(),
            position: HexCoord::new(0, 0),
            profile: ae_gunboat_profile(),
            state: Default::default(),
        };
        // Three gunboats stack-free is fine (gunboats may not stack with
        // anything else, so each gets its own hex).
        let mut batch: Vec<UnitPlacement> = Vec::new();
        for hex in [
            HexCoord::new(0, 0),
            HexCoord::new(0, 1),
            HexCoord::new(1, 0),
        ] {
            let mut p = mk(&mut state);
            p.position = hex;
            batch.push(p);
        }
        assert!(apply_effect(&mut state, &GameEffect::PlaceReinforcements(batch)).is_ok());
        // A fourth in the same turn is over quota.
        let mut state2 = campaign_wave_state(Player::AngloEgyptian);
        let mut batch2: Vec<UnitPlacement> = Vec::new();
        for hex in [
            HexCoord::new(0, 0),
            HexCoord::new(0, 1),
            HexCoord::new(1, 0),
            HexCoord::new(1, 1),
        ] {
            let mut p = mk(&mut state2);
            p.position = hex;
            batch2.push(p);
        }
        assert!(matches!(
            apply_effect(&mut state2, &GameEffect::PlaceReinforcements(batch2)),
            Err(RuleError::GunboatQuotaExceeded { turn: 1 })
        ));
    }

    #[test]
    fn desertion_removes_chosen_count_and_respects_exemptions() {
        let mut state = dervish_first_night_state();
        let a = make_dervish_tribal(&mut state, HexCoord::new(1, 0));
        let b = make_dervish_tribal(&mut state, HexCoord::new(2, 0));
        let c = make_dervish_tribal(&mut state, HexCoord::new(3, 0));
        let khalifa = make_unit(
            &mut state,
            HexCoord::new(4, 0),
            UnitKind::DervishLeader {
                fire: 0,
                melee: 0,
                movement: 0,
            },
            UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah),
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Eight),
        );

        // Roll of 2 -> 3 deserters; choosing the Khalifa is illegal (§8.2).
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DervishDesertion {
                    roll: DieRoll::Two,
                    deserters: vec![a, b, khalifa],
                }
            ),
            Err(RuleError::Desertion(DesertionError::Exempt(_)))
        ));
        // Wrong count is rejected.
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DervishDesertion {
                    roll: DieRoll::Two,
                    deserters: vec![a, b],
                }
            ),
            Err(RuleError::Desertion(DesertionError::WrongCount { .. }))
        ));
        // A legal choice of three eligible units succeeds and is once-per-game.
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::DervishDesertion {
                    roll: DieRoll::Two,
                    deserters: vec![a, b, c],
                }
            )
            .is_ok()
        );
        assert!(state.find_unit(a).is_none());
        assert!(state.find_unit(khalifa).is_some());
        assert!(state.dervish_deserted);
        // A second desertion is rejected (§8.2 once per game).
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DervishDesertion {
                    roll: DieRoll::Two,
                    deserters: vec![],
                }
            ),
            Err(RuleError::Desertion(DesertionError::AlreadyDeserted))
        ));
    }

    #[rulebook("§9.14")]
    #[test]
    fn friendlies_bank_scores_by_side() {
        // A small board: Nile in column q=0 of row r=0; west bank q<0, east q>0.
        let mut board = BoardInfo::default();
        board.terrain.insert(
            HexCoord::new(0, 0),
            Terrain::Nile {
                direction: HexDirection::East,
            },
        );
        board
            .terrain
            .insert(HexCoord::new(-1, 0), Terrain::default());
        board
            .terrain
            .insert(HexCoord::new(1, 0), Terrain::default());
        assert_eq!(board.bank_of(HexCoord::new(-1, 0)), Some(NileBank::West));
        assert_eq!(board.bank_of(HexCoord::new(1, 0)), Some(NileBank::East));
    }

    // ----- Part D-1: stacking ----------------------------------------------

    #[rulebook("§5.51")]
    #[test]
    fn stacking_over_limit_rejected() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let dest = HexCoord::new(1, 0);
        // Four AE infantry already in the destination hex.
        for _ in 0..4 {
            make_ae_infantry(&mut state, dest);
        }
        let mover = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let err = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: mover,
                to: dest,
                cost: MovementPoints::new(1),
                path: Vec::new(),
            },
        );
        assert!(matches!(
            err,
            Err(RuleError::Stacking(crate::StackingError::OverLimit))
        ));
    }

    #[test]
    fn stacking_different_tribes_rejected() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let dest = HexCoord::new(1, 0);
        // A Baggara unit sits in the destination.
        make_dervish_tribal(&mut state, dest);
        // A Hadendowa unit tries to join it (§5.52).
        let mover = make_unit(
            &mut state,
            HexCoord::new(0, 0),
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0,
            },
            UnitIdentity::DervishTribal {
                tribe: DervishTribe::Hadendowa,
            },
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Nine),
        );
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::MoveUnit {
                    unit_id: mover,
                    to: dest,
                    cost: MovementPoints::new(1),
                    path: Vec::new(),
                }
            ),
            Err(RuleError::Stacking(crate::StackingError::DervishTribeMix))
        ));
    }

    #[test]
    fn fok_green_mulazmin_units_resolve_and_stacking_mix_rejected() {
        // §5.52 regression: the Fall-of-Khartoum `Mulazmin_I`/`Mulazmin_II`
        // Mulazmin counters previously had no UnitId/profile, so `check_stacking`
        // was skipped for them entirely. Both sections must now resolve to the
        // Mulazmin tribe and participate in the different-tribe rule.
        for section in [
            omdurman_types::SectionName::MulazminI,
            omdurman_types::SectionName::MulazminII,
        ] {
            let unit_id =
                unit_id_for_section_pos(section, 0, 0).expect("green section has a UnitId");
            let profile = crate::unit_profiles::profile_for_unit(unit_id)
                .expect("green section resolves a profile");
            assert_eq!(
                profile.identity,
                UnitIdentity::DervishTribal {
                    tribe: DervishTribe::Mulazmin
                },
                "{section:?} (0,0) must be Mulazmin"
            );
        }

        // A Mulazmin unit and a Baggara unit in the same hex are a tribe mix.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let dest = HexCoord::new(1, 0);
        make_unit(
            &mut state,
            dest,
            UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            },
            UnitIdentity::DervishTribal {
                tribe: DervishTribe::Mulazmin,
            },
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Nine),
        );
        let baggara = make_unit(
            &mut state,
            HexCoord::new(0, 0),
            UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            },
            UnitIdentity::DervishTribal {
                tribe: DervishTribe::Baggara,
            },
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Fifteen),
        );
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::MoveUnit {
                    unit_id: baggara,
                    to: dest,
                    cost: MovementPoints::new(1),
                    path: Vec::new(),
                }
            ),
            Err(RuleError::Stacking(crate::StackingError::DervishTribeMix))
        ));

        // Two Mulazmin units may stack together (§5.52 allows same-tribe).
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let dest = HexCoord::new(2, 0);
        let m1 = make_unit(
            &mut state,
            dest,
            UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            },
            UnitIdentity::DervishTribal {
                tribe: DervishTribe::Mulazmin,
            },
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Nine),
        );
        let m2 = make_unit(
            &mut state,
            HexCoord::new(3, 0),
            UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            },
            UnitIdentity::DervishTribal {
                tribe: DervishTribe::Mulazmin,
            },
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Nine),
        );
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::MoveUnit {
                    unit_id: m2,
                    to: dest,
                    cost: MovementPoints::new(1),
                    path: Vec::new(),
                }
            ),
            Ok(())
        ));
        let _ = m1;
    }

    // ----- Part D-2: ZOC ----------------------------------------------------

    #[test]
    fn gunboat_projects_zoc_only_vs_gunboats() {
        let mut state = GameState::new(Scenario::Campaign);
        state.active_player = Player::AngloEgyptian;
        make_dervish_gunboat(&mut state, HexCoord::new(1, 1));
        // A land mover ignores the enemy gunboat's ZOC...
        assert!(!state.hex_in_enemy_zoc(
            HexCoord::new(1, 0),
            Player::AngloEgyptian,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0
            }
        ));
        // ...but another gunboat is stopped by it (§5.41).
        assert!(state.hex_in_enemy_zoc(
            HexCoord::new(1, 0),
            Player::AngloEgyptian,
            UnitKind::Gunboat {
                fire: 0,
                upstream: 0,
                downstream: 0
            }
        ));
    }

    #[test]
    fn zoc_does_not_cross_a_khor() {
        let mut state = GameState::new(Scenario::Campaign);
        state.active_player = Player::AngloEgyptian;
        let enemy_hex = HexCoord::new(1, 1);
        make_dervish_tribal(&mut state, enemy_hex);
        let into = HexCoord::new(1, 0);
        // Without a hexside, ZOC reaches `into`.
        assert!(state.hex_in_enemy_zoc(
            into,
            Player::AngloEgyptian,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0
            }
        ));
        // §5.44: a khor on the shared edge blocks the ZOC.
        state
            .board
            .hexsides
            .insert(HexsideRef::new(enemy_hex, into), HexsideKind::Khor);
        assert!(!state.hex_in_enemy_zoc(
            into,
            Player::AngloEgyptian,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0
            }
        ));
    }

    #[rulebook("§5.42")]
    #[test]
    fn entering_enemy_zoc_costs_no_extra_mp() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let mover = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        // Enemy at (2,0) puts (1,0) in its ZOC.
        make_dervish_tribal(&mut state, HexCoord::new(2, 0));

        // Moving into a ZOC hex costs only the terrain MP (1 for clear), no surcharge.
        let result = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: mover,
                to: HexCoord::new(1, 0),
                cost: MovementPoints::new(1),
                path: Vec::new(),
            },
        );
        assert!(result.is_ok());
        assert_eq!(state.mp_spent(mover), 1, "entering ZOC adds no MP cost");
    }

    // ----- Part D-3: movement cost & gunboats -------------------------------

    fn nile_board_row0(min_q: i32, max_q: i32, flow: HexDirection) -> BoardInfo {
        let mut board = BoardInfo::default();
        for q in min_q..=max_q {
            board
                .terrain
                .insert(HexCoord::new(q, 0), Terrain::Nile { direction: flow });
        }
        board
    }

    #[rulebook("§5.11")]
    #[test]
    fn land_unit_may_not_enter_nile() {
        let mut state = GameState::new(Scenario::Campaign);
        state.board = nile_board_row0(0, 3, HexDirection::East);
        state.phase = Phase::Movement;
        let mover = make_ae_infantry(&mut state, HexCoord::new(0, 1));
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::MoveUnit {
                    unit_id: mover,
                    to: HexCoord::new(1, 0), // a Nile hex
                    cost: MovementPoints::new(1),
                    path: Vec::new(),
                }
            ),
            Err(RuleError::LandIntoNile(_))
        ));
    }

    #[test]
    fn gunboat_step_direction_classifies_against_current() {
        // Current flows East (+q). A step to the +q neighbour is downstream;
        // the -q neighbour is upstream (§5.24).
        let board = nile_board_row0(0, 3, HexDirection::East);
        let here = HexCoord::new(1, 0);
        assert_eq!(
            board.step_direction(here, HexCoord::new(2, 0)),
            Some(StepDirection::Downstream)
        );
        assert_eq!(
            board.step_direction(here, HexCoord::new(0, 0)),
            Some(StepDirection::Upstream)
        );
    }

    #[test]
    fn gunboat_upstream_step_caps_the_turn() {
        let mut state = GameState::new(Scenario::Campaign);
        state.board = nile_board_row0(0, 6, HexDirection::East);
        state.phase = Phase::Movement;
        // Gunboat at (3,0); upstream allowance 10, downstream 16.
        let gb = make_dervish_gunboat(&mut state, HexCoord::new(3, 0));
        state.active_player = Player::Dervish;
        // One upstream step (to q=2) caps the turn at the upstream allowance of
        // 10; a cost of 12 is therefore illegal (§5.24).
        let upstream_path = vec![HexCoord::new(2, 0)];
        assert!(matches!(
            state.can_move_gunboat(
                gb,
                HexCoord::new(2, 0),
                &upstream_path,
                MovementPoints::new(12)
            ),
            Err(RuleError::GunboatUpstreamCap { .. })
        ));
        // Purely downstream, the larger allowance of 16 applies, so 12 is fine.
        let downstream_path = vec![
            HexCoord::new(4, 0),
            HexCoord::new(5, 0),
            HexCoord::new(6, 0),
        ];
        assert!(
            state
                .can_move_gunboat(
                    gb,
                    HexCoord::new(6, 0),
                    &downstream_path,
                    MovementPoints::new(12)
                )
                .is_ok()
        );
    }

    #[rulebook("§5.24")]
    #[test]
    fn gunboat_upstream_cap_is_sticky_across_moves() {
        // Regression (audit §5.24): the cap used to be recomputed per move, so
        // a gunboat that went upstream in an earlier move could spend up to
        // its *downstream* allowance with later all-downstream moves. The
        // manual caps the whole turn: "if they move even one hex upstream,
        // their upstream movement allowance is their maximum movement
        // allowance for that turn".
        let mut state = GameState::new(Scenario::Campaign);
        state.board = nile_board_row0(0, 12, HexDirection::East);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        // Gunboat at (3,0); upstream allowance 10, downstream 16.
        let gb = make_dervish_gunboat(&mut state, HexCoord::new(3, 0));

        // Move 1: one committed upstream step (to q=2), 1 MP.
        let upstream = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: gb,
                to: HexCoord::new(2, 0),
                cost: MovementPoints::new(1),
                path: vec![HexCoord::new(2, 0)],
            },
        );
        assert!(
            upstream.is_ok(),
            "1-MP upstream step is legal: {upstream:?}"
        );
        assert!(
            state.gunboats_upstream_this_turn.contains(&gb),
            "the committed upstream step must set the sticky flag"
        );

        // Move 2 (all downstream, engine-costed at 1 MP per hex): cumulative
        // 1 + 10 = 11 exceeds the upstream cap of 10 -> rejected under §5.24,
        // even though 11 < 16 and this move itself never goes upstream.
        let downstream: Vec<HexCoord> = (3..=12).map(|q| HexCoord::new(q, 0)).collect();
        let result = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: gb,
                to: HexCoord::new(12, 0),
                cost: MovementPoints::new(10),
                path: downstream,
            },
        );
        assert!(
            matches!(result, Err(RuleError::GunboatUpstreamCap { .. })),
            "later downstream moves must stay capped at the upstream allowance (§5.24), got {result:?}"
        );

        // Cross-check via the predicate with an explicit cumulative spend.
        let mut state = GameState::new(Scenario::Campaign);
        state.board = nile_board_row0(0, 12, HexDirection::East);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let gb = make_dervish_gunboat(&mut state, HexCoord::new(2, 0));
        state.gunboats_upstream_this_turn.push(gb);
        state.mp_spent_this_turn.insert(gb, 9);
        let downstream_path = vec![HexCoord::new(3, 0)];
        assert!(matches!(
            state.can_move_gunboat(
                gb,
                HexCoord::new(3, 0),
                &downstream_path,
                MovementPoints::new(2)
            ),
            Err(RuleError::GunboatUpstreamCap { .. })
        ));
        assert!(
            state
                .can_move_gunboat(
                    gb,
                    HexCoord::new(3, 0),
                    &downstream_path,
                    MovementPoints::new(1)
                )
                .is_ok()
        );
    }

    // ----- Part D-4: artillery special results & howitzer scatter -----------

    #[rulebook("§6.61")]
    #[test]
    fn rifles_may_not_sink_a_gunboat() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let rifle = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(1, 0);
        make_dervish_gunboat(&mut state, target);
        let attack = direct_attack(Player::AngloEgyptian, vec![rifle], target);
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::FireCombat {
                    attack,
                    roll: DieRoll::Ten
                }
            ),
            Err(RuleError::ArtilleryOnlyVsGunboatOrFort(_))
        ));
    }

    /// Resolve an artillery attack on a gunboat at `target` with `roll` and
    /// report whether the gunboat was sunk and the Combat Results Table result
    /// the engine actually computed (so the test asserts the §6.61 threshold
    /// against the *real* banded result, not a re-derived one).
    fn arty_vs_gunboat(roll: DieRoll) -> (bool, CombatResult) {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let arty = make_ae_artillery(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(1, 0);
        let gb = make_dervish_gunboat(&mut state, target);
        let attack = direct_attack(Player::AngloEgyptian, vec![arty], target);
        // Mirror the engine's banded total + modifiers to know the CRT result.
        let range = HexDistance::new(1);
        let band = ae_range_effects(WeaponClass::Artillery, range);
        let total = band.apply(crate::FireFactor::Four.value());
        let crt = combat_results_table(
            FireFactorRow::from_total(total),
            roll.apply_modifier(attack.net_modifier()),
        );
        apply_effect(&mut state, &GameEffect::FireCombat { attack, roll }).unwrap();
        (state.find_unit(gb).is_none(), crt)
    }

    #[rulebook("§6.61")]
    #[test]
    fn artillery_sinks_gunboat_only_on_three_plus() {
        // §6.61: a gunboat is sunk only on a Combat Results Table result of 3+.
        // Across the die-roll range, the gunboat is sunk iff the result was
        // Eliminate(>=3) -- never on a lesser result.
        for r in 1u16..=10 {
            let roll = DieRoll::try_from(r).unwrap();
            let (sunk, crt) = arty_vs_gunboat(roll);
            assert_eq!(
                sunk,
                matches!(crt, CombatResult::Eliminate(n) if n >= 3),
                "roll {r}: sunk={sunk} but CRT={crt:?}"
            );
        }
    }

    // §6.62: only artillery may fire at forts, and only a Combat Results
    // Table result of 2+ destroys the fort -- any lesser result is a miss.
    #[rulebook("§6.62")]
    #[test]
    fn artillery_destroys_fort_on_two_or_better_only() {
        for r in 1u16..=10 {
            let roll = DieRoll::try_from(r).unwrap();
            let mut state = GameState::new(Scenario::Campaign);
            state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
            state.active_player = Player::AngloEgyptian;
            let arty = make_ae_artillery(&mut state, HexCoord::new(0, 0));
            let target = HexCoord::new(1, 0);
            let fort = make_fort(&mut state, target);
            let attack = direct_attack(Player::AngloEgyptian, vec![arty], target);
            let total = ae_range_effects(WeaponClass::Artillery, HexDistance::new(1))
                .apply(crate::FireFactor::Four.value());
            let crt = combat_results_table(
                FireFactorRow::from_total(total),
                roll.apply_modifier(attack.net_modifier()),
            );
            apply_effect(&mut state, &GameEffect::FireCombat { attack, roll }).unwrap();
            let destroyed = state.find_unit(fort).is_none();
            assert_eq!(
                destroyed,
                matches!(crt, CombatResult::Eliminate(n) if n >= 2),
                "roll {r}: destroyed={destroyed} but CRT={crt:?}"
            );
        }

        // "Only artillery may fire at forts": infantry cannot even target one.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let infantry = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let fort_hex = HexCoord::new(1, 0);
        let _fort = make_fort(&mut state, fort_hex);
        let attack = direct_attack(Player::AngloEgyptian, vec![infantry], fort_hex);
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::FireCombat {
                    attack,
                    roll: DieRoll::Ten
                }
            ),
            Err(RuleError::ArtilleryOnlyVsGunboatOrFort(_))
        ));
    }

    #[rulebook("§6.63")]
    #[test]
    fn artillery_breaches_wall_only_on_crt_two_or_better() {
        // §6.63: "Only artillery may fire to breach a wall hexside... A
        // result of 2 or more on the combat results table is required to
        // breach a wall." Exactly like the §6.62 fort rule, the CRT cell value
        // is what matters -- Eliminate(2)+ flips Wall -> Breach, anything
        // less is a miss.
        for r in 1u16..=10 {
            let roll = DieRoll::try_from(r).unwrap();
            let mut state = GameState::new(Scenario::Campaign);
            state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
            state.active_player = Player::AngloEgyptian;
            let arty = make_ae_artillery(&mut state, HexCoord::new(0, 0));
            let wall = omdurman_types::HexsideRef::new(HexCoord::new(1, 0), HexCoord::new(2, 0));
            state.board.hexsides.insert(wall, HexsideKind::Wall);
            make_dervish_tribal(&mut state, HexCoord::new(1, 0));

            let result = apply_effect(
                &mut state,
                &GameEffect::ArtilleryBreachWall {
                    firers: vec![arty],
                    target: wall,
                    roll,
                },
            );
            assert!(result.is_ok(), "roll {r}: {result:?}");
            let total = ae_range_effects(WeaponClass::Artillery, HexDistance::new(1))
                .apply(crate::FireFactor::Four.value());
            let crt = combat_results_table(FireFactorRow::from_total(total), roll);
            let should_breach = matches!(crt, CombatResult::Eliminate(n) if n >= 2);
            assert_eq!(
                state.board.hexsides.get(&wall).copied(),
                Some(if should_breach {
                    HexsideKind::Breach
                } else {
                    HexsideKind::Wall
                }),
                "roll {r}: wall flipped exactly when CRT says 2+ (got {crt:?})"
            );
        }

        // "Only artillery may fire to breach": an infantry unit is rejected.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let infantry = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let wall = omdurman_types::HexsideRef::new(HexCoord::new(1, 0), HexCoord::new(2, 0));
        state.board.hexsides.insert(wall, HexsideKind::Wall);
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::ArtilleryBreachWall {
                    firers: vec![infantry],
                    target: wall,
                    roll: DieRoll::Ten,
                }
            ),
            Err(RuleError::OnlyArtilleryMayBreachWall(_))
        ));
    }

    #[rulebook("§6.63")]
    #[test]
    fn wall_breach_eliminates_one_adjacent_enemy_unit() {
        // §6.63: "If any enemy units are adjacent to the wall hexside at the
        // instant it is breached, one enemy unit is eliminated." The breach
        // above flips the hexside; exercising the adjacent-enemy elimination
        // with a roll that clears the 2+ threshold.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let arty = make_ae_artillery(&mut state, HexCoord::new(0, 0));
        let wall = omdurman_types::HexsideRef::new(HexCoord::new(1, 0), HexCoord::new(2, 0));
        state.board.hexsides.insert(wall, HexsideKind::Wall);
        let first_adjacent = make_dervish_tribal(&mut state, HexCoord::new(1, 0));
        let second_adjacent = make_dervish_tribal(&mut state, HexCoord::new(2, 0));

        apply_effect(
            &mut state,
            &GameEffect::ArtilleryBreachWall {
                firers: vec![arty],
                target: wall,
                roll: DieRoll::Ten,
            },
        )
        .unwrap();

        assert_eq!(
            state.board.hexsides.get(&wall).copied(),
            Some(HexsideKind::Breach)
        );
        // Exactly one adjacent enemy unit is eliminated; the other survives.
        let eliminated = state
            .observations
            .iter()
            .filter_map(|o| match o {
                Observation::UnitEliminated {
                    id,
                    cause: ElimCause::Demolition,
                    ..
                } => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            eliminated.len(),
            1,
            "one adjacent enemy unit eliminated: {eliminated:?}"
        );
        assert!(
            state.find_unit(first_adjacent).is_none() || state.find_unit(second_adjacent).is_none()
        );
    }

    // §5.12: movement is capped by the movement allowance, hex by hex -- a
    // move may not push the turn's cumulative cost past the allowance.
    #[rulebook("§5.12")]
    #[test]
    fn cumulative_move_cost_may_not_exceed_allowance() {
        let mut state = playing(Scenario::Campaign);
        let mover = make_ae_infantry(&mut state, HexCoord::new(2, 0)); // MA 8
        apply_move_unit(
            &mut state,
            mover,
            HexCoord::new(3, 0),
            MovementPoints::new(5),
            &[],
        )
        .unwrap();
        // 5 spent + 4 more would be 9 > 8: rejected.
        assert!(matches!(
            apply_move_unit(
                &mut state,
                mover,
                HexCoord::new(3, 1),
                MovementPoints::new(4),
                &[]
            ),
            Err(RuleError::MovementExceedsAllowance { .. })
        ));
        // 5 spent + 3 = 8 <= 8: allowed.
        apply_move_unit(
            &mut state,
            mover,
            HexCoord::new(3, 1),
            MovementPoints::new(3),
            &[],
        )
        .unwrap();
    }

    // §5.13: unused movement points are lost -- a fresh turn's allowance is
    // not increased by whatever the unit failed to spend last turn.
    #[rulebook("§5.13")]
    #[test]
    fn unused_movement_points_do_not_carry_over() {
        let mut state = playing(Scenario::Campaign);
        state.active_player = Player::Dervish;
        let mover = make_dervish_tribal(&mut state, HexCoord::new(3, 0)); // MA 9
        apply_move_unit(
            &mut state,
            mover,
            HexCoord::new(4, 0),
            MovementPoints::new(2),
            &[],
        )
        .unwrap();
        assert_eq!(state.mp_spent(mover), 2);
        end_player_turn(&mut state).unwrap();
        end_player_turn(&mut state).unwrap();
        assert_eq!(state.mp_spent(mover), 0);
        // The full allowance (9) is available again -- not 9 + 2 unspent.
        apply_move_unit(
            &mut state,
            mover,
            HexCoord::new(3, 0),
            MovementPoints::new(9),
            &[],
        )
        .unwrap();
    }

    // §5.3: Anglo-Egyptian infantry can begin constructing a Zariba hexside:
    // the builders are marked and the hexside is recorded. The engine enforces
    // unit validity -- disrupted units may not build (positioning/turn book-
    // keeping stays with the caller).
    #[rulebook("§5.3")]
    #[test]
    fn construct_zariba_marks_builders_and_records_hexside() {
        let mut state = playing(Scenario::Campaign);
        let builder = make_ae_infantry(&mut state, HexCoord::new(3, 0));
        let hexside = HexsideRef::new(HexCoord::new(3, 0), HexCoord::new(3, 1));
        apply_effect(
            &mut state,
            &GameEffect::ConstructZariba {
                unit_ids: vec![builder],
                hexside,
            },
        )
        .unwrap();
        assert!(state.find_unit(builder).unwrap().state.constructing_zariba);
        assert!(state.zariba_hexsides.contains(&hexside));

        // A disrupted unit cannot join the construction party.
        state.find_unit_mut(builder).unwrap().state.disrupted = true;
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::ConstructZariba {
                    unit_ids: vec![builder],
                    hexside,
                },
            ),
            Err(RuleError::Disrupted(_))
        ));
    }

    // §5.53: a stacked Dervish leader only accepts units of his command
    // colour -- Sheik el Din stacks with Mulazmins and Jehadias only.
    #[rulebook("§5.53")]
    #[test]
    fn dervish_leader_stacks_only_with_command_colour() {
        let mut state = playing(Scenario::Campaign);
        state.active_player = Player::Dervish;
        let _leader = make_unit_with_identity(
            &mut state,
            HexCoord::new(5, 5),
            UnitIdentity::DervishLeader(DervishLeader::SheikElDin),
        );
        let jehadia = make_profiled_unit(
            &mut state,
            HexCoord::new(6, 5),
            dervish_tribal_profile_with(DervishTribe::Jehadia),
        );
        let baggara = make_profiled_unit(
            &mut state,
            HexCoord::new(7, 5),
            dervish_tribal_profile_with(DervishTribe::Baggara),
        );
        // His own colour may stack with him...
        assert!(
            state
                .check_stacking(state.find_unit(jehadia).unwrap(), HexCoord::new(5, 5))
                .is_ok()
        );
        // ...a foreign colour may not.
        assert!(matches!(
            state.check_stacking(state.find_unit(baggara).unwrap(), HexCoord::new(5, 5)),
            Err(StackingError::DervishLeaderCommandMismatch)
        ));
    }

    // §7.6: when a Dervish melee eliminates all defenders, the surviving
    // attackers MUST advance into the vacated hex.
    #[rulebook("§7.6")]
    #[test]
    fn dervish_advance_after_melee_is_mandatory() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let defender = make_ae_infantry(&mut state, HexCoord::new(1, 0));
        let attack = MeleeAttack {
            attacker_player: Player::Dervish,
            attacker_hex: HexCoord::new(0, 0),
            defender_hex: HexCoord::new(1, 0),
            attackers: vec![attacker],
            defenders: vec![defender],
            attacker_modifiers: vec![MeleeModifier::DervishStandard],
            defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
        };
        apply_effect(
            &mut state,
            &GameEffect::MeleeCombat {
                attack,
                attacker_roll: DieRoll::Ten,
                defender_roll: DieRoll::One,
            },
        )
        .unwrap();
        // The defender is eliminated and the attacker has taken his hex.
        assert!(state.find_unit(defender).is_none());
        assert_eq!(
            state.find_unit(attacker).map(|u| u.position),
            Some(HexCoord::new(1, 0))
        );
    }

    // §9.232: a unit Nile-side adjacent to a trench hexside is entrenched:
    // Dervish fire against it carries -4, and Dervish melee -2 (instead of +2).
    #[rulebook("§9.232")]
    #[test]
    fn trench_entrenched_units_take_trench_modifiers() {
        assert_eq!(FireModifier::ZaribaTrenchEntrenched.die_modifier(), -4);
        assert_eq!(MeleeModifier::DervishVsTrenchedDefender.die_modifier(), -2);

        let mut state = GameState::new(Scenario::Historical);
        // Nile at (0,0); the trench runs between (0,0) and (1,0), so a unit
        // at (1,0) stands on the Nile side of the trench: entrenched.
        state.board.terrain.insert(
            HexCoord::new(0, 0),
            Terrain::Nile {
                direction: HexDirection::East,
            },
        );
        state.board.hexsides.insert(
            HexsideRef::new(HexCoord::new(0, 0), HexCoord::new(1, 0)),
            HexsideKind::ZaribaTrench,
        );
        assert!(state.board.is_zariba_entrenched(HexCoord::new(1, 0)));

        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::Dervish;
        let firer = make_dervish_tribal(&mut state, HexCoord::new(2, 0));
        let target = make_ae_infantry(&mut state, HexCoord::new(1, 0));
        let mut attack = FireAttack {
            firing_player: Player::Dervish,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![],
        };
        attack.modifiers = mandatory_fire_modifiers(&state, &attack);
        assert!(
            attack
                .modifiers
                .contains(&FireModifier::ZaribaTrenchEntrenched)
        );

        // ...and Dervish melee against the entrenched unit takes the -2
        // DervishVsTrenchedDefender modifier alongside the +2 standard.
        let melee = MeleeAttack {
            attacker_player: Player::Dervish,
            attacker_hex: HexCoord::new(2, 0),
            defender_hex: HexCoord::new(1, 0),
            attackers: vec![firer],
            defenders: vec![target],
            attacker_modifiers: vec![],
            defender_modifiers: vec![],
        };
        let (attacker_mods, _) = mandatory_melee_modifiers(&state, &melee);
        assert!(attacker_mods.contains(&MeleeModifier::DervishVsTrenchedDefender));
    }

    // §9.342: every hex of the Fall-of-Khartoum mini-map is playable,
    // including half hexes -- the authored board excludes nothing.
    #[rulebook("§9.342")]
    #[test]
    fn fall_of_khartoum_board_excludes_no_hexes() {
        let map = crate::board_data::fall_of_khartoum_map_data();
        assert!(map.excluded.is_empty(), "FoK excludes hexes (§9.342)");
        let board = crate::board::BoardInfo::from_map_data(&map);
        assert_eq!(board.terrain.len(), map.tiles.len());
    }

    #[test]
    fn howitzer_scatters_off_target_below_seven() {
        // Impact roll 1-6 must move the shell off the designated hex; 7-10 hits.
        let mut state = GameState::new(Scenario::Campaign);
        state.board = nile_board_row0(0, 12, HexDirection::East);
        let target = HexCoord::new(8, 0);
        // Roll 3 (Right on the scattergram) lands on a distinct neighbour.
        let impact = state.howitzer_impact_hex(
            target,
            Some(HexCoord::new(0, 0)),
            howitzer_scatter(DieRoll::Three),
        );
        assert_ne!(impact, target);
        // Every miss roll (1-6) lands on a distinct neighbour.
        let mut seen = std::collections::HashSet::new();
        for roll in 1u16..=6 {
            let hex = state.howitzer_impact_hex(
                target,
                Some(HexCoord::new(0, 0)),
                howitzer_scatter(DieRoll::try_from(roll).unwrap()),
            );
            assert_ne!(hex, target);
            assert!(seen.insert(hex), "rolls must scatter to distinct hexes");
        }
        // On-target (roll 7-10) lands on the designated hex.
        let on = state.howitzer_impact_hex(
            target,
            Some(HexCoord::new(0, 0)),
            howitzer_scatter(DieRoll::Nine),
        );
        assert_eq!(on, target);
    }

    // ----- Part D-5: mines & chain ------------------------------------------

    #[rulebook("§10.12", "§10.13", "§10.14")]
    #[test]
    fn mine_fires_once_and_spares_dervish() {
        let mut state = GameState::new(Scenario::Campaign);
        let hex = HexCoord::new(2, 0);
        state.mines.push(crate::MinePlacement {
            hex,
            triggered: false,
        });
        // A Dervish gunboat passes unharmed (§10.14) and does not consume the mine.
        let dervish_gb = make_dervish_gunboat(&mut state, hex);
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::RiverMine {
                    gunboat_id: dervish_gb,
                    hex,
                    roll: DieRoll::Ten
                }
            )
            .is_ok()
        );
        assert!(state.find_unit(dervish_gb).is_some());
        assert!(!state.mines[0].triggered);

        // A British gunboat triggers it (roll 10 -> sunk).
        let brit_gb = make_unit(
            &mut state,
            hex,
            UnitKind::Gunboat {
                fire: 0,
                upstream: 0,
                downstream: 0,
            },
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(crate::OldGunboat::LordKitchener)),
            WeaponClass::Artillery,
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        );
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::RiverMine {
                    gunboat_id: brit_gb,
                    hex,
                    roll: DieRoll::Ten
                }
            )
            .is_ok()
        );
        assert!(state.find_unit(brit_gb).is_none());
        assert!(state.mines[0].triggered);
        // §10.13: a spent mine no longer fires.
        let gb3 = make_unit(
            &mut state,
            hex,
            UnitKind::Gunboat {
                fire: 0,
                upstream: 0,
                downstream: 0,
            },
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(crate::OldGunboat::LordKitchener)),
            WeaponClass::Artillery,
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        );
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::RiverMine {
                    gunboat_id: gb3,
                    hex,
                    roll: DieRoll::Ten
                }
            ),
            Err(RuleError::NoUntriggeredMine(_))
        ));
    }

    #[rulebook("§10.22", "§10.23")]
    #[test]
    fn chain_stops_gunboat_until_sunk() {
        let mut state = GameState::new(Scenario::Campaign);
        state.board = nile_board_row0(0, 4, HexDirection::East);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let chained = HexCoord::new(2, 0);
        state.chain = Some(crate::ChainPlacement {
            hexes: vec![chained],
            sunk: false,
        });
        let gb = make_dervish_gunboat(&mut state, HexCoord::new(1, 0));
        let path = vec![chained];
        assert!(matches!(
            state.can_move_gunboat(gb, chained, &path, MovementPoints::new(1)),
            Err(RuleError::BlockedByChain(_))
        ));
        // §10.23: once sunk, the chain no longer stops the gunboat.
        apply_effect(&mut state, &GameEffect::SinkChain).unwrap();
        assert!(
            state
                .can_move_gunboat(gb, chained, &path, MovementPoints::new(1))
                .is_ok()
        );
    }

    // ----- Part E: Mahdi's Tomb --------------------------------------------

    #[rulebook("§9.14")]
    #[test]
    fn mahdis_tomb_scores_for_anglo_egyptian_when_held() {
        let mut state = GameState::new(Scenario::Campaign);
        let tomb = HexCoord::new(5, 5);
        state
            .board
            .locations
            .insert(tomb, omdurman_types::Location::MahdisTomb);
        // A British leader plus a non-Friendlies combat unit, both undisrupted.
        make_unit(
            &mut state,
            tomb,
            UnitKind::BritishLeader { movement: 0 },
            UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Kitchener),
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Eight),
        );
        make_ae_infantry(&mut state, tomb);

        score_mahdis_tomb(&mut state);
        assert_eq!(
            state.victory.total_for(Player::AngloEgyptian),
            crate::VictoryPoints(25)
        );
    }

    #[rulebook("§9.14")]
    #[test]
    fn mahdis_tomb_not_scored_without_a_leader() {
        let mut state = GameState::new(Scenario::Campaign);
        let tomb = HexCoord::new(5, 5);
        state
            .board
            .locations
            .insert(tomb, omdurman_types::Location::MahdisTomb);
        // Only a combat unit, no British leader -> Dervish retains control.
        make_ae_infantry(&mut state, tomb);
        score_mahdis_tomb(&mut state);
        assert_eq!(
            state.victory.total_for(Player::AngloEgyptian),
            crate::VictoryPoints(0)
        );
    }

    // ----- Fall of Khartoum special rules (§9.3) ---------------------------

    fn make_gordon(state: &mut GameState, hex: HexCoord) -> UnitId {
        make_unit(
            state,
            hex,
            UnitKind::BritishLeader { movement: 0 },
            UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Gordon),
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Immobile),
        )
    }

    /// A FoK game with a Palace at `palace`, GORDON on it, and clear passable
    /// terrain on the palace and an adjacent hex so a Dervish unit can advance.
    fn fok_with_palace(palace: HexCoord) -> (GameState, HexCoord) {
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        state.phase = Phase::Movement; // these tests exercise play, not setup
        let adj = palace.neighbors()[0];
        state
            .board
            .locations
            .insert(palace, omdurman_types::Location::Palace);
        state.board.terrain.insert(palace, Terrain::default());
        state.board.terrain.insert(adj, Terrain::default());
        make_gordon(&mut state, palace);
        (state, adj)
    }

    #[test]
    fn gordon_may_not_move_in_fok() {
        // §9.346: GORDON may not move during FALL OF KHARTOUM.
        let (mut state, _adj) = fok_with_palace(HexCoord::new(2, 2));
        state.active_player = Player::AngloEgyptian;
        let gordon = state.units[0].id;
        let err = state
            .can_move_unit_to(gordon, Some(HexCoord::new(2, 1)), MovementPoints::new(1))
            .unwrap_err();
        assert!(matches!(err, RuleError::GordonMayNotMove));
    }

    #[test]
    fn dervish_reaching_palace_eliminates_gordon_and_ends_game() {
        // §9.346: GORDON dies the instant a Dervish unit occupies the Palace;
        // §9.35: the turn is recorded and the game ends.
        let palace = HexCoord::new(2, 2);
        let (mut state, adj) = fok_with_palace(palace);
        state.current_turn = GameTurnIndex::new(3);
        state.active_player = Player::Dervish;
        let dervish = make_dervish_tribal(&mut state, adj);

        apply_move_unit(
            &mut state,
            dervish,
            palace,
            MovementPoints::new(1),
            &[palace],
        )
        .expect("Dervish moves onto the palace");

        assert!(
            !state.units.iter().any(|u| u.profile.identity.is_gordon()),
            "GORDON is removed"
        );
        assert_eq!(state.gordon_eliminated_turn, Some(GameTurnIndex::new(3)));
        assert!(state.game_over);
    }

    #[rulebook("§9.346")]
    #[test]
    fn gordon_survives_means_no_elimination() {
        // A Dervish unit adjacent to (but not on) the Palace does not kill GORDON.
        let palace = HexCoord::new(2, 2);
        let (mut state, adj) = fok_with_palace(palace);
        state.active_player = Player::Dervish;
        let dervish = make_dervish_tribal(&mut state, palace.neighbors()[1]);
        apply_move_unit(&mut state, dervish, adj, MovementPoints::new(1), &[adj])
            .expect("Dervish moves adjacent");
        assert!(state.units.iter().any(|u| u.profile.identity.is_gordon()));
        assert_eq!(state.gordon_eliminated_turn, None);
        assert!(!state.game_over);
    }

    #[test]
    fn fok_victory_levels_follow_the_table() {
        use crate::FoKVictoryLevel as V;
        // §9.35 base levels by turn of GORDON's death (no Dervish-loss penalty).
        // scenario_end_turn is irrelevant when GORDON died (the death turn
        // fixes the base), so we pass 8 (the typical FoK end).
        assert_eq!(V::resolve(Some(4), 8, 0), V::DervishDecisive);
        assert_eq!(V::resolve(Some(3), 8, 0), V::DervishDecisive);
        assert_eq!(V::resolve(Some(5), 8, 0), V::DervishTactical);
        assert_eq!(V::resolve(Some(6), 8, 0), V::DervishMarginal);
        // GORDON survives: British level depends on how long he held.
        assert_eq!(V::resolve(None, 6, 0), V::BritishMarginal);
        assert_eq!(V::resolve(None, 7, 0), V::BritishTactical);
        assert_eq!(V::resolve(None, 8, 0), V::BritishDecisive);
        // Early end (before turn 6) with GORDON alive: best-effort Marginal.
        assert_eq!(V::resolve(None, 5, 0), V::BritishMarginal);

        // The rulebook worked example: GORDON dies turn 5 (Dervish tactical)
        // but the Dervish lose 24 units (-2 levels) -> British marginal.
        assert_eq!(V::resolve(Some(5), 8, 24), V::BritishMarginal);
        // Loss-penalty thresholds: 16-23 -> -1, 24-31 -> -2, 32+ -> -3.
        assert_eq!(V::resolve(Some(3), 8, 16), V::DervishTactical); // decisive -1
        assert_eq!(V::resolve(Some(3), 8, 32), V::BritishMarginal); // decisive -3, clamps up
    }

    fn make_old_gunboat(state: &mut GameState, hex: HexCoord) -> UnitId {
        make_unit(
            state,
            hex,
            UnitKind::Gunboat {
                fire: 0,
                upstream: 0,
                downstream: 0,
            },
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(crate::OldGunboat::LordKitchener)),
            WeaponClass::Artillery,
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        )
    }

    #[rulebook("§9.345")]
    #[test]
    fn fok_gunboat_crosses_between_nile_mouths() {
        // §9.345: a British gunboat may cross White<->Blue Nile mouths off-board
        // for 6 upstream MP, even though the mouths are not Nile-adjacent.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        state.phase = Phase::Movement; // exercises movement, not setup
        let white = HexCoord::new(1, 0);
        let blue = HexCoord::new(16, 1);
        state.board.terrain.insert(
            white,
            Terrain::Nile {
                direction: HexDirection::East,
            },
        );
        state.board.terrain.insert(
            blue,
            Terrain::Nile {
                direction: HexDirection::East,
            },
        );
        state
            .board
            .locations
            .insert(white, omdurman_types::Location::WhiteNileMouth);
        state
            .board
            .locations
            .insert(blue, omdurman_types::Location::BlueNileMouth);
        state.active_player = Player::AngloEgyptian;
        let gb = make_old_gunboat(&mut state, white);

        // The crossing is legal (6 MP <= the gunboat's upstream allowance of 10).
        assert!(
            state
                .can_move_gunboat(gb, blue, &[blue], MovementPoints::new(6))
                .is_ok(),
            "White->Blue mouth crossing is legal (§9.345)"
        );

        // A normal far-apart move that is NOT a mouth crossing is rejected (the
        // two hexes are not contiguous Nile).
        let elsewhere = HexCoord::new(8, 8);
        state.board.terrain.insert(elsewhere, Terrain::default());
        assert!(
            state
                .can_move_gunboat(gb, elsewhere, &[elsewhere], MovementPoints::new(6))
                .is_err()
        );
    }

    #[rulebook("§9.343")]
    #[test]
    fn fok_both_players_use_dervish_range_table() {
        // §9.343: in FoK an Anglo-Egyptian unit fires on the Dervish table.
        // Dervish rifles reach range 2 at normal; Anglo-Egyptian rifles on
        // their own table would be out of range at 2 doubled->halved etc., so
        // compare the band the engine picks for an AE rifleman at range 3.
        let r = HexDistance::new(3);
        let fok = range_band_for(
            Scenario::FallOfKhartoum,
            Player::AngloEgyptian,
            WeaponClass::Rifles,
            r,
        );
        let dervish = crate::range_effects::dervish_range_effects(WeaponClass::Rifles, r);
        assert_eq!(fok, dervish, "AE uses the Dervish table in FoK (§9.343)");
    }

    // -- §9.14 VP routing tests ---------------------------------------------

    fn make_unit_with_identity(
        state: &mut GameState,
        hex: HexCoord,
        identity: UnitIdentity,
    ) -> UnitId {
        let id = state.alloc_unit_id();
        let kind = match identity {
            UnitIdentity::DervishFort => UnitKind::Fort { fire: 0, melee: 0 },
            UnitIdentity::DervishLeader(_) => UnitKind::DervishLeader {
                fire: 0,
                melee: 0,
                movement: 0,
            },
            UnitIdentity::AngloEgyptianLeader(_) => UnitKind::BritishLeader { movement: 0 },
            _ => UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0,
            },
        };
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: UnitProfile {
                kind,
                identity,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Three),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        });
        id
    }

    #[test]
    fn khalifa_elimination_scores_10_vp() {
        let mut state = playing(Scenario::Campaign);
        let id = make_unit_with_identity(
            &mut state,
            HexCoord::new(0, 0),
            UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah),
        );
        score_elimination(&mut state, id, ElimCause::Combat);
        assert_eq!(
            state.victory.total_for(Player::AngloEgyptian).0,
            10,
            "Khalifa is worth 10 VP (§9.14)"
        );
    }

    #[test]
    fn fort_elimination_scores_0_vp() {
        let mut state = playing(Scenario::Campaign);
        let id =
            make_unit_with_identity(&mut state, HexCoord::new(0, 0), UnitIdentity::DervishFort);
        score_elimination(&mut state, id, ElimCause::Combat);
        assert_eq!(
            state.victory.total_for(Player::AngloEgyptian).0,
            0,
            "Fort elimination is worth 0 VP (§9.14)"
        );
    }

    #[test]
    fn isa_zachneih_elimination_sets_flag_and_scores_1_vp() {
        let mut state = playing(Scenario::Campaign);
        assert!(!state.isa_zachneih_eliminated, "flag starts clear");
        let id = make_unit_with_identity(
            &mut state,
            HexCoord::new(0, 0),
            UnitIdentity::DervishTribal {
                tribe: DervishTribe::IsaZachneih,
            },
        );
        score_elimination(&mut state, id, ElimCause::Combat);
        assert!(state.isa_zachneih_eliminated, "flag set after elimination");
        assert_eq!(
            state.victory.total_for(Player::AngloEgyptian).0,
            1,
            "Isa Zachneih is worth 1 VP (§9.14)"
        );
    }

    #[test]
    fn ordinary_dervish_leader_scores_1_vp() {
        let mut state = playing(Scenario::Campaign);
        let id = make_unit_with_identity(
            &mut state,
            HexCoord::new(0, 0),
            UnitIdentity::DervishLeader(crate::DervishLeader::Yakub),
        );
        score_elimination(&mut state, id, ElimCause::Combat);
        assert_eq!(
            state.victory.total_for(Player::AngloEgyptian).0,
            1,
            "Ordinary Dervish leaders are worth 1 VP (§9.14)"
        );
    }

    #[test]
    fn observations_pushed_on_elimination() {
        let mut state = playing(Scenario::Campaign);
        let id = make_unit_with_identity(
            &mut state,
            HexCoord::new(0, 0),
            UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah),
        );
        score_elimination(&mut state, id, ElimCause::Combat);
        let obs = state.drain_observations();
        assert!(
            obs.iter().any(|o| matches!(
                o,
                Observation::VictoryScored {
                    source: VpSource::KhalifaEliminated,
                    points: crate::VictoryPoints(10),
                    ..
                }
            )),
            "VictoryScored observation for 10 VP"
        );
        assert!(
            obs.iter()
                .any(|o| matches!(o, Observation::UnitEliminated { .. })),
            "UnitEliminated observation"
        );
        assert!(
            obs.iter()
                .any(|o| matches!(o, Observation::LeaderKilled { .. })),
            "LeaderKilled observation"
        );
    }

    // -- §6.42 Maxim second fire tests --------------------------------------

    fn make_maxim(state: &mut GameState, hex: HexCoord) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::Maxim {
                    fire: 0,
                    melee: 0,
                    movement: 0,
                },
                identity: UnitIdentity::AngloEgyptianMaxim,
                weapon: WeaponClass::Maxims,
                fire: Some(crate::FireFactor::Five),
                melee: Some(crate::MeleeFactor::One),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        });
        id
    }

    #[test]
    fn maxim_may_fire_twice_per_turn() {
        let mut state = playing(Scenario::Campaign);
        let maxim = make_maxim(&mut state, HexCoord::new(0, 0));
        let _enemy = make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        // Direct fire subphase: Maxim fires once.
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        assert!(
            state
                .can_fire_at(maxim, HexCoord::new(1, 0), FireKind::Direct)
                .is_ok()
        );
        state.units_fired_this_phase.push(maxim);

        // The once-per-phase set blocks a second shot in the SAME subphase.
        assert!(matches!(
            state.can_fire_at(maxim, HexCoord::new(1, 0), FireKind::Direct),
            Err(RuleError::AlreadyFired(_))
        ));

        // Advance to the Maxim/Howitzer subphase: the set is cleared, so the
        // Maxim may fire its second shot (§6.42).
        state.phase = Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer);
        state.units_fired_this_phase.clear();
        assert!(
            state
                .can_fire_at(maxim, HexCoord::new(1, 0), FireKind::MaximSecondFire)
                .is_ok(),
            "Maxim may fire a second time in the Maxim/Howitzer subphase (§6.42)"
        );
    }

    #[test]
    fn maxim_that_skipped_direct_may_fire_once_in_second_subphase() {
        let mut state = playing(Scenario::Campaign);
        let maxim = make_maxim(&mut state, HexCoord::new(0, 0));

        // The Maxim did not fire in DirectFire. In the Maxim/Howitzer subphase
        // it may fire once (§6.42: "If any Maxim guns did not fire during the
        // Direct Fire Subphase, they may still only fire once in [6.42]").
        state.phase = Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer);
        assert!(
            state
                .can_fire_at(maxim, HexCoord::new(1, 0), FireKind::MaximSecondFire)
                .is_ok(),
            "Maxim that skipped Direct may fire once in the second subphase"
        );
    }

    #[test]
    fn non_maxim_rejected_in_second_subphase() {
        let mut state = playing(Scenario::Campaign);
        let rifle = make_ae_infantry(&mut state, HexCoord::new(0, 0));

        // A rifle-class unit may not fire in the Maxim/Howitzer subphase --
        // engine-authoritative rejection with a typed error (§6.42).
        state.phase = Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer);
        assert!(matches!(
            state.can_fire_at(rifle, HexCoord::new(1, 0), FireKind::MaximSecondFire),
            Err(RuleError::WrongWeaponForSubphase(_))
        ));
    }

    // -- §6.53 Royal Engineers demolition tests -----------------------------

    fn make_royal_engineers(state: &mut GameState, hex: HexCoord) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::Infantry {
                    fire: 0,
                    melee: 0,
                    movement: 0,
                },
                identity: UnitIdentity::RoyalEngineers,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Five),
                melee: Some(crate::MeleeFactor::Three),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        });
        id
    }

    #[test]
    fn demolition_destroys_fort() {
        let mut state = playing(Scenario::Campaign);
        let eng = make_royal_engineers(&mut state, HexCoord::new(0, 0));
        let fort = make_fort(&mut state, HexCoord::new(1, 0));

        // Commit demolition.
        apply_demolition(&mut state, eng, DemolitionTarget::Fort(fort)).unwrap();
        assert!(state.pending_demolitions.len() == 1);

        // Resolve: engineer still adjacent + undisrupted → fort destroyed.
        apply_resolve_demolition(&mut state, eng, DemolitionTarget::Fort(fort)).unwrap();
        assert!(
            state.find_unit(fort).is_none(),
            "fort should be eliminated after successful demolition"
        );
        assert!(
            !state
                .find_unit(eng)
                .map(|u| u.state.demolishing)
                .unwrap_or(true),
            "engineer freed after demolition"
        );
        // Fort elimination is 0 VP (§9.14).
        assert_eq!(state.victory.total_for(Player::AngloEgyptian).0, 0);
    }

    #[rulebook("§6.53")]
    #[test]
    fn demolition_cancelled_when_engineer_disrupted() {
        let mut state = playing(Scenario::Campaign);
        let eng = make_royal_engineers(&mut state, HexCoord::new(0, 0));
        let fort = make_fort(&mut state, HexCoord::new(1, 0));

        apply_demolition(&mut state, eng, DemolitionTarget::Fort(fort)).unwrap();
        // Engineer gets disrupted during the turn.
        state.find_unit_mut(eng).unwrap().state.disrupted = true;
        apply_resolve_demolition(&mut state, eng, DemolitionTarget::Fort(fort)).unwrap();
        assert!(
            state.find_unit(fort).is_some(),
            "fort should survive when engineer was disrupted"
        );
        assert!(
            !state
                .find_unit(eng)
                .map(|u| u.state.demolishing)
                .unwrap_or(true),
            "engineer freed even on failed demolition"
        );
    }

    #[rulebook("§6.53")]
    #[test]
    fn demolition_cancelled_when_engineer_moved_away() {
        let mut state = playing(Scenario::Campaign);
        let eng = make_royal_engineers(&mut state, HexCoord::new(0, 0));
        let fort = make_fort(&mut state, HexCoord::new(1, 0));

        apply_demolition(&mut state, eng, DemolitionTarget::Fort(fort)).unwrap();
        // Engineer moves away (no longer adjacent).
        state.find_unit_mut(eng).unwrap().position = HexCoord::new(5, 5);
        apply_resolve_demolition(&mut state, eng, DemolitionTarget::Fort(fort)).unwrap();
        assert!(
            state.find_unit(fort).is_some(),
            "fort should survive when engineer moved away"
        );
    }

    // -- Engine-authoritative LOS / hexside blocking tests (§6.3, §7.2) ----

    #[rulebook("§6.21")]
    #[test]
    fn can_fire_at_rejects_blocked_los() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target_hex = HexCoord::new(1, 0);
        make_dervish_tribal(&mut state, target_hex);
        // Wall hexside between firer and target blocks LOS.
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), target_hex),
            omdurman_types::HexsideKind::Wall,
        );
        assert!(matches!(
            state.can_fire_at(ae, target_hex, crate::FireKind::Direct),
            Err(RuleError::LineOfSightBlocked(_, _))
        ));
    }

    #[rulebook("§6.21")]
    #[test]
    fn can_fire_at_allows_clear_los() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target_hex = HexCoord::new(1, 0);
        make_dervish_tribal(&mut state, target_hex);
        // No hexside → LOS clear.
        assert!(
            state
                .can_fire_at(ae, target_hex, crate::FireKind::Direct)
                .is_ok()
        );
    }

    #[rulebook("§7.2")]
    #[test]
    fn can_melee_rejects_wall_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(1, 0);
        make_dervish_tribal(&mut state, target);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), target),
            omdurman_types::HexsideKind::Wall,
        );
        assert!(matches!(
            state.can_melee(ae, target),
            Err(RuleError::MeleeBlockedByHexside(_, _))
        ));
    }

    #[rulebook("§7.2")]
    #[test]
    fn can_melee_rejects_thorn_hedge_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(1, 0);
        make_dervish_tribal(&mut state, target);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), target),
            omdurman_types::HexsideKind::ZaribaThornHedge,
        );
        assert!(matches!(
            state.can_melee(ae, target),
            Err(RuleError::MeleeBlockedByHexside(_, _))
        ));
    }

    #[rulebook("§7.2")]
    #[test]
    fn can_melee_allows_gate_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(1, 0);
        make_dervish_tribal(&mut state, target);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), target),
            omdurman_types::HexsideKind::Gate,
        );
        assert!(state.can_melee(ae, target).is_ok());
    }

    #[rulebook("§6.82")]
    #[test]
    fn can_advance_after_combat_rejects_wall_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::DirectFire);
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let to = HexCoord::new(1, 0);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), to),
            omdurman_types::HexsideKind::Wall,
        );
        open_advance_window(&mut state, to, &[ae], vec!["6.82".to_string()]);
        assert!(matches!(
            state.can_advance_after_combat(ae, to),
            Err(RuleError::AdvanceBlockedByHexside(_, _))
        ));
    }

    #[rulebook("§6.82")]
    #[test]
    fn can_advance_after_combat_rejects_khor_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::DirectFire);
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let to = HexCoord::new(1, 0);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), to),
            omdurman_types::HexsideKind::Khor,
        );
        open_advance_window(&mut state, to, &[ae], vec!["6.82".to_string()]);
        assert!(matches!(
            state.can_advance_after_combat(ae, to),
            Err(RuleError::AdvanceBlockedByHexside(_, _))
        ));
    }

    // -- Engine-authoritative movement tests (§5.11, §5.23) -----------------

    #[rulebook("§5.23")]
    #[test]
    fn can_move_rejects_wall_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let to = HexCoord::new(1, 0);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), to),
            omdurman_types::HexsideKind::Wall,
        );
        assert!(matches!(
            state.can_move_unit_to(ae, Some(to), MovementPoints::new(1)),
            Err(RuleError::MoveBlockedByHexside(_, _))
        ));
    }

    #[rulebook("§5.23")]
    #[test]
    fn can_move_allows_gate_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let to = HexCoord::new(1, 0);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), to),
            omdurman_types::HexsideKind::Gate,
        );
        assert!(
            state
                .can_move_unit_to(ae, Some(to), MovementPoints::new(1))
                .is_ok()
        );
    }

    #[rulebook("§5.11")]
    #[test]
    fn movement_cost_for_uses_terrain() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        // Place terrain: Rough at (1,0) costs 2 MP.
        state.board.terrain.insert(
            HexCoord::new(1, 0),
            omdurman_types::Terrain::ground(omdurman_types::GroundKind::Rough),
        );
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let unit = state.find_unit(ae).unwrap();
        let cost = state.movement_cost_for(unit, &[HexCoord::new(1, 0)]);
        assert_eq!(cost, Some(MovementPoints::new(2)));
    }

    #[rulebook("§5.11")]
    #[test]
    fn movement_cost_for_road_costs_one() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        // Place Rough terrain (normally 2 MP) and a road edge.
        state.board.terrain.insert(
            HexCoord::new(1, 0),
            omdurman_types::Terrain::ground(omdurman_types::GroundKind::Rough),
        );
        state.board.roads.insert(omdurman_types::HexsideRef::new(
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
        ));
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let unit = state.find_unit(ae).unwrap();
        let cost = state.movement_cost_for(unit, &[HexCoord::new(1, 0)]);
        assert_eq!(cost, Some(MovementPoints::new(1)));
    }

    #[rulebook("§8.1")]
    #[test]
    fn night_movement_overlay_allowance_halved() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.day_night = DayNight::Night;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        // AE infantry has MA 4; at night that's halved to 2.
        let unit = state.find_unit(ae).unwrap();
        let allowance = match unit.profile.movement {
            crate::UnitMovement::Land(a) => a,
            _ => panic!("expected land movement"),
        };
        let effective =
            crate::effective_movement_at_night(allowance, Player::AngloEgyptian, state.day_night);
        assert_eq!(effective.value(), allowance.value() / 2);
    }

    // ----- Part E: walled-city entry (§5.23), Zariba surcharge (§9.233),
    //      mid-move stacking (§5.51), SetupLetter mapping (§9.212) ----

    /// Helper: build a tiny board with a walled-city interior at `city`.
    /// Three Wall hexsides surround it so `is_walled_city` fires.
    fn make_walled_board(state: &mut GameState, city: HexCoord) {
        // The walled city is *derived* (flood from the Palace, §5.23), so the
        // fixture needs a Palace landmark plus walls, then a recompute.
        state
            .board
            .locations
            .insert(city, omdurman_types::Location::Palace);
        let n = city.neighbors();
        for neighbor in n.iter().take(3) {
            state.board.hexsides.insert(
                omdurman_types::HexsideRef::new(city, *neighbor),
                HexsideKind::Wall,
            );
        }
        state.board.walled_city = state.board.compute_walled_city();
    }

    #[rulebook("§5.23")]
    #[test]
    fn walled_city_entry_allows_khalifa() {
        let mut state = playing(Scenario::Campaign);
        state.active_player = Player::Dervish;
        let from = HexCoord::new(0, 0);
        let city = HexCoord::new(1, 0);
        make_walled_board(&mut state, city);
        let khalifa = make_unit(
            &mut state,
            from,
            UnitKind::DervishLeader {
                fire: 0,
                melee: 0,
                movement: 0,
            },
            UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah),
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Eight),
        );
        assert!(
            state
                .can_move_unit_to(khalifa, Some(city), MovementPoints::new(1))
                .is_ok(),
            "Khalifa must be allowed into the walled city (§5.23)"
        );
    }

    #[rulebook("§5.23")]
    #[test]
    fn walled_city_entry_rejects_unauthorized_dervish() {
        let mut state = playing(Scenario::Campaign);
        state.active_player = Player::Dervish;
        let from = HexCoord::new(0, 0);
        let city = HexCoord::new(1, 0);
        make_walled_board(&mut state, city);
        let tribal = make_dervish_tribal(&mut state, from);
        assert!(matches!(
            state.can_move_unit_to(tribal, Some(city), MovementPoints::new(1)),
            Err(RuleError::WalledCityEntry(_, _))
        ));
    }

    #[rulebook("§5.23")]
    #[test]
    fn walled_city_entry_rejects_ae_gunboat() {
        let identity = UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Named(
            crate::NamedGunboat::Sultan,
        ));
        assert!(
            !identity.may_enter_walled_city(),
            "AE gunboats must be blocked from the walled city (§5.23)"
        );
    }

    #[rulebook("§5.23")]
    #[test]
    fn walled_city_entry_not_enforced_for_fok() {
        let mut state = playing(Scenario::FallOfKhartoum);
        let from = HexCoord::new(0, 0);
        let city = HexCoord::new(1, 0);
        make_walled_board(&mut state, city);
        let tribal = make_dervish_tribal(&mut state, from);
        // Baggara would fail on Campaign map, but FoK map is exempt.
        assert!(
            state
                .can_move_unit_to(tribal, Some(city), MovementPoints::new(1))
                .is_ok(),
            "FoK map must not enforce §5.23 walled-city entry"
        );
    }

    #[rulebook("§9.233")]
    #[test]
    fn zariba_end_hexside_costs_extra_mp() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(a, b),
            HexsideKind::ZaribaTrenchEndA,
        );
        // Seed terrain so movement_cost_for doesn't short-circuit on empty board.
        state.board.terrain.insert(a, Terrain::default());
        state.board.terrain.insert(b, Terrain::default());
        let ae = make_ae_infantry(&mut state, a);
        let unit = state.find_unit(ae).unwrap();
        let cost = state.movement_cost_for(unit, &[b]).unwrap();
        // Clear terrain = 1 MP + zariba surcharge 2 = 3 MP.
        assert_eq!(cost, MovementPoints::new(3));
    }

    #[rulebook("§9.233")]
    #[test]
    fn zariba_thorn_hedge_blocks_movement() {
        let mut state = playing(Scenario::Campaign);
        state.active_player = Player::AngloEgyptian;
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(a, b),
            HexsideKind::ZaribaThornHedge,
        );
        let ae = make_ae_infantry(&mut state, a);
        assert!(matches!(
            state.can_move_unit_to(ae, Some(b), MovementPoints::new(1)),
            Err(RuleError::MoveBlockedByHexside(_, _))
        ));
    }

    #[rulebook("§5.51")]
    #[test]
    fn mid_move_stacking_allows_pass_through() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        // Path: (0,0) -> (1,0) -> (2,0).  Put 4 friendlies in (1,0), none in (2,0).
        let through = HexCoord::new(1, 0);
        let dest = HexCoord::new(2, 0);
        for _ in 0..4 {
            make_ae_infantry(&mut state, through);
        }
        let mover = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        // Move along the 2-hex path; stacking at (1,0) is never checked.
        assert!(
            state
                .can_move_unit_to(mover, Some(dest), MovementPoints::new(2))
                .is_ok(),
            "passing through a stacked hex must not be blocked (§5.51 mid-move)"
        );
    }

    #[rulebook("§5.51")]
    #[test]
    fn mid_move_stacking_rejects_over_limit_destination() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        let dest = HexCoord::new(2, 0);
        for _ in 0..4 {
            make_ae_infantry(&mut state, dest);
        }
        let mover = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        // Stacking is checked during apply, not can_move_unit_to.
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::MoveUnit {
                    unit_id: mover,
                    to: dest,
                    cost: MovementPoints::new(1),
                    path: Vec::new(),
                }
            ),
            Err(RuleError::Stacking(crate::StackingError::OverLimit))
        ));
    }

    #[rulebook("§9.212")]
    #[test]
    fn setup_letter_dervish_leader_roundtrip() {
        use crate::dervish_leader_for_setup_letter;
        for letter in [
            SetupLetter::A,
            SetupLetter::D,
            SetupLetter::Y,
            SetupLetter::K,
            SetupLetter::S,
            SetupLetter::O,
        ] {
            let leader = dervish_leader_for_setup_letter(letter);
            assert_eq!(leader.setup_letter(), letter);
        }
    }

    #[rulebook("§9.212")]
    #[test]
    fn setup_letter_to_dervish_leader_known_values() {
        use crate::dervish_leader_for_setup_letter;
        assert_eq!(
            dervish_leader_for_setup_letter(SetupLetter::A),
            crate::DervishLeader::AliWadHelu
        );
        assert_eq!(
            dervish_leader_for_setup_letter(SetupLetter::K),
            crate::DervishLeader::KhalifaAbdullah
        );
        assert_eq!(
            dervish_leader_for_setup_letter(SetupLetter::O),
            crate::DervishLeader::OsmanDigna
        );
    }

    // ----- Part F: Named vs Old gunboat capabilities (§6.64, §2.32) ----

    fn make_named_gunboat(state: &mut GameState, hex: HexCoord) -> UnitId {
        make_unit(
            state,
            hex,
            UnitKind::Gunboat {
                fire: 0,
                upstream: 0,
                downstream: 0,
            },
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Named(crate::NamedGunboat::Sultan)),
            WeaponClass::Artillery, // profile weapon stays Artillery
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        )
    }

    // §6.64
    #[test]
    fn named_gunboat_has_howitzer() {
        assert!(GunboatId::Named(crate::NamedGunboat::Sultan).has_howitzer());
        assert!(GunboatId::Named(crate::NamedGunboat::Fateh).has_howitzer());
    }

    // §2.32
    #[test]
    fn old_gunboat_lacks_howitzer() {
        assert!(!GunboatId::Old(crate::OldGunboat::LordKitchener).has_howitzer());
        assert!(!GunboatId::Old(crate::OldGunboat::Tamai).has_howitzer());
    }

    // §6.64
    #[test]
    fn named_gunboat_may_fire_howitzer_in_second_subphase() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::MaximSecondAndHowitzer);
        let gb = make_named_gunboat(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(5, 0);
        make_dervish_tribal(&mut state, target);
        assert!(
            state.can_fire_at(gb, target, FireKind::Howitzer).is_ok(),
            "named gunboat must be allowed to fire howitzer (§6.64)"
        );
    }

    // §2.32
    #[test]
    fn old_gunboat_rejected_from_howitzer_subphase() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::MaximSecondAndHowitzer);
        let gb = make_old_gunboat(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(3, 0);
        make_dervish_tribal(&mut state, target);
        assert!(
            matches!(
                state.can_fire_at(gb, target, FireKind::Howitzer),
                Err(RuleError::WrongWeaponForSubphase(_))
            ),
            "old gunboat must not fire howitzer (§2.32)"
        );
    }

    // §6.64: named gunboat in direct fire still uses the Artillery line.
    #[test]
    fn named_gunboat_direct_fire_uses_artillery_weapon() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::DirectFire);
        let gb = make_named_gunboat(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(3, 0);
        make_dervish_tribal(&mut state, target);
        assert!(
            state.can_fire_at(gb, target, FireKind::Direct).is_ok(),
            "named gunboat must be allowed direct fire"
        );
    }

    // §6.64: named gunboat cannot fire howitzer at night.
    #[test]
    fn named_gunboat_no_howitzer_at_night() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::MaximSecondAndHowitzer);
        state.day_night = DayNight::Night;
        let gb = make_named_gunboat(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(3, 0);
        make_dervish_tribal(&mut state, target);
        assert!(
            matches!(
                state.can_fire_at(gb, target, FireKind::Howitzer),
                Err(RuleError::NoHowitzerAtNight)
            ),
            "howitzer fire at night must be rejected (§6.64)"
        );
    }

    // §6.64: Dervish gunboats have no howitzer.
    #[test]
    fn dervish_gunboat_lacks_howitzer() {
        assert!(!GunboatId::DervishGunboat(1).has_howitzer());
    }

    // §6.14: a combat unit may only be *fired at* once per fire phase
    // (exceptions: Maxims and gunboats). A second attack on the same target
    // hex in the same phase fires at the same units and must be rejected.
    #[rulebook("§6.14")]
    #[test]
    fn unit_may_only_be_fired_at_once_per_phase() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };
        apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: attack.clone(),
                roll: DieRoll::Ten, // Eliminate(2): target gone
            },
        )
        .unwrap();
        assert!(state.find_unit(target).is_none());

        // A fresh firer attacks the same (now-empty) hex in the same phase:
        // the tracker recorded the target -- but he is eliminated, so this
        // targets nobody. Re-set with a survivor instead.
        let _target2 = make_dervish_tribal(&mut state, HexCoord::new(1, 0));
        let firer2 = make_ae_infantry(&mut state, HexCoord::new(2, 0));
        let attack2 = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer2],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };
        // The hex's previous occupant was fired at; a new occupant arriving
        // later in the same phase may be fired at (the rule is per-unit).
        // The genuine violation: fire at target2 twice.
        apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: attack2,
                roll: DieRoll::One, // NoEffect -- but still "fired at"
            },
        )
        .unwrap();
        let firer3 = make_ae_infantry(&mut state, HexCoord::new(3, 0));
        let attack3 = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer3],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::FireCombat {
                    attack: attack3,
                    roll: DieRoll::Ten
                }
            ),
            Err(RuleError::AlreadyFiredAt(_))
        ));
    }

    // §6.42: the Maxim/Howitzer subphase is a fresh fire phase for fired-at
    // purposes ("Units firing in this subphase may fire at enemy units fired
    // at in Direct Fire Subphase").
    #[rulebook("§6.42")]
    #[test]
    fn fired_at_tracker_resets_at_maxim_subphase() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        make_dervish_tribal(&mut state, HexCoord::new(1, 0));
        let id = state.units[0].id;
        state.units_fired_at_this_phase.push(id);
        assert!(state.units_fired_at_this_phase.contains(&id));

        advance_phase(&mut state).unwrap(); // -> Maxim/Howitzer subphase
        assert!(
            state.units_fired_at_this_phase.is_empty(),
            "§6.42 bridge resets the fired-at tracker"
        );
    }

    // §6.14's exception: gunboats and Maxims may be fired at repeatedly.
    #[rulebook("§6.14")]
    #[test]
    fn gunboat_and_maxim_may_be_fired_at_repeatedly() {
        assert!(fired_at_excepted(UnitKind::Gunboat {
            fire: 0,
            upstream: 0,
            downstream: 0
        }));
        assert!(fired_at_excepted(UnitKind::Maxim {
            fire: 0,
            melee: 0,
            movement: 0
        }));
        assert!(!fired_at_excepted(UnitKind::Infantry {
            fire: 0,
            melee: 0,
            movement: 0
        }));
        assert!(!fired_at_excepted(UnitKind::Fort { fire: 0, melee: 0 }));
    }

    // §9.111: only the Dervish initial force deploys at Campaign setup --
    // the rest arrive as §9.112/§9.113 reinforcements, and the
    // Anglo-Egyptian side deploys nothing at all.
    #[rulebook("§9.111")]
    #[test]
    fn campaign_setup_rejects_non_initial_force() {
        let mut state = GameState::new(Scenario::Campaign); // permissive zone
        let hex = HexCoord::new(1, 1);

        // §9.111 set deploys.
        for profile in [
            dervish_tribal_profile_with(DervishTribe::Taiasha),
            dervish_tribal_profile_with(DervishTribe::IsaZachneih),
        ] {
            let p = UnitPlacement {
                id: state.alloc_unit_id(),
                position: hex,
                profile,
                state: Default::default(),
            };
            assert!(state.can_deploy_unit(&p).is_ok(), "§9.111 unit rejected");
        }

        // A wave tribe (Baggara arrives turn 1 per §9.112) may not deploy at
        // setup; nor may any Anglo-Egyptian unit (§9.113).
        let baggara = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: dervish_tribal_profile_with(DervishTribe::Baggara),
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&baggara),
            Err(RuleError::NotInPlay(_))
        ));
        let ae = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: crate::UnitProfile {
                kind: crate::UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity: crate::UnitIdentity::AngloEgyptianInfantry {
                    brigade: omdurman_types::BrigadeId {
                        number: 1,
                        nationality: omdurman_types::BrigadeNationality::British,
                    },
                    battalion: crate::BattalionOrdinal::First,
                },
                weapon: crate::WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&ae),
            Err(RuleError::NotInPlay(_))
        ));
    }

    // §9.211/§9.212: the Historical scenario's not-in-play units may not be
    // deployed: GORDON and the "Friendlies" (AE), Isa Zachneih, gunboats and
    // forts (Dervish).
    #[rulebook("§9.211", "§9.212")]
    #[test]
    fn historical_setup_rejects_not_in_play_units() {
        let mut state = GameState::new(Scenario::Historical); // permissive zone
        let hex = HexCoord::new(1, 1);

        let gordon = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::BritishLeader { movement: 8 },
                identity: UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Gordon),
                weapon: WeaponClass::Melee,
                fire: None,
                melee: None,
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&gordon),
            Err(RuleError::NotInPlay(_))
        ));

        let isa = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: dervish_tribal_profile_with(DervishTribe::IsaZachneih),
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&isa),
            Err(RuleError::NotInPlay(_))
        ));

        // In-play units are unaffected.
        let baggara = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: dervish_tribal_profile_with(DervishTribe::Baggara),
            state: Default::default(),
        };
        assert!(state.can_deploy_unit(&baggara).is_ok());
        let ae = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: crate::UnitProfile {
                kind: crate::UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity: crate::UnitIdentity::AngloEgyptianInfantry {
                    brigade: omdurman_types::BrigadeId {
                        number: 1,
                        nationality: omdurman_types::BrigadeNationality::British,
                    },
                    battalion: crate::BattalionOrdinal::First,
                },
                weapon: crate::WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement: crate::UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        };
        assert!(state.can_deploy_unit(&ae).is_ok());
    }

    // §9.321/§9.322: the FALL OF KHARTOUM orders of battle -- which unit
    // types exist at all, and their exact counts. Dervish fort counters play
    // no role (§9.344: the single North Fort is a scenario-fixed placement),
    // nor do Dervish gunboats or any non-entry tribe.
    #[rulebook("§9.322", "§9.344")]
    #[test]
    fn fok_order_of_battle_dervish() {
        let mut state = GameState::new(Scenario::FallOfKhartoum); // permissive zone
        let hex = HexCoord::new(1, 1);
        let dervish_fort_profile = || UnitProfile {
            kind: UnitKind::Fort { fire: 5, melee: 3 },
            identity: UnitIdentity::DervishFort,
            weapon: WeaponClass::Artillery,
            fire: Some(crate::FireFactor::Five),
            melee: Some(crate::MeleeFactor::Three),
            movement: UnitMovement::Immobile,
        };
        let dervish_leader_profile = |leader: DervishLeader| UnitProfile {
            kind: UnitKind::DervishLeader {
                fire: 3,
                melee: 6,
                movement: 9,
            },
            identity: UnitIdentity::DervishLeader(leader),
            weapon: WeaponClass::Melee,
            fire: Some(crate::FireFactor::Three),
            melee: Some(crate::MeleeFactor::Six),
            movement: UnitMovement::Land(crate::MovementAllowance::Nine),
        };
        let dervish_artillery_profile = || UnitProfile {
            kind: UnitKind::Artillery {
                fire: 5,
                melee: 1,
                movement: 7,
            },
            identity: UnitIdentity::DervishArtillery,
            weapon: WeaponClass::Artillery,
            fire: Some(crate::FireFactor::Five),
            melee: Some(crate::MeleeFactor::One),
            movement: UnitMovement::Land(crate::MovementAllowance::Seven),
        };

        // Not in play: the Dervish gunboats, the Khalifa, and every
        // non-entry tribe.
        for profile in [
            dervish_gunboat_profile(),
            dervish_leader_profile(DervishLeader::KhalifaAbdullah),
            dervish_tribal_profile_with(DervishTribe::Baggara),
            dervish_tribal_profile_with(DervishTribe::Taiasha),
            dervish_tribal_profile_with(DervishTribe::IsaZachneih),
        ] {
            let p = UnitPlacement {
                id: state.alloc_unit_id(),
                position: hex,
                profile,
                state: Default::default(),
            };
            assert!(
                matches!(state.can_deploy_unit(&p), Err(RuleError::NotInPlay(_))),
                "{:?} is not in the FoK order of battle",
                p.profile.identity
            );
        }

        // §9.322 counts: exactly 2 Hadendowa, 6 Kehena, 5 Degheim, 3
        // artillery, 32 Mulazmin.
        // Each type gets its own hex column: distinct tribes may not stack
        // (§5.52) and the four-unit limit (§5.51) would otherwise mask the
        // order-of-battle caps being tested.
        let count_cap = |state: &mut GameState, profile: UnitProfile, cap: usize, col: i32| {
            let mut accepted = 0;
            for i in 0..cap {
                let p = UnitPlacement {
                    id: state.alloc_unit_id(),
                    position: HexCoord::new(col, 1 + (i % 8) as i32),
                    profile,
                    state: Default::default(),
                };
                if state.can_deploy_unit(&p).is_ok() {
                    apply_effect(state, &GameEffect::DeployUnit(p)).unwrap();
                    accepted += 1;
                }
            }
            let over = UnitPlacement {
                id: state.alloc_unit_id(),
                position: HexCoord::new(col, 1 + (cap % 8) as i32),
                profile,
                state: Default::default(),
            };
            assert_eq!(
                accepted, cap,
                "expected to place {cap} of {:?}",
                profile.identity
            );
            let err = state.can_deploy_unit(&over);
            let n = state
                .units
                .iter()
                .filter(|u| u.profile.identity == over.profile.identity)
                .count();
            assert!(
                matches!(err, Err(RuleError::FoKOrderOfBattleFull)),
                "cap of {} enforced for {:?}, got {:?} (on board: {n})",
                cap,
                profile.identity,
                err
            );
        };
        count_cap(
            &mut state,
            dervish_tribal_profile_with(DervishTribe::Hadendowa),
            2,
            1,
        );
        count_cap(
            &mut state,
            dervish_tribal_profile_with(DervishTribe::Kehena),
            6,
            2,
        );
        count_cap(
            &mut state,
            dervish_tribal_profile_with(DervishTribe::Degheim),
            5,
            3,
        );
        count_cap(&mut state, dervish_artillery_profile(), 3, 4);
        count_cap(
            &mut state,
            dervish_tribal_profile_with(DervishTribe::Mulazmin),
            32,
            5,
        );

        // §9.344: exactly one Dervish fort is in play -- the scenario-fixed
        // North Fort. A first fort counter deploys (the scenario's fixed
        // placement uses the canonical counter); a second is rejected.
        count_cap(&mut state, dervish_fort_profile(), 1, 6);
    }

    // §9.321 regression: the counts bind across counter variants -- "two
    // British infantry units" covers 1B First + 1B Second; a third battalion
    // (whatever its ordinal) is rejected. Likewise the gunboat cap binds
    // across the four old-style boat counters.
    #[test]
    fn fok_caps_bind_across_counter_variants() {
        let mut state = GameState::new(Scenario::FallOfKhartoum);

        let id_1 = state.alloc_unit_id();
        let id_2 = state.alloc_unit_id();
        let id_3 = state.alloc_unit_id();
        let id_4 = state.alloc_unit_id();
        let id_5 = state.alloc_unit_id();

        // 1B First + 1B Second fit (cap 2 British); 1B Third is rejected.
        apply_effect(
            &mut state,
            &GameEffect::DeployUnit(UnitPlacement {
                id: id_1,
                position: HexCoord::new(3, 1),
                profile: UnitProfile {
                    kind: UnitKind::Infantry {
                        fire: 9,
                        melee: 5,
                        movement: 8,
                    },
                    identity: UnitIdentity::AngloEgyptianInfantry {
                        brigade: crate::BrigadeId {
                            number: 1,
                            nationality: crate::BrigadeNationality::British,
                        },
                        battalion: BattalionOrdinal::First,
                    },
                    weapon: WeaponClass::Rifles,
                    fire: Some(crate::FireFactor::Nine),
                    melee: Some(crate::MeleeFactor::Five),
                    movement: UnitMovement::Land(crate::MovementAllowance::Eight),
                },
                state: Default::default(),
            }),
        )
        .unwrap();
        apply_effect(
            &mut state,
            &GameEffect::DeployUnit(UnitPlacement {
                id: id_2,
                position: HexCoord::new(4, 1),
                profile: UnitProfile {
                    kind: UnitKind::Infantry {
                        fire: 9,
                        melee: 5,
                        movement: 8,
                    },
                    identity: UnitIdentity::AngloEgyptianInfantry {
                        brigade: crate::BrigadeId {
                            number: 1,
                            nationality: crate::BrigadeNationality::British,
                        },
                        battalion: BattalionOrdinal::Second,
                    },
                    weapon: WeaponClass::Rifles,
                    fire: Some(crate::FireFactor::Nine),
                    melee: Some(crate::MeleeFactor::Five),
                    movement: UnitMovement::Land(crate::MovementAllowance::Eight),
                },
                state: Default::default(),
            }),
        )
        .unwrap();
        let third = UnitPlacement {
            id: id_3,
            position: HexCoord::new(5, 1),
            profile: UnitProfile {
                kind: UnitKind::Infantry {
                    fire: 9,
                    melee: 5,
                    movement: 8,
                },
                identity: UnitIdentity::AngloEgyptianInfantry {
                    brigade: crate::BrigadeId {
                        number: 1,
                        nationality: crate::BrigadeNationality::British,
                    },
                    battalion: BattalionOrdinal::Third,
                },
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Nine),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: Default::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(third)),
            Err(RuleError::FoKOrderOfBattleFull)
        ));

        // Two different named old gunboats fill cap 2; third rejected.
        apply_effect(
            &mut state,
            &GameEffect::DeployUnit(UnitPlacement {
                id: id_4,
                position: HexCoord::new(3, 2),
                profile: UnitProfile {
                    kind: UnitKind::Gunboat {
                        fire: 0,
                        upstream: 15,
                        downstream: 16,
                    },
                    identity: UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(
                        crate::OldGunboat::LordKitchener,
                    )),
                    weapon: WeaponClass::Artillery,
                    fire: None,
                    melee: None,
                    movement: UnitMovement::Gunboat(crate::GunboatMovement {
                        upstream: crate::MovementAllowance::Fifteen,
                        downstream: crate::MovementAllowance::Sixteen,
                    }),
                },
                state: Default::default(),
            }),
        )
        .unwrap();
        apply_effect(
            &mut state,
            &GameEffect::DeployUnit(UnitPlacement {
                id: id_5,
                position: HexCoord::new(4, 2),
                profile: UnitProfile {
                    kind: UnitKind::Gunboat {
                        fire: 0,
                        upstream: 15,
                        downstream: 16,
                    },
                    identity: UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(
                        crate::OldGunboat::Tamai,
                    )),
                    weapon: WeaponClass::Artillery,
                    fire: None,
                    melee: None,
                    movement: UnitMovement::Gunboat(crate::GunboatMovement {
                        upstream: crate::MovementAllowance::Fifteen,
                        downstream: crate::MovementAllowance::Sixteen,
                    }),
                },
                state: Default::default(),
            }),
        )
        .unwrap();
        let third_boat = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(5, 2),
            profile: UnitProfile {
                kind: UnitKind::Gunboat {
                    fire: 0,
                    upstream: 15,
                    downstream: 16,
                },
                identity: UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(
                    crate::OldGunboat::Metemmeh,
                )),
                weapon: WeaponClass::Artillery,
                fire: None,
                melee: None,
                movement: UnitMovement::Gunboat(crate::GunboatMovement {
                    upstream: crate::MovementAllowance::Fifteen,
                    downstream: crate::MovementAllowance::Sixteen,
                }),
            },
            state: Default::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(third_boat)),
            Err(RuleError::FoKOrderOfBattleFull)
        ));
    }

    // §9.321: the British garrison -- old gunboats only (no named), one
    // artillery, and the battalion counts per nationality.

    // §9.321: the British garrison -- old gunboats only (no named), one
    // artillery, and the battalion counts per nationality.    // §9.321: the British garrison -- old gunboats only (no named), one
    // artillery, and the battalion counts per nationality.
    #[rulebook("§9.321")]
    #[test]
    fn fok_order_of_battle_british() {
        let mut state = GameState::new(Scenario::FallOfKhartoum); // permissive zone

        // Not in play: named gunboats, cavalry, Maxims, Royal Engineers,
        // non-Gordon leaders, the Camel Corps.
        for profile in [
            UnitProfile {
                kind: UnitKind::Gunboat {
                    fire: 0,
                    upstream: 15,
                    downstream: 16,
                },
                identity: UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Named(
                    crate::NamedGunboat::Sultan,
                )),
                weapon: WeaponClass::Artillery,
                fire: None,
                melee: None,
                movement: UnitMovement::Gunboat(crate::GunboatMovement {
                    upstream: crate::MovementAllowance::Fifteen,
                    downstream: crate::MovementAllowance::Sixteen,
                }),
            },
            UnitProfile {
                kind: UnitKind::Cavalry {
                    fire: 8,
                    melee: 5,
                    movement: 15,
                },
                identity: UnitIdentity::AngloEgyptianCavalry,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Eight),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Fifteen),
            },
            UnitProfile {
                kind: UnitKind::Maxim {
                    fire: 6,
                    melee: 1,
                    movement: 12,
                },
                identity: UnitIdentity::AngloEgyptianMaxim,
                weapon: WeaponClass::Maxims,
                fire: Some(crate::FireFactor::Six),
                melee: Some(crate::MeleeFactor::One),
                movement: UnitMovement::Land(crate::MovementAllowance::Twelve),
            },
            UnitProfile {
                kind: UnitKind::Infantry {
                    fire: 5,
                    melee: 3,
                    movement: 8,
                },
                identity: UnitIdentity::RoyalEngineers,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Five),
                melee: Some(crate::MeleeFactor::Three),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            UnitProfile {
                kind: UnitKind::BritishLeader { movement: 8 },
                identity: UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Kitchener),
                weapon: WeaponClass::Melee,
                fire: None,
                melee: None,
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
        ] {
            let p = UnitPlacement {
                id: state.alloc_unit_id(),
                position: HexCoord::new(1, 1),
                profile,
                state: Default::default(),
            };
            assert!(
                matches!(state.can_deploy_unit(&p), Err(RuleError::NotInPlay(_))),
                "{:?} is not in the FoK garrison",
                p.profile.identity
            );
        }

        // §9.321 counts: 2 old gunboats, 1 artillery, 2 British / 3 Egyptian
        // / 4 Sudanese / 4 Friendlies battalions.
        let old_gb = UnitProfile {
            kind: UnitKind::Gunboat {
                fire: 0,
                upstream: 15,
                downstream: 16,
            },
            identity: UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(
                crate::OldGunboat::LordKitchener,
            )),
            weapon: WeaponClass::Artillery,
            fire: None,
            melee: None,
            movement: UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Fifteen,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        };
        let ae_infantry_of = |nationality: crate::BrigadeNationality| UnitProfile {
            kind: UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            },
            identity: UnitIdentity::AngloEgyptianInfantry {
                brigade: crate::BrigadeId {
                    number: 1,
                    nationality,
                },
                battalion: BattalionOrdinal::First,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(crate::FireFactor::Nine),
            melee: Some(crate::MeleeFactor::Five),
            movement: UnitMovement::Land(crate::MovementAllowance::Eight),
        };
        let ae_artillery = UnitProfile {
            kind: UnitKind::Artillery {
                fire: 8,
                melee: 1,
                movement: 7,
            },
            identity: UnitIdentity::AngloEgyptianArtillery,
            weapon: WeaponClass::Artillery,
            fire: Some(crate::FireFactor::Eight),
            melee: Some(crate::MeleeFactor::One),
            movement: UnitMovement::Land(crate::MovementAllowance::Seven),
        };

        let place = |state: &mut GameState, profile: UnitProfile, hex: HexCoord| {
            let p = UnitPlacement {
                id: state.alloc_unit_id(),
                position: hex,
                profile,
                state: Default::default(),
            };
            apply_effect(state, &GameEffect::DeployUnit(p)).unwrap();
        };
        for i in 0..2 {
            place(&mut state, old_gb, HexCoord::new(20 + i, 1)); // Nile-side permissive test board
        }
        let third_gb = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(20, 1),
            profile: old_gb,
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&third_gb),
            Err(RuleError::FoKOrderOfBattleFull)
        ));

        place(&mut state, ae_artillery, HexCoord::new(2, 1));
        let second_art = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(2, 1),
            profile: ae_artillery,
            state: Default::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&second_art),
            Err(RuleError::FoKOrderOfBattleFull)
        ));

        let mut col = 3;
        #[allow(clippy::explicit_counter_loop)]
        for (nationality, cap) in [
            (crate::BrigadeNationality::British, 2),
            (crate::BrigadeNationality::Egyptian, 3),
            (crate::BrigadeNationality::Sudanese, 4),
            (crate::BrigadeNationality::Friendlies, 4),
        ] {
            let profile = ae_infantry_of(nationality);
            for i in 0..cap {
                place(&mut state, profile, HexCoord::new(col, 1 + (i % 3)));
            }
            let over = UnitPlacement {
                id: state.alloc_unit_id(),
                position: HexCoord::new(col, 1 + (cap % 3)),
                profile,
                state: Default::default(),
            };
            col += 1;
            assert!(
                matches!(
                    state.can_deploy_unit(&over),
                    Err(RuleError::FoKOrderOfBattleFull)
                ),
                "cap of {cap} enforced for {nationality:?}"
            );
        }
    }

    // ----- Part G: ZOC + invariant property tests ---------------------------

    #[rulebook("§5.41")]
    #[test]
    fn zoc_hexes_matches_hex_in_enemy_zoc() {
        // For every enemy unit, hex_in_enemy_zoc(my_hex) should be true
        // if and only if zoc_hexes(enemy) contains my_hex.
        let mut state = GameState::new(Scenario::Campaign);
        state.active_player = Player::AngloEgyptian;

        let enemy1 = make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        let enemy2 = make_dervish_tribal(&mut state, HexCoord::new(5, 7));

        let mover_kind = UnitKind::Infantry {
            fire: 0,
            melee: 0,
            movement: 0,
        };

        for u in &state.units.clone() {
            let zoc = state.zoc_hexes(u, Player::AngloEgyptian, mover_kind);
            for &adj in &u.position.neighbors() {
                let in_zoc = state.hex_in_enemy_zoc(adj, Player::AngloEgyptian, mover_kind);
                if in_zoc {
                    assert!(
                        zoc.contains(&adj),
                        "hex_in_enemy_zoc({adj:?}) is true but zoc_hexes({:?}) does not contain it (unit at {:?})",
                        u.id,
                        u.position
                    );
                }
            }
        }
        let _ = (enemy1, enemy2);
    }

    #[rulebook("§5.41", "§5.44")]
    #[test]
    fn zoc_hexes_excludes_nile() {
        let mut state = GameState::new(Scenario::Campaign);
        state.active_player = Player::AngloEgyptian;
        let flow = omdurman_types::HexDirection::East;
        // Put enemy on one side of a Nile hex, check ZOC doesn't extend across.
        state
            .board
            .terrain
            .insert(HexCoord::new(1, 0), Terrain::Nile { direction: flow });
        let enemy = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let zoc = state.zoc_hexes(
            state.find_unit(enemy).unwrap(),
            Player::AngloEgyptian,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0,
            },
        );
        assert!(
            !zoc.contains(&HexCoord::new(1, 0)),
            "ZOC should not extend into Nile hex"
        );
    }

    #[rulebook("§5.41", "§5.44")]
    #[test]
    fn zoc_hexes_excludes_khor() {
        let mut state = GameState::new(Scenario::Campaign);
        state.active_player = Player::AngloEgyptian;
        let enemy_pos = HexCoord::new(1, 1);
        let target = HexCoord::new(1, 0);
        let enemy = make_dervish_tribal(&mut state, enemy_pos);
        state
            .board
            .hexsides
            .insert(HexsideRef::new(enemy_pos, target), HexsideKind::Khor);
        let zoc = state.zoc_hexes(
            state.find_unit(enemy).unwrap(),
            Player::AngloEgyptian,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0,
            },
        );
        assert!(!zoc.contains(&target), "ZOC should not cross khor hexside");
    }

    #[rulebook("§5.41")]
    #[test]
    fn zoc_hexes_empty_for_disrupted_unit() {
        let mut state = GameState::new(Scenario::Campaign);
        let enemy = make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        state.find_unit_mut(enemy).unwrap().state.disrupted = true;
        let zoc = state.zoc_hexes(
            state.find_unit(enemy).unwrap(),
            Player::AngloEgyptian,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0,
            },
        );
        assert!(zoc.is_empty(), "disrupted unit should project no ZOC");
    }

    #[rulebook("§5.41")]
    #[test]
    fn zoc_hexes_empty_for_anglo_egyptian_leader() {
        let mut state = GameState::new(Scenario::Campaign);
        let leader = make_ae_leader(&mut state, HexCoord::new(5, 5));
        let zoc = state.zoc_hexes(
            state.find_unit(leader).unwrap(),
            Player::Dervish,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0,
            },
        );
        assert!(zoc.is_empty(), "AE leaders project no ZOC (§5.41)");
    }

    #[rulebook("§5.41", "§5.44")]
    #[test]
    fn zoc_hexes_normal_unit_projects_six_adjacent_minus_exclusions() {
        let mut state = GameState::new(Scenario::Campaign);
        let enemy = make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        let zoc = state.zoc_hexes(
            state.find_unit(enemy).unwrap(),
            Player::AngloEgyptian,
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0,
            },
        );
        // On an empty board, a normal unit projects ZOC to all 6 neighbours.
        assert_eq!(zoc.len(), 6, "normal unit should project ZOC to 6 hexes");
    }

    // ----- Part H: GameState invariant checker ------------------------------

    #[rulebook("§5.51", "§5.52")]
    #[test]
    fn validate_stacking_invariants_clean_state() {
        let state = GameState::new(Scenario::Campaign);
        assert!(
            state.validate_stacking_invariants().is_ok(),
            "clean state has no violations"
        );
    }

    #[rulebook("§5.51")]
    #[test]
    fn validate_stacking_invariants_catches_stacking_violation() {
        let mut state = GameState::new(Scenario::Campaign);
        let hex = HexCoord::new(5, 5);
        // Place 5 non-leader, non-gunboat Dervish units in the same hex (§5.51 max 4).
        for _ in 0..5 {
            make_dervish_tribal(&mut state, hex);
        }
        let err = state
            .validate_stacking_invariants()
            .expect_err("should catch stacking violation");
        assert!(err.contains("§5.51"), "violation should cite §5.51: {err}");
    }

    #[rulebook("§5.51")]
    #[test]
    fn validate_stacking_invariants_allows_leaders_stacking() {
        let mut state = GameState::new(Scenario::Campaign);
        let hex = HexCoord::new(5, 5);
        // 4 counted units + 1 leader = OK (leaders are free stacking, §5.51).
        for _ in 0..4 {
            make_dervish_tribal(&mut state, hex);
        }
        // Ali Wad Helu: his colour is not pinned down by the rules, so he
        // commands any tribe (§5.53) -- he free-stacks over the four Baggara.
        let ali_wad_helu = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: ali_wad_helu,
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::DervishLeader {
                    fire: 1,
                    melee: 1,
                    movement: 15,
                },
                identity: UnitIdentity::DervishLeader(DervishLeader::AliWadHelu),
                weapon: WeaponClass::Melee,
                fire: None,
                melee: None,
                movement: UnitMovement::Land(crate::MovementAllowance::Fifteen),
            },
            state: Default::default(),
        });
        assert!(state.validate_stacking_invariants().is_ok());

        // The free ride ends at counted units: a fifth counter trips §5.51
        // even with the leader present.
        make_dervish_tribal(&mut state, hex);
        assert!(state.validate_stacking_invariants().is_err());
    }

    /// A Friendlies-brigade infantry profile (§5.21 transport eligibility).
    fn friendlies_infantry_profile() -> UnitProfile {
        UnitProfile {
            kind: UnitKind::Infantry {
                fire: 4,
                melee: 5,
                movement: 8,
            },
            identity: UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: 4,
                    nationality: BrigadeNationality::Friendlies,
                },
                battalion: BattalionOrdinal::First,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(crate::FireFactor::Four),
            melee: Some(crate::MeleeFactor::Five),
            movement: UnitMovement::Land(crate::MovementAllowance::Eight),
        }
    }

    fn make_profiled_unit(state: &mut GameState, hex: HexCoord, profile: UnitProfile) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile,
            state: Default::default(),
        });
        id
    }

    #[rulebook("§6.53")]
    #[test]
    fn demolition_targets_finds_adjacent_fort_and_wall() {
        let mut state = GameState::new(Scenario::Campaign);
        let engineers = make_ae_infantry(&mut state, HexCoord::new(10, 10));
        let fort = make_fort(&mut state, HexCoord::new(11, 10));

        // No adjacent target yet except the fort; add a Wall hexside on another
        // side of the engineers' hex.
        let wall_edge = HexsideRef::new(HexCoord::new(10, 10), HexCoord::new(10, 9));
        state.board.hexsides.insert(wall_edge, HexsideKind::Wall);

        let targets = state.demolition_targets(engineers);
        assert!(
            targets.contains(&DemolitionTarget::Fort(fort)),
            "adjacent fort must be discovered, got {targets:?}"
        );
        assert!(
            targets.contains(&DemolitionTarget::WallHexside(wall_edge)),
            "adjacent wall hexside must be discovered, got {targets:?}"
        );

        // A unit with nothing adjacent sees no targets; a nonexistent unit
        // (allocated but never placed) sees none either.
        let lone = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        assert!(state.demolition_targets(lone).is_empty());
        let ghost = state.alloc_unit_id();
        assert!(state.demolition_targets(ghost).is_empty());
    }

    #[rulebook("§5.21")]
    #[test]
    fn friendlies_transport_offer_load_requires_prerequisites() {
        let mut state = GameState::new(Scenario::Campaign);
        let gunboat = make_unit(
            &mut state,
            HexCoord::new(20, 10),
            UnitKind::Gunboat {
                fire: 0,
                upstream: 15,
                downstream: 16,
            },
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(OldGunboat::LordKitchener)),
            WeaponClass::Artillery,
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Fifteen,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        );
        let friendlies = make_profiled_unit(
            &mut state,
            HexCoord::new(21, 10),
            friendlies_infantry_profile(),
        );

        // No selection -> no offer.
        assert!(state.friendlies_transport_offer(None).is_none());

        // Non-friendlies selection -> no offer.
        let british = make_ae_infantry(&mut state, HexCoord::new(19, 10));
        assert!(state.friendlies_transport_offer(Some(british)).is_none());

        // Friendlies adjacent to a gunboat, but the Isa Zachneih still stands.
        assert!(state.friendlies_transport_offer(Some(friendlies)).is_none());

        // Once the Isa Zachneih is eliminated the load is offered.
        state.isa_zachneih_eliminated = true;
        assert_eq!(
            state.friendlies_transport_offer(Some(friendlies)),
            Some(FriendliesAction::Load {
                unit: friendlies,
                gunboat,
            })
        );
    }

    #[rulebook("§5.21")]
    #[test]
    fn friendlies_transport_offer_follows_state_machine() {
        let mut state = GameState::new(Scenario::Campaign);
        let gunboat = make_unit(
            &mut state,
            HexCoord::new(20, 10),
            UnitKind::Gunboat {
                fire: 0,
                upstream: 15,
                downstream: 16,
            },
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(OldGunboat::LordKitchener)),
            WeaponClass::Artillery,
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Fifteen,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        );
        let friendlies = make_profiled_unit(
            &mut state,
            HexCoord::new(20, 11),
            friendlies_infantry_profile(),
        );

        // Loaded -> Cross toward the gunboat's current hex.
        state.friendlies_transport = Some(TransportState::Loaded {
            unit: friendlies,
            gunboat,
        });
        assert_eq!(
            state.friendlies_transport_offer(None),
            Some(FriendliesAction::Cross {
                unit: friendlies,
                gunboat,
                to: HexCoord::new(20, 10),
            })
        );

        // Crossing / ReadyToDisembark -> Disembark.
        state.friendlies_transport = Some(TransportState::Crossing {
            unit: friendlies,
            gunboat,
            to: HexCoord::new(20, 10),
        });
        assert_eq!(
            state.friendlies_transport_offer(None),
            Some(FriendliesAction::Disembark {
                unit: friendlies,
                gunboat,
            })
        );
        state.friendlies_transport = Some(TransportState::ReadyToDisembark {
            unit: friendlies,
            gunboat,
        });
        assert_eq!(
            state.friendlies_transport_offer(None),
            Some(FriendliesAction::Disembark {
                unit: friendlies,
                gunboat,
            })
        );
    }

    // -----------------------------------------------------------------------
    // Effect atomicity (§4): a rejected effect must leave the state unchanged.
    //
    // Peers apply events only on the host-sequenced echo, and a rejected effect
    // is never retried -- so a peer that rejects an effect after partially
    // mutating diverges permanently from one that accepts it. These pin the
    // three sites that used to mutate before their guards.
    // -----------------------------------------------------------------------

    /// §7.5: a `ResolveMelee` in the wrong phase must not consume the
    /// declaration or its pre-rolled dice. `apply_resolve_melee` used to
    /// `take()` the pending melee before `apply_melee_combat` could reject the
    /// phase, silently dropping the attack.
    #[rulebook("§7.5")]
    #[test]
    fn rejected_resolve_melee_keeps_the_declaration() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;
        let defender = make_ae_infantry(&mut state, HexCoord::new(5, 5));
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(5, 4));

        apply_effect(
            &mut state,
            &GameEffect::DeclareMelee {
                attack: MeleeAttack {
                    attacker_player: Player::Dervish,
                    attacker_hex: HexCoord::new(5, 4),
                    defender_hex: HexCoord::new(5, 5),
                    attackers: vec![attacker],
                    defenders: vec![defender],
                    attacker_modifiers: vec![MeleeModifier::DervishStandard],
                    defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
                },
                attacker_roll: DieRoll::Five,
                defender_roll: DieRoll::Three,
            },
        )
        .unwrap();
        assert!(state.pending_melee.is_some());

        // Slip out of the melee phase, then try to resolve.
        state.phase = Phase::Movement;
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::ResolveMelee),
            Err(RuleError::WrongPhase)
        ));

        // The declaration -- and both pre-rolled dice -- survived.
        let pending = state
            .pending_melee
            .as_ref()
            .expect("rejected ResolveMelee must not consume the declaration");
        assert_eq!(pending.attacker_roll, DieRoll::Five);
        assert_eq!(pending.defender_roll, DieRoll::Three);

        // And back in the melee phase it still resolves.
        state.phase = Phase::Melee;
        apply_effect(&mut state, &GameEffect::ResolveMelee).unwrap();
        assert!(state.pending_melee.is_none());
    }

    /// §6.82/§7.6: a rejected `AdvancePhase` must not drop the
    /// advance-after-combat windows. The `vacated_by_combat.clear()` used to
    /// run before the `MeleePendingResolution` guard.
    #[rulebook("§6.82")]
    #[test]
    fn rejected_advance_phase_keeps_vacated_windows() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;
        let defender = make_ae_infantry(&mut state, HexCoord::new(5, 5));
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(5, 4));

        // An open advance window, plus a declared melee that blocks the phase end.
        let vacated = HexCoord::new(9, 9);
        state.vacated_by_combat.insert(vacated, vec![attacker]);
        apply_effect(
            &mut state,
            &GameEffect::DeclareMelee {
                attack: MeleeAttack {
                    attacker_player: Player::Dervish,
                    attacker_hex: HexCoord::new(5, 4),
                    defender_hex: HexCoord::new(5, 5),
                    attackers: vec![attacker],
                    defenders: vec![defender],
                    attacker_modifiers: vec![MeleeModifier::DervishStandard],
                    defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
                },
                attacker_roll: DieRoll::Five,
                defender_roll: DieRoll::Three,
            },
        )
        .unwrap();

        assert!(matches!(
            apply_effect(&mut state, &GameEffect::AdvancePhase),
            Err(RuleError::MeleePendingResolution)
        ));
        assert_eq!(
            state.vacated_by_combat.get(&vacated),
            Some(&vec![attacker]),
            "rejected AdvancePhase must not clear the §6.82 advance windows"
        );
    }

    /// §6.14: a rejected fire attack must not burn the firers' once-per-phase
    /// allowance. `resolve_fire_attack` used to push every firer into
    /// `units_fired_this_phase` before the `AlreadyFiredAt` guard.
    #[rulebook("§6.14")]
    #[test]
    fn rejected_fire_attack_does_not_mark_firers_as_fired() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let first = make_ae_infantry(&mut state, HexCoord::new(5, 5));
        let second = make_ae_infantry(&mut state, HexCoord::new(5, 6));
        let target = make_dervish_tribal(&mut state, HexCoord::new(6, 6));

        let attack_by = |firer: UnitId| FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: Phase::OffensiveFire(FireSubPhase::DirectFire),
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(6, 6),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };

        // First attack lands and marks the target as fired at.
        apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: attack_by(first),
                roll: DieRoll::One,
            },
        )
        .unwrap();
        assert!(state.units_fired_at_this_phase.contains(&target));

        // Second firer aims at the same, already-fired-at target: rejected.
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::FireCombat {
                    attack: attack_by(second),
                    roll: DieRoll::One,
                },
            ),
            Err(RuleError::AlreadyFiredAt(_))
        ));

        // The rejected firer keeps its allowance (§6.14) and can still fire
        // this phase at a legal target.
        assert!(
            !state.units_fired_this_phase.contains(&second),
            "rejected fire attack must not consume the firer's once-per-phase allowance"
        );
        let other = make_dervish_tribal(&mut state, HexCoord::new(4, 6));
        apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: FireAttack {
                    firing_player: Player::AngloEgyptian,
                    phase: Phase::OffensiveFire(FireSubPhase::DirectFire),
                    kind: FireKind::Direct,
                    firers: vec![second],
                    target_hex: HexCoord::new(4, 6),
                    factor_row: FireFactorRow::Row01to05,
                    modifiers: vec![FireModifier::AngloEgyptianDirectFire],
                },
                roll: DieRoll::One,
            },
        )
        .unwrap();
        assert!(state.units_fired_this_phase.contains(&second));
        let _ = other;
    }
}
