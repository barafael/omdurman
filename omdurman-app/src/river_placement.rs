//! Hex-click handler for river-mine and river-chain placement (§10.11, §10.21).
//!
//! During the Setup phase, the Dervish player may place up to two river mines on
//! Nile hexes, or a single river chain spanning up to four contiguous Nile hexes.
//! This system checks for a hex click and emits the corresponding `GameEffect`.

use bevy::prelude::*;

use crate::board_click::RiverPlacementClick;
use crate::ui_plugin::OptionalRulePlacement;
use crate::{GameStateResource, PendingEdits};
use omdurman_net::GameEvent;

/// Emits `PlaceMine` or `PlaceChain` effects when the Dervish player clicks a
/// Nile hex during Setup with the placement UI active. (Phase and faction
/// gate: `board_click::click_mode` routes a [`RiverPlacementClick`] only in
/// Setup, only while a placement is armed, and only for a seat that
/// `may_act(Dervish)`.) The engine decides which hexes are legal (§10.11,
/// §10.21: Nile hexes south of the Khor Shambat's mouth, the chain a line of
/// adjacent hexes); a refused mine or chain hex posts its slip.
pub(crate) fn handle_optional_rule_click(
    mut clicks: bevy::ecs::message::MessageReader<RiverPlacementClick>,
    game_state: Option<Res<GameStateResource>>,
    mut placement: ResMut<OptionalRulePlacement>,
    mut submit: crate::submit::CheckedSubmit,
) {
    let Some(&RiverPlacementClick(hex)) = clicks.read().last() else {
        return;
    };
    let Some(gs) = game_state else { return };

    if placement.placing_mine {
        if submit.submit(
            &gs.0,
            GameEvent::Effect(omdurman_rules::effects::GameEffect::PlaceMine { hex }),
        ) {
            placement.placing_mine = false;
        }
        return;
    }

    if placement.placing_chain {
        let mut hexes = placement.chain_hexes.clone();
        hexes.push(hex);
        // A refused hex says why (not adjacent to the line, not Nile, north
        // of the khor's mouth), like a refused mine.
        if let Err(error) = gs.0.can_plot_chain(&hexes) {
            if let Some(dispatches) = submit.dispatches.as_deref_mut() {
                dispatches.push(crate::submit::REFUSED_HEADER, error.to_string());
            }
            return;
        }
        placement.chain_hexes = hexes;
        if placement.chain_hexes.len() == omdurman_rules::effects::MAX_CHAIN_HEXES {
            submit.submit(
                &gs.0,
                GameEvent::Effect(omdurman_rules::effects::GameEffect::PlaceChain {
                    hexes: std::mem::take(&mut placement.chain_hexes),
                }),
            );
            placement.placing_chain = false;
        }
    }
}

/// §10.12: "the Dervish player then resolves the effect of the mine's blast
/// by rolling" -- the Dervish seat's app rolls for a British gunboat that
/// struck a mine, once per strike. (An AI-held Dervish seat rolls through the
/// bot driver.) On any screen, against the live state (`timeline::LiveGame`):
/// the A-E End Phase waits for this roll, so a Dervish player in the menu or
/// a review must not hold the game up.
pub(crate) fn roll_for_struck_mine(
    live: crate::timeline::LiveGame,
    peers: crate::peers::Peers,
    seats: Res<crate::seats::Seats>,
    mut game_rng: ResMut<crate::GameRng>,
    mut pending: ResMut<PendingEdits>,
    mut rolled_for: Local<Option<omdurman_rules::StruckMine>>,
) {
    let Some(struck) = live.state().pending_mine() else {
        *rolled_for = None;
        return;
    };
    let human_dervish = peers.may_act(omdurman_types::Player::Dervish)
        && !crate::seats::ai_factions(&seats.0).contains(&omdurman_types::Player::Dervish);
    if !human_dervish || *rolled_for == Some(struck) {
        return;
    }
    *rolled_for = Some(struck);
    pending.submit_game(GameEvent::Effect(
        omdurman_rules::effects::GameEffect::RiverMine {
            gunboat_id: struck.gunboat,
            hex: struck.hex,
            roll: game_rng.roll_d10(),
        },
    ));
}

