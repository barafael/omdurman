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
use omdurman_net::{GameEvent, PlayerKey, Seat, SeatHolder, SeatRequestKind};
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
    /// The key for this app instance. Native: persisted in a locked slot file
    /// in the user's config directory, so a relaunch reclaims the seat (see
    /// `player_key_store`). Web: persisted per browser tab in
    /// `sessionStorage`, so a reload keeps the seat.
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
        // Tests build the net plugin too; keep them out of the user's
        // config directory.
        #[cfg(all(not(target_arch = "wasm32"), not(test)))]
        {
            Self(crate::player_key_store::load_or_create())
        }
        #[cfg(all(not(target_arch = "wasm32"), test))]
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

/// Apply a `GameEvent::SeatAssigned`: seat `seat` passes from `previous`
/// to `holder`. Rejected (returns `false`, table untouched) when the index is
/// out of range, the seat is no longer held by `previous`, or a human
/// `holder` already holds another seat. Deterministic, so every peer and
/// every replay agree.
pub(crate) fn assign_seat(
    seats: &mut [Seat],
    seat: u8,
    previous: SeatHolder,
    holder: SeatHolder,
) -> bool {
    let index = usize::from(seat);
    let Some(current) = seats.get(index) else {
        return false;
    };
    if current.holder != previous {
        return false;
    }
    if let SeatHolder::Human(key) = holder
        && seat_of(seats, key).is_some_and(|(i, _)| i != index)
    {
        return false;
    }
    seats[index].holder = holder;
    true
}

/// Whether `scope` is a non-empty set of `faction`'s own kind (tribes for
/// the Dervish, brigades for the Anglo-Egyptian) -- the only scopes a
/// sub-faction takeover may carve.
pub(crate) fn carvable_scope(faction: Player, scope: &CommandScope) -> bool {
    !scope.claims_nothing()
        && matches!(
            (faction, scope),
            (Player::Dervish, CommandScope::Tribes(_))
                | (Player::AngloEgyptian, CommandScope::Brigades(_))
        )
}

/// Apply a `GameEvent::SeatCarved`: `holder` (who must hold no seat) gets a
/// new seat commanding `scope` of `faction`, and the carved tribes/brigades
/// leave every other seat of the faction (a set emptied that way becomes
/// `Army`). Whole-faction and `Army` seats are unchanged: the scope gate
/// already keeps them off units another seat claims. Rejected (returns
/// `false`, table untouched) for a non-carvable scope or a seated holder.
pub(crate) fn carve_seat(
    seats: &mut Vec<Seat>,
    faction: Player,
    scope: &CommandScope,
    holder: PlayerKey,
) -> bool {
    if !carvable_scope(faction, scope) || seat_of(seats, holder).is_some() {
        return false;
    }
    for seat in seats.iter_mut().filter(|s| s.faction == faction) {
        if let Some(own @ (CommandScope::Tribes(_) | CommandScope::Brigades(_))) = &seat.scope {
            seat.scope = Some(own.without(scope));
        }
    }
    seats.push(Seat {
        faction,
        scope: Some(scope.clone()),
        holder: SeatHolder::Human(holder),
    });
    true
}

/// Apply a seat event (`SeatAssigned` / `SeatCarved`) to a seat table.
/// `false` for a rejected seat event or any other event.
pub(crate) fn apply_seat_event(seats: &mut Vec<Seat>, event: &GameEvent) -> bool {
    match event {
        GameEvent::SeatAssigned {
            seat,
            previous,
            holder,
        } => assign_seat(seats, *seat, *previous, *holder),
        GameEvent::SeatCarved {
            faction,
            scope,
            holder,
        } => carve_seat(seats, *faction, scope, *holder),
        _ => false,
    }
}

/// The seat table after the given (e.g. still unconfirmed) seat events --
/// what a decision made now will be applied against.
pub(crate) fn projected_seats<'a>(
    seats: &[Seat],
    pending: impl IntoIterator<Item = &'a GameEvent>,
) -> Vec<Seat> {
    let mut table = seats.to_vec();
    for event in pending {
        apply_seat_event(&mut table, event);
    }
    table
}

