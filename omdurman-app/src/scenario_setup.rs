//! Automated scenario set-up -- emits the unambiguous, fixed-hex unit
//! placements for a scenario as ordinary [`GameEvent::PlaceUnit`] events.
//!
//! The rulebook's set-up (§9) is mostly *player choice* ("all green units set
//! up within three hexes of Sheik El Din", "anywhere in the walled city"). Only
//! a handful of units have a single fixed hex -- chiefly the six Dervish leaders,
//! which the Historical scenario pins to the lettered set-up hexes A/D/Y/K/S/O
//! (§9.212). Those letters exist on the campaign board as [`Terrain`] variants,
//! so we resolve each leader's destination from the loaded [`GameMap`] rather
//! than hard-coding coordinates -- the placement stays correct if the board is
//! re-annotated.
//!
//! Everything else (tribal retinues, brigades, gunboats) is left for players to
//! drag from the picker, near the leaders this anchors. Placements flow through
//! the same [`GameEvent::PlaceUnit`] path as interactive placement
//! (`apply_pending_placement`), so they are netcode-ordered and acquire rules
//! `UnitId`s identically -- see [[project_netcode_host_relay]] in memory.

use bevy::prelude::*;
use omdurman_hexmap::GameMap;
use omdurman_net::{GameEvent, NetMsg};
use omdurman_types::{HexCoord, MapKind, Scenario, SectionName, SetupLetter};

/// Which board a scenario plays on. Both the Campaign game (§9.1) and the
/// Historical scenario (§9.2) are the Battle of Omdurman fought on the main
/// Omdurman mapsheet -- they differ only in set-up, length, and victory, not
/// terrain -- so both use the campaign map (the lettered set-up hexes A/D/Y/K/S/O
/// of §9.212 live on it). Only the Fall-of-Khartoum bonus game (§9.3) uses the
/// separate tactical mini-map.
pub fn map_kind_for_scenario(scenario: Scenario) -> MapKind {
    match scenario {
        Scenario::Campaign | Scenario::Historical => MapKind::Campaign,
        Scenario::FallOfKhartoum => MapKind::FallOfKhartoum,
    }
}

/// What unambiguously fixes a counter's set-up hex on the board.
enum Anchor {
    /// A lettered set-up hex (Historical scenario, §9.212).
    Letter(SetupLetter),
    /// A named landmark hex (e.g. the Palace for GORDON, §9.321/§9.346).
    Location(omdurman_types::Location),
}

/// One fixed-hex placement: which counter (`section`/`col`/`row` on the sprite
/// sheet) goes onto the single hex identified by `anchor`.
struct FixedPlacement {
    section: SectionName,
    col: u32,
    row: u32,
    anchor: Anchor,
}

/// The six Dervish leaders and their Historical-scenario lettered set-up hexes
/// (§9.212). Two leaders (Yakub, Osman Digna) have no sprite section of their
/// own and ride in a tribal block -- see `omdurman_rules::unit_profiles::identity_for_section`,
/// which resolves those specific counters as leaders.
const HISTORICAL_LEADERS: &[FixedPlacement] = &[
    // A: Ali Wad Helu
    FixedPlacement {
        section: SectionName::AliWadHelu,
        col: 0,
        row: 0,
        anchor: Anchor::Letter(SetupLetter::A),
    },
    // D: Sheik El Din
    FixedPlacement {
        section: SectionName::SheikElDin,
        col: 0,
        row: 0,
        anchor: Anchor::Letter(SetupLetter::D),
    },
    // Y: Yakub (first counter of the upper_Jaalin block)
    FixedPlacement {
        section: SectionName::UpperJaalin,
        col: 0,
        row: 0,
        anchor: Anchor::Letter(SetupLetter::Y),
    },
    // K: Khalifa Abdullah
    FixedPlacement {
        section: SectionName::KhalifaAbdullah,
        col: 0,
        row: 0,
        anchor: Anchor::Letter(SetupLetter::K),
    },
    // S: Sherif
    FixedPlacement {
        section: SectionName::Sherif,
        col: 0,
        row: 0,
        anchor: Anchor::Letter(SetupLetter::S),
    },
    // O: Osman Digna (second counter of the Hadendowa block)
    FixedPlacement {
        section: SectionName::Hadendowa,
        col: 1,
        row: 0,
        anchor: Anchor::Letter(SetupLetter::O),
    },
];

