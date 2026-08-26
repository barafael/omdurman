//! Strategy doctrine loader: reads the checked-in corpus under
//! `docs/strategy/*.md` and assembles the per-side LLM brief.
//!
//! The corpus is a set of standalone markdown files of tactical advice, each
//! item citing the rulebook section it relies on (validated against
//! `docs/traceability.toml` by `tests/strategy_corpus.rs`). A side's brief is
//! the common doctrine plus its faction file (plus the Fall-of-Khartoum file
//! for that scenario). The brief is prepended to the advisor system prompt via
//! [`AgentStrategy::LlmAdvised`].

use std::fs;
use std::path::{Path, PathBuf};

use omdurman_types::{Player, Scenario};

/// Where the doctrine corpus lives, relative to the workspace root.
fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs")
        .join("strategy")
}

fn read(name: &str) -> Option<String> {
    fs::read_to_string(corpus_dir().join(name)).ok()
}

/// Assemble the strategic brief for `player` in `scenario` from the corpus.
/// Missing files degrade gracefully (empty sections are skipped), so the bot
/// still works on a partial checkout.
pub fn doctrine_brief(player: Player, scenario: Scenario) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(common) = read("common_doctrine.md") {
        parts.push(common);
    }

    let faction = match player {
        Player::AngloEgyptian => "anglo_egyptian_doctrine.md",
        Player::Dervish => "dervish_doctrine.md",
    };
    if let Some(text) = read(faction) {
        parts.push(text);
    }

    if matches!(scenario, Scenario::FallOfKhartoum)
        && let Some(fok) = read("fall_of_khartoum_doctrine.md")
    {
        parts.push(fok);
    }

    parts.join("\n\n---\n\n")
}

/// The list of corpus files that should exist, in load order.
pub fn corpus_files() -> &'static [&'static str] {
    &[
        "common_doctrine.md",
        "anglo_egyptian_doctrine.md",
        "dervish_doctrine.md",
        "fall_of_khartoum_doctrine.md",
    ]
}

/// The storm-the-Palace brief: the standard Dervish FoK doctrine plus
/// explicit override orders for an all-out assault replay (the `storm`
/// CLI preset). Not part of the corpus — this is a scenario script, not
/// general doctrine, so it does not affect other LLM runs.
pub fn storm_brief(scenario: Scenario) -> String {
    let mut brief = doctrine_brief(Player::Dervish, scenario);
    brief.push_str(
        "\n\n---\n\n\
STORM ORDERS — the Mahdi has decreed: KHARTOUM MUST FALL AND GORDON MUST DIE. \
Where these conflict with the doctrine above, the storm orders win.\n\n\
1. OBJECTIVE. The objective hex is GORDON's own hex — find \
AngloEgyptianLeader(Gordon) in the enemy unit list each turn; he never moves \
(the Palace, \u{a7}9.346). The game ends INSTANTLY in a Dervish victory the \
moment any Dervish unit occupies his hex. Everything else is secondary.\n\n\
2. THE CLOCK. This scenario lasts only 8 turns; you cannot win a slow game. \
From turn 1 every unit marches on the Palace in every movement phase \
(\u{a7}5.11) — spend all movement points closing distance. Do NOT detach \
flank guards and do NOT hold reserves.\n\n\
3. PLAN SHAPE. Your plan is matched by INTENT: a planned action stays in \
force until it is applied or becomes illegal, even as the candidate list \
shifts. Plan LONG (30+ entries) and lead with movement: every MoveUnit that \
steps toward the Palace first, then melees, then fire. Do NOT include \
AdvancePhase — it is ignored in plans; phases end on their own when you run \
out of actions. Units you do not mention follow this brief automatically.\
\n\n\
4. COMBAT. When adjacent, melee — the Dervish melee modifiers are superior. \
NEVER RetreatBeforeMelee (\u{a7}7.5): dying at the walls is cheaper than \
losing time. Take AdvanceAfterCombat (\u{a7}6.82/\u{a7}7.6) whenever offered — \
it converts kills into forward progress.\n\n\
5. WALLS. The Palace sits inside the city walls. Bring the artillery up \
behind the infantry and breach the wall hexsides on your approach axis \
(\u{a7}6.63), then pour the tribes through the gap.\n\n\
6. SETUP. Deploy forward and tight in the part of the deployment zone \
closest to the city (\u{a7}9.322): stack each tribe with its leader \
(\u{a7}5.53) so whole commands move as a fist.",
    );
    brief
}