/// The host's verdict on a seat request.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SeatDecision {
    /// Allowed outright: submit the recorded event.
    Submit(GameEvent),
    /// Needs the unanimous vote of the other seated humans; `summary`
    /// completes "{requester} asks to ...".
    Vote { event: GameEvent, summary: String },
    /// Refused, with the reason shown to the requester.
    Deny(String),
}

/// Decide a seat request against the (projected) seat table. `abandoned`
/// reports whether a key's seat is abandoned in the deciding host's view.
pub(crate) fn decide_request(
    seats: &[Seat],
    requester: PlayerKey,
    kind: &SeatRequestKind,
    abandoned: impl Fn(PlayerKey) -> bool,
) -> SeatDecision {
    let seated = seat_of(seats, requester).is_some();
    let seat_at = |seat: u8| seats.get(usize::from(seat));
    let decision = match kind {
        SeatRequestKind::ClaimAbandoned { seat } => match seat_at(*seat) {
            None => SeatDecision::Deny("That seat does not exist.".into()),
            Some(_) if seated => SeatDecision::Deny("You already hold a seat.".into()),
            Some(s) => match s.holder {
                SeatHolder::Human(key) if abandoned(key) => {
                    SeatDecision::Submit(GameEvent::SeatAssigned {
                        seat: *seat,
                        previous: s.holder,
                        holder: SeatHolder::Human(requester),
                    })
                }
                _ => SeatDecision::Deny("That seat is not abandoned.".into()),
            },
        },
        SeatRequestKind::HandToAi { seat } => match seat_at(*seat) {
            None => SeatDecision::Deny("That seat does not exist.".into()),
            Some(s) => match s.holder {
                SeatHolder::Human(key) if abandoned(key) => SeatDecision::Vote {
                    event: GameEvent::SeatAssigned {
                        seat: *seat,
                        previous: s.holder,
                        holder: SeatHolder::Ai,
                    },
                    summary: format!("hand the abandoned seat {} to the AI", seat_label(s)),
                },
                _ => SeatDecision::Deny("Only an abandoned seat can go to the AI.".into()),
            },
        },
        SeatRequestKind::ClaimFromAi { seat } => match seat_at(*seat) {
            None => SeatDecision::Deny("That seat does not exist.".into()),
            Some(_) if seated => SeatDecision::Deny("You already hold a seat.".into()),
            Some(s) if s.holder == SeatHolder::Ai => SeatDecision::Vote {
                event: GameEvent::SeatAssigned {
                    seat: *seat,
                    previous: SeatHolder::Ai,
                    holder: SeatHolder::Human(requester),
                },
                summary: format!("take over the AI seat {}", seat_label(s)),
            },
            Some(_) => SeatDecision::Deny("That seat is not held by the AI.".into()),
        },
        SeatRequestKind::TakeOver { faction, scope } => {
            if seated {
                SeatDecision::Deny("You already hold a seat.".into())
            } else if !carvable_scope(*faction, scope) {
                SeatDecision::Deny("Pick tribes (Dervish) or brigades (Anglo-Egyptian).".into())
            } else if !human_seats(seats).any(|(_, s)| s.faction == *faction) {
                SeatDecision::Deny(
                    "No human commands that side; ask for its AI seat instead.".into(),
                )
            } else {
                SeatDecision::Vote {
                    event: GameEvent::SeatCarved {
                        faction: *faction,
                        scope: scope.clone(),
                        holder: requester,
                    },
                    summary: format!("take over {scope} ({})", crate::ui::faction_name(*faction)),
                }
            }
        }
    };
    // Belt and braces: whatever is allowed must apply to this table.
    match &decision {
        SeatDecision::Submit(event) | SeatDecision::Vote { event, .. }
            if !apply_seat_event(&mut seats.to_vec(), event) =>
        {
            SeatDecision::Deny("That seat change no longer applies.".into())
        }
        _ => decision,
    }
}

