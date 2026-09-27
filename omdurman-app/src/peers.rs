//! Peer entities.
//!
//! Each connected peer (plus the local peer) is an [`Entity`] carrying its
//! [`PeerKey`] (the session `PeerId`) and any data we hold about that peer:
//! the stable [`PeerPlayerKey`], name and colour announced via
//! `Ephemeral::PlayerInfo` ([`PeerName`] / [`PeerColor`]), the live cursor
//! position (`Ephemeral::CursorPos`, [`PeerCursor`]), and the pre-commit lobby
//! picks (`Ephemeral::FactionChoice` / `SpectatorChoice`,
//! [`LobbyPick`] / [`Spectator`]).
//!
//! Seat bindings do *not* live here: they are the committed seat table
//! ([`crate::seats::Seats`], written only by the recorded-event apply path)
//! keyed by stable [`PlayerKey`]s. [`Peers`] reads that table plus the
//! [`LocalPlayerKey`] for every action gate. [`sync_peer_entities`] keeps the
//! set of peer entities reconciled with `NetState::peers` each frame.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;
use omdurman_net::{NetState, PlayerKey};
use omdurman_rules::UnitIdentity;
use omdurman_types::{CommandScope, Player};
use std::collections::{HashMap, HashSet};

use crate::seats::{self, LocalPlayerKey, Seats};

/// Marker component for a peer entity (one per connected peer, plus the local
/// peer). Used to despawn the whole set during a timeline scrub teardown.
#[derive(Component)]
pub struct Peer;

/// The `PeerId` backing a peer entity.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct PeerKey(pub PeerId);

/// The stable [`PlayerKey`] a remote peer announced in
/// `Ephemeral::PlayerInfo`: how its seat is found in the seat table.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct PeerPlayerKey(pub PlayerKey);

/// Pre-commit lobby command pick received via `Ephemeral::CommandChoice`
/// (live preview only; the binding is committed by `StartGame`'s seats).
#[derive(Component, Clone, Default)]
pub struct CommandPick(pub Option<CommandScope>);

/// Setup-phase member readiness (`Ephemeral::SetupReady`, §9.2/§9.3): the
/// peer has deployed everything their §1.1 command requires. The faction's
/// engine-level `ConfirmSetupReady` fires once every member is ready. The
/// local peer's own flag lives in [`LocalSetupReady`] (its broadcasts are
/// not echoed back to it).
#[derive(Component, Clone, Copy, Default)]
pub struct SetupReadyFlag(pub bool);

/// The local peer's setup-readiness flag (see [`SetupReadyFlag`]).
#[derive(Resource, Default)]
pub struct LocalSetupReady(pub bool);

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

/// Query data backing [`Peers`]' setup-readiness check: each connected
/// peer's stable key and readiness flag.
pub type PeerReadyQueryData = (&'static PeerPlayerKey, Option<&'static SetupReadyFlag>);

/// Read-only view of the seat table from the local player's side, used by
/// the per-player action gates (§lobby).
#[derive(SystemParam)]
pub struct Peers<'w, 's> {
    seats: Res<'w, Seats>,
    key: Res<'w, LocalPlayerKey>,
    presence: Res<'w, seats::SeatPresence>,
    ready: Query<'w, 's, PeerReadyQueryData, With<Peer>>,
}

impl Peers<'_, '_> {
    /// The faction the local player's seat commands, if they hold one.
    pub fn local(&self) -> Option<Player> {
        seats::seat_of(&self.seats.0, self.key.0).map(|(_, s)| s.faction)
    }

    /// The command scope of the local player's seat, if any (§1.1).
    pub fn local_scope(&self) -> Option<CommandScope> {
        seats::seat_of(&self.seats.0, self.key.0).and_then(|(_, s)| s.scope.clone())
    }

    /// Whether any human seat carries a command scope.
    pub fn any_commands(&self) -> bool {
        seats::any_commands(&self.seats.0)
    }

    /// How many human seats (the local one included) belong to `player`'s
    /// faction.
    pub fn faction_size(&self, player: Player) -> usize {
        seats::human_seats(&self.seats.0)
            .filter(|(_, s)| s.faction == player)
            .count()
    }

    /// Whether every *other* human seat of `player`'s faction has flagged
    /// itself ready for setup (§9.2/§9.3 per-member readiness). Counted per
    /// seat, not per connected peer: a teammate who is not connected is not
    /// ready. `true` when the local player has no teammates on the side.
    pub fn faction_others_ready(&self, player: Player) -> bool {
        let me = self.key.0;
        seats::human_seats(&self.seats.0)
            .filter(|(key, s)| *key != me && s.faction == player)
            .all(|(key, _)| {
                self.ready
                    .iter()
                    .any(|(peer_key, flag)| peer_key.0 == key && flag.is_some_and(|f| f.0))
            })
    }

