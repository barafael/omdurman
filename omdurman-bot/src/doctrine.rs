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

/// The last-stand defender brief (the `laststand` CLI preset): a layered,
/// prolonged defence of Khartoum. The garrison holds in depth — gate
/// garrisons, the western-gap guard, an interior ring, and a bodyguard at
/// GORDON's side — plugging every gap so the assault is ground down layer
/// by layer. The defence cannot win outright; the objective is to hold
/// GORDON alive as long as the clock allows.
pub fn defender_brief(scenario: Scenario) -> String {
    let mut brief = doctrine_brief(Player::AngloEgyptian, scenario);
    brief.push_str(
        "\n\n---\n\n\
LAST-STAND ORDERS — General Gordon has decreed: KHARTOUM WILL BE HELD TO \
THE LAST MAN. Where these conflict with the doctrine above, these orders \
win.\n\n\
1. OBJECTIVE. You cannot win a battle of manoeuvre — you can only make \
the Dervish pay for every hex. You win the campaign by keeping GORDON \
alive (\u{a7}9.346): every turn he lives is a victory turn. Fight a \
defence in DEPTH and never, ever sally out of the walls.\n\n\
2. DEPLOYMENT (setup, \u{a7}9.322) — four layers:\n\n\
   a) GATE GARRISONS: one Maxim + two infantry at each southern gate \
(Buri, Messalamia, Kalakla), on the hex directly behind the gate \
hexside. They project ZOC out through the gate (\u{a7}5.44) and force \
every attacker to stop and fight.\n\n\
   b) WESTERN GAP GUARD: the west wall has a washed-away section \
(\u{a7}2.1) — the only breach-free approach. One Maxim + infantry \
stands on the inside hex covering it.\n\n\
   c) INTERIOR RING: the remaining infantry deploy on a ring of hexes \
around the Palace, one to two hexes out. This is the second line that \
plugs whatever the gates lose.\n\n\
   d) BODYGUARD: GORDON never moves (\u{a7}9.346). Two infantry stay \
stacked on his hex for the entire game.\n\n\
3. MAXIM GUNS. A Maxim behind a wall hexside fires with a +2 bonus \
(\u{a7}6.64). Never move a Maxim once placed; stack infantry with it \
(\u{a7}6.51). Every fire phase, every Maxim fires at the largest \
Dervish stack in range.\n\n\
4. PLUG GAPS. After every Dervish move, look for vacated or lost front \
hexes: move an interior-ring stack into the gap the same movement \
phase. Always keep at least two units adjacent to GORDON — refill the \
ring the moment it thins.\n\n\
5. NEVER RETREAT. Never RetreatBeforeMelee (\u{a7}7.5). A unit that \
dies holding a gate has bought GORDON a turn. Disrupted units recover \
and hold their hex (\u{a7}5.41).\n\n\
6. PLAN SHAPE. Your plan is matched by INTENT: a planned action stays \
in force until applied or illegal. Plan LONG (30+ entries). Lead with \
DeployUnit (garrisons first, then the ring, then the bodyguard), then \
each turn's MoveUnit gap-plugging and FireCombat from the Maxims. Do \
NOT include AdvancePhase — it is ignored in plans. Units you do not \
mention follow these orders automatically.",
    );
    brief
}

/// The last-stand besieger brief (the `laststand` CLI preset): a
/// systematic, paced reduction of Khartoum. The horde masses as tribal \
/// stacks at one gate, breaches the wall, and annihilates the defence \
/// layer by layer before the final assault on the Palace in the closing \
/// turns — a scripted-feel siege rather than a headlong rush.
pub fn besieger_brief(scenario: Scenario) -> String {
    let mut brief = doctrine_brief(Player::Dervish, scenario);
    brief.push_str(
        "\n\n---\n\n\
BESIEGER ORDERS — the Mahdi has decreed: ANNIHILATE THE GARRISON, THEN \
CLAIM THE PALACE. Where these conflict with the doctrine above, these \
orders win.\n\n\
1. OBJECTIVE. The game ends the moment any Dervish unit occupies \
GORDON's hex (\u{a7}9.346) — but the Mahdi demands the garrison's \
destruction FIRST. A rushed Palace is a hollow victory: reduce the \
defence layer by layer, then strike.\n\n\
2. THE PACING (follow it exactly):\n\n\
   T1: Mass the whole army before ONE southern gate (Buri, Messalamia, \
or Kalakla — pick one and stay with it). Artillery moves up to \
breaching range of that gate's wall.\n\n\
   T2: Artillery breaches the wall (\u{a7}6.63). First melees against \
the gate garrison — melee may be made through a gate (\u{a7}7.2).\n\n\
   T3-4: Destroy the gate garrison. Pour the tribes through the breach. \
Melee the second line stack by stack.\n\n\
   T5-6: Clear the city interior. No Dervish unit may end its move \
within two hexes of the Palace before turn 5 — the Mahdi forbids a \
premature rush. Annihilate every field unit outside the bodyguard \
first.\n\n\
   T7-8: THE FINAL ASSAULT. Melee the bodyguard, then take the Palace \
hex. GORDON falls and Khartoum is claimed.\n\n\
3. COHESION — MOVE AS FISTS, NOT AS INDIVIDUALS. A fist = one tribe \
stacked with its leader (\u{a7}5.53). When you plan movement, move \
EVERY unit of a stack to the same next hex, one after another, before \
touching any other stack. The army advances wall by wall of shields, \
never as scattered individuals.\n\n\
4. COMBAT. Melee whenever adjacent (\u{a7}7.3) — the Dervish melee \
modifiers are superior. NEVER RetreatBeforeMelee (\u{a7}7.5). Take \
AdvanceAfterCombat (\u{a7}6.82/\u{a7}7.6) to convert kills into \
ground.\n\n\
5. THE WESTERN GAP. The washed-away west wall section (\u{a7}2.1) is \
surely guarded — do NOT split your assault there. Concentrate on your \
chosen gate; the west is a feint for another day.\n\n\
6. CASUALTIES. Irrelevant. Momentum and annihilation are everything.\n\n\
7. SETUP. Deploy forward and tight nearest the city (\u{a7}9.322), \
each tribe stacked with its leader. Artillery behind the first wave, \
within range of the chosen gate.\n\n\
8. PLAN SHAPE. Your plan is matched by INTENT: a planned action stays \
in force until applied or illegal. Plan LONG (30+ entries): movement \
of whole stacks first, then melees, then fire. Do NOT include \
AdvancePhase — it is ignored in plans. Units you do not mention \
follow these orders automatically.",
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

    #[test]
    fn last_stand_briefs_carry_the_pacing_and_layers() {
        let db = defender_brief(Scenario::FallOfKhartoum);
        assert!(db.contains("LAST-STAND"));
        assert!(db.contains("DEPTH"));
        assert!(db.contains("BODYGUARD"));
        let bb = besieger_brief(Scenario::FallOfKhartoum);
        assert!(bb.contains("BESIEGER"));
        assert!(bb.contains("PACING"));
        assert!(bb.contains("FISTS"));
    }
}
