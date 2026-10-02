//! Late-joiner / replay tests, extracted from `main.rs` to keep the binary
//! entry point focused on wiring.
//!
//! These are internal unit tests (`#[cfg(test)] mod tests;` in `main.rs`) so
//! they can access `pub(crate)` items directly.

#[cfg(test)]
mod late_joiner_tests {
    use crate::{
        LoadedAnnotations, PendingEdits, PendingIncoming, PendingMapLoad, TurnState, game_apply,
        game_record, map_kind_for_scenario, picker::UnitPaths, rebuild_state_to,
        timeline::RebuildState,
    };
    use bevy::ecs::world::CommandQueue;
    use bevy::prelude::*;
    use chrono::Utc;
    use omdurman_hexmap::{GameMap, load_map_data};
    use omdurman_net::{
        GameEvent, GameRecord, InitialGameState, NetState, RecordedEvent, new_seed,
    };
    use omdurman_rules::MovementPoints;
    use omdurman_rules::effects::GameState;
    use omdurman_types::{HexCoord, MapKind, SectionName, SpriteRef, Terrain};

    /// Build a minimal GameRecord from a list of events.
    fn make_record(events: Vec<GameEvent>) -> GameRecord {
        let events = events
            .into_iter()
            .enumerate()
            .map(|(i, payload)| RecordedEvent {
                utc: Utc::now(),
                sender_idx: Some(0),
                seq: i as u32,
                uid: None,
                payload,
            })
            .collect();
        GameRecord {
            initial_state: InitialGameState { seed: new_seed() },
            events,
        }
    }

    /// Common setup for replay tests: holds all the mutable state that
    /// [`rebuild_state_to`] reads and writes, so the triplicated
    /// `World::new()` / `Commands::new()` / `GameMap::default()` / ...
    /// stanza lives in one place.
    struct TestHarness {
        world: World,
        queue: CommandQueue,
        game_map: GameMap,
        game_state: GameState,
        seats: crate::seats::Seats,
        local_setup_ready: crate::peers::LocalSetupReady,
        bot_driver: crate::bot_player::BotDriver,
        loaded_annotations: LoadedAnnotations,
        pending_map_load: PendingMapLoad,
        unit_paths: UnitPaths,
        press: crate::telegram::TelegramLog,
    }

    impl TestHarness {
        fn new() -> Self {
            let mut game_map = GameMap::default();
            let loaded_annotations = LoadedAnnotations::from_board_ron();
            load_map_data(
                loaded_annotations.map(MapKind::FallOfKhartoum),
                &mut game_map,
            );
            Self {
                world: World::new(),
                queue: CommandQueue::default(),
                game_map,
                game_state: GameState::new(omdurman_types::Scenario::Campaign),
                seats: crate::seats::Seats::default(),
                local_setup_ready: crate::peers::LocalSetupReady::default(),
                bot_driver: crate::bot_player::BotDriver::default(),
                loaded_annotations,
                pending_map_load: PendingMapLoad::default(),
                unit_paths: UnitPaths::default(),
                press: crate::telegram::TelegramLog::default(),
            }
        }

        /// The event-application sinks over this harness's state.
        fn sinks(&mut self) -> game_apply::EventSinks<'_> {
            game_apply::EventSinks {
                game_state: &mut self.game_state,
                seats: &mut self.seats,
                local_setup_ready: &mut self.local_setup_ready,
                bot_driver: &mut self.bot_driver,
                loaded_annotations: &mut self.loaded_annotations,
                pending_map_load: &mut self.pending_map_load,
                unit_paths: &mut self.unit_paths,
                press: &mut self.press,
            }
        }