    /// Whether a seat table exists (a `StartGame` committed seats).
    pub fn any_assigned(&self) -> bool {
        seats::any_bound(&self.seats.0)
    }

    /// Whether the local player may act right now: their seat's faction is
    /// the rules engine's active player. Before any seat table exists (no
    /// lobby) this returns `true` so the game stays playable; once one
    /// exists the local player must hold a seat (§lobby). Nobody may act
    /// while the game is paused waiting for an absent seat holder: every
    /// action gate inherits the pause from here.
    pub fn may_act(&self, active: Player) -> bool {
        !self.presence.paused() && seats::may_act(&self.seats.0, self.key.0, active)
    }

    /// Whether the local player's seat commands `player`'s faction (or the
    /// session is unbound), regardless of the pause. For labels ("(you)"),
    /// not for gating actions -- use [`may_act`](Self::may_act) for that.
    pub fn commands_faction(&self, player: Player) -> bool {
        seats::may_act(&self.seats.0, self.key.0, player)
    }

    /// Whether play is suspended waiting for an absent seat holder.
    pub fn paused(&self) -> bool {
        self.presence.paused()
    }

    /// Whether the local seat's §1.1 command scope lets it act on this unit
    /// (the command-scope half of the action gates; pair with
    /// [`may_act`](Self::may_act) for the turn/faction gate). `true`
    /// whenever no seat carries a scope: the local scope must claim the
    /// unit's tribe/brigade, unless the unit is communal (no scope claims
    /// it), which every faction member may act on.
    pub fn scope_allows(&self, identity: &UnitIdentity) -> bool {
        seats::scope_allows(&self.seats.0, self.key.0, identity)
    }

    /// Whether the local player is a spectator: a seat table exists but no
    /// seat is held by the local key, so it watches only.
    pub fn is_spectator(&self) -> bool {
        self.any_assigned() && self.local().is_none()
    }
}

/// Roster view of the peer set: one lobby row per peer, with the announced
/// display data (`PeerName`/`PeerColor`), the pre-commit faction pick, the
/// pre-commit command scope, and the spectator marker. Used by the lobby
/// roster UI.
pub type RosterQueryData = (
    &'static PeerKey,
    Option<&'static PeerPlayerKey>,
    Option<&'static PeerName>,
    Option<&'static PeerColor>,
    Option<&'static LobbyPick>,
    Option<&'static CommandPick>,
    Has<Spectator>,
);
pub type RosterQuery<'w, 's> = Query<'w, 's, RosterQueryData, With<Peer>>;

/// Routing view of the peer set: resolves which entity an incoming ephemeral
/// event belongs to, plus the live cursor and announced name.
pub type PeerRouteQueryData = (
    Entity,
    &'static PeerKey,
    Option<&'static PeerCursor>,
    Option<&'static PeerName>,
);
pub type PeerRouteQuery<'w, 's> = Query<'w, 's, PeerRouteQueryData, With<Peer>>;

/// Remote-cursor overlay view: name/colour for the label plus the live cursor
/// component (mutated to advance the display interpolation).
pub type PeerCursorQueryData = (
    Entity,
    &'static PeerKey,
    Option<&'static PeerName>,
    Option<&'static PeerColor>,
    &'static mut PeerCursor,
);
pub type PeerCursorQuery<'w, 's> = Query<'w, 's, PeerCursorQueryData, With<Peer>>;

/// Reconcile peer entities with `NetState::peers` each frame: spawn new
/// peers, despawn peers that left, and re-point the local peer at its current
/// `PeerId`. A cheap no-op in the common case. Seat bindings are keyed by
/// [`PlayerKey`], so nothing needs carrying across a reconnect. Gated off
/// while spectating, where the scrubber owns the peer set.
pub(crate) fn sync_peer_entities(
    mut commands: Commands,
    net: Res<NetState>,
    mut local: ResMut<LocalPeer>,
    peers: Query<(Entity, &PeerKey)>,
) {
    let desired: HashSet<PeerId> = {
        let mut s = net.peers.iter().copied().collect::<HashSet<_>>();
        if let Some(my) = net.my_id {
            s.insert(my);
        }
        s
    };

    let mut by_key: HashMap<PeerId, Entity> = HashMap::new();
    for (entity, key) in &peers {
        by_key.insert(key.0, entity);
    }

    for &id in &desired {
        by_key
            .entry(id)
            .or_insert_with(|| commands.spawn((Peer, PeerKey(id))).id());
    }

    local.0 = net.my_id.map(|my| by_key[&my]);

    for (entity, key) in &peers {
        if !desired.contains(&key.0) {
            commands.entity(entity).despawn();
        }
    }
}
