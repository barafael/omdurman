//! The seat table: who commands what in the running game.
//!
//! `GameEvent::StartGame` commits a list of [`Seat`]s -- faction, optional
//! §1.1 command scope, and holder (a human by stable [`PlayerKey`], or the
//! AI). The [`Seats`] resource mirrors it and is written *only* by
//! `game_apply::apply_game_event`, so the live echo and every replay agree.
//! The local player's seat is found by the process-stable
//! [`LocalPlayerKey`], never by the session `PeerId`: a reconnect rebuilds
//! the socket (fresh `PeerId`) but keeps the key, so the returning player is
//! bound to their seat again as soon as the history is replayed.
//!
//! The query functions below are pure over `(&[Seat], PlayerKey)` so the
//! action gates in [`crate::peers::Peers`] are unit-testable.
//!
//! [`SeatPresence`] is the local, unrecorded view of which human seat
//! holders are connected right now. Any absent holder pauses the game
//! (every `Peers::may_act` gate and the host's AI driver stop); a holder
//! absent for [`SEAT_ABANDON_SECS`] leaves an *abandoned* seat.

use bevy::prelude::*;
use omdurman_net::{PlayerKey, Seat, SeatHolder};
use omdurman_rules::UnitIdentity;
use omdurman_rules::unit_profiles::command_owns_unit;
use omdurman_types::{CommandScope, Player};
use std::collections::{HashMap, HashSet};

/// The committed seat table of the running game (empty before any
/// `StartGame`). Written only by the recorded-event apply path.
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct Seats(pub Vec<Seat>);

/// This app instance's stable player identity (see [`PlayerKey`]).
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalPlayerKey(pub PlayerKey);

impl LocalPlayerKey {
    /// The key for this process (native: fresh per launch) or browser tab
    /// (web: persisted in `sessionStorage`, so a reload keeps the seat).
    pub fn load_or_create() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            const STORAGE_KEY: &str = "omdurman.player_key";
            let storage = web_sys::window().and_then(|w| w.session_storage().ok().flatten());
            if let Some(storage) = storage.as_ref()
                && let Ok(Some(stored)) = storage.get_item(STORAGE_KEY)
                && let Ok(value) = u64::from_str_radix(&stored, 16)
            {
                return Self(PlayerKey(value));
            }
            let key = PlayerKey::random();
            if let Some(storage) = storage {
                let _ = storage.set_item(STORAGE_KEY, &key.to_string());
            }
            Self(key)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(PlayerKey::random())
        }
    }
}

/// The seat `key` holds, if any (with its index in the table). A key holds
/// at most one seat: the lobby emits one per roster row and the seat arbiter
/// only hands seats to seatless keys.
pub(crate) fn seat_of(seats: &[Seat], key: PlayerKey) -> Option<(usize, &Seat)> {
    seats
        .iter()
        .enumerate()
        .find(|(_, s)| s.holder == SeatHolder::Human(key))
}

/// Whether a seat table exists at all (a `StartGame` committed seats). Once
/// it does, only seat holders may act; before it, the game is unbound and
/// every peer may (solo play, legacy records).
pub(crate) fn any_bound(seats: &[Seat]) -> bool {
    !seats.is_empty()
}

/// Human-held seats only. AI seats never gate a human: their scopes do not
/// claim units (a sub-faction handed to the AI while humans remain on the
/// side becomes communal for those humans).
pub(crate) fn human_seats(seats: &[Seat]) -> impl Iterator<Item = (PlayerKey, &Seat)> {
    seats
        .iter()
        .filter_map(|s| s.holder.human().map(|key| (key, s)))
}

/// Whether any human seat carries a §1.1 command scope.
pub(crate) fn any_commands(seats: &[Seat]) -> bool {
    human_seats(seats).any(|(_, s)| s.scope.is_some())
}

/// Whether any human seat's scope claims `identity`'s tribe/brigade. Units
/// nobody claims are the faction's communal pool (§1.1).
pub(crate) fn any_claims(seats: &[Seat], identity: &UnitIdentity) -> bool {
    human_seats(seats).any(|(_, s)| {
        s.scope
            .as_ref()
            .is_some_and(|scope| command_owns_unit(scope, identity))
    })
}

/// Whether `key` may act for `active`: unbound sessions allow everyone;
/// once seats exist, the key must hold a seat of `active`'s faction.
pub(crate) fn may_act(seats: &[Seat], key: PlayerKey, active: Player) -> bool {
    match seat_of(seats, key) {
        Some((_, seat)) => seat.faction == active,
        None => !any_bound(seats),
    }
}