/// Board markers for placed river mines (§10.11) and the river chain
/// (§10.21). Both are *secretly recorded*, so the Dervish seat (and unbound
/// seats) sees them whole; every other seat sees only what the British have
/// found -- the mines their gunboats struck (§10.12) and the chained hexes
/// they ran onto (§10.22).
pub(crate) fn mine_chain_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    game_state: Res<GameStateResource>,
    peers: crate::peers::Peers,
    existing: Query<Entity, With<MineChainMarker>>,
    mut last: Local<Option<MineChainKey>>,
    (generation, mut seen_generation): (Res<crate::picker::OverlayGeneration>, Local<u32>),
) {
    // Rebuilt only when what is shown changes (or the overlays were
    // cleared), not every frame.
    if generation.invalidates(&mut seen_generation) {
        *last = None;
    }
    let gs = game_state;
    let whole = peers.may_act(omdurman_types::Player::Dervish);
    let key = MineChainKey {
        mines: gs
            .0
            .mines
            .iter()
            .filter(|m| whole || m.triggered)
            .map(|m| m.hex)
            .collect(),
        chain: gs
            .0
            .chain
            .as_ref()
            .map(|c| {
                if whole {
                    c.hexes.clone()
                } else {
                    c.struck.clone()
                }
            })
            .unwrap_or_default(),
        whole,
    };
    if last.as_ref() == Some(&key) {
        return;
    }
    let mut rings = crate::overlay::ring_batch(&mut commands, &hex, existing.iter());

    // Mines: compact red rings on their Nile hexes.
    for &mine in &key.mines {
        rings.ring(MineChainMarker, mine, 0.7, 0.35, &hex.assets.red);
    }
    // Chain: a grey ring on each known chained hex, and -- for the seat that
    // knows the whole line -- grey bars spanning consecutive hexes.
    for &chained in &key.chain {
        rings.ring(MineChainMarker, chained, 0.7, 0.35, &hex.assets.gray);
    }
    let chain = if key.whole {
        key.chain.clone()
    } else {
        Vec::new()
    };
    *last = Some(key);
    let origin = rings.origin();
    let size = rings.size();
    let params = rings.params();
    let bars: Vec<_> = chain
        .windows(2)
        .map(|pair| {
            let a = omdurman_hexmap::hex_world_pos(pair[0], origin, params);
            let b = omdurman_hexmap::hex_world_pos(pair[1], origin, params);
            let mid = (a + b) * 0.5;
            let len = a.distance(b).max(0.001);
            let dir = (b - a) / len;
            let angle = (-dir.z).atan2(dir.x);
            (mid, len, angle)
        })
        .collect();
    for (mid, len, angle) in bars {
        rings.commands().spawn((
            MineChainMarker,
            Mesh3d(hex.assets.unit_square.clone()),
            MeshMaterial3d(hex.assets.gray.clone()),
            Transform::from_translation(Vec3::new(mid.x, 0.7, mid.z))
                .with_rotation(
                    Quat::from_rotation_y(angle)
                        * Quat::from_rotation_x(-std::f32::consts::PI / 2.0),
                )
                .with_scale(Vec3::new(len, size * 0.12, 1.0)),
            Visibility::Visible,
        ));
    }
}

/// What [`mine_chain_overlay_mesh`] last drew: the mine and chain hexes
/// known to this seat, and whether it knows the whole chain line.
#[derive(PartialEq, Eq, Debug)]
pub(crate) struct MineChainKey {
    mines: Vec<omdurman_types::HexCoord>,
    chain: Vec<omdurman_types::HexCoord>,
    whole: bool,
}

/// Marker component for mine/chain board markers.
#[derive(Component)]
pub(crate) struct MineChainMarker;