/// Fall-of-Khartoum fixed placements (§9.321/§9.344/§9.346):
/// - GORDON is the one counter with a single, unambiguous hex -- he starts in
///   (and may never leave) the Palace.
/// - The North Fort at `(19,3)` is Dervish-controlled per §9.344. The engine
///   treats it as a `Fort` unit placed at the `Location::NorthFort` landmark;
///   its artillery factor fires on the Artillery line and it is enclosed by
///   its own wall ring (it cannot be entered by the British).
///
/// The rest of the British garrison and the Dervish entry forces are
/// player-placed (§9.321 "anywhere in the walled city", §9.322 map-edge entry).
/// GORDON is the "GEN. GORDON" counter at British_Boats (3,1); the North Fort
/// uses a campaign HadendowaForts counter (one of the spare fort sprites).
const FALL_OF_KHARTOUM_SETUP: &[FixedPlacement] = &[
    FixedPlacement {
        section: SectionName::BritishBoats,
        col: 3,
        row: 1,
        anchor: Anchor::Location(omdurman_types::Location::Palace),
    },
    FixedPlacement {
        section: SectionName::HadendowaForts,
        col: 0,
        row: 0,
        anchor: Anchor::Location(omdurman_types::Location::NorthFort),
    },
];

/// The single hex carrying `setup_letter` on the loaded map, if exactly one does.
/// The lettered set-up hexes are unique, so "first match" is the intended one;
/// returns `None` if the letter isn't on the board (e.g. the wrong scenario's
/// map is loaded).
fn hex_with_setup_letter(game_map: &GameMap, letter: SetupLetter) -> Option<HexCoord> {
    game_map
        .hexes
        .iter()
        .find(|(_, data)| data.setup_letter == Some(letter))
        .map(|(coord, _)| *coord)
}

/// The single hex whose name resolves to `location` on the loaded map (e.g. the
/// Palace), if any. Mirrors how the engine derives `board.locations` from tile
/// names, so the app and engine agree on where landmarks are.
fn hex_with_location(game_map: &GameMap, location: omdurman_types::Location) -> Option<HexCoord> {
    game_map
        .hexes
        .iter()
        .find(|(_, data)| {
            data.name
                .as_deref()
                .and_then(omdurman_types::Location::from_tile_name)
                == Some(location)
        })
        .map(|(coord, _)| *coord)
}

/// Resolve a placement's anchor to a hex on the loaded map.
fn resolve_anchor(game_map: &GameMap, anchor: &Anchor) -> Option<HexCoord> {
    match anchor {
        Anchor::Letter(letter) => hex_with_setup_letter(game_map, *letter),
        Anchor::Location(location) => hex_with_location(game_map, *location),
    }
}

/// Outcome of building a scenario's fixed placements: the `PlaceUnit` events to
/// broadcast, plus the names of any anchors that could not be resolved on the
/// current map (so the caller can surface them rather than silently dropping).
pub struct SetupPlan {
    pub placements: Vec<GameEvent>,
    pub unresolved: Vec<&'static str>,
}