        /// Run `rebuild_state_to` with this harness's state. `upto = None`
        /// means full replay; `Some(i)` scrubs to event `i`. Applies the
        /// command queue afterwards so spawned entities are visible.
        fn replay(&mut self, record: &GameRecord, upto: Option<usize>) {
            {
                let mut commands = Commands::new(&mut self.queue, &self.world);
                let mut state = RebuildState {
                    commands: &mut commands,
                    game_map: &mut self.game_map,
                    sinks: game_apply::EventSinks {
                        game_state: &mut self.game_state,
                        seats: &mut self.seats,
                        local_setup_ready: &mut self.local_setup_ready,
                        bot_driver: &mut self.bot_driver,
                        loaded_annotations: &mut self.loaded_annotations,
                        pending_map_load: &mut self.pending_map_load,
                        unit_paths: &mut self.unit_paths,
                        press: &mut self.press,
                    },
                };
                rebuild_state_to(record, upto, &mut state);
            }
            self.queue.apply(&mut self.world);
        }
    }

    // -- one application path: sprite events + effects in seq order ----------

    /// The sprite-shaped event that expresses `effect` *exactly* in `state`
    /// (as `game_apply::sprite_event_effect` would translate it back), or the
    /// plain `Effect` event when no sprite event does.
    fn as_recorded_event(
        effect: &omdurman_rules::effects::GameEffect,
        state: &GameState,
    ) -> GameEvent {
        use omdurman_rules::effects::GameEffect;
        let sprite_of = |id: omdurman_rules::UnitId| {
            let (section_name, col, row) = id.section_pos();
            SpriteRef {
                section_name,
                col: u32::from(col),
                row: u32::from(row),
            }
        };
        let candidate = match effect {
            GameEffect::DeployUnit(p) => Some(GameEvent::PlaceUnit {
                sprite: sprite_of(p.id),
                coord: p.position,
                is_boat: p.profile.kind.is_boat(),
            }),
            GameEffect::PlaceReinforcements(ps) if ps.len() == 1 => Some(GameEvent::PlaceUnit {
                sprite: sprite_of(ps[0].id),
                coord: ps[0].position,
                is_boat: ps[0].profile.kind.is_boat(),
            }),
            GameEffect::MoveUnit {
                unit_id,
                to,
                cost,
                path,
            } => Some(GameEvent::MoveUnit {
                sprite: sprite_of(*unit_id),
                to_q: to.q,
                to_r: to.r,
                cost: *cost,
                path: path.clone(),
            }),
            _ => None,
        };
        candidate
            .filter(|ev| {
                game_apply::sprite_event_effect(ev, state).map(|e| format!("{e:?}"))
                    == Some(format!("{effect:?}"))
            })
            .unwrap_or_else(|| GameEvent::Effect(effect.clone()))
    }

    /// Play up to `max_actions` AI decisions (both factions AI) after a
    /// `StartGame`, recording placements and moves as the sprite events a
    /// player's clicks produce, interleaved with every other effect. Returns
    /// the record and the reference engine state after each recorded event,
    /// built by applying the underlying *effects* directly, in order.
    fn interleaved_ai_record(
        scenario: omdurman_types::Scenario,
        max_actions: usize,
    ) -> (GameRecord, Vec<GameState>) {
        use omdurman_rules::effects::apply_effect;
        use omdurman_types::Player;
        let ai = vec![Player::AngloEgyptian, Player::Dervish];
        let start = GameEvent::StartGame {
            seats: ai
                .iter()
                .map(|&faction| omdurman_net::Seat {
                    faction,
                    scope: None,
                    holder: omdurman_net::SeatHolder::Ai,
                })
                .collect(),
            scenario,
            optional_rules: Vec::new(),
        };
        let mut h = TestHarness::new();
        assert!(game_apply::apply_game_event(&start, &mut h.sinks()));
        let mut state = h.game_state.clone();
        let mut after = vec![state.clone()];
        let mut events = vec![start];
        let mut rng = omdurman_bot::rng::BotRng::from_seed(7);
        for _ in 0..max_actions {
            if state.game_over {
                break;
            }
            let chooser = state.phase_player();
            let effect = crate::bot_player::next_ai_action(&state, chooser, &ai, &mut rng);
            let event = as_recorded_event(&effect, &state);
            apply_effect(&mut state, &effect).expect("the AI submits only validated effects");
            state.drain_observations();
            events.push(event);
            after.push(state.clone());
        }
        (make_record(events), after)
    }

    /// A canonical, order-stable fingerprint of the engine state (the state
    /// holds hash maps, whose iteration order differs between instances).
    fn state_json(state: &GameState) -> String {
        let mut units: Vec<String> = state
            .units
            .iter()
            .map(|u| {
                format!(
                    "{:?}@{:?} {:?} mp={:?}",
                    u.id,
                    u.position,
                    u.state,
                    state.mp_spent(u.id)
                )
            })
            .collect();
        units.sort();
        format!(
            "phase={:?} turn={:?} active={:?} day={:?} over={:?} result={:?} deserted={:?}\n\
             events={:?}\nunits={units:#?}",
            state.phase,
            state.current_turn,
            state.active_player,
            state.day_night,
            state.game_over,
            state.game_result,
            state.dervish_deserted,
            state.turn_events,
        )
    }

    /// C1: `PlaceUnit` / `MoveUnit` / `RemoveUnit` reach the engine in the
    /// same seq-ordered pass as `Effect`s -- the replayed engine state equals
    /// applying the underlying effects in record order, and so does the live
    /// path (one event at a time through the same function).
    #[test]
    fn interleaved_sprite_and_effect_events_replay_in_seq_order() {
        let (record, reference) =
            interleaved_ai_record(omdurman_types::Scenario::FallOfKhartoum, 220);
        let count = |pred: fn(&GameEvent) -> bool| {
            record.events.iter().filter(|e| pred(&e.payload)).count()
        };
        let places = count(|e| matches!(e, GameEvent::PlaceUnit { .. }));
        let moves = count(|e| matches!(e, GameEvent::MoveUnit { .. }));
        let effects = count(|e| matches!(e, GameEvent::Effect(_)));
        assert!(
            places > 0 && moves > 0 && effects > 0,
            "the record must interleave sprite events with effects \
             (places={places}, moves={moves}, effects={effects})"
        );
        let expected = reference.last().expect("at least the StartGame state");

        // Replay (late join / heal / scrub to the end).
        let mut replayed = TestHarness::new();
        replayed.replay(&record, None);
        assert_eq!(
            state_json(&replayed.game_state),
            state_json(expected),
            "replayed engine state diverges from in-order application"
        );

        // Live: each sequenced echo applied as it arrives.
        let mut live = TestHarness::new();
        for event in &record.events {
            game_apply::apply_game_event(&event.payload, &mut live.sinks());
            live.game_state.drain_observations();
        }
        assert_eq!(
            state_json(&live.game_state),
            state_json(expected),
            "live engine state diverges from in-order application"
        );
    }

    /// The timeline scrub (bounded rebuild) shows exactly the state after the
    /// cursor's event, including sprite events.
    #[test]
    fn scrub_applies_only_events_up_to_index() {
        let (record, reference) =
            interleaved_ai_record(omdurman_types::Scenario::FallOfKhartoum, 60);
        let first_place = record
            .events
            .iter()
            .position(|e| matches!(e.payload, GameEvent::PlaceUnit { .. }))
            .expect("the AI deploys");
        for upto in [first_place - 1, first_place, record.events.len() - 1] {
            let mut h = TestHarness::new();
            h.replay(&record, Some(upto));
            assert_eq!(
                state_json(&h.game_state),
                state_json(&reference[upto]),
                "scrub to {upto} must show the state after event {upto}"
            );
        }
    }

    /// C6: observations produced while replaying history are dropped, so the
    /// next live effect does not flush the whole history into the UI.
    #[test]
    fn rebuild_discards_history_observations() {
        let (record, _) = interleaved_ai_record(omdurman_types::Scenario::FallOfKhartoum, 40);
        let mut h = TestHarness::new();
        h.replay(&record, None);
        assert!(h.game_state.drain_observations().is_empty());
    }

    /// A `PlaceUnit` is a deployment during Setup and a reinforcement entry
    /// during a Movement phase.
    // §9.112
    #[traceability_macro::rulebook("§9.112")]
    #[test]
    fn place_unit_translates_by_phase() {
        use omdurman_rules::effects::GameEffect;
        let event = GameEvent::PlaceUnit {
            sprite: SpriteRef {
                section_name: SectionName::Baggara,
                col: 0,
                row: 0,
            },
            coord: HexCoord::new(5, 6),
            is_boat: false,
        };
        let mut gs = GameState::new(omdurman_types::Scenario::Campaign);
        gs.phase = omdurman_rules::Phase::Setup;
        assert!(matches!(
            game_apply::sprite_event_effect(&event, &gs),
            Some(GameEffect::DeployUnit(p)) if p.position == HexCoord::new(5, 6)
        ));
        gs.phase = omdurman_rules::Phase::Movement;
        assert!(matches!(
            game_apply::sprite_event_effect(&event, &gs),
            Some(GameEffect::PlaceReinforcements(ps)) if ps.len() == 1
        ));
    }

    /// A `MoveUnit` names the counter's deterministic rules id and carries the
    /// route unchanged.
    #[test]
    fn move_unit_translates_to_engine_move() {
        use omdurman_rules::effects::GameEffect;
        let sprite = SpriteRef {
            section_name: SectionName::Baggara,
            col: 0,
            row: 0,
        };
        let expected_id = omdurman_rules::unit_id_for_section_pos(SectionName::Baggara, 0, 0)
            .expect("Baggara 0,0 is a counter");
        let event = GameEvent::MoveUnit {
            sprite,
            to_q: 7,
            to_r: 8,
            cost: MovementPoints::new(2),
            path: vec![HexCoord::new(7, 7), HexCoord::new(7, 8)],
        };
        let gs = GameState::new(omdurman_types::Scenario::Campaign);
        match game_apply::sprite_event_effect(&event, &gs) {
            Some(GameEffect::MoveUnit {
                unit_id, to, path, ..
            }) => {
                assert_eq!(unit_id, expected_id);
                assert_eq!(to, HexCoord::new(7, 8));
                assert_eq!(path.len(), 2);
            }
            other => panic!("expected an engine MoveUnit, got {other:?}"),
        }
    }

    // -- map is cleared before replay ----------------------------------------

    #[test]
    fn map_cleared_before_replay() {
        // Pre-populate the map with a hex that is NOT in the record.
        // After replay it must be gone. The map is seeded from the board RON
        // data, then rebuild_state_to clears it and re-seeds the default
        // board (map edits no longer travel as events; the boards are data
        // files authored by tools/map-editor).
        let record = make_record(vec![GameEvent::PlaceUnit {
            sprite: SpriteRef {
                section_name: SectionName::HadendowaForts,
                col: 0,
                row: 0,
            },
            coord: HexCoord::new(1, 1),
            is_boat: false,
        }]);

        let mut h = TestHarness::new();
        h.game_map.hexes.insert(
            HexCoord::new(99, 99),
            omdurman_types::HexData::new(
                Terrain::Swamp {
                    road: omdurman_types::Road::None,
                },
                None,
            ),
        );
        h.replay(&record, None);

        assert!(
            !h.game_map.hexes.contains_key(&HexCoord::new(99, 99)),
            "stale hex must be cleared before replay"
        );
        // The default board is re-seeded after the clear.
        assert!(h.game_map.hexes.contains_key(&HexCoord::new(3, 2)));
    }

    // -- scenario selects the board (§dual-map) -------------------------------

    // §9.31
    #[traceability_macro::rulebook("§9.31")]
    #[test]
    fn scenario_maps_to_board() {
        use omdurman_types::Scenario;
        assert_eq!(map_kind_for_scenario(Scenario::Campaign), MapKind::Campaign);
        // The Historical scenario is the Battle of Omdurman on the main map.
        assert_eq!(
            map_kind_for_scenario(Scenario::Historical),
            MapKind::Campaign
        );
        assert_eq!(
            map_kind_for_scenario(Scenario::FallOfKhartoum),
            MapKind::FallOfKhartoum
        );
    }

    /// A replayed `StartGame { scenario: Campaign }` must request the campaign
    /// board, and `LoadedAnnotations` (initialised from compiled codegen data)
    /// must keep both boards' data regardless of which board is live.
    // §9.31
    #[traceability_macro::rulebook("§9.31")]
    #[test]
    fn start_game_scenario_selects_board() {
        use omdurman_types::Scenario;

        let record = make_record(vec![GameEvent::StartGame {
            seats: vec![],
            scenario: Scenario::Campaign,
            optional_rules: Vec::new(),
        }]);

        let mut h = TestHarness::new();
        h.replay(&record, None);

        // StartGame requested the campaign board...
        assert_eq!(h.pending_map_load.0, Some(MapKind::Campaign));
        // ...and both boards' data survived in the in-memory file.
        assert!(
            h.loaded_annotations.campaign.tiles.contains_key(&(7, 8)),
            "campaign tile present in LoadedAnnotations"
        );
        assert_eq!(
            h.loaded_annotations.fall_of_khartoum.image,
            "fall_of_khartoum_1885.webp"
        );
    }

    // §1.1: a replayed StartGame installs the seat table with its per-human
    // command scopes (live and replay paths share `apply_start_game`, so a
    // late joiner gates on the same commands) and restarts the local
    // member's setup readiness.
    #[traceability_macro::rulebook("§1.1")]
    #[test]
    fn replayed_start_game_stages_commands_and_resets_ready() {
        use omdurman_types::{CommandScope, DervishTribe};
        use std::collections::BTreeSet;

        let me = omdurman_net::PlayerKey(7);
        let scope = CommandScope::Tribes(BTreeSet::from([DervishTribe::Hadendowa]));
        let seats = vec![omdurman_net::Seat {
            faction: omdurman_types::Player::Dervish,
            scope: Some(scope),
            holder: omdurman_net::SeatHolder::Human(me),
        }];
        let record = make_record(vec![GameEvent::StartGame {
            seats: seats.clone(),
            scenario: omdurman_types::Scenario::Campaign,
            optional_rules: Vec::new(),
        }]);

        let mut h = TestHarness::new();
        h.local_setup_ready.0 = true; // stale flag from a previous game
        h.replay(&record, None);

        assert_eq!(h.seats.0, seats, "StartGame installs the seat table");
        assert!(
            !h.local_setup_ready.0,
            "a fresh game restarts per-member setup readiness"
        );
    }

    /// The reconnect regression: a player whose socket was rebuilt (fresh
    /// `PeerId`) re-installs the history and is bound to their seat again,
    /// because seats are keyed by the process-stable player key. A stranger
    /// with another key replaying the same record is a spectator.
    #[test]
    fn history_install_rebinds_the_same_player_key() {
        use crate::seats;
        use omdurman_net::{PlayerKey, Seat, SeatHolder};
        use omdurman_types::Player;

        let me = PlayerKey(0xfeed);
        let foe = PlayerKey(0xbeef);
        let record = make_record(vec![GameEvent::StartGame {
            seats: vec![
                Seat {
                    faction: Player::Dervish,
                    scope: None,
                    holder: SeatHolder::Human(me),
                },
                Seat {
                    faction: Player::AngloEgyptian,
                    scope: None,
                    holder: SeatHolder::Human(foe),
                },
            ],
            scenario: omdurman_types::Scenario::Campaign,
            optional_rules: Vec::new(),
        }]);

        // First session.
        let mut live = TestHarness::new();
        live.replay(&record, None);
        // Reconnect: a fresh harness (wiped state), same key, same record.
        let mut rejoined = TestHarness::new();
        rejoined.replay(&record, None);
        assert_eq!(rejoined.seats, live.seats);
        assert_eq!(
            seats::seat_of(&rejoined.seats.0, me).map(|(_, s)| s.faction),
            Some(Player::Dervish)
        );
        assert!(seats::may_act(&rejoined.seats.0, me, Player::Dervish));
        let stranger = PlayerKey(1);
        assert!(seats::seat_of(&rejoined.seats.0, stranger).is_none());
        assert!(!seats::may_act(
            &rejoined.seats.0,
            stranger,
            Player::Dervish
        ));
    }

    /// Seat events are recorded state: applying `StartGame` -> `SeatCarved`
    /// -> `SeatAssigned` live and rebuilding the same record from scratch
    /// yield the same seat table -- including a stale assignment that both
    /// paths reject identically.
    #[test]
    fn seat_events_replay_to_the_live_seat_table() {
        use omdurman_net::{PlayerKey, Seat, SeatHolder};
        use omdurman_types::{CommandScope, DervishTribe, Player};
        use std::collections::BTreeSet;

        let (a, b, c) = (PlayerKey(1), PlayerKey(2), PlayerKey(3));
        let events = vec![
            GameEvent::StartGame {
                seats: vec![
                    Seat {
                        faction: Player::Dervish,
                        scope: Some(CommandScope::Tribes(BTreeSet::from([
                            DervishTribe::Baggara,
                            DervishTribe::Jaalin,
                        ]))),
                        holder: SeatHolder::Human(a),
                    },
                    Seat {
                        faction: Player::AngloEgyptian,
                        scope: None,
                        holder: SeatHolder::Human(b),
                    },
                ],
                scenario: omdurman_types::Scenario::Campaign,
                optional_rules: Vec::new(),
            },
            GameEvent::SeatCarved {
                faction: Player::Dervish,
                scope: CommandScope::Tribes(BTreeSet::from([DervishTribe::Jaalin])),
                holder: c,
            },
            GameEvent::SeatAssigned {
                seat: 1,
                previous: SeatHolder::Human(b),
                holder: SeatHolder::Ai,
            },
            // Stale: seat 1 is no longer held by `b`.
            GameEvent::SeatAssigned {
                seat: 1,
                previous: SeatHolder::Human(b),
                holder: SeatHolder::Human(a),
            },
        ];

        let mut live = TestHarness::new();
        let accepted: Vec<bool> = events
            .iter()
            .map(|e| game_apply::apply_game_event(e, &mut live.sinks()))
            .collect();
        assert_eq!(accepted, vec![true, true, true, false]);

        let mut replayed = TestHarness::new();
        replayed.replay(&make_record(events), None);
        assert_eq!(replayed.seats, live.seats);
        let seats = &replayed.seats.0;
        assert_eq!(
            seats[0].scope,
            Some(CommandScope::Tribes(BTreeSet::from([
                DervishTribe::Baggara
            ])))
        );
        assert_eq!(seats[1].holder, SeatHolder::Ai);
        assert_eq!(seats[2].holder, SeatHolder::Human(c));
    }

    /// The telegram and Gazette are recorded press events: applying them live
    /// and replaying the record file the same texts, one telegram per turn
    /// (the first recorded wins), so every peer and every replay reads the
    /// host's words.
    #[test]
    fn press_events_file_once_live_and_on_replay() {
        let events = vec![
            GameEvent::StartGame {
                seats: Vec::new(),
                scenario: omdurman_types::Scenario::FallOfKhartoum,
                optional_rules: Vec::new(),
            },
            GameEvent::Telegram {
                turn: 1,
                text: "Night assault repulsed.".into(),
            },
            // A second writer (a failover host) for the same turn loses.
            GameEvent::Telegram {
                turn: 1,
                text: "A different account.".into(),
            },
            GameEvent::Gazette {
                paragraphs: vec!["Khartoum holds.".into()],
            },
        ];
        let mut live = TestHarness::new();
        let filed: Vec<bool> = events
            .iter()
            .map(|e| game_apply::apply_game_event(e, &mut live.sinks()))
            .collect();
        assert_eq!(filed, vec![true, true, false, true]);

        let mut replayed = TestHarness::new();
        replayed.replay(&make_record(events), None);
        assert_eq!(replayed.press.entries, live.press.entries);
        assert_eq!(
            replayed.press.entries,
            vec![(1, "Night assault repulsed.".to_string())]
        );
        assert_eq!(
            replayed.press.gazette,
            Some(vec!["Khartoum holds.".to_string()])
        );
    }

    /// Only the host writes the press: a guest asks no model and submits
    /// nothing; the host submits the turn's telegram as a recorded event
    /// (its echo files it everywhere).
    #[test]
    fn only_the_host_writes_telegrams() {
        use omdurman_rules::turn_summary::TurnSummary;
        let written = |is_host: bool| -> Vec<GameEvent> {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            let mut state = GameState::new(omdurman_types::Scenario::FallOfKhartoum);
            state.turn_summaries.push(TurnSummary {
                turn: omdurman_rules::GameTurnIndex::new(1),
                time: omdurman_rules::turn_track::GameTime::FourAM,
                day_night: omdurman_types::DayNight::Night,
                first_player: omdurman_types::Player::Dervish,
                events: Vec::new(),
            });
            app.insert_resource(crate::GameStateResource(state));
            // No key: the host publishes the fallback at once (and a test
            // never calls a model).
            app.insert_resource(crate::llm::LlmConfig {
                api_key: None,
                ..Default::default()
            });
            app.insert_resource(crate::llm::PendingCompletions::default());
            app.insert_resource(crate::telegram::TelegramLog::default());
            let mut net = omdurman_net::NetState::default();
            net.is_host = is_host;
            app.insert_resource(net);
            app.insert_resource(crate::PendingEdits::default());
            app.add_systems(Update, crate::telegram::generate_telegrams);
            app.update();
            app.update();
            let pending = app.world().resource::<crate::PendingEdits>();
            assert!(
                app.world()
                    .resource::<crate::telegram::TelegramLog>()
                    .entries
                    .is_empty(),
                "filed only by the recorded echo"
            );
            pending.unconfirmed.iter().map(|(_, e)| e.clone()).collect()
        };
        assert!(written(false).is_empty(), "a guest writes nothing");
        let host = written(true);
        assert_eq!(host.len(), 1, "one telegram per turn, once");
        assert!(matches!(host[0], GameEvent::Telegram { turn: 1, .. }));
    }

    /// Make sure any pre-existing on-disk game record still parses against
    /// the current schema. Run only on native; on WASM there are no files.
    /// Scans the per-game directories (`game_*/events.jsonl`).
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn saved_games_still_load() {
        let games_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../games");
        let Ok(entries) = std::fs::read_dir(games_dir) else {
            return;
        };
        let mut record_files = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !path.is_dir() || !name.starts_with("game_") {
                continue;
            }
            let events = path.join("events.jsonl");
            if events.is_file() {
                record_files.push(events);
            }
        }
        let mut found = 0;
        for path in record_files {
            let content = std::fs::read_to_string(&path).expect("read saved game");
            let mut lines = content.lines();
            // First line: {"seed": <n>}
            let header = lines
                .next()
                .unwrap_or_else(|| panic!("{}: empty file", path.display()));
            let seed: u64 = serde_json::from_str(header)
                .map(|v: serde_json::Value| {
                    v.get("seed")
                        .and_then(|s| s.as_u64())
                        .expect("missing seed")
                })
                .unwrap_or_else(|e| panic!("{}: bad header: {e}", path.display()));
            let mut events = Vec::new();
            let mut skipped = false;
            for (i, line) in lines.enumerate() {
                match serde_json::from_str::<RecordedEvent>(line) {
                    Ok(ev) => events.push(ev),
                    Err(e) => {
                        // Old format (e.g. tuple variants) may not parse with
                        // the current schema -- skip this file gracefully.
                        eprintln!(
                            "{}:{}: skipping file due to format change: {e}",
                            path.display(),
                            i + 2
                        );
                        skipped = true;
                        break;
                    }
                }
            }
            if skipped {
                continue;
            }
            let rec = GameRecord {
                initial_state: InitialGameState { seed },
                events,
            };
            // A game abandoned right after it started records only its
            // `StartGame` -- still a valid record.
            assert!(
                rec.events.iter().any(|e| matches!(
                    e.payload,
                    GameEvent::StartGame { .. }
                        | GameEvent::PlaceUnit { .. }
                        | GameEvent::MoveUnit { .. }
                        | GameEvent::Effect(_)
                )) || rec.events.is_empty(),
                "record {} has events but none of the expected variants",
                path.display()
            );
            found += 1;
        }
        if found > 0 {
            eprintln!("verified {found} saved game record(s)");
        }
    }

    /// Serialises tests that swap the process-wide working directory (the
    /// recorder's `games/` path is CWD-relative): the test harness runs them
    /// on parallel threads otherwise.
    #[cfg(not(target_arch = "wasm32"))]
    static CWD_SWAP_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Run the game recording pipeline in isolation: create a JSONL file by
    /// starting the recorder, recording a PlaceUnit event the way
    /// `handle_socket` does on a host-sequenced receipt (`push_event` with a
    /// canonical seq), then flushing and reading back to verify it is present.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn jsonl_records_place_unit() {
        let _cwd_guard = CWD_SWAP_LOCK.lock().unwrap();
        let tmp = tempfile::TempDir::new().unwrap();
        let orig_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Resources the pipeline needs
        app.insert_resource(game_record::GameRecorder::default());
        app.insert_resource(PendingEdits::default());
        app.insert_resource(PendingIncoming::default());
        app.insert_resource(NetState::default());
        app.insert_resource(TurnState::default());

        // Pipeline systems, run in order each frame
        app.add_systems(
            Update,
            (
                game_record::init_game_record,
                game_record::flush_game_record,
            )
                .chain(),
        );

        // Frame 1: init_game_record creates the recorder + seed file.
        app.update();

        // Record a PlaceUnit the way `handle_socket` does when it applies a
        // host-sequenced event: `push_event` with the canonical seq.
        app.world_mut()
            .resource_mut::<game_record::GameRecorder>()
            .push_event(
                &GameEvent::PlaceUnit {
                    sprite: SpriteRef {
                        section_name: SectionName::BritishArmy,
                        col: 0,
                        row: 0,
                    },
                    coord: HexCoord::new(0, 0),
                    is_boat: false,
                },
                Some(0),
                0,
                None,
            );

        // Frame 2: flush_game_record appends the recorded event to the JSONL.
        app.update();

        // Restore CWD before reading / asserting (TempDir cleans up on drop).
        std::env::set_current_dir(&orig_cwd).unwrap();

        let games_dir = tmp.path().join("games");
        // The recorder writes one directory per game; find its events.jsonl.
        let mut jsonl_path = None;
        for entry in std::fs::read_dir(&games_dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if path.is_dir() && name.starts_with("game_") {
                let events = path.join("events.jsonl");
                assert!(events.is_file(), "missing events.jsonl in {name}");
                jsonl_path = Some(events);
                break;
            }
        }
        let jsonl_path = jsonl_path.expect("no jsonl file found in games/");

        let content = std::fs::read_to_string(&jsonl_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert!(
            lines.len() >= 2,
            "expected >= 2 lines (seed + events), got {}",
            lines.len()
        );

        // Line 0: seed header.
        let seed_val: serde_json::Value =
            serde_json::from_str(lines[0]).expect("seed line must be valid JSON");
        assert!(
            seed_val.get("seed").and_then(|s| s.as_u64()).is_some(),
            "first line must contain seed"
        );

        // At least one line must contain a PlaceUnit payload.
        let has_place = lines[1..].iter().any(|l| l.contains("PlaceUnit"));
        assert!(has_place, "expected a PlaceUnit event in JSONL:\n{content}");
    }

    /// The flavour-text artifacts (telegrams, newspaper) land next to the
    /// event log in the game's `games/<game>/` directory (native only).
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn flavour_artifacts_written() {
        let _cwd_guard = CWD_SWAP_LOCK.lock().unwrap();
        let tmp = tempfile::TempDir::new().unwrap();
        let orig_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(game_record::GameRecorder::default());
        app.add_systems(Update, game_record::init_game_record);
        app.update(); // init_game_record creates games/<game>/

        // Seed a completed telegram log + newspaper report and run the savers.
        app.insert_resource(crate::telegram::TelegramLog {
            entries: vec![(2, "Second.".to_string()), (1, "First report.".to_string())],
            ..Default::default()
        });
        app.insert_resource(crate::newspaper::NewspaperReport {
            masthead: "THE LONDON GAZETTE".to_string(),
            date_line: "September 1898".to_string(),
            headline: "DECISIVE BATTLE".to_string(),
            subhead: "Full details inside".to_string(),
            scenario: "Campaign".to_string(),
            turns_played: 7,
            result_key: "anglo_victory".to_string(),
            paragraphs: vec!["The forces met at dawn.".to_string()],
        });
        app.insert_resource(crate::newspaper::NewspaperLlmState {
            dispatched: true,
            completed: true,
            ..Default::default()
        });
        app.add_systems(
            Update,
            (
                crate::telegram::save_telegram_artifacts,
                crate::newspaper::save_newspaper_artifact,
            ),
        );
        app.update();

        // Restore CWD before reading / asserting (TempDir cleans up on drop).
        std::env::set_current_dir(&orig_cwd).unwrap();

        let games_dir = tmp.path().join("games");
        let mut game_dir = None;
        for entry in std::fs::read_dir(&games_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                game_dir = Some(path);
                break;
            }
        }
        let game_dir = game_dir.expect("no game directory found in games/");

        let telegrams = std::fs::read_to_string(game_dir.join("telegrams.md")).unwrap();
        assert!(telegrams.contains("# Military telegrams"));
        // Sorted by turn regardless of arrival order.
        let turn1 = telegrams.find("First report.").expect("turn 1 entry");
        let turn2 = telegrams.find("Second.").expect("turn 2 entry");
        assert!(turn1 < turn2, "telegrams not sorted by turn:\n{telegrams}");

        let newspaper = std::fs::read_to_string(game_dir.join("newspaper.md")).unwrap();
        assert!(newspaper.contains("THE LONDON GAZETTE"));
        assert!(newspaper.contains("DECISIVE BATTLE"));
        assert!(newspaper.contains("The forces met at dawn."));
        assert!(newspaper.contains("Result: anglo_victory"));
    }

    #[test]
    fn net_plugin_registers_seat_resources() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<crate::AppState>();
        app.add_plugins(crate::net_plugin::NetPlugin);
        assert!(app.world().contains_resource::<crate::seats::Seats>());
        assert!(
            app.world()
                .contains_resource::<crate::seats::LocalPlayerKey>()
        );
    }
}