/// Whether `key`'s seat scope lets it act on `identity` (the command-scope
/// half of the action gates). `true` whenever no human seat carries a
/// scope; otherwise the unit must be in the key's scope or communal.
pub(crate) fn scope_allows(seats: &[Seat], key: PlayerKey, identity: &UnitIdentity) -> bool {
    if !any_commands(seats) {
        return true;
    }
    let local_scope: Option<&CommandScope> =
        seat_of(seats, key).and_then(|(_, s)| s.scope.as_ref());
    match local_scope {
        Some(scope) => command_owns_unit(scope, identity) || !any_claims(seats, identity),
        // Whole-faction or seatless: the communal pool only.
        None => !any_claims(seats, identity),
    }
}

/// The factions the AI plays: every seat of the faction is an AI seat. A
/// faction that still has a human seat is played by its humans (an AI
/// sub-seat's units are then communal, see [`human_seats`]).
pub(crate) fn ai_factions(seats: &[Seat]) -> Vec<Player> {
    [Player::AngloEgyptian, Player::Dervish]
        .into_iter()
        .filter(|&f| {
            let mut of_faction = seats.iter().filter(|s| s.faction == f).peekable();
            of_faction.peek().is_some() && of_faction.all(|s| s.holder == SeatHolder::Ai)
        })
        .collect()
}

/// Short human-readable description of a seat ("Dervish · Tribes Baggara").
pub(crate) fn seat_label(seat: &Seat) -> String {
    let faction = crate::ui::faction_name(seat.faction);
    match &seat.scope {
        None => faction.to_string(),
        Some(scope) => format!("{faction} \u{b7} {scope}"),
    }
}

/// How long a seat holder must be gone before the seat counts as abandoned
/// (claimable by a newcomer without a vote).
pub const SEAT_ABANDON_SECS: f64 = 60.0;

/// Connection state of one human seat holder, as this peer sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Presence {
    Connected,
    /// Not connected since `since` (app clock seconds).
    Disconnected {
        since: f64,
    },
}

/// Local (never recorded) presence of every human seat holder, refreshed each
/// frame by [`update_seat_presence`]. Drives the pause and abandonment.
#[derive(Resource, Default, Debug)]
pub struct SeatPresence {
    status: HashMap<PlayerKey, Presence>,
    /// Last announced display name per key; outlives the peer's entity so
    /// the pause notice can name who we are waiting for.
    names: HashMap<PlayerKey, String>,
    /// App clock at the last update.
    now: f64,
    paused: bool,
}

impl SeatPresence {
    /// Refresh from the seat table and the keys connected right now.
    /// Holders newly missing start their abandonment clock at `now`; keys
    /// that no longer hold a human seat are forgotten. The game is paused
    /// while any holder is missing, unless it is over.
    pub fn update(
        &mut self,
        seats: &[Seat],
        connected: &HashSet<PlayerKey>,
        now: f64,
        game_over: bool,
    ) {
        self.now = now;
        let held: HashSet<PlayerKey> = human_seats(seats).map(|(key, _)| key).collect();
        self.status.retain(|key, _| held.contains(key));
        for key in held {
            let entry = self
                .status
                .entry(key)
                .or_insert(Presence::Disconnected { since: now });
            if connected.contains(&key) {
                *entry = Presence::Connected;
            } else if *entry == Presence::Connected {
                *entry = Presence::Disconnected { since: now };
            }
        }
        self.paused = !game_over
            && self
                .status
                .values()
                .any(|p| matches!(p, Presence::Disconnected { .. }));
    }

    /// Whether play is suspended waiting for a seat holder.
    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Seconds `key` has been gone, if it holds a seat and is disconnected.
    pub fn absent_secs(&self, key: PlayerKey) -> Option<f64> {
        match self.status.get(&key)? {
            Presence::Connected => None,
            Presence::Disconnected { since } => Some((self.now - since).max(0.0)),
        }
    }

    /// Whether `key`'s seat is abandoned: its holder has been gone for at
    /// least [`SEAT_ABANDON_SECS`].
    pub fn abandoned(&self, key: PlayerKey) -> bool {
        self.absent_secs(key)
            .is_some_and(|secs| secs >= SEAT_ABANDON_SECS)
    }