/// The voters on `requester`'s request: every connected human seat holder
/// except the requester (sorted, deduplicated).
pub(crate) fn vote_voters(
    seats: &[Seat],
    requester: PlayerKey,
    connected: impl Fn(PlayerKey) -> bool,
) -> Vec<PlayerKey> {
    let mut voters: Vec<PlayerKey> = human_seats(seats)
        .map(|(key, _)| key)
        .filter(|key| *key != requester && connected(*key))
        .collect();
    voters.sort();
    voters.dedup();
    voters
}

/// How long a seat vote stays open before it is denied.
pub const SEAT_VOTE_SECS: f64 = 60.0;

/// One open seat vote on the host.
#[derive(Clone, Debug)]
struct OpenVote {
    request_id: u64,
    event: GameEvent,
    voters: Vec<PlayerKey>,
    approvals: HashSet<PlayerKey>,
    deadline: f64,
}

/// A decided vote.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum VoteOutcome {
    Approved { request_id: u64, event: GameEvent },
    Denied { request_id: u64, reason: String },
}

/// Host-side vote state machine: unanimous approval of the listed voters
/// passes a request; any "no" or the deadline denies it. Host-local: a host
/// failover drops open votes (clients expire their ballots at the deadline).
#[derive(Resource, Default, Debug)]
pub struct VoteBook {
    open: Vec<OpenVote>,
}

impl VoteBook {
    /// Open a vote. With no voters the request passes at once (returned);
    /// a duplicate `request_id` is ignored.
    pub(crate) fn open(
        &mut self,
        request_id: u64,
        event: GameEvent,
        voters: Vec<PlayerKey>,
        now: f64,
    ) -> Option<VoteOutcome> {
        if self.is_open(request_id) {
            return None;
        }
        if voters.is_empty() {
            return Some(VoteOutcome::Approved { request_id, event });
        }
        self.open.push(OpenVote {
            request_id,
            event,
            voters,
            approvals: HashSet::new(),
            deadline: now + SEAT_VOTE_SECS,
        });
        None
    }

    /// Whether `request_id` is being voted on.
    pub(crate) fn is_open(&self, request_id: u64) -> bool {
        self.open.iter().any(|v| v.request_id == request_id)
    }

    /// Record `voter`'s ballot. Ballots from non-voters or on unknown votes
    /// are ignored. Returns the outcome once the vote is decided.
    pub(crate) fn vote(
        &mut self,
        request_id: u64,
        voter: PlayerKey,
        approve: bool,
    ) -> Option<VoteOutcome> {
        let index = self.open.iter().position(|v| v.request_id == request_id)?;
        let vote = &mut self.open[index];
        if !vote.voters.contains(&voter) {
            return None;
        }
        if !approve {
            self.open.remove(index);
            return Some(VoteOutcome::Denied {
                request_id,
                reason: "A commander declined.".into(),
            });
        }
        vote.approvals.insert(voter);
        if vote.voters.iter().all(|v| vote.approvals.contains(v)) {
            let vote = self.open.remove(index);
            return Some(VoteOutcome::Approved {
                request_id,
                event: vote.event,
            });
        }
        None
    }

    /// Deny every vote past its deadline.
    pub(crate) fn tick(&mut self, now: f64) -> Vec<VoteOutcome> {
        let mut expired = Vec::new();
        self.open.retain(|v| {
            if now >= v.deadline {
                expired.push(VoteOutcome::Denied {
                    request_id: v.request_id,
                    reason: "The vote timed out.".into(),
                });
                false
            } else {
                true
            }
        });
        expired
    }

    /// Drop every open vote (this peer is no longer the host).
    pub(crate) fn clear(&mut self) {
        self.open.clear();
    }
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

    // -- seat changes ---------------------------------------------------

