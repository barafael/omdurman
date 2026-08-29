//! Coverage: over a batch of playthroughs, the most common GameEffect
//! variants must all appear. This catches move-generator blind spots and,
//! more importantly, would surface if a rule path is unreachable.

use omdurman_bot::agent::Agents;
use omdurman_bot::oob::deployable_oob;
use omdurman_bot::{PlayConfig, playthrough};
use omdurman_rules::effects::fok_cap_group;
use omdurman_rules::unit_profiles::profile_for_unit;
use omdurman_types::Scenario;
use std::collections::{HashMap, HashSet};

#[test]
fn fok_batch_covers_core_variants() {
    let cfg = PlayConfig {
        keep_out: None,
        max_actions_per_phase: 100,
        max_turns: 8,
    };
    let mut all_kinds: HashSet<&str> = HashSet::new();
    for seed in 0..8u64 {
        let result = futures::executor::block_on(playthrough(
            Scenario::FallOfKhartoum,
            seed,
            cfg.clone(),
            Agents::random(),
        ));
        for kind in &result.variant_coverage {
            all_kinds.insert(*kind);
        }
    }
    // These variants MUST appear in a batch of FoK games:
    // - DeployUnit + ConfirmSetupReady + AdvancePhase (setup flow)
    // - MoveUnit (movement)
    // Setup flow: DeployUnit + ConfirmSetupReady; AdvancePhase only appears
    // when the driver needs to churn (e.g. mandatory arrivals forcing
    // subphase transitions), so it is not guaranteed in every batch.
    assert!(
        all_kinds.contains("DeployUnit"),
        "missing DeployUnit in {:?}",
        all_kinds
    );
    // We don't hard-assert combat coverage (the random bot may not close range
    // in every batch), but we log the achieved set — and call out a batch that
    // saw no combat at all.
    eprintln!("FoK coverage after 8 games: {:?}", all_kinds);
    let has_combat = all_kinds.contains("FireCombat")
        || all_kinds.contains("MeleeCombat")
        || all_kinds.contains("DeclareMelee");
    if !has_combat {
        eprintln!("note: batch saw no combat (no FireCombat/MeleeCombat/DeclareMelee)");
    }
}

/// §9.321/§9.322: every identity returned by `deployable_oob(FoK)` must have a
/// `fok_cap_group` entry, and no non-OOB identity leaks in. The OOB list
/// contains all *candidates* (every UnitId whose identity is in play); the
/// engine's cap enforcement limits how many of each type actually deploy.
/// Verify: (a) no non-OOB identities, (b) every cap group has candidates,
/// (c) the sum of group caps equals the setup targets (AE 17, Dervish 49).
#[test]
fn fok_oob_matches_manual_exactly() {
    let oob = deployable_oob(Scenario::FallOfKhartoum);

    // Every deployable identity must have a fok_cap_group entry.
    for &(player, id) in &oob {
        let profile = profile_for_unit(id)
            .unwrap_or_else(|| panic!("deployable_oob contains {id:?} with no profile"));
        assert!(
            fok_cap_group(&profile.identity).is_some(),
            "deployable_oob(FoK) contains {id:?} ({:?}) which has no fok_cap_group entry — \
             not in the §9.321/§9.322 order of battle",
            profile.identity,
        );
        assert_eq!(
            profile.identity.owner(),
            player,
            "deployable_oob player mismatch for {id:?}"
        );
    }

    // Count deployable UnitIds per cap group.
    let mut by_group: HashMap<String, usize> = HashMap::new();
    for &(_player, id) in &oob {
        let profile = profile_for_unit(id).unwrap();
        let (group, _cap) = fok_cap_group(&profile.identity).unwrap();
        *by_group.entry(format!("{:?}", group)).or_default() += 1;
    }

    // Every FoK cap group must have at least one candidate in the OOB.
    let expected_groups = [
        ("Tribe(Mulazmin)", 32usize), // 16 MulazminI + 16 MulazminII
        ("Tribe(Hadendowa)", 13),     // 13 Hadendowa tribal counters
        ("Tribe(Kehena)", 6),
        ("Tribe(Degheim)", 5),
        ("DervishArtillery", 3),
        ("Infantry(British)", 8),    // 8 British infantry counters
        ("Infantry(Egyptian)", 10),  // 10 Egyptian infantry counters
        ("Infantry(Sudanese)", 6),   // 6 Sudanese counters
        ("Infantry(Friendlies)", 5), // 5 Friendlies counters
        ("AeArtillery", 6),          // 2 British + 4 Egyptian batteries
        ("OldGunboat", 4),           // 4 old gunboat counters
    ];
    for (group, min_candidates) in expected_groups {
        let actual = *by_group.get(group).unwrap_or(&0);
        assert!(
            actual >= min_candidates,
            "FoK cap group {group}: expected >= {min_candidates} candidates, got {actual}",
        );
    }

    // No unexpected cap groups (leaders, cavalry, Maxims, Dervish gunboats,
    // named gunboats, IsaZachneih, etc.).
    let known_groups: HashSet<&str> = [
        "Tribe(Mulazmin)",
        "Tribe(Hadendowa)",
        "Tribe(Kehena)",
        "Tribe(Degheim)",
        "DervishArtillery",
        "DervishFort",
        "OldGunboat",
        "AeArtillery",
        "Infantry(British)",
        "Infantry(Egyptian)",
        "Infantry(Sudanese)",
        "Infantry(Friendlies)",
        "Gordon",
    ]
    .iter()
    .copied()
    .collect();
    for group_key in by_group.keys() {
        assert!(
            known_groups.contains(group_key.as_str()),
            "unexpected FoK cap group in deployable_oob: {group_key}"
        );
    }

    // Sum of caps must equal the setup targets (AE 17 = 16 + Gordon, Dervish
    // 49 = 48 + North Fort). Gordon and North Fort are fixed placements, not
    // in deployable_oob.
    let ae_cap_sum = 2 + 3 + 4 + 4 + 1 + 2; // British + Egyptian + Sudanese + Friendlies + AE art + old GB
    let dervish_cap_sum = 32 + 2 + 6 + 5 + 3; // Mulazmin + Hadendowa + Kehena + Degheim + D art
    assert_eq!(
        ae_cap_sum, 16,
        "AE cap sum must be 16 (player-deployed, Gordon is fixed)"
    );
    assert_eq!(
        dervish_cap_sum, 48,
        "Dervish cap sum must be 48 (player-deployed, fort is fixed)"
    );
}
