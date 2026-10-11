//! Watch one commander-vs-commander game turn by turn: what each side moved
//! where, who fired at whom with what result, the melees, and a map of every
//! stack at the end of each player turn.
//!
//! ```sh
//! cargo run --release -p omdurman-bot --example observe -- historical 1000 [last_turn] [current|baseline] [current|baseline]
//! ```
//!
//! Map symbols (one per hex, the stack's first unit): Anglo-Egyptian upper
//! case -- I infantry (British brigades B), F "Friendlies", C cavalry,
//! J camel corps, A artillery, M Maxim, G gunboat, L leader, E engineers,
//! T fort; Dervish lower case, by the leader whose colour the tribe wears --
//! k Taiasha, y Baggara/Jaalin, o Hadendowa, d Mulazmin/Jehadia, s Danagla,
//! a Kehena/Degheim, z Isa Zachneih, x artillery, g gunboat, t fort, and the
//! leaders as `@`. Terrain: `~` Nile, `#` building, `h` huts, `^` hilltop,
//! `:` rough, `.` clear.

use std::collections::BTreeMap;

use omdurman_bot::arena::{Version, play_observed};
use omdurman_bot::describe::{describe_effect, describe_observation, unit_name};
use omdurman_bot::fire_plan::melee_value;
use omdurman_rules::effects::{GameEffect, GameState, build_melee_attack};
use omdurman_rules::{Phase, UnitId, UnitIdentity};
use omdurman_types::{DervishTribe, HexCoord, Player, Scenario, Terrain};

fn symbol(identity: &UnitIdentity) -> char {
    use UnitIdentity::*;
    match identity {
        AngloEgyptianInfantry { .. } if identity.is_friendlies() => 'F',
        AngloEgyptianInfantry { .. } => {
            let british = identity
                .brigade()
                .is_some_and(|b| b.nationality == omdurman_types::BrigadeNationality::British);
            if british { 'B' } else { 'I' }
        }
        AngloEgyptianCavalry => 'C',
        AngloEgyptianCamelCorps => 'J',
        AngloEgyptianArtillery => 'A',
        AngloEgyptianMaxim => 'M',
        AngloEgyptianGunboat(_) => 'G',
        AngloEgyptianLeader(_) => 'L',
        RoyalEngineers => 'E',
        AngloEgyptianFort => 'T',
        DervishTribal { tribe } => match tribe {
            DervishTribe::Taiasha => 'k',
            DervishTribe::Baggara | DervishTribe::Jaalin => 'y',
            DervishTribe::Hadendowa => 'o',
            DervishTribe::Mulazmin | DervishTribe::Jehadia => 'd',
            DervishTribe::Danagla => 's',
            DervishTribe::Kehena | DervishTribe::Degheim => 'a',
            DervishTribe::IsaZachneih => 'z',
        },
        DervishLeader(_) => '@',
        DervishArtillery => 'x',
        DervishGunboat(_) => 'g',
        DervishFort => 't',
    }
}

fn terrain_char(t: Option<&Terrain>) -> char {
    match t {
        None => ' ',
        Some(Terrain::Nile { .. }) => '~',
        Some(Terrain::Building { .. }) => '#',
        Some(Terrain::Huts { .. }) => 'h',
        Some(Terrain::Hilltop { .. }) => '^',
        Some(Terrain::Rough { .. }) => ':',
        Some(Terrain::Swamp { .. }) => ',',
        Some(Terrain::Trees { .. }) => 't',
        Some(_) => '.',
    }
}