/// Fixture generator: turns a headless bot replay record into a full app-side
/// game directory (events.jsonl + telegrams.md + newspaper.md) by driving the
/// *real* telegram/newspaper systems against the replayed engine state.
///
/// Telegrams are generated per completed game turn; the newspaper needs the
/// game to be over (`game_result` set), so only completed records qualify.
///
/// Run explicitly (it performs LLM calls and writes into the workspace's
/// `games/` directory):
///
/// ```shell
/// ARTIFACT_RECORDS="games/game_bot_<a>/events.jsonl,games/game_bot_<b>/events.jsonl" \
///   cargo test -p omdurman-app generate_artifact_fixtures -- --ignored --nocapture
/// ```
#[cfg(test)]
mod artifact_fixture_tests {
    use bevy::prelude::*;
    use omdurman_net::GameEvent;
    use omdurman_rules::effects::{GameState, apply_effect};
    use omdurman_types::Scenario;

    use crate::LoadedAnnotations;
    use crate::game_record::{self, GameRecorder};
    use crate::llm::{LlmConfig, PendingCompletions};
    use crate::newspaper::{NewspaperLlmState, NewspaperReport};
    use crate::state::GameStateResource;
    use crate::telegram::TelegramLog;

    /// Serialises against the other CWD-swapping tests.
    static CWD_SWAP_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    #[ignore = "fixture generator: performs LLM calls and writes into games/"]
    fn generate_artifact_fixtures() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let records: Vec<String> = std::env::var("ARTIFACT_RECORDS")
                .expect("set ARTIFACT_RECORDS to a comma-separated list of events.jsonl paths")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            assert!(!records.is_empty(), "no record paths given");