    #[test]
    fn carving_moves_tribes_to_the_new_seat() {
        let mut seats = table();
        seats[0].scope = tribes(&[DervishTribe::Baggara, DervishTribe::Hadendowa]);
        let newcomer = PlayerKey(50);
        let carved = CommandScope::Tribes(BTreeSet::from([DervishTribe::Hadendowa]));
        assert!(carve_seat(&mut seats, Player::Dervish, &carved, newcomer));
        assert_eq!(seats[0].scope, tribes(&[DervishTribe::Baggara]));
        assert_eq!(seats[1].scope, tribes(&[DervishTribe::Jaalin]), "untouched");
        assert_eq!(seats[3].holder, SeatHolder::Human(newcomer));
        assert!(scope_allows(
            &seats,
            newcomer,
            &tribal(DervishTribe::Hadendowa)
        ));
        assert!(!scope_allows(&seats, ME, &tribal(DervishTribe::Hadendowa)));
    }

    #[test]
    fn carving_everything_leaves_army() {
        let mut seats = table();
        let carved = CommandScope::Tribes(BTreeSet::from([DervishTribe::Jaalin]));
        assert!(carve_seat(
            &mut seats,
            Player::Dervish,
            &carved,
            PlayerKey(50)
        ));
        assert_eq!(seats[1].scope, Some(CommandScope::Army));
    }

    #[test]
    fn carving_rejects_bad_scopes_and_seated_holders() {
        let mut seats = table();
        let before = seats.clone();
        let jaalin = CommandScope::Tribes(BTreeSet::from([DervishTribe::Jaalin]));
        assert!(!carve_seat(
            &mut seats,
            Player::AngloEgyptian,
            &jaalin,
            PlayerKey(50)
        ));
        assert!(!carve_seat(
            &mut seats,
            Player::Dervish,
            &CommandScope::Army,
            PlayerKey(50)
        ));
        assert!(!carve_seat(&mut seats, Player::Dervish, &jaalin, FOE));
        assert_eq!(seats, before);
    }

    #[test]
    fn assignment_requires_the_expected_previous_holder() {
        let mut seats = table();
        let newcomer = SeatHolder::Human(PlayerKey(50));
        assert!(!assign_seat(&mut seats, 1, SeatHolder::Ai, newcomer));
        assert!(!assign_seat(
            &mut seats,
            9,
            SeatHolder::Human(MATE),
            newcomer
        ));
        // A human may not take a second seat.
        assert!(!assign_seat(
            &mut seats,
            1,
            SeatHolder::Human(MATE),
            SeatHolder::Human(ME)
        ));
        assert!(assign_seat(
            &mut seats,
            1,
            SeatHolder::Human(MATE),
            newcomer
        ));
        // The same decision replayed a second time no longer applies.
        assert!(!assign_seat(
            &mut seats,
            1,
            SeatHolder::Human(MATE),
            newcomer
        ));
    }

    #[test]
    fn claiming_an_abandoned_seat_needs_no_vote_but_needs_abandonment() {
        let seats = table();
        let newcomer = PlayerKey(50);
        let claim = SeatRequestKind::ClaimAbandoned { seat: 1 };
        assert!(matches!(
            decide_request(&seats, newcomer, &claim, |_| false),
            SeatDecision::Deny(_)
        ));
        assert_eq!(
            decide_request(&seats, newcomer, &claim, |k| k == MATE),
            SeatDecision::Submit(GameEvent::SeatAssigned {
                seat: 1,
                previous: SeatHolder::Human(MATE),
                holder: SeatHolder::Human(newcomer),
            })
        );
        // A seated player cannot grab a second seat.
        assert!(matches!(
            decide_request(&seats, ME, &claim, |k| k == MATE),
            SeatDecision::Deny(_)
        ));
    }

