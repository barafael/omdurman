//! Play a whole game headless, Kitchener against the Khalifa, and print its
//! press: each turn's telegram as the host would file it, then the front
//! page. The quickest way to read what an edit of the phrase banks
//! (`omdurman-rules/src/press/phrases.rs`) does to real games.
//!
//! ```shell
//! cargo run -p omdurman-bot --example press_sample -- historical 7
//! cargo run -p omdurman-bot --example press_sample -- fok 31
//! cargo run -p omdurman-bot --example press_sample -- campaign 17
//! ```
//!
//! With `PRESS_RECORD=<path>` it also writes the game as a replayable
//! record (telegrams included), for `OMDURMAN_REPLAY=<path> cargo run -p
//! omdurman-app` to open on the front page.
use omdurman_bot::rng::BotRng;
use omdurman_rules::Phase;
use omdurman_rules::effects::{GameEffect, GameState, apply_effect};
use omdurman_types::{Player, Scenario};

fn main() {
    let scenario = match std::env::args().nth(1).as_deref() {
        Some("fok") => Scenario::FallOfKhartoum,
        Some("campaign") => Scenario::Campaign,
        _ => Scenario::Historical,
    };
    let seed: u64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(7);
    let board = omdurman_bot::playthrough::board_for_scenario(scenario);
    let mut state = GameState::with_board(scenario, board);
    let mut rng = BotRng::from_seed(seed);
    let mut memory = omdurman_bot::move_memory::MoveMemory::new();
    let mut telegrams: Vec<(u8, String)> = Vec::new();
    // The record: StartGame with two AI seats, every applied effect, and
    // each turn's telegram where the host would file it.
    let mut record: Vec<omdurman_net::GameEvent> = vec![omdurman_net::GameEvent::StartGame {
        seats: [Player::AngloEgyptian, Player::Dervish]
            .into_iter()
            .map(|faction| omdurman_net::Seat {
                faction,
                scope: None,
                holder: omdurman_net::SeatHolder::Ai,
            })
            .collect(),
        scenario,
        optional_rules: Vec::new(),
    }];
    let mut steps = 0;
    let mut stuck = 0;
    while !state.game_over && steps < 200_000 {
        steps += 1;
        let chooser = if state.pending_mine.is_some() {
            Player::Dervish
        } else {
            state.player_to_act().unwrap_or(state.phase_player())
        };
        let candidates = if state.phase == Phase::Setup {
            omdurman_bot::actions::legal_actions_deep_setup(&state, &mut rng)
        } else {
            omdurman_bot::actions::legal_actions(&state, &mut rng)
        };
        let effect = if let Some(e) = candidates
            .iter()
            .find(|e| matches!(e, GameEffect::DervishDesertion { .. }))
        {
            e.clone()
        } else if state.phase == Phase::Setup {
            omdurman_bot::commanders::pick_setup_validated(
                &state,
                &candidates,
                Some(chooser),
                &mut rng,
            )
        } else {
            omdurman_bot::commanders::pick_validated(
                &state,
                chooser,
                &candidates,
                &mut rng,
                &mut memory,
            )
        };
        let before = state.turn_summaries.len();
        if apply_effect(&mut state, &effect).is_err() {
            stuck += 1;
            if stuck > 3 {
                if apply_effect(&mut state, &GameEffect::AdvancePhase).is_ok() {
                    record.push(omdurman_net::GameEvent::Effect(GameEffect::AdvancePhase));
                }
                stuck = 0;
            }
        } else {
            stuck = 0;
            record.push(omdurman_net::GameEvent::Effect(effect.clone()));
        }
        if state.turn_summaries.len() > before {
            let s = state.turn_summaries.last().unwrap().clone();
            let t = omdurman_rules::press::telegram::telegram(&state, &s);
            println!("TURN {} TELEGRAM: {t}", s.turn.value());
            record.push(omdurman_net::GameEvent::Telegram {
                turn: s.turn.value(),
                text: t.clone(),
            });
            telegrams.push((s.turn.value(), t));
        }
    }
    if let Ok(path) = std::env::var("PRESS_RECORD") {
        let mut out = format!("{{\"seed\":{seed}}}\n");
        for (seq, payload) in record.into_iter().enumerate() {
            let event = omdurman_net::RecordedEvent {
                utc: chrono::Utc::now(),
                sender_idx: Some(0),
                seq: seq as u32,
                uid: Some(seq as u64 + 1),
                payload,
            };
            out.push_str(&serde_json::to_string(&event).expect("serialisable"));
            out.push('\n');
        }
        std::fs::write(&path, out).expect("write the record");
        eprintln!("record written to {path}");
    }
    let page = omdurman_rules::press::gazette::front_page(&state, &telegrams);
    println!(
        "\n==== {} | {} | {} | {}",
        page.masthead, page.issue, page.date, page.price
    );
    println!("HEADLINE: {}", page.headline);
    println!("--- {} / {:?}", page.lead.head, page.lead.decks);
    for (i, p) in page.lead.paragraphs.iter().enumerate() {
        if i == 0 {
            println!("{} {}", page.lead.dateline.clone().unwrap_or_default(), p);
        } else {
            println!("{p}");
        }
    }
    for (h, t) in &page.chronicle {
        println!("CHRONICLE {h}.— {t}");
    }
    println!("ROLL: {:?}", page.roll_of_honour);
    for f in &page.features {
        println!("* {}: {}", f.head, f.paragraphs.join(" "));
    }
    for a in &page.adverts {
        println!("AD {} {:?}", a.head, a.lines);
    }
}