            // LLM config reads the key from the environment at construction.
            dotenvy::dotenv().ok();

            let _cwd_guard = CWD_SWAP_LOCK.lock().unwrap();
            // The recorder's games/ dir is CWD-relative; cargo test starts in
            // the crate dir, so swap to the workspace root.
            let crate_dir = std::env::current_dir().unwrap();
            let workspace_root = crate_dir
                .parent()
                .expect("crate dir has a parent")
                .to_path_buf();
            let orig_cwd = std::env::current_dir().unwrap();
            std::env::set_current_dir(&workspace_root).unwrap();

            for path in &records {
                let dir = run_fixture(path);
                eprintln!("artifacts written to {dir}");
            }

            std::env::set_current_dir(&orig_cwd).unwrap();
        }
    }

    /// Replay one record through the real artifact systems; returns the game
    /// directory the artifacts landed in.
    #[cfg(not(target_arch = "wasm32"))]
    fn run_fixture(record_path: &str) -> String {
        let record = game_record::load_record_from_jsonl(record_path)
            .unwrap_or_else(|e| panic!("load {record_path}: {e}"));

        // Rebuild the final engine state by applying the record's effects —
        // the same path the spectator rebuild uses (dice ride in the effects,
        // so no RNG is consumed). The scenario's compiled board must be
        // attached exactly as the bot driver does (`board_for_scenario`), or
        // map-dependent effects (wall breaching, Nile movement, ZOC) reject
        // on replay.
        let scenario = record
            .events
            .iter()
            .find_map(|e| match &e.payload {
                GameEvent::StartGame { scenario, .. } => Some(*scenario),
                _ => None,
            })
            .unwrap_or(Scenario::Campaign);
        let loaded = LoadedAnnotations::from_board_ron();
        let map_data = match scenario {
            Scenario::Campaign | Scenario::Historical => &loaded.campaign,
            Scenario::FallOfKhartoum => &loaded.fall_of_khartoum,
        };
        let board = omdurman_rules::board::BoardInfo::from_map_data(map_data);
        let mut state = GameState::with_board(scenario, board);
        for event in &record.events {
            if let GameEvent::Effect(effect) = &event.payload {
                apply_effect(&mut state, effect)
                    .unwrap_or_else(|e| panic!("replay {record_path}: {e}"));
            }
        }
        assert!(
            state.game_over && state.game_result.is_some(),
            "{record_path}: record is not a completed game (game_over=false); \
             the newspaper artifact requires a finished game"
        );

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // A fresh game directory under games/ with the record's own seed
        // header, then the bot's event log installed so `flush_game_record`
        // appends the full trace after it.
        let mut recorder = GameRecorder::init(record.initial_state.seed);
        recorder.install_history(record);
        app.insert_resource(recorder);
        app.insert_resource(GameStateResource(state));
        app.insert_resource(LlmConfig::default());
        app.insert_resource(TelegramLog::default());
        app.insert_resource(NewspaperReport::default());
        app.insert_resource(NewspaperLlmState::default());
        app.insert_resource(PendingCompletions::default());
        app.add_systems(
            Update,
            (
                crate::telegram::generate_telegrams,
                crate::telegram::poll_telegram_completions,
                crate::telegram::save_telegram_artifacts,
                crate::newspaper::generate_newspaper,
                crate::newspaper::poll_newspaper_completion,
                crate::newspaper::adopt_filed_gazette,
                crate::newspaper::save_newspaper_artifact,
                game_record::flush_game_record,
            ),
        );

        // Pump frames until everything drains: all telegram entries flushed,
        // the newspaper saved, no completions in flight. The savers fall back
        // to stub text on LLM failure, so this always terminates.
        let mut iterations = 0usize;
        loop {
            app.update();
            let telegram_log = app.world().resource::<TelegramLog>();
            let newspaper = app.world().resource::<NewspaperLlmState>();
            let pending = app.world().resource::<PendingCompletions>();
            let done = newspaper.saved
                && telegram_log.flushed == telegram_log.entries.len()
                && pending.items.is_empty()
                && !telegram_log.entries.is_empty();
            iterations += 1;
            if done || iterations > 6_000 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let telegram_log = app.world().resource::<TelegramLog>();
        let newspaper = app.world().resource::<NewspaperLlmState>();
        assert!(
            newspaper.saved,
            "newspaper artifact was not written for {record_path}"
        );
        assert!(
            !telegram_log.entries.is_empty() && telegram_log.flushed == telegram_log.entries.len(),
            "telegram artifact was not fully written for {record_path}"
        );
        eprintln!(
            "{record_path}: {} telegram entries in {} update() iterations",
            telegram_log.entries.len(),
            iterations
        );

        let dir = app
            .world()
            .resource::<GameRecorder>()
            .artifacts_dir()
            .expect("recorder has a game dir");

        // Sanity: the flushed record parses back and still carries the seed.
        let reloaded = game_record::load_record_from_jsonl(&format!("{dir}/events.jsonl"))
            .unwrap_or_else(|e| panic!("reload {dir}/events.jsonl: {e}"));
        assert_eq!(
            reloaded.initial_state.seed,
            app.world()
                .resource::<GameRecorder>()
                .record
                .as_ref()
                .unwrap()
                .initial_state
                .seed
        );
        dir
    }
}