    /// Seconds until `key`'s seat becomes abandoned (0 once it is), or
    /// `None` while its holder is connected.
    pub fn secs_until_abandoned(&self, key: PlayerKey) -> Option<f64> {
        self.absent_secs(key)
            .map(|secs| (SEAT_ABANDON_SECS - secs).max(0.0))
    }

    /// Whether `key` holds a seat and is connected.
    pub fn is_connected(&self, key: PlayerKey) -> bool {
        self.status.get(&key) == Some(&Presence::Connected)
    }

    /// Remember `key`'s display name.
    pub fn remember_name(&mut self, key: PlayerKey, name: &str) {
        if self.names.get(&key).map(String::as_str) != Some(name) {
            self.names.insert(key, name.to_owned());
        }
    }

    /// `key`'s last announced display name, or a neutral fallback.
    pub fn name(&self, key: PlayerKey) -> String {
        self.names
            .get(&key)
            .cloned()
            .unwrap_or_else(|| "a player".to_string())
    }
}

/// The seat table plus live presence, bundled for systems near Bevy's
/// parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct SeatView<'w> {
    pub seats: Res<'w, Seats>,
    pub presence: Res<'w, SeatPresence>,
}

/// Refresh [`SeatPresence`] from the connected peers' announced keys (plus
/// our own) every frame. Cheap: a handful of seats and peers.
pub(crate) fn update_seat_presence(
    time: Res<Time>,
    seats: Res<Seats>,
    local_key: Res<LocalPlayerKey>,
    settings: Res<crate::settings::LocalPlayerSettings>,
    game_state: Res<crate::GameStateResource>,
    peers: Query<(
        &crate::peers::PeerPlayerKey,
        Option<&crate::peers::PeerName>,
    )>,
    mut presence: ResMut<SeatPresence>,
) {
    let mut connected: HashSet<PlayerKey> = HashSet::with_capacity(peers.iter().len() + 1);
    connected.insert(local_key.0);
    for (key, name) in &peers {
        connected.insert(key.0);
        if let Some(name) = name {
            presence.remember_name(key.0, &name.0);
        }
    }
    presence.remember_name(local_key.0, &settings.name);
    presence.update(
        &seats.0,
        &connected,
        time.elapsed_secs_f64(),
        game_state.0.game_over,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_types::DervishTribe;
    use std::collections::BTreeSet;

    const ME: PlayerKey = PlayerKey(1);
    const MATE: PlayerKey = PlayerKey(2);
    const FOE: PlayerKey = PlayerKey(3);

    fn seat(faction: Player, scope: Option<CommandScope>, holder: SeatHolder) -> Seat {
        Seat {
            faction,
            scope,
            holder,
        }
    }

    fn tribes(t: &[DervishTribe]) -> Option<CommandScope> {
        Some(CommandScope::Tribes(
            t.iter().copied().collect::<BTreeSet<_>>(),
        ))
    }

    fn tribal(tribe: DervishTribe) -> UnitIdentity {
        UnitIdentity::DervishTribal { tribe }
    }

    fn table() -> Vec<Seat> {
        vec![
            seat(
                Player::Dervish,
                tribes(&[DervishTribe::Baggara]),
                SeatHolder::Human(ME),
            ),
            seat(
                Player::Dervish,
                tribes(&[DervishTribe::Jaalin]),
                SeatHolder::Human(MATE),
            ),
            seat(Player::AngloEgyptian, None, SeatHolder::Human(FOE)),
        ]
    }

    #[test]
    fn unbound_session_lets_everyone_act() {
        assert!(may_act(&[], ME, Player::Dervish));
        assert!(may_act(&[], ME, Player::AngloEgyptian));
        assert!(scope_allows(&[], ME, &tribal(DervishTribe::Jaalin)));
    }

    #[test]
    fn seat_holder_acts_only_for_their_faction() {
        let seats = table();
        assert!(may_act(&seats, ME, Player::Dervish));
        assert!(!may_act(&seats, ME, Player::AngloEgyptian));
        assert!(may_act(&seats, FOE, Player::AngloEgyptian));
    }

    #[test]
    fn seatless_key_is_a_spectator() {
        let seats = table();
        let stranger = PlayerKey(99);
        assert!(seat_of(&seats, stranger).is_none());
        assert!(!may_act(&seats, stranger, Player::Dervish));
        assert!(!may_act(&seats, stranger, Player::AngloEgyptian));
    }

    #[test]
    fn an_ai_only_table_still_binds() {
        let seats = vec![
            seat(Player::Dervish, None, SeatHolder::Ai),
            seat(Player::AngloEgyptian, None, SeatHolder::Ai),
        ];
        assert!(!may_act(&seats, ME, Player::Dervish));
        assert_eq!(
            ai_factions(&seats),
            vec![Player::AngloEgyptian, Player::Dervish]
        );
    }

    #[test]
    fn scope_gates_claimed_units_and_shares_communal_ones() {
        let seats = table();
        assert!(scope_allows(&seats, ME, &tribal(DervishTribe::Baggara)));
        assert!(!scope_allows(&seats, ME, &tribal(DervishTribe::Jaalin)));
        // Nobody claims the Hadendowa: communal.
        assert!(scope_allows(&seats, ME, &tribal(DervishTribe::Hadendowa)));
        assert!(scope_allows(&seats, MATE, &tribal(DervishTribe::Jaalin)));
    }

    #[test]
    fn ai_sub_seat_units_are_communal_for_remaining_humans() {
        let mut seats = table();
        seats[1].holder = SeatHolder::Ai;
        assert!(scope_allows(&seats, ME, &tribal(DervishTribe::Jaalin)));
        // A faction with a human seat left is played by its humans.
        assert!(ai_factions(&seats).is_empty());
    }

    #[test]
    fn rebinding_by_key_survives_a_new_peer_id() {
        // The seat names the key only: whatever PeerId the holder connects
        // under, the same key finds the same seat.
        let seats = table();
        assert_eq!(seat_of(&seats, MATE).map(|(i, _)| i), Some(1));
    }

    fn presence_after(seats: &[Seat], connected: &[PlayerKey], now: f64) -> SeatPresence {
        let mut p = SeatPresence::default();
        p.update(seats, &connected.iter().copied().collect(), 0.0, false);
        p.update(seats, &connected.iter().copied().collect(), now, false);
        p
    }

    #[test]
    fn everyone_connected_is_not_paused() {
        let p = presence_after(&table(), &[ME, MATE, FOE], 5.0);
        assert!(!p.paused());
        assert!(p.is_connected(MATE));
        assert_eq!(p.secs_until_abandoned(MATE), None);
    }

    #[test]
    fn a_missing_holder_pauses_immediately_and_abandons_after_the_timeout() {
        let seats = table();
        let all: HashSet<PlayerKey> = [ME, MATE, FOE].into_iter().collect();
        let without_mate: HashSet<PlayerKey> = [ME, FOE].into_iter().collect();
        let mut p = SeatPresence::default();
        p.update(&seats, &all, 10.0, false);
        p.update(&seats, &without_mate, 12.0, false);
        assert!(p.paused(), "pause is immediate");
        assert!(!p.abandoned(MATE));
        assert_eq!(p.secs_until_abandoned(MATE), Some(SEAT_ABANDON_SECS));
        p.update(&seats, &without_mate, 12.0 + SEAT_ABANDON_SECS - 1.0, false);
        assert!(!p.abandoned(MATE));
        p.update(&seats, &without_mate, 12.0 + SEAT_ABANDON_SECS, false);
        assert!(p.abandoned(MATE));
        assert_eq!(p.secs_until_abandoned(MATE), Some(0.0));
        // The holder returns (same key, any PeerId): resumed, clock reset.
        p.update(&seats, &all, 100.0, false);
        assert!(!p.paused());
        assert!(!p.abandoned(MATE));
    }

    #[test]
    fn a_finished_game_never_pauses() {
        let seats = table();
        let mut p = SeatPresence::default();
        p.update(&seats, &[ME].into_iter().collect(), 1.0, true);
        assert!(!p.paused());
    }

    #[test]
    fn ai_seats_and_empty_tables_never_pause() {
        let seats = vec![seat(Player::Dervish, None, SeatHolder::Ai)];
        let p = presence_after(&seats, &[], 100.0);
        assert!(!p.paused());
        let p = presence_after(&[], &[], 100.0);
        assert!(!p.paused());
    }

    #[test]
    fn keys_that_lose_their_seat_are_forgotten() {
        let mut seats = table();
        let mut p = SeatPresence::default();
        p.update(&seats, &[ME, FOE].into_iter().collect(), 0.0, false);
        assert!(p.paused());
        seats[1].holder = SeatHolder::Ai;
        p.update(&seats, &[ME, FOE].into_iter().collect(), 1.0, false);
        assert!(!p.paused());
        assert_eq!(p.absent_secs(MATE), None);
    }
}
