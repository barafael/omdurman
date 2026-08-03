//! Peer entities.
//!
//! Each connected peer (plus the local peer) is an [`Entity`] carrying its
//! [`PeerKey`] and any data we hold about that peer: the authoritative faction
//! binding from `StartGame` ([`AssignedFaction`]), the name/colour announced
//! via `Ephemeral::PlayerInfo` ([`PeerName`] / [`PeerColor`]), the live cursor
//! position (`Ephemeral::CursorPos`, [`PeerCursor`]), and the pre-commit lobby
//! picks (`Ephemeral::FactionChoice` / `SpectatorChoice`,
//! [`LobbyPick`] / [`Spectator`]).
//!
//! This replaces the old per-peer resource maps (`PlayerFactions`,
//! `PlayerInfoMap`, `CursorPositions`, `LobbyChoices`) with ECS components.
//! [`sync_peer_entities`] keeps the set of peer entities reconciled with
//! `NetState::peers` each frame (spawning on connect, despawning on leave,
//! transferring the local faction binding across a reconnect); faction
//! bindings produced by a `StartGame` handler (live, replayed, or restored
//! from a snapshot) are staged in [`QueuedFactions`] and applied by
//! [`apply_faction_bindings`] once the entities exist.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;
use omdurman_net::NetState;
use omdurman_types::Player;
use std::collections::{HashMap, HashSet};

/// Marker component for a peer entity (one per connected peer, plus the local
/// peer). Used to despawn the whole set during a timeline scrub teardown.
#[derive(Component)]
pub struct Peer;

/// The `PeerId` backing a peer entity.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct PeerKey(pub PeerId);

/// The authoritative faction binding established by `GameEvent::StartGame`
/// (§lobby). `Some(player)` for a playing peer; `None` once the game started
/// without assigning this peer a faction (a spectator).
#[derive(Component, Clone, Copy)]
pub struct AssignedFaction(pub Option<Player>);

/// Display name announced via `Ephemeral::PlayerInfo`.
#[derive(Component)]
pub struct PeerName(pub String);

/// Display colour announced via `Ephemeral::PlayerInfo`.
#[derive(Component)]
pub struct PeerColor(pub bevy_egui::egui::Color32);

/// Live cursor position in world space (`Vec2(world.x, world.z)` — the
/// cursor's hit point on the ground plane) received via
/// `Ephemeral::CursorPos`, plus the interpolation state `cursor_overlay_ui`
/// maintains (previous position, update timestamp, smoothed display value).
#[derive(Component, Default)]
pub struct PeerCursor {
    pub current: Option<Vec2>,
    pub previous: Option<Vec2>,
    pub last_update: f64,
    pub display: Option<Vec2>,
}

/// Pre-commit lobby faction pick received via `Ephemeral::FactionChoice`.
#[derive(Component, Clone, Copy, Default)]
pub struct LobbyPick(pub Option<Player>);

/// Marker: the peer chose to spectate (via `Ephemeral::SpectatorChoice`), so it
/// is never assigned a faction and is ignored by the start-readiness check.
#[derive(Component)]
pub struct Spectator;

/// The local peer's entity. Managed by [`sync_peer_entities`] each frame.
#[derive(Resource, Default)]
pub struct LocalPeer(pub Option<Entity>);

/// Faction bindings staged by a `StartGame` handler (live `handle_socket`,
/// replay `rebuild_state_to`, or snapshot restore) and applied to peer entities
/// by [`apply_faction_bindings`]. Staged because the entities may not exist yet
/// when the handler runs (they are spawned by [`sync_peer_entities`]).
#[derive(Resource, Default)]
pub struct QueuedFactions(pub Option<Vec<(PeerId, Player)>>);

/// Read-only view of the peer set used by the per-player action gates (§lobby).
#[derive(SystemParam)]
pub struct Peers<'w, 's> {
    local: Res<'w, LocalPeer>,
    query: Query<'w, 's, (&'static PeerKey, Option<&'static AssignedFaction>)>,
}

impl Peers<'_, '_> {
    /// The faction the local peer commands, if the game has assigned one.
    pub fn local(&self) -> Option<Player> {
        let entity = self.local.0?;
        self.query
            .get(entity)
            .ok()
            .and_then(|(_, faction)| faction.and_then(|f| f.0))
    }

    /// Whether any peer has an assigned faction (i.e. a `StartGame` binding
    /// exists).
    pub fn any_assigned(&self) -> bool {
        self.query
            .iter()
            .any(|(_, faction)| faction.is_some_and(|f| f.0.is_some()))
    }