#[cfg(test)]
mod ui_gating_tests {
    use bevy_egui::egui;

    /// Drag-and-drop placement: a counter dragged out of a panel onto the
    /// map. egui "uses" the pointer for the whole drag -- the press landed
    /// on its widget -- so the plain gate stays shut over the map and the
    /// drop's release never reached the board (found in a click-through
    /// play-test: the hint said "drag", only click-then-hex worked). While
    /// the drag is carried, only an egui surface under the pointer blocks.
    #[test]
    fn a_counter_dragged_out_of_a_panel_reaches_the_board() {
        use omdurman_board_ui::panels::{board_pointer_blocked, register_panel_blocker};
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));
        let counter = egui::pos2(40.0, 40.0);
        let over_map = egui::pos2(600.0, 300.0);
        let over_panel = egui::pos2(50.0, 300.0);
        let pass = |ctx: &egui::Context, t: f64, events: Vec<egui::Event>| {
            ctx.begin_pass(egui::RawInput {
                screen_rect: Some(screen),
                events,
                time: Some(t),
                ..Default::default()
            });
            let mut ui = egui::Ui::new(
                ctx.clone(),
                egui::Id::new("test_panel_ui"),
                egui::UiBuilder::new()
                    .layer_id(egui::LayerId::background())
                    .max_rect(screen),
            );
            register_panel_blocker(&mut ui, "test_panel", screen);
            egui::Panel::left("test_panel").show(&mut ui, |ui| {
                // A sidebar counter: click to pick up, drag to carry.
                ui.allocate_exact_size(egui::vec2(48.0, 48.0), egui::Sense::click_and_drag());
            });
            let mut out = ctx.end_pass();
            out.textures_delta.clear();
        };
        let button = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        pass(&ctx, 0.0, vec![egui::Event::PointerMoved(counter)]);
        pass(&ctx, 0.1, vec![button(counter, true)]);
        for (i, x) in [100.0, 300.0, 600.0].into_iter().enumerate() {
            let pos = egui::pos2(x, over_map.y);
            pass(
                &ctx,
                0.2 + i as f64 * 0.1,
                vec![egui::Event::PointerMoved(pos)],
            );
        }
        // The button is still down over the map: egui is using the pointer.
        assert!(crate::ui_plugin::egui_wants_pointer_input(&ctx));
        assert!(
            board_pointer_blocked(&ctx, false),
            "the plain gate stays shut"
        );
        assert!(
            !board_pointer_blocked(&ctx, true),
            "a carried drag reaches the board"
        );
        // Carried back over the panel, the panel still blocks the board.
        pass(&ctx, 0.6, vec![egui::Event::PointerMoved(over_panel)]);
        assert!(board_pointer_blocked(&ctx, true));
    }

    /// The panel-unification contract: a click-sensed full-rect blocker
    /// (`Ui::interact`, see `panels::register_panel_blocker`) makes
    /// `egui_wants_pointer_input` true over *blank* panel areas -- the one
    /// thing the deleted `PanelRects` registry used to provide -- while the
    /// map stays unblocked. Replicates the app's sidebar construction
    /// (background-layer Ui + `egui::Panel`) headlessly.
    #[test]
    fn panel_blocker_registers_pointer_interest() {
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));
        // Panel default width is 96pt; probe a blank spot *inside* it.
        let over_blank_panel = egui::pos2(50.0, 300.0);
        let over_map = egui::pos2(600.0, 300.0);

        let pointer_at = |ctx: &egui::Context, pos: egui::Pos2| {
            ctx.begin_pass(egui::RawInput {
                screen_rect: Some(screen),
                events: vec![egui::Event::PointerMoved(pos)],
                time: Some(0.0),
                ..Default::default()
            });
            let mut ui = egui::Ui::new(
                ctx.clone(),
                egui::Id::new("test_panel_ui"),
                egui::UiBuilder::new()
                    .layer_id(egui::LayerId::background())
                    .max_rect(screen),
            );
            omdurman_board_ui::panels::register_panel_blocker(
                &mut ui,
                "test_panel",
                // First frame: no PanelState yet, so the fallback rect gates.
                screen,
            );
            egui::Panel::left("test_panel")
                .frame(egui::Frame::default().fill(egui::Color32::from_gray(44)))
                .show(&mut ui, |_ui| {});
            // egui 0.36 asserts TexturesDelta is drained before dropping (a
            // real renderer turns it into texture uploads; headless clears).
            let mut out = ctx.end_pass();
            out.textures_delta.clear();
        };

        pointer_at(&ctx, over_blank_panel);
        assert!(
            crate::ui_plugin::egui_wants_pointer_input(&ctx),
            "pointer over a blank panel area must read as UI interest"
        );
        pointer_at(&ctx, over_map);
        assert!(
            !crate::ui_plugin::egui_wants_pointer_input(&ctx),
            "pointer over the map must NOT read as UI interest"
        );
    }

    /// Regression test: the panel blocker must be registered *before* the
    /// panel's content. egui hit-tests back-to-front within a layer, so a
    /// blocker registered after the content sat on top of every widget in
    /// the panel and swallowed all their clicks and hovers (this took the
    /// lobby's buttons dead when the PanelRects registry was replaced).
    #[test]
    fn panel_blocker_does_not_steal_widget_clicks() {
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));

        // Pass 1: build the panel with a button, blocker registered first
        // (via `register_panel_blocker`, exactly like production code).
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(screen),
            events: vec![egui::Event::PointerMoved(egui::pos2(50.0, 20.0))],
            time: Some(0.0),
            ..Default::default()
        });
        {
            let mut ui = egui::Ui::new(
                ctx.clone(),
                egui::Id::new("test_panel_ui"),
                egui::UiBuilder::new()
                    .layer_id(egui::LayerId::background())
                    .max_rect(screen),
            );
            omdurman_board_ui::panels::register_panel_blocker(&mut ui, "test_panel", screen);
            egui::Panel::left("test_panel").show(&mut ui, |ui| {
                let _ = ui.button("Lobby");
            });
        }
        // egui 0.36 asserts TexturesDelta is drained before dropping (a real
        // renderer turns it into texture uploads; headless clears).
        let mut out = ctx.end_pass();
        out.textures_delta.clear();

        // Pass 2: press + release over the button. egui resolves clicks
        // against the *previous* pass's widget rects, so this is where the
        // click (or the blocker's theft of it) lands.
        let press_release = |pressed: bool| egui::Event::PointerButton {
            pos: egui::pos2(50.0, 20.0),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(screen),
            events: vec![press_release(true), press_release(false)],
            time: Some(1.0),
            ..Default::default()
        });
        let mut ui = egui::Ui::new(
            ctx.clone(),
            egui::Id::new("test_panel_ui"),
            egui::UiBuilder::new()
                .layer_id(egui::LayerId::background())
                .max_rect(screen),
        );
        omdurman_board_ui::panels::register_panel_blocker(&mut ui, "test_panel", screen);
        let mut clicked = false;
        let mut hovering = false;
        egui::Panel::left("test_panel").show(&mut ui, |ui| {
            let resp = ui.button("Lobby");
            clicked = resp.clicked();
            hovering = resp.hovered();
        });
        // egui 0.36 asserts TexturesDelta is drained before dropping (a real
        // renderer turns it into texture uploads; headless clears).
        let mut out = ctx.end_pass();
        out.textures_delta.clear();

        assert!(
            clicked,
            "a button inside the panel must receive clicks despite the blocker"
        );
        assert!(
            hovering,
            "a button inside the panel must receive hover despite the blocker"
        );
    }

    /// Same contract for painter-only fullscreen overlays (splash, event
    /// viewer): an `interact` blocker inside the Area's Ui (see
    /// `splash::splash_ui`) must cover the blank backdrop.
    #[test]
    fn overlay_blocker_registers_pointer_interest() {
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));

        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(screen),
            events: vec![egui::Event::PointerMoved(egui::pos2(400.0, 300.0))],
            time: Some(0.0),
            ..Default::default()
        });
        egui::Area::new(egui::Id::new("test_overlay"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(&ctx, |ui| {
                ui.interact(
                    screen,
                    egui::Id::new("test_overlay_blocker"),
                    egui::Sense::click(),
                );
                // Painter-only backdrop: no widgets, like the splash.
                ui.painter()
                    .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(255));
            });
        // egui 0.36 asserts TexturesDelta is drained before dropping (a real
        // renderer turns it into texture uploads; headless clears).
        let mut out = ctx.end_pass();
        out.textures_delta.clear();

        assert!(
            crate::ui_plugin::egui_wants_pointer_input(&ctx),
            "pointer over a painter-only overlay backdrop must read as UI interest"
        );
    }
}