/// The fortress brief: the standard Anglo-Egyptian FoK doctrine plus
/// override orders for a defensive replay (the `siege` CLI preset). The
/// garrison spreads inside the walled city, prioritises maxim-gun
/// strongpoints, and holds ground rather than counterattacking.
pub fn fortress_brief(scenario: Scenario) -> String {
    let mut brief = doctrine_brief(Player::AngloEgyptian, scenario);
    brief.push_str(
        "\n\n---\n\n\
FORTRESS ORDERS — General Gordon has decreed: KHARTOUM WILL BE HELD. \
Where these conflict with the doctrine above, the fortress orders win.\n\n\
1. OBJECTIVE. DEFEND the walled city. You win by surviving all 8 turns \
with GORDON alive at the Palace (\u{a7}9.346). Every turn the Dervish do \
NOT occupy the Palace hex is a turn closer to an Anglo-Egyptian victory. \
Do NOT seek decisive battle outside the walls.\n\n\
2. DEPLOYMENT PRIORITY — GATE GARRISONS. The Dervish approach from the \
south and must pass through one of the three southern gates (Buri, \
Messalamia, Kalakla) to enter the walled city. However, the city wall has \
a GAP on the western end (the washed-away wall section, \u{a7}2.1) — this \
is the only approach that bypasses the gates entirely. You MUST cover both:\n\n\
   a) Gate garrisons: deploy your strongest stacks AT each southern gate \
hex (one hex inside the south wall, directly behind the gate hexside). A \
stack of 4 units (\u{a7}5.51) at each gate blocks the Dervish advance and \
forces them to fight through your garrison before reaching the interior.\n\n\
   b) Western gap guard: place a strong stack (Maxim + infantry) on a hex \
inside the city that covers the western gap. The Dervish will try to flank \
through this gap to reach the Palace without breaching the walls — do NOT \
let them through.\n\n\
   c) Gordon's bodyguard: stack at least one infantry section with Gordon \
at the Palace hex (\u{a7}9.346, he cannot move in FoK). He is the last \
line — if the Dervish reach him, the game ends.\n\n\
3. MAXIM GUNS. Place Maxims at the gate garrisons and the western gap. \
A Maxim behind a wall hexside fires with a +2 wall bonus (\u{a7}6.64). \
Stack each Maxim with infantry for protection — a lone Maxim is overrun \
in passing (\u{a7}6.51). Once placed, never move a Maxim voluntarily.\n\n\
4. PLAN SHAPE. Your plan is matched by INTENT: a planned action stays in \
force until it is applied or becomes illegal, even as the candidate list \
shifts. Plan LONG (30+ entries). Lead with DeployUnit to place Maxims and \
infantry at gate garrisons during setup, then MoveUnit to spread into \
defensive positions. Do NOT include AdvancePhase — it is ignored in plans. \
Units you do not mention follow this brief automatically.\n\n\
5. FIRE COMBAT. When Dervish units are within range, open fire from your \
Maxim strongpoints (\u{a7}6.64). Prioritise fire over melee — the \
Anglo-Egyptian fire modifiers are superior. Do NOT pursue retreating \
Dervish units outside the walls.\n\n\
6. HOLD. Never RetreatBeforeMelee (\u{a7}7.5) from a wall hex — the wall \
bonus makes the defence profitable. Accept losses at the gates rather than \
yielding ground. The Palace must never fall.",
    );
    brief
}