    #[test]
    fn takeover_hand_to_ai_and_claim_from_ai_go_to_a_vote() {
        let mut seats = table();
        let newcomer = PlayerKey(50);
        let takeover = SeatRequestKind::TakeOver {
            faction: Player::Dervish,
            scope: CommandScope::Tribes(BTreeSet::from([DervishTribe::Jaalin])),
        };
        assert!(matches!(
            decide_request(&seats, newcomer, &takeover, |_| false),
            SeatDecision::Vote {
                event: GameEvent::SeatCarved { .. },
                ..
            }
        ));
        let hand = SeatRequestKind::HandToAi { seat: 1 };
        assert!(matches!(
            decide_request(&seats, ME, &hand, |_| false),
            SeatDecision::Deny(_)
        ));
        assert!(matches!(
            decide_request(&seats, ME, &hand, |k| k == MATE),
            SeatDecision::Vote { .. }
        ));
        seats[1].holder = SeatHolder::Ai;
        let back = SeatRequestKind::ClaimFromAi { seat: 1 };
        assert!(matches!(
            decide_request(&seats, newcomer, &back, |_| false),
            SeatDecision::Vote { .. }
        ));
        // An all-AI side is claimed through its AI seat, not carved.
        let all_ai = vec![seat(Player::Dervish, None, SeatHolder::Ai)];
        assert!(matches!(
            decide_request(&all_ai, newcomer, &takeover, |_| false),
            SeatDecision::Deny(_)
        ));
    }

    #[test]
    fn projection_sees_unconfirmed_seat_events() {
        let seats = table();
        let newcomer = PlayerKey(50);
        let pending = [GameEvent::SeatAssigned {
            seat: 1,
            previous: SeatHolder::Human(MATE),
            holder: SeatHolder::Human(newcomer),
        }];
        let projected = projected_seats(&seats, &pending);
        // A second claim of the same seat is refused against the projection.
        let other = PlayerKey(51);
        assert!(matches!(
            decide_request(
                &projected,
                other,
                &SeatRequestKind::ClaimAbandoned { seat: 1 },
                |k| k == MATE
            ),
            SeatDecision::Deny(_)
        ));
    }

    // -- votes ----------------------------------------------------------

    fn carve_event() -> GameEvent {
        GameEvent::SeatCarved {
            faction: Player::Dervish,
            scope: CommandScope::Tribes(BTreeSet::from([DervishTribe::Jaalin])),
            holder: PlayerKey(50),
        }
    }

    #[test]
    fn voters_are_connected_seat_holders_except_the_requester() {
        let seats = table();
        assert_eq!(
            vote_voters(&seats, PlayerKey(50), |_| true),
            vec![ME, MATE, FOE]
        );
        assert_eq!(vote_voters(&seats, ME, |k| k != FOE), vec![MATE]);
    }

    #[test]
    fn unanimous_approval_passes() {
        let mut book = VoteBook::default();
        assert_eq!(book.open(7, carve_event(), vec![ME, FOE], 0.0), None);
        assert_eq!(book.vote(7, ME, true), None);
        assert_eq!(book.vote(7, PlayerKey(50), true), None, "not a voter");
        assert_eq!(
            book.vote(7, FOE, true),
            Some(VoteOutcome::Approved {
                request_id: 7,
                event: carve_event()
            })
        );
        assert!(!book.is_open(7));
    }

    #[test]
    fn any_no_denies() {
        let mut book = VoteBook::default();
        book.open(7, carve_event(), vec![ME, FOE], 0.0);
        book.vote(7, ME, true);
        assert!(matches!(
            book.vote(7, FOE, false),
            Some(VoteOutcome::Denied { request_id: 7, .. })
        ));
        assert_eq!(book.vote(7, FOE, true), None, "closed");
    }

    #[test]
    fn a_silent_vote_times_out() {
        let mut book = VoteBook::default();
        book.open(7, carve_event(), vec![ME], 10.0);
        assert!(book.tick(10.0 + SEAT_VOTE_SECS - 1.0).is_empty());
        assert!(matches!(
            book.tick(10.0 + SEAT_VOTE_SECS).as_slice(),
            [VoteOutcome::Denied { request_id: 7, .. }]
        ));
        assert!(!book.is_open(7));
    }

    #[test]
    fn zero_voters_pass_at_once() {
        let mut book = VoteBook::default();
        assert!(matches!(
            book.open(7, carve_event(), Vec::new(), 0.0),
            Some(VoteOutcome::Approved { request_id: 7, .. })
        ));
        assert!(!book.is_open(7));
    }
}
