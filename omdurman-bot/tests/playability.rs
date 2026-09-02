//! Playability cross-check for the Fall of Khartoum two-player UI: every
//! `GameEffect` variant the headless playthrough actually emits must have a
//! clickable or otherwise active path in `omdurman-app`, otherwise a human
//! pair cannot reproduce the game the engine and bot consider legal.
//!
//! The bot's `variant_coverage` (what the engine accepted in real play) is
//! matched against `UI_SUPPORTED` — an explicit, documented allowlist. An
//! effect that starts occurring in play but has no UI caller fails here, and
//! the fix is either wiring the UI or documenting the exclusion with a
//! reason, never silently growing the allowlist.

use std::collections::BTreeSet;

use omdurman_bot::agent::Agents;
use omdurman_bot::{PlayConfig, playthrough};
use omdurman_types::Scenario;

/// Effects with an active caller in `omdurman-app/src`, keyed by the
/// `strum::IntoStaticStr` variant name the playthrough coverage reports.
/// The comment is the clickable path (checked 2026-09; update when the UI
/// moves).
const UI_SUPPORTED: &[(&str, &str)] = &[
    (
        "DeployUnit",
        "setup picker click → GameEvent::PlaceUnit (picker.rs) / auto fixed-placement (scenario_setup.rs)",
    ),
    (
        "RemoveDeployedUnit",
        "setup picker right-click/re-pickup → GameEvent::RemoveUnit (picker.rs)",
    ),
    (
        "ConfirmSetupReady",
        "\"Ready\" button in the setup control section (ui_plugin.rs)",
    ),
    (
        "AdvancePhase",
        "\"End Phase\" button (own turn) / \"Begin battle\" (ui_plugin.rs)",
    ),
    (
        "MoveUnit",
        "select unit → click destination (picker.rs, staged stack moves)",
    ),
    (
        "FireCombat",
        "select battery → click target → allocation panel \"Execute All\" (fire_allocation.rs)",
    ),
    (
        "HowitzerFire",
        "howitzer allocation in the Maxim/Howitzer sub-phase → \"Execute All\" (fire_allocation.rs)",
    ),
    (
        "ArtilleryBreachWall",
        "select artillery in a fire phase → wall buttons in the breach card (ui_plugin.rs artillery_breach_ui)",
    ),
    (
        "DeclareMelee",
        "select melee unit → click adjacent enemy (melee.rs)",
    ),
    (
        "ResolveMelee",
        "attacker's \"Resolve Melee\" button in the reaction panel (melee.rs)",
    ),
    (
        "RetreatBeforeMelee",
        "defender clicks a highlighted retreat hex (retreat.rs)",
    ),
    (
        "AdvanceAfterCombat",
        "click the vacated hex with the advancing unit selected (melee.rs)",
    ),
    (
        "RecoverUnit",
        "automatic in end_player_turn; the engine also accepts it from any peer (no button needed)",
    ),
    (
        "DervishDesertion",
        "\"Confirm desertion\" panel (desertion.rs; Campaign-only, never emitted in FoK)",
    ),
    (
        "Demolition",
        "Royal Engineers \"Commit to Demolition\" card (ui_plugin.rs)",
    ),
    (
        "ResolveDemolition",
        "automatic in end_player_turn (no button needed)",
    ),
    (
        "ConstructZariba",
        "\"Construct Zariba (§5.3)\" hexside buttons (ui_plugin.rs; Campaign/Historical)",
    ),
];