/// Build the fixed-hex placements for `scenario` against the loaded `game_map`.
///
/// The Historical scenario pins its six Dervish leaders to lettered hexes
/// (§9.212); Fall of Khartoum pins GORDON to the Palace (§9.321/§9.346). The
/// Campaign game leaves all set-up to the players, so it returns an empty plan.
/// Everything not anchored here (tribal retinues, brigades, gunboats, the
/// Dervish entry forces) is player-placed.
pub fn build_setup_plan(scenario: Scenario, game_map: &GameMap) -> SetupPlan {
    let fixed: &[FixedPlacement] = match scenario {
        Scenario::Historical => HISTORICAL_LEADERS,
        Scenario::FallOfKhartoum => FALL_OF_KHARTOUM_SETUP,
        // The Campaign game leaves all set-up to the players.
        Scenario::Campaign => &[],
    };

    let mut placements = Vec::new();
    let mut unresolved = Vec::new();
    for fp in fixed {
        match resolve_anchor(game_map, &fp.anchor) {
            Some(coord) => placements.push(GameEvent::PlaceUnit {
                sprite: omdurman_types::SpriteRef {
                    section_name: fp.section,
                    col: fp.col,
                    row: fp.row,
                },
                coord,
                is_boat: false,
            }),
            None => unresolved.push(fp.section.display_name()),
        }
    }
    SetupPlan {
        placements,
        unresolved,
    }
}

/// Check whether a fixed-placement event has already been resolved by the
/// rules engine (i.e. the unit is on the board).
///
/// Rules-engine [`UnitId`]s are allocated sequentially and are opaque tokens
/// (a counter's id does not correspond to its sprite-sheet position), so a
/// canonical-id lookup cannot tell whether this counter was placed. Match by
/// the counter's profile identity at its target hex instead.
fn placement_already_on_board(
    ev: &GameEvent,
    gs: &omdurman_rules::effects::GameState,
) -> bool {
    let GameEvent::PlaceUnit {
        sprite,
        coord,
        ..
    } = ev
    else {
        return true;
    };
    let Some(uid) = omdurman_rules::unit_id_for_section_pos(
        sprite.section_name,
        sprite.col as u8,
        sprite.row as u8,
    ) else {
        return false;
    };
    let Some(profile) = omdurman_rules::unit_profiles::profile_for_unit(uid) else {
        return false;
    };
    gs.units
        .iter()
        .any(|u| u.position == *coord && u.profile.identity == profile.identity)
}