/// The horde brief: the standard Dervish FoK doctrine plus override orders
/// for an all-out assault that must eventually overcome a fortified defence
/// (the `siege` CLI preset). More aggressive than `storm_brief` — the
/// horde accepts enormous casualties to breach the walls.
pub fn horde_brief(scenario: Scenario) -> String {
    let mut brief = doctrine_brief(Player::Dervish, scenario);
    brief.push_str(
        "\n\n---\n\n\
HORDE ORDERS — the Mahdi has decreed: OVERWHELM THE FORTRESS AND CLAIM \
KHARTOUM. Where these conflict with the doctrine above, the horde orders \
win.\n\n\
1. OBJECTIVE. The objective hex is GORDON's own hex — find \
AngloEgyptianLeader(Gordon) in the enemy unit list each turn; he never \
moves (the Palace, \u{a7}9.346). The game ends INSTANTLY in a Dervish \
victory the moment any Dervish unit occupies his hex.\n\n\
2. THE CLOCK. This scenario lasts only 8 turns. The enemy has a strong \
defensive position behind walled hexsides with Maxim guns at the gates. \
You MUST breach the walls and storm the Palace within the time limit.\n\n\
3. APPROACH AND BREACH. The city is walled on the south and west sides. \
The three southern gates (Buri, Messalamia, Kalakla) are the main entry \
points. The western wall has a gap (washed-away section, \u{a7}2.1) but \
the enemy WILL guard it with a strong stack — do NOT waste your assault \
trying to sneak through. Instead, concentrate on the southern gates.\n\n\
   a) Concentrate your assault on ONE southern gate — do NOT split across \
all three. A concentrated force overwhelms the garrison at that gate while \
a dispersed force is defeated in detail.\n\n\
   b) Bring infantry to the base of the wall first, then commit artillery \
to create a breach (\u{a7}6.63). Once a gate is breached, pour ALL units \
through the gap toward the Palace.\n\n\
   c) The Palace is deep inside the walled city, roughly 8 hexes north of \
the south wall. Once inside, march straight for it — every movement \
point spent on anything other than reaching the Palace is wasted.\n\n\
4. PLAN SHAPE. Your plan is matched by INTENT: a planned action stays in \
force until it is applied or becomes illegal, even as the candidate list \
shifts. Plan LONG (30+ entries). Lead with movement toward the \
concentrated gate, then BreachWall at the gate hexsides, then melee \
through the gap, then more movement toward the Palace. Do NOT include \
AdvancePhase — it is ignored in plans; phases end on their own. \
Units you do not mention follow this brief automatically.\n\n\
5. COMBAT. When adjacent, melee — the Dervish melee modifiers are \
superior (\u{a7}7.3). NEVER RetreatBeforeMelee (\u{a7}7.5): the cost of \
retreating and re-approaching exceeds the cost of accepting losses. Take \
AdvanceAfterCombat (\u{a7}6.82/\u{a7}7.6) whenever offered — it \
converts kills into forward progress.\n\n\
6. CASUALTIES. Accept enormous losses. A tribe that is eliminated trading \
for a wall hex or a Maxim position has done its duty. Do not preserve \
units at the cost of momentum.\n\n\
7. SETUP. Deploy forward and tight in the part of the deployment zone \
closest to the city (\u{a7}9.322). Stack each tribe with its leader \
(\u{a7}5.53) so whole commands move as a fist. Artillery and engineers \
deploy behind the first wave, within striking distance of the chosen gate.",
    );
    brief
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_corpus_files_exist_and_are_nonempty() {
        for name in corpus_files() {
            let text = read(name).unwrap_or_else(|| panic!("missing corpus file {name}"));
            assert!(!text.trim().is_empty(), "corpus file {name} is empty");
        }
    }

    #[test]
    fn briefs_are_faction_specific() {
        let ae = doctrine_brief(Player::AngloEgyptian, Scenario::Campaign);
        let derv = doctrine_brief(Player::Dervish, Scenario::Campaign);
        assert!(ae.contains("Anglo-Egyptian"));
        assert!(derv.contains("Dervish"));
        assert_ne!(ae, derv);
    }

    #[test]
    fn fok_brief_includes_the_scenario_delta() {
        let fok = doctrine_brief(Player::Dervish, Scenario::FallOfKhartoum);
        assert!(fok.contains("GORDON"));
    }

    #[test]
    fn fortress_brief_mentions_maxim_guns() {
        let fb = fortress_brief(Scenario::FallOfKhartoum);
        assert!(fb.contains("Maxim"));
        assert!(fb.contains("FORTRESS"));
    }

    #[test]
    fn horde_brief_mentions_overwhelming() {
        let hb = horde_brief(Scenario::FallOfKhartoum);
        assert!(hb.contains("HORDE"));
        assert!(hb.contains("GORDON"));
    }
}
