//! Late-joiner / replay tests, extracted from `main.rs` to keep the binary
//! entry point focused on wiring.
//!
//! These are internal unit tests (`#[cfg(test)] mod tests;` in `main.rs`) so
//! they can access `pub(crate)` items directly.

#[cfg(test)]
mod late_joiner_tests {
    use crate::{
        LoadedAnnotations, PendingEdits, PendingIncoming, PendingMapLoad,
        TurnState, map_kind_for_scenario, peers::QueuedFactions, rebuild_state_to,
        editor::HexEditor, game_record, render::HexOverlay,
        timeline::RebuildState, units::UnitViewer,
    };
    use bevy::ecs::world::CommandQueue;
    use bevy::prelude::*;
    use bevy_matchbox::prelude::PeerId;
    use chrono::Utc;
    use omdurman_hexmap::{GameMap, load_map_data};
    use omdurman_net::{GameEvent, GameRecord, InitialGameState, NetState, RecordedEvent, new_seed};
    use omdurman_rules::board_data;
    use omdurman_rules::effects::GameState;
    use omdurman_rules::MovementPoints;
    use omdurman_types::{
        HexCoord, MapKind, OverlayParams, SectionName, SpriteRef, Terrain,
    };
    use uuid::Uuid;

    /// Build a minimal GameRecord from a list of events.
    fn make_record(events: Vec<GameEvent>) -> GameRecord {
        let events = events
            .into_iter()
            .enumerate()
            .map(|(i, payload)| RecordedEvent {
                utc: Utc::now(),
                sender_idx: Some(0),
                seq: i as u32,
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
        overlay: HexOverlay,
        editor: HexEditor,
        viewer: UnitViewer,
        incoming: Vec<(GameEvent, PeerId)>,
        history_peer: PeerId,
        game_state: GameState,
        queued_factions: QueuedFactions,
        loaded_annotations: LoadedAnnotations,
        pending_map_load: PendingMapLoad,
    }

    impl TestHarness {
        fn new() -> Self {
            let mut game_map = GameMap::default();
            let loaded_annotations = LoadedAnnotations {
                campaign: board_data::campaign_map_data(),
                fall_of_khartoum: board_data::fall_of_khartoum_map_data(),
            };
            load_map_data(loaded_annotations.map(MapKind::FallOfKhartoum), &mut game_map);
            let overlay = HexOverlay {
                params: game_map.overlay.clone(),
            };
            Self {
                world: World::new(),
                queue: CommandQueue::default(),
                game_map,
                overlay,
                editor: HexEditor::default(),
                viewer: UnitViewer {
                    grids: vec![],
                    grids_dirty: false,
                    dirty_grids: std::collections::HashSet::new(),
                },
                incoming: vec![],
                history_peer: PeerId(Uuid::nil()),
                game_state: GameState::new(omdurman_types::Scenario::Campaign),
                queued_factions: QueuedFactions::default(),
                loaded_annotations,
                pending_map_load: PendingMapLoad::default(),
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
                    overlay: &mut self.overlay,
                    editor: &mut self.editor,
                    viewer: &mut self.viewer,
                    replay: &mut self.incoming,
                    game_state: &mut self.game_state,
                    queued_factions: &mut self.queued_factions,
                    loaded_annotations: &mut self.loaded_annotations,
                    pending_map_load: &mut self.pending_map_load,
                };
                rebuild_state_to(record, upto, self.history_peer, &mut state);
            }
            self.queue.apply(&mut self.world);
        }
    }

    // -- bounded rebuild (timeline scrub) -------------------------------------

    /// Rebuild to a bounded event index and return the resulting map (mirrors
    /// `run_replay` but exercises the `upto` scrub path used by the spectator
    /// timeline).
    fn run_replay_upto(record: &GameRecord, upto: usize) -> GameMap {
        let mut h = TestHarness::new();
        h.replay(record, Some(upto));
        h.game_map
    }

    #[test]
    fn scrub_applies_only_events_up_to_index() {
        // Two edits at distinct hexes on separate events. The map is
        // pre-populated from compiled codegen data (see TestHarness::new).
        // FOK board (OddR Rectangle 18×16) — q ranges depend on row:
        // r=0 → q in [1..18]; r=2 → q in [2..19]; r=4 → q in [3..20].
        let rough = Terrain::ground(omdurman_types::GroundKind::Rough);
        let mk_edit = |q, r, name: &str| GameEvent::MapEdit {
            map: omdurman_types::MapKind::FallOfKhartoum,
            coord: HexCoord::new(q, r),
            terrain: rough,
            name: name.into(),
        };
        let record = make_record(vec![
            mk_edit(2, 2, "first"),  // idx 0 — q=2 in [2..19] for r=2
            mk_edit(4, 4, "second"), // idx 1 — q=4 in [3..20] for r=4
        ]);

        // Scrub to idx 0: only the first edit is applied.
        let at_0 = run_replay_upto(&record, 0);
        assert_eq!(
            at_0.hexes.get(&HexCoord::new(2, 2)).map(|h| h.terrain),
            Some(rough),
            "first edit should be present at idx 0"
        );
        assert!(
            at_0.hexes
                .get(&HexCoord::new(4, 4))
                .is_none_or(|h| h.terrain != rough),
            "second edit must NOT be present at idx 0"
        );

        // Scrub to idx 1: both edits are applied.
        let at_1 = run_replay_upto(&record, 1);
        assert_eq!(
            at_1.hexes.get(&HexCoord::new(4, 4)).map(|h| h.terrain),
            Some(rough),
            "second edit should be present at idx 1"
        );
    }

    // -- map edit --------------------------------------------------------------

    #[test]
    fn map_edit_replayed() {
        // MapEdit applies to on-map coords; the map is pre-populated from
        // compiled codegen data (see TestHarness::new).
        // FOK board (OddR Rectangle 18×16) has q=[1..18] at r=0,
        // q=[2..19] at r=2, etc. — use (3, 2) which is well within bounds.
        let coord = HexCoord::new(3, 2);
        let record = make_record(vec![GameEvent::MapEdit {
            map: omdurman_types::MapKind::FallOfKhartoum,
            coord,
            terrain: Terrain::Rough { road: omdurman_types::Road::None },
            name: "Khartoum".into(),
        }]);
        let mut h = TestHarness::new();
        h.replay(&record, None);
        let hex = h
            .game_map
            .hexes
            .get(&coord)
            .expect("hex not found");
        assert_eq!(hex.terrain, Terrain::Rough { road: omdurman_types::Road::None });
        assert_eq!(hex.name.as_deref(), Some("Khartoum"));
    }

    // -- overlay update synced ------------------------------------------------

    #[test]
    fn overlay_update_replayed() {
        let params = OverlayParams {
            hex_size: 99.0,
            ..Default::default()
        };
        let record = make_record(vec![GameEvent::OverlayUpdate {
            map: omdurman_types::MapKind::FallOfKhartoum,
            params: params.clone(),
        }]);
        let mut h = TestHarness::new();
        h.replay(&record, None);
        assert_eq!(h.overlay.params.hex_size, 99.0);
    }

    // -- unit placement queued for apply_pending_placement --------------------

    #[test]
    fn place_unit_queued_in_incoming() {
        let record = make_record(vec![GameEvent::PlaceUnit {
            sprite: SpriteRef {
                section_name: SectionName::Baggara,
                col: 2,
                row: 3,
            },
            coord: HexCoord::new(5, 6),
            is_boat: false,
        }]);
        let mut h = TestHarness::new();
        h.replay(&record, None);
        assert_eq!(h.incoming.len(), 1);
        match &h.incoming[0].0 {
            GameEvent::PlaceUnit {
                sprite,
                coord,
                is_boat,
            } => {
                assert_eq!(sprite.section_name, SectionName::Baggara);
                assert_eq!(sprite.col, 2);
                assert_eq!(sprite.row, 3);
                assert_eq!(coord, &HexCoord::new(5, 6));
                assert!(!is_boat);
            }
            other => panic!("expected PlaceUnit, got {other:?}"),
        }
    }

    // -- move unit queued -----------------------------------------------------

    #[test]
    fn move_unit_queued_in_incoming() {
        let record = make_record(vec![
            GameEvent::PlaceUnit {
                sprite: SpriteRef {
                    section_name: SectionName::HadendowaForts,
                    col: 0,
                    row: 0,
                },
                coord: HexCoord::new(1, 1),
                is_boat: false,
            },
            GameEvent::MoveUnit {
                sprite: SpriteRef {
                    section_name: SectionName::HadendowaForts,
                    col: 0,
                    row: 0,
                },
                to_q: 7,
                to_r: 8,
                cost: MovementPoints::new(0),
                path: vec![],
            },
        ]);
        let mut h = TestHarness::new();
        h.replay(&record, None);
        assert_eq!(h.incoming.len(), 2);
        match &h.incoming[1].0 {
            GameEvent::MoveUnit { to_q, to_r, .. } => {
                assert_eq!(*to_q, 7);
                assert_eq!(*to_r, 8);
            }
            other => panic!("expected MoveUnit, got {other:?}"),
        }
    }

    // -- show terrain overlay -------------------------------------------------

    #[test]
    fn show_terrain_overlay_replayed() {
        let record = make_record(vec![GameEvent::ShowTerrainOverlay(true)]);
        let mut h = TestHarness::new();
        h.replay(&record, None);
        assert!(h.editor.show_terrain_overlay);
    }

    // -- unit grids synced ----------------------------------------------------

    #[test]
    fn unit_grids_replayed() {
        use omdurman_types::UnitGrid;
        let grids = vec![UnitGrid {
            name: "test_section".into(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            cols: 4,
            rows: 2,
        }];
        let record = make_record(vec![GameEvent::UpdateUnitGrids { grids: grids.clone() }]);
        let mut h = TestHarness::new();
        h.replay(&record, None);
        assert_eq!(h.viewer.grids.len(), 1);
        assert_eq!(h.viewer.grids[0].name, "test_section");
    }

    // -- move after place in same batch ---------------------------------------

    #[test]
    fn move_after_place_queued_in_order() {
        // PlaceUnit at (1,1) then MoveUnit to (7,8) -- both in the same replay
        // batch.  The incoming queue must contain both events in order so that
        // apply_pending_placement can use the just_placed fallback map to apply
        // the move even though Bevy hasn't flushed the spawn command yet.
        let record = make_record(vec![
            GameEvent::PlaceUnit {
                sprite: SpriteRef {
                    section_name: SectionName::Baggara,
                    col: 0,
                    row: 0,
                },
                coord: HexCoord::new(1, 1),
                is_boat: false,
            },
            GameEvent::MoveUnit {
                sprite: SpriteRef {
                    section_name: SectionName::Baggara,
                    col: 0,
                    row: 0,
                },
                to_q: 7,
                to_r: 8,
                cost: MovementPoints::new(0),
                path: vec![],
            },
        ]);
        let mut h = TestHarness::new();
        h.replay(&record, None);
        assert_eq!(
            h.incoming.len(),
            2,
            "both PlaceUnit and MoveUnit must be queued"
        );
        // PlaceUnit comes first
        assert!(matches!(
            &h.incoming[0].0,
            GameEvent::PlaceUnit {
                coord: HexCoord { q: 1, r: 1 },
                ..
            }
        ));
        // MoveUnit comes second, with the target coords
        assert!(matches!(
            &h.incoming[1].0,
            GameEvent::MoveUnit {
                to_q: 7,
                to_r: 8,
                ..
            }
        ));
    }

    // -- map is cleared before replay ----------------------------------------

    #[test]
    fn map_cleared_before_replay() {
        // Pre-populate the map with a hex that is NOT in the record.
        // After replay it must be gone, and the new hex must be present.
        // The map is pre-populated from compiled codegen data, then
        // rebuild_state_to clears it and replays only the record events.
        // FOK (OddR Rectangle 18×16): r=2 has q in [2..19].
        let coord = HexCoord::new(3, 2);
        let record = make_record(vec![
            GameEvent::MapEdit {
                map: MapKind::FallOfKhartoum,
                coord,
                terrain: Terrain::Rough { road: omdurman_types::Road::None },
                name: "".into(),
            },
        ]);

        let mut h = TestHarness::new();
        h.game_map.hexes.insert(
            HexCoord::new(99, 99),
            omdurman_types::HexData::new(Terrain::Swamp { road: omdurman_types::Road::None }, None),
        );
        h.replay(&record, None);

        assert!(
            !h.game_map.hexes.contains_key(&HexCoord::new(99, 99)),
            "stale hex must be cleared before replay"
        );
        assert!(h.game_map.hexes.contains_key(&coord));
    }

    // -- scenario selects the board (§dual-map) -------------------------------

    // §9.1
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
    // §9.2
    #[test]
    fn start_game_scenario_selects_board() {
        use omdurman_types::Scenario;

        let record = make_record(vec![GameEvent::StartGame {
            assignments: vec![],
            scenario: Scenario::Campaign,
            optional_rule: None,
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
            assert!(
                rec.events.iter().any(|e| matches!(
                    e.payload,
                    GameEvent::MapEdit { .. }
                        | GameEvent::PlaceUnit { .. }
                        | GameEvent::MoveUnit { .. }
                        | GameEvent::OverlayUpdate { .. }
                        | GameEvent::UpdateUnitGrids { .. }
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
            entries: vec![
                (2, "Second.".to_string()),
                (1, "First report.".to_string()),
            ],
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
        let map_data = match scenario {
            Scenario::Campaign | Scenario::Historical => {
                omdurman_rules::board_data::campaign_map_data()
            }
            Scenario::FallOfKhartoum => {
                omdurman_rules::board_data::fall_of_khartoum_map_data()
            }
        };
        let board = omdurman_rules::board::BoardInfo::from_map_data(&map_data);
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
            app.world().resource::<GameRecorder>()
                .record
                .as_ref()
                .unwrap()
                .initial_state
                .seed
        );
        dir
    }
}