/// Auto-emit the fixed-hex scenario setup on the host when the game begins.
///
/// For Campaign there are no fixed placements so this is a no-op. For Historical
/// and Fall-of-Khartoum the host broadcasts the resolved placements once; guests
/// receive them via normal netcode relay. The system is idempotent: it re-runs
/// each frame but `build_setup_plan` + `placement_already_on_board` gate it so
/// events are emitted at most once.
pub(crate) fn auto_trigger_scenario_setup(
    game_state: Option<Res<crate::GameStateResource>>,
    game_map: Option<Res<GameMap>>,
    gate: crate::FactionGate<'_>,
    mut pending: ResMut<crate::PendingEdits>,
    mut done_scenario: Local<Option<Scenario>>,
) {
    let Some(state) = game_state else { return };
    let Some(map) = game_map else { return };

    // Only the host auto-triggers.
    if !gate.net.is_host {
        return;
    }

    // Only trigger during setup.
    if !matches!(state.0.phase, omdurman_rules::Phase::Setup) {
        *done_scenario = None; // reset for next game
        return;
    }

    // Already triggered this scenario -- skip.
    if *done_scenario == Some(state.0.scenario) {
        return;
    }

    let plan = build_setup_plan(state.0.scenario, &map);
    if plan.placements.is_empty() {
        if plan.unresolved.is_empty() {
            *done_scenario = Some(state.0.scenario); // Campaign -- nothing to do
        }
        // If unresolved is non-empty, the map hasn't loaded yet (anchors
        // not found).  Retry next frame.
        return;
    }

    // Wait until all placements are already on the board before declaring
    // "done" -- on the first frame the board may not be loaded yet, so we
    // simply re-emit until they stick.
    if plan
        .placements
        .iter()
        .all(|ev| placement_already_on_board(ev, &state.0))
    {
        *done_scenario = Some(state.0.scenario);
        return;
    }

    // Emit the placements -- they'll flow through host-relay sequencing
    // and be applied on the next frame.  Do NOT mark done yet: the
    // placement_already_on_board check above will confirm them on a
    // subsequent frame, and we retry each frame until they land.
    for ev in plan.placements {
        pending
            .outgoing_broadcast
            .push(NetMsg::Game(ev));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_types::{HexData, Road, Terrain};

    fn map_with(letters: &[(i32, i32, SetupLetter)]) -> GameMap {
        let mut m = GameMap::default();
        for &(q, r, l) in letters {
            let mut data = HexData::new(Terrain::Clear { road: Road::None }, None);
            data.setup_letter = Some(l);
            m.hexes.insert(HexCoord::new(q, r), data);
        }
        m
    }

    // §9.212
    #[test]
    fn historical_places_all_six_leaders_when_anchors_present() {
        let map = map_with(&[
            (15, 4, SetupLetter::A),
            (20, 10, SetupLetter::D),
            (26, 17, SetupLetter::Y),
            (28, 21, SetupLetter::K),
            (31, 23, SetupLetter::S),
            (33, 23, SetupLetter::O),
        ]);
        let plan = build_setup_plan(Scenario::Historical, &map);
        assert_eq!(plan.placements.len(), 6);
        assert!(plan.unresolved.is_empty());

        // The Khalifa counter must land on the K hex.
        let khalifa = plan
            .placements
            .iter()
            .find_map(|e| match e {
                GameEvent::PlaceUnit {
                    sprite,
                    coord,
                    ..
                } if sprite.section_name == SectionName::KhalifaAbdullah => {
                    Some((coord.q, coord.r))
                }
                _ => None,
            })
            .expect("Khalifa placement present");
        assert_eq!(khalifa, (28, 21));
    }

    // §9.212
    #[test]
    fn missing_anchor_is_reported_not_dropped_silently() {
        // Only the A hex exists; the other five leaders are unresolved.
        let map = map_with(&[(15, 4, SetupLetter::A)]);
        let plan = build_setup_plan(Scenario::Historical, &map);
        assert_eq!(plan.placements.len(), 1);
        assert_eq!(plan.unresolved.len(), 5);
    }

    // §9.1
    #[test]
    fn campaign_has_no_fixed_placements() {
        let map = map_with(&[(28, 21, SetupLetter::K)]);
        let plan = build_setup_plan(Scenario::Campaign, &map);
        assert!(plan.placements.is_empty());
        assert!(plan.unresolved.is_empty());
    }

    fn map_with_named(named: &[(i32, i32, &str)]) -> GameMap {
        let mut m = GameMap::default();
        for &(q, r, name) in named {
            let data = HexData::new(Terrain::Clear { road: Road::None }, Some(name.to_string()));
            m.hexes.insert(HexCoord::new(q, r), data);
        }
        m
    }

    // §9.321, §9.344, §9.346
    #[test]
    fn fall_of_khartoum_places_gordon_in_the_palace() {
        // §9.321/§9.346: GORDON (British_Boats 3,1) starts in the Palace hex.
        // §9.344: the North Fort is Dervish-controlled -- a fort counter is
        // auto-placed there alongside GORDON.
        let map = map_with_named(&[(7, 9, "Palace"), (3, 0, "North Fort")]);
        let plan = build_setup_plan(Scenario::FallOfKhartoum, &map);
        assert_eq!(plan.placements.len(), 2);
        assert!(plan.unresolved.is_empty());

        let gordon = plan
            .placements
            .iter()
            .find_map(|e| match e {
                GameEvent::PlaceUnit { sprite, coord, .. }
                    if sprite.section_name == SectionName::BritishBoats
                        && (sprite.col, sprite.row) == (3, 1) =>
                {
                    Some(*coord)
                }
                _ => None,
            })
            .expect("GORDON placement present");
        assert_eq!(gordon, HexCoord::new(7, 9));

        let fort = plan
            .placements
            .iter()
            .find_map(|e| match e {
                GameEvent::PlaceUnit { sprite, coord, .. }
                    if sprite.section_name == SectionName::HadendowaForts =>
                {
                    Some(*coord)
                }
                _ => None,
            })
            .expect("North Fort placement present");
        assert_eq!(fort, HexCoord::new(3, 0));
    }

    // §9.321
    #[test]
    fn fall_of_khartoum_reports_missing_palace() {
        // No Palace on the map -> GORDON is surfaced as unresolved, not dropped.
        // (The North Fort also resolves to nothing on this map.)
        let map = map_with_named(&[(0, 0, "Barracks")]);
        let plan = build_setup_plan(Scenario::FallOfKhartoum, &map);
        assert!(plan.placements.is_empty());
        assert_eq!(plan.unresolved.len(), 2);
    }

    // §9.321/§9.344 -- the FoK map's fort landmarks sit at the correct hexes:
    // Fort Makran at (4,1) is an AE set-up fort (§9.321); the Dervish-
    // controlled North Fort is at (19,3) (§9.344).
    #[test]
    fn fall_of_khartoum_fort_landmarks_sit_at_the_correct_hexes() {
        let map = omdurman_rules::board_data::fall_of_khartoum_map_data();
        let name_at = |q: i32, r: i32| {
            map.tiles
                .get(&(q, r))
                .and_then(|t| t.name.as_deref())
                .unwrap_or("")
        };
        assert_eq!(name_at(4, 1), "Fort Makran");
        assert_eq!(name_at(19, 3), "North Fort");

        // Each fort landmark appears exactly once, at its correct hex.
        assert!(map.tiles.iter().all(|((q, r), t)| {
            t.name.as_deref() != Some("North Fort") || (*q, *r) == (19, 3)
        }));
        assert!(map.tiles.iter().all(|((q, r), t)| {
            t.name.as_deref() != Some("Fort Makran") || (*q, *r) == (4, 1)
        }));

        // The setup plan must land the Dervish fort on the North Fort hex.
        let mut gm = GameMap::default();
        omdurman_hexmap::load_map_data(&map, &mut gm);
        let plan = build_setup_plan(Scenario::FallOfKhartoum, &gm);
        assert!(plan.unresolved.is_empty());
        let fort = plan
            .placements
            .iter()
            .find_map(|e| match e {
                GameEvent::PlaceUnit { sprite, coord, .. }
                    if sprite.section_name == SectionName::HadendowaForts =>
                {
                    Some(*coord)
                }
                _ => None,
            })
            .expect("North Fort placement present");
        assert_eq!(fort, HexCoord::new(19, 3));
    }

    // §9.344 -- the auto-setup done-gate must not key off allocated UnitIds,
    // which are sequential opaque tokens unrelated to the sprite position.
    #[test]
    fn placement_done_gate_matches_by_identity_not_allocated_id() {
        let map = map_with_named(&[(7, 9, "Palace"), (4, 1, "North Fort")]);
        let plan = build_setup_plan(Scenario::FallOfKhartoum, &map);
        assert_eq!(plan.placements.len(), 2);

        let mut gs = omdurman_rules::effects::GameState::new(Scenario::FallOfKhartoum);

        // No units placed yet -> nothing is "already on the board".
        assert!(
            !plan
                .placements
                .iter()
                .all(|ev| placement_already_on_board(ev, &gs))
        );

        // Place GORDON first (allocating `Gordon`, ALL[0]) and the North Fort
        // second (allocating the *next* sequential id, NOT HadendowaForts_0_0).
        for ev in &plan.placements {
            let GameEvent::PlaceUnit { sprite, coord, .. } = ev else {
                panic!("expected PlaceUnit");
            };
            let uid = omdurman_rules::unit_id_for_section_pos(
                sprite.section_name,
                sprite.col as u8,
                sprite.row as u8,
            )
            .expect("fixed placement has a unit id");
            let profile = omdurman_rules::unit_profiles::profile_for_unit(uid)
                .expect("fixed placement has a profile");
            let id = gs.alloc_unit_id();
            gs.units.push(omdurman_rules::UnitPlacement {
                id,
                position: *coord,
                profile,
                state: Default::default(),
            });
        }

        // Both placements must now count as resolved, even though the fort's
        // allocated id is not `HadendowaForts_0_0`.
        assert!(
            plan.placements
                .iter()
                .all(|ev| placement_already_on_board(ev, &gs))
        );
    }
}