    /// Whether the local player may act right now: their faction is the rules
    /// engine's active player. Before any binding exists (no lobby) this
    /// returns `true` so the game stays playable; once a binding exists the
    /// local peer must be in it (§lobby).
    pub fn may_act(&self, active: Player) -> bool {
        match self.local() {
            Some(mine) => mine == active,
            None => !self.any_assigned(),
        }
    }

    /// Whether the local peer is a spectator: a faction binding exists but this
    /// peer isn't in it, so it joined to watch only.
    pub fn is_spectator(&self) -> bool {
        self.any_assigned() && self.local().is_none()
    }

    /// The full faction binding as `(peer_id, faction)` pairs (for snapshots).
    pub fn assignments(&self) -> Vec<(PeerId, Player)> {
        self.query
            .iter()
            .filter_map(|(key, faction)| faction.and_then(|f| f.0).map(|f| (key.0, f)))
            .collect()
    }
}

/// Reconcile peer entities with `NetState::peers` each frame: spawn new peers,
/// despawn peers that left, and re-point the local peer at its current
/// `PeerId`, carrying the faction binding across a reconnect. A cheap no-op in
/// the common case. Gated off while spectating, where the scrubber owns the
/// peer set (rebuilt from the reviewed record).
pub(crate) fn sync_peer_entities(
    mut commands: Commands,
    net: Res<NetState>,
    mut local: ResMut<LocalPeer>,
    peers: Query<(Entity, &PeerKey, Option<&AssignedFaction>)>,
) {
    let desired: HashSet<PeerId> = {
        let mut s = net.peers.iter().copied().collect::<HashSet<_>>();
        if let Some(my) = net.my_id {
            s.insert(my);
        }
        s
    };

    let mut by_key: HashMap<PeerId, Entity> = HashMap::new();
    for (entity, key, _) in &peers {
        by_key.insert(key.0, entity);
    }

    for &id in &desired {
        if !by_key.contains_key(&id) {
            let entity = commands.spawn((Peer, PeerKey(id))).id();
            by_key.insert(id, entity);
        }
    }

    if let Some(my) = net.my_id {
        let my_entity = by_key[&my];
        // A reconnect issues a fresh `PeerId`: if the local entity just moved
        // to a new id, carry the old one's faction binding across so the
        // player isn't silently demoted to a spectator of their own game.
        // Faction is the durable player identity here (there are exactly two
        // playable sides), so reclaiming "my" faction is unambiguous.
        //
        // Best-effort: if the disconnect was processed a frame earlier the old
        // entity is already despawned, `peers.get(old)` returns `Err`, and we
        // fall through without copying -- the binding is then re-established
        // by the next `apply_faction_bindings` from the staged `QueuedFactions`.
        if let Some(old) = local.0
            && old != my_entity
            && let Ok((_, _, faction)) = peers.get(old)
            && let Some(AssignedFaction(Some(f))) = faction
        {
            commands
                .entity(my_entity)
                .insert(AssignedFaction(Some(*f)));
            info!("transferred local faction binding across reconnect");
        }
        local.0 = Some(my_entity);
    } else {
        local.0 = None;
    }

    for (entity, key, _) in &peers {
        if !desired.contains(&key.0) {
            commands.entity(entity).despawn();
        }
    }
}

/// Apply a staged faction binding to the peer entities, clearing the
/// `AssignedFaction` on peers not in the binding (spectators). Spawns entities
/// for bindings that have no peer yet (e.g. bindings reconstructed from a
/// replayed record).
pub(crate) fn apply_faction_bindings(
    mut queued: ResMut<QueuedFactions>,
    mut commands: Commands,
    peers: Query<(Entity, &PeerKey, Option<&AssignedFaction>)>,
) {
    let Some(assignments) = queued.0.take() else {
        return;
    };
    let by_key: HashMap<PeerId, Entity> =
        peers.iter().map(|(e, k, _)| (k.0, e)).collect();

    for (entity, key, current) in &peers {
        let faction = assignments
            .iter()
            .find(|(pid, _)| *pid == key.0)
            .map(|(_, f)| *f);
        if current.map(|c| c.0) != Some(faction) {
            commands.entity(entity).insert(AssignedFaction(faction));
        }
    }
    for &(pid, faction) in &assignments {
        if !by_key.contains_key(&pid) {
            commands.spawn((Peer, PeerKey(pid), AssignedFaction(Some(faction))));
        }
    }
}