#[cfg(test)]
mod layout_tests {
    use bevy_egui::egui;

    /// The left-rail contract : rail panels chain side by side
    /// below the top bar instead of superimposing at the window edge (this
    /// used to overlap the unit picker and unit overview sidebars).
    #[test]
    fn left_rail_panels_chain_without_overlap() {
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600.0, 900.0));
        let mut layout = crate::ScreenLayout::default();

        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(screen),
            time: Some(0.0),
            ..Default::default()
        });

        let mut rects = Vec::new();
        for (root, panel) in [
            ("rail_root_1", "rail_panel_1"),
            ("rail_root_2", "rail_panel_2"),
        ] {
            let mut rect = egui::Rect::NOTHING;
            crate::layout::left_rail_panel(&ctx, &mut layout, root, panel, 216.0, |ui| {
                rect = egui::Panel::left(panel)
                    .resizable(true)
                    .default_size(200.0)
                    .show(ui, |_ui| {})
                    .response
                    .rect;
                rect
            });
            rects.push(rect);
        }
        // egui 0.36 asserts TexturesDelta is drained before dropping (a real
        // renderer turns it into texture uploads; headless clears).
        let mut out = ctx.end_pass();
        out.textures_delta.clear();

        assert_eq!(rects.len(), 2);
        // Both start below the top bar.
        for rect in &rects {
            assert!(
                rect.min.y >= crate::layout::TOP_BAR_HEIGHT - f32::EPSILON,
                "rail panels must start below the top bar"
            );
        }
        // No horizontal overlap: the second panel starts at (or right of)
        // the first panel's right edge.
        assert!(
            rects[1].min.x >= rects[0].max.x - f32::EPSILON,
            "rail panels must chain side by side, not overlap: {rects:?}"
        );
    }

    /// The top-center stack contract : stacked cards accumulate
    /// downward from below the top bar instead of sharing a fixed y (this
    /// used to superimpose the phase banner, previews, and prompts).
    #[test]
    fn stacked_cards_accumulate_downward() {
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600.0, 900.0));
        let mut layout = crate::ScreenLayout::default();

        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(screen),
            time: Some(0.0),
            ..Default::default()
        });

        let rect_a =
            crate::ui::stacked_card(&ctx, &mut layout, "card_a", egui::Frame::default(), |ui| {
                ui.label("banner");
                ui.min_rect()
            })
            .unwrap();
        let rect_b =
            crate::ui::stacked_card(&ctx, &mut layout, "card_b", egui::Frame::default(), |ui| {
                ui.label("preview");
                ui.min_rect()
            })
            .unwrap();
        // egui 0.36 asserts TexturesDelta is drained before dropping (a real
        // renderer turns it into texture uploads; headless clears).
        let mut out = ctx.end_pass();
        out.textures_delta.clear();

        assert!(
            rect_a.min.y >= crate::layout::TOP_BAR_HEIGHT - f32::EPSILON,
            "stacked cards start below the top bar"
        );
        assert!(
            rect_b.min.y >= rect_a.max.y,
            "stacked cards must accumulate downward, not overlap: a={rect_a:?} b={rect_b:?}"
        );
    }

    /// Play-test repro: a melee target hex lying under the hover preview card
    /// could not be clicked -- the card claimed the pointer. A hover-only
    /// (passive) card must leave the board unblocked; an ordinary card with a
    /// widget still blocks it.
    #[test]
    fn passive_preview_cards_do_not_block_the_board() {
        let blocks = |passive: bool| {
            let ctx = egui::Context::default();
            let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600.0, 900.0));
            let card = |ctx: &egui::Context| {
                let mut layout = crate::ScreenLayout::default();
                let contents = |ui: &mut egui::Ui| {
                    let _ = ui.button("§7.7 Melee");
                    ui.min_rect()
                };
                if passive {
                    crate::ui::passive_stacked_card(
                        ctx,
                        &mut layout,
                        "preview",
                        egui::Frame::default(),
                        contents,
                    )
                } else {
                    crate::ui::stacked_card(
                        ctx,
                        &mut layout,
                        "preview",
                        egui::Frame::default(),
                        contents,
                    )
                }
                .unwrap()
            };
            ctx.begin_pass(egui::RawInput {
                screen_rect: Some(screen),
                time: Some(0.0),
                ..Default::default()
            });
            let rect = card(&ctx);
            ctx.end_pass().textures_delta.clear();
            // Second pass: the pointer rests in the middle of the card.
            ctx.begin_pass(egui::RawInput {
                screen_rect: Some(screen),
                time: Some(0.1),
                events: vec![egui::Event::PointerMoved(rect.center())],
                ..Default::default()
            });
            card(&ctx);
            let blocked = omdurman_board_ui::pointer_over_egui_surface(&ctx);
            ctx.end_pass().textures_delta.clear();
            blocked
        };
        assert!(
            blocks(false),
            "an interactive card blocks the board under it"
        );
        assert!(!blocks(true), "a hover preview card lets the click through");
    }
}