/// The board as text: columns are q, rows r - q/2 (the axial skew undone so
/// the Nile runs down the page).
fn render(state: &GameState) -> String {
    let hexes: Vec<HexCoord> = state.board.terrain.keys().copied().collect();
    let row = |h: HexCoord| h.r - h.q.div_euclid(2);
    let (q0, q1) = (
        hexes.iter().map(|h| h.q).min().unwrap_or(0),
        hexes.iter().map(|h| h.q).max().unwrap_or(0),
    );
    let (y0, y1) = (
        hexes.iter().map(|&h| row(h)).min().unwrap_or(0),
        hexes.iter().map(|&h| row(h)).max().unwrap_or(0),
    );
    let mut stacks: BTreeMap<HexCoord, Vec<char>> = BTreeMap::new();
    for u in &state.units {
        stacks
            .entry(u.position)
            .or_default()
            .push(symbol(&u.profile.identity));
    }
    let mut out = String::new();
    out += "      ";
    for q in q0..=q1 {
        out.push(char::from_digit((q.rem_euclid(100) / 10) as u32, 10).unwrap());
    }
    out += "\n      ";
    for q in q0..=q1 {
        out.push(char::from_digit(q.rem_euclid(10) as u32, 10).unwrap());
    }
    out.push('\n');
    for y in y0..=y1 {
        let mut line = format!("{y:>5} ");
        let mut any = false;
        for q in q0..=q1 {
            let h = HexCoord::new(q, y + q.div_euclid(2));
            let c = match stacks.get(&h) {
                Some(units) => {
                    any = true;
                    // A leader shows only when alone; otherwise the troops.
                    *units
                        .iter()
                        .find(|&&c| c != 'L' && c != '@')
                        .unwrap_or(&units[0])
                }
                None => terrain_char(state.board.terrain.get(&h)),
            };
            line.push(c);
        }
        if any || y % 5 == 0 {
            out += line.trim_end();
            out.push('\n');
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let scenario = match args.first().map(|s| s.as_str()) {
        Some("fok") => Scenario::FallOfKhartoum,
        Some("historical") => Scenario::Historical,
        _ => Scenario::Campaign,
    };
    let seed: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1000);
    let last_turn: u8 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(u8::MAX);
    let version = |i: usize| match args.get(i).map(|s| s.as_str()) {
        Some("baseline") => Version::Baseline,
        _ => Version::Current,
    };
    // Movement per unit within one movement phase: first hex, last hex.
    let mut moved: BTreeMap<UnitId, (HexCoord, HexCoord)> = BTreeMap::new();
    let header = |state: &GameState| {
        format!(
            "\n=== turn {} {:?} {:?} {:?} -- lost AE {} D {} ===",
            state.current_turn.value(),
            state.day_night,
            state.active_player,
            state.phase,
            state.victory.units_eliminated_by(Player::Dervish),
            state.victory.units_eliminated_by(Player::AngloEgyptian),
        )
    };
    let mut observe = |before: &GameState, effect: &GameEffect, after: &GameState| {
        if let GameEffect::MoveUnit { unit_id, to, .. } = effect {
            let from = before
                .find_unit(*unit_id)
                .map(|u| u.position)
                .unwrap_or(*to);
            moved.entry(*unit_id).or_insert((from, *to)).1 = *to;
        }
        let interesting = !matches!(
            effect,
            GameEffect::MoveUnit { .. } | GameEffect::AdvancePhase | GameEffect::DeployUnit(_)
        );
        if interesting {
            println!(
                "  [{:?} {:?}] {}",
                before.phase_player(),
                before.phase,
                describe_effect(effect, before)
            );
            for obs in &after.observations {
                println!("      -> {}", describe_observation(obs));
            }
        }
        if matches!(effect, GameEffect::AdvancePhase)
            && before.phase == Phase::Movement
            && !moved.is_empty()
        {
            println!("  moves of {:?}:", before.active_player);
            for (id, (from, to)) in std::mem::take(&mut moved) {
                println!(
                    "    {:<28} ({},{}) -> ({},{})  {} hexes",
                    unit_name(id),
                    from.q,
                    from.r,
                    to.q,
                    to.r,
                    from.distance(to)
                );
            }
        }
        // A melee phase ending: the melees left undeclared, with the bot's
        // expected value of each (`fire_plan::melee_value`).
        if matches!(effect, GameEffect::AdvancePhase) && before.phase == Phase::Melee {
            let side = before.active_player;
            let mut hexes: Vec<HexCoord> = before
                .units
                .iter()
                .filter(|u| u.profile.identity.owner() == side)
                .map(|u| u.position)
                .collect();
            hexes.sort_by_key(|h| (h.q, h.r));
            hexes.dedup();
            for a in hexes {
                for d in a.neighbors() {
                    if let Some(attack) = build_melee_attack(before, a, d) {
                        println!(
                            "  undeclared melee ({},{})->({},{}) {} vs {}: value {:.2}",
                            a.q,
                            a.r,
                            d.q,
                            d.r,
                            attack.attackers.len(),
                            attack.defenders.len(),
                            melee_value(before, &attack)
                        );
                    }
                }
            }
        }
        // End of a player turn (melee over) or of set-up: the map.
        let turn_over = (before.phase == Phase::Melee || before.phase == Phase::Setup)
            && (after.phase != before.phase || after.active_player != before.active_player);
        if turn_over {
            println!("{}", header(after));
            print!("{}", render(after));
        }
    };
    let r = play_observed(
        scenario,
        seed,
        version(3),
        version(4),
        last_turn,
        &mut observe,
    );
    println!("\n{}", r.line());
}