/// Effects deliberately without a UI, with the reason. These must NOT appear
/// in a FoK playthrough's coverage; if one does, the test fails and forces a
/// decision (wire it up or explain why the engine can emit it in FoK play).
const UI_EXCLUDED: &[(&str, &str)] = &[
    (
        "MeleeCombat",
        "legacy single-shot melee; the app uses Declare/Resolve (§7.5)",
    ),
    (
        "PlaceReinforcements",
        "Campaign-only arrival waves; FoK deploys everything in setup",
    ),
    (
        "PlaceMine",
        "§10 river-mines optional rule is Campaign-only",
    ),
    ("PlaceChain", "§10 chain optional rule is Campaign-only"),
    (
        "PlaceZariba",
        "pre-placed Historical zariba, never authorable in-game",
    ),
    (
        "SinkChain",
        "§10 chain-cutting optional rule is Campaign-only",
    ),
    (
        "RiverMine",
        "in-play mine resolution under the Campaign-only optional rule",
    ),
    (
        "DriftGunboat",
        "automatic engines-lost drift; engine has no in-app trigger yet (known gap, §10)",
    ),
    (
        "FriendliesTransport",
        "Campaign gunboat ferry; FoK has no Friendlies counters",
    ),
];

fn ui_status(variant: &str) -> Result<Option<&'static str>, String> {
    if let Some((_, path)) = UI_SUPPORTED.iter().find(|(name, _)| *name == variant) {
        return Ok(Some(path));
    }
    if UI_EXCLUDED.iter().any(|(name, _)| *name == variant) {
        return Ok(None);
    }
    Err(format!(
        "effect `{variant}` is neither in UI_SUPPORTED nor UI_EXCLUDED — \
         wire a UI path for it or document the exclusion"
    ))
}

fn variants_of(effects: &[&'static str]) -> BTreeSet<&'static str> {
    effects.iter().copied().collect()
}

#[test]
fn fok_playthrough_effects_all_have_ui_paths() {
    for seed in [7u64, 42, 2026] {
        let cfg = PlayConfig {
            max_actions_per_phase: 80,
            max_turns: 12,
            keep_out: None,
        };
        let result = futures::executor::block_on(playthrough(
            Scenario::FallOfKhartoum,
            seed,
            cfg,
            Agents::random(),
        ));
        assert!(
            result.actions_taken > 0,
            "seed {seed}: playthrough took no actions"
        );
        assert!(
            result.final_state.game_over || result.actions_taken > 100,
            "seed {seed}: neither finished nor meaningfully progressed"
        );

        for variant in variants_of(&result.variant_coverage) {
            match ui_status(variant) {
                Ok(Some(_)) => {}
                Ok(None) => panic!(
                    "seed {seed}: FoK play emitted `{variant}`, which is UI-excluded as \
                     out-of-FoK-scope — the scenario played something humans cannot"
                ),
                Err(why) => panic!("seed {seed}: {why}"),
            }
        }

        // The setup handoff must complete one of its two legal exits: both
        // factions confirming ready, or `AdvancePhase` straight out of Setup
        // once `setup_complete` holds. Either way the game must not still be
        // deploying when it finished or stalled.
        assert!(
            result.final_state.game_over
                || !matches!(result.final_state.phase, omdurman_rules::Phase::Setup),
            "seed {seed}: still in Setup when the playthrough ended"
        );
    }
}

/// The effects a *played* FoK game is expected to exercise. If the bot stops
/// emitting one of these, either the bot regressed or the UI allowlist above
/// is covering an effect no human will ever need — either way, look.
#[test]
fn fok_playthrough_exercises_the_core_action_set() {
    let cfg = PlayConfig {
        max_actions_per_phase: 80,
        max_turns: 12,
        keep_out: None,
    };
    let result = futures::executor::block_on(playthrough(
        Scenario::FallOfKhartoum,
        42,
        cfg,
        Agents::random(),
    ));
    let seen = variants_of(&result.variant_coverage);
    for expected in [
        "DeployUnit",
        "ConfirmSetupReady",
        "MoveUnit",
        "AdvancePhase",
        "FireCombat",
    ] {
        assert!(seen.contains(expected), "coverage missing `{expected}`");
    }
    // Fire phases must actually be fire phases: every FireCombat-shaped
    // effect is gated on them in the engine, so their presence implies the
    // sub-phase machine ran.
    assert!(
        seen.contains("FireCombat") || seen.contains("ArtilleryBreachWall"),
        "no fire action at all"
    );
}