/// C2: the Game view keeps no snapshot -- a round trip through the menu and
/// the lobby returns to the *live* engine state, including events applied
/// while the menu was shown.
#[cfg(test)]
mod mode_transition_tests {
    use bevy::prelude::*;
    use omdurman_rules::effects::GameState;

    use crate::state::{AppMode, AppState, GameStateResource};

    fn switch(app: &mut App, mode: AppMode, state: AppState) {
        app.world_mut()
            .resource_mut::<NextState<AppMode>>()
            .set(mode);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(state);
        app.update();
    }

    #[test]
    fn menu_round_trip_keeps_live_engine_state() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<AppState>().init_state::<AppMode>();
        app.insert_resource(GameStateResource(GameState::new(
            omdurman_types::Scenario::Campaign,
        )));
        app.insert_resource(crate::PendingEdits::default());
        crate::mode_transitions::add_lobby_snapshot_systems(&mut app);

        switch(&mut app, AppMode::Game, AppState::InGame);
        switch(&mut app, AppMode::Menu, AppState::InGame);
        // A sequenced event lands while the menu is shown.
        app.world_mut()
            .resource_mut::<GameStateResource>()
            .0
            .game_over = true;
        switch(&mut app, AppMode::Lobby, AppState::Lobby);
        switch(&mut app, AppMode::Game, AppState::InGame);

        assert!(
            app.world().resource::<GameStateResource>().0.game_over,
            "re-entering the Game view must not roll the engine back"
        );
    }

    #[test]
    fn game_in_progress_follows_start_or_record() {
        let mut turn = crate::TurnState::default();
        let mut recorder = crate::game_record::GameRecorder::default();
        assert!(!crate::game_in_progress(&turn, &recorder));
        turn.game_started = true;
        assert!(crate::game_in_progress(&turn, &recorder));
        turn.game_started = false;
        recorder.install_history(omdurman_net::GameRecord {
            initial_state: omdurman_net::InitialGameState { seed: 1 },
            events: vec![omdurman_net::RecordedEvent {
                utc: chrono::Utc::now(),
                sender_idx: None,
                seq: 0,
                uid: None,
                payload: omdurman_net::GameEvent::Effect(
                    omdurman_rules::effects::GameEffect::AdvancePhase,
                ),
            }],
        });
        assert!(crate::game_in_progress(&turn, &recorder));
    }
}

/// The whole game, headless: no window server (`primary_window` optional,
/// no winit), no GPU, no global log subscriber (tests share a process).
fn headless_game_app(window: bool) -> bevy::app::App {
    use bevy::prelude::*;
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: window.then(Window::default),
                exit_condition: bevy::window::ExitCondition::DontExit,
                ..default()
            })
            .set(bevy::render::RenderPlugin {
                render_creation: bevy::render::settings::WgpuSettings {
                    backends: None,
                    ..default()
                }
                .into(),
                ..default()
            })
            .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::log::LogPlugin>(),
    );
    crate::add_game(&mut app, "headless".into());
    app
}

/// Every system the game registers has a valid parameter set. Bevy reports a
/// conflicting pair -- two `ResMut` of one resource (B0002), say one hidden
/// inside a `SystemParam` -- only when a schedule is initialised, i.e. when
/// the app first runs, so no other test sees it. Build the whole game
/// headless (no window, no GPU) and initialise every schedule without
/// running a frame.
#[test]
fn every_system_has_valid_parameters() {
    use bevy::prelude::*;
    let mut app = headless_game_app(false);
    app.finish();
    app.cleanup();
    // (Initialising a schedule may itself touch `Schedules`, so take it out
    // of the world rather than `resource_scope` it.)
    let world = app.world_mut();
    let mut schedules = world
        .remove_resource::<Schedules>()
        .expect("the app has schedules");
    for (label, schedule) in schedules.iter_mut() {
        schedule
            .initialize(world)
            .unwrap_or_else(|e| panic!("schedule {label:?}: {e}"));
    }
    world.insert_resource(schedules);
}

/// Play `scenario` headless, both factions held by the AI on an offline
/// self-hosting instance, for up to `frames` frames of 250 ms: every system
/// of every phase runs for real, so a system asking for a resource that
/// does not exist yet, or a command on a vanished entity -- runtime panics
/// in Bevy -- fails here. Returns the final engine state.
fn ai_plays_headless(
    scenario: omdurman_types::Scenario,
    frames: usize,
) -> omdurman_rules::effects::GameState {
    use bevy::prelude::*;
    let games = tempfile::tempdir().expect("temp dir");
    let mut app = headless_game_app(true);
    app.insert_resource(crate::net_plugin::OfflineMode(true))
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(250),
        ))
        // No flavour-text model calls, and records into the temp dir.
        .insert_resource(omdurman_net::llm::LlmConfig {
            api_key: None,
            ..Default::default()
        })
        .insert_resource(crate::game_record::GameRecorder::init_in(
            games.path().to_str().expect("utf-8 temp path"),
            7,
        ))
        .insert_resource(crate::GameRng::from_seed(7));
    for _ in 0..10 {
        app.update();
    }
    // Into the lobby, as the title screen's Lobby button does.
    app.world_mut()
        .resource_mut::<NextState<crate::AppState>>()
        .set(crate::AppState::Lobby);
    app.world_mut()
        .resource_mut::<NextState<crate::AppMode>>()
        .set(crate::AppMode::Lobby);
    for _ in 0..10 {
        app.update();
    }
    let seats = [
        omdurman_types::Player::AngloEgyptian,
        omdurman_types::Player::Dervish,
    ]
    .map(|faction| omdurman_net::Seat {
        faction,
        scope: None,
        holder: omdurman_net::SeatHolder::Ai,
    })
    .to_vec();
    app.world_mut()
        .resource_mut::<crate::PendingEdits>()
        .submit_game(omdurman_net::GameEvent::StartGame {
            seats,
            scenario,
            optional_rules: Vec::new(),
        });
    let mut after_the_end = 0;
    for _ in 0..frames {
        app.update();
        // Keep going a while after the end: the result, the Gazette and the
        // end-of-game screens run then.
        if app
            .world()
            .resource::<crate::GameStateResource>()
            .0
            .game_over
        {
            after_the_end += 1;
            if after_the_end > 120 {
                break;
            }
        }
    }
    let final_state = app.world().resource::<crate::GameStateResource>().0.clone();
    // Then review the game on the timeline, as the saved-games list and the
    // Gazette's "Review timeline" do: jump about, then play it back.
    let record = app
        .world()
        .resource::<crate::game_record::GameRecorder>()
        .record
        .clone()
        .expect("the game was recorded");
    let len = record.events.len();
    app.world_mut()
        .resource_mut::<crate::timeline::SpectatorTimeline>()
        .open(record, "headless".into());
    app.world_mut()
        .resource_mut::<NextState<crate::AppState>>()
        .set(crate::AppState::Spectating);
    for cursor in [len / 2, 0, 1, len / 3, len.saturating_sub(2), len / 5] {
        let mut timeline = app
            .world_mut()
            .resource_mut::<crate::timeline::SpectatorTimeline>();
        timeline.cursor = cursor;
        timeline.dirty = true;
        app.update();
        app.update();
    }
    app.world_mut()
        .resource_mut::<crate::timeline::SpectatorTimeline>()
        .playing = true;
    for _ in 0..60 {
        app.update();
    }
    assert_eq!(
        *app.world().resource::<State<crate::AppState>>().get(),
        crate::AppState::Spectating
    );
    assert!(
        !app.world()
            .resource::<crate::timeline::SpectatorTimeline>()
            .dirty,
        "every scrub was rebuilt"
    );
    // ...and back to the menu.
    app.world_mut()
        .resource_mut::<NextState<crate::AppMode>>()
        .set(crate::AppMode::Menu);
    for _ in 0..5 {
        app.update();
    }
    final_state
}

/// The Historical scenario, AI against AI, to the §9.24 result. About a
/// minute in a debug build; run with `cargo test -p omdurman-app -- --ignored`.
#[traceability_macro::rulebook("§9.24")]
#[test]
#[ignore = "long: a full Historical game headless"]
fn ai_plays_historical_headless() {
    let state = ai_plays_headless(omdurman_types::Scenario::Historical, 20000);
    assert!(state.game_over, "the four turns are played out");
    assert!(matches!(
        state.game_result,
        Some(omdurman_rules::GameResult::Historical { .. })
    ));
}

/// The Campaign, AI against AI, through a dozen turns (night, desertion,
/// reinforcements). A few minutes in a debug build; `--ignored` to run.
#[test]
#[ignore = "long: a Campaign game headless"]
fn ai_plays_campaign_headless() {
    let state = ai_plays_headless(omdurman_types::Scenario::Campaign, 40000);
    assert!(
        state.current_turn.value() >= 8,
        "turn {:?}",
        state.current_turn
    );
}

/// FALL OF KHARTOUM, AI against AI, from set-up to the result, then the
/// timeline review (about 20 s in a debug build).
#[test]
fn ai_plays_fall_of_khartoum_headless() {
    let state = ai_plays_headless(omdurman_types::Scenario::FallOfKhartoum, 20000);
    assert_eq!(state.scenario, omdurman_types::Scenario::FallOfKhartoum);
    assert!(state.game_over, "the AI plays the siege to its end");
    assert!(matches!(
        state.game_result,
        Some(omdurman_rules::GameResult::FoK(_))
    ));
}
