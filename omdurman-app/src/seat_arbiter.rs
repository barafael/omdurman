//! Seat claims and votes over the network.
//!
//! Guests never submit seat events. A player asks the elected host with a
//! reliable `Control::SeatRequest`; the host decides it against the
//! (projected) seat table ([`seats::decide_request`]): an abandoned seat is
//! handed over at once, anything else goes to a unanimous vote of the other
//! connected seat holders ([`VoteBook`]). Only the host submits the recorded
//! `GameEvent::SeatAssigned` / `SeatCarved`, through the ordinary
//! sequenced path, so the seat table stays a pure function of the log.
//!
//! [`SeatClient`] is every peer's side of the conversation: the requests it
//! sent (for the join panel's status lines) and the ballots it was asked to
//! cast (the vote popup). UI systems only touch [`SeatClient`]'s outboxes;
//! [`seat_control`] owns all the traffic.

use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;
use omdurman_net::{Control, NetMsg, NetState, PlayerKey, SeatRequestKind};
use std::collections::VecDeque;

use crate::seats::{self, SEAT_VOTE_SECS, SeatDecision, VoteBook, VoteOutcome};
use crate::{PendingEdits, PendingIncoming};

/// Dispatch-slip header for seat request outcomes.
pub(crate) const SEAT_HEADER: &str = "Seat Request";

/// A request this peer sent, as the join panel shows it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MyRequest {
    pub request_id: u64,
    pub kind: SeatRequestKind,
    pub status: RequestStatus,
    /// App clock when sent (for the no-answer timeout).
    pub sent_at: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RequestStatus {
    /// Sent; the host has not answered yet.
    Pending,
    /// The other commanders are voting until `deadline` (app clock).
    Voting {
        deadline: f64,
    },
    Approved,
    Denied(String),
}

/// A vote this peer is asked to cast.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Ballot {
    pub request_id: u64,
    pub requester: PlayerKey,
    pub summary: String,
    /// App clock after which the ballot is dropped.
    pub deadline: f64,
}

/// Every peer's seat-request state (see the module docs).
#[derive(Resource, Default, Debug)]
pub(crate) struct SeatClient {
    pub requests: Vec<MyRequest>,
    pub ballots: Vec<Ballot>,
    /// Requests the UI wants sent.
    pub outbox: Vec<SeatRequestKind>,
    /// Ballots the UI cast: `(request_id, approve)`.
    pub cast: Vec<(u64, bool)>,
    /// The spectator folded the join panel away ("Keep watching").
    pub join_panel_folded: bool,
}

impl SeatClient {
    /// Whether a request of this kind is still awaiting its outcome.
    pub(crate) fn is_pending(&self, kind: &SeatRequestKind) -> bool {
        self.requests.iter().any(|r| {
            r.kind == *kind
                && matches!(
                    r.status,
                    RequestStatus::Pending | RequestStatus::Voting { .. }
                )
        })
    }

    /// Whether any request is awaiting its outcome.
    pub(crate) fn any_pending(&self) -> bool {
        self.requests.iter().any(|r| {
            matches!(
                r.status,
                RequestStatus::Pending | RequestStatus::Voting { .. }
            )
        })
    }
}

/// A request unanswered for this long (e.g. the host left mid-vote) is
/// given up on.
const NO_ANSWER_SECS: f64 = SEAT_VOTE_SECS + 10.0;

/// Network half of [`seat_control`], bundled under the parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct SeatTraffic<'w> {
    net: Res<'w, NetState>,
    pending: ResMut<'w, PendingEdits>,
    incoming: ResMut<'w, PendingIncoming>,
}

/// Seat state half of [`seat_control`].
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct SeatBooks<'w> {
    view: seats::SeatView<'w>,
    local_key: Res<'w, seats::LocalPlayerKey>,
    book: ResMut<'w, VoteBook>,
    client: ResMut<'w, SeatClient>,
    dispatches: Option<ResMut<'w, crate::dispatch::Dispatches>>,
}

/// Where a message produced this frame goes.
enum Route {
    /// To the elected host (looped back locally when we are the host).
    Host(Control),
    /// To every peer, and to ourselves.
    All(Control),
}

/// Drive seat requests and votes: send what the UI queued, arbitrate on the
/// host, update the client state, expire stale ballots/requests.
pub(crate) fn seat_control(time: Res<Time>, mut traffic: SeatTraffic, mut books: SeatBooks) {
    let now = time.elapsed_secs_f64();
    let is_host = traffic.net.is_host;
    let me = books.local_key.0;
    let my_id = traffic.net.my_id().unwrap_or(PeerId(uuid::Uuid::nil()));
    if !is_host {
        // Votes live on the host; a demoted host drops its open votes
        // (clients expire their ballots at the deadline).
        books.book.clear();
    }

    let mut queue: VecDeque<(Control, PeerId)> = traffic.incoming.seat_control.drain(..).collect();
    let mut out: Vec<Route> = Vec::new();

    // -- this peer's own requests and ballots --
    for kind in std::mem::take(&mut books.client.outbox) {
        if books.client.is_pending(&kind) {
            continue;
        }
        let request_id: u64 = rand::random();
        books.client.requests.push(MyRequest {
            request_id,
            kind: kind.clone(),
            status: RequestStatus::Pending,
            sent_at: now,
        });
        out.push(Route::Host(Control::SeatRequest {
            request_id,
            requester: me,
            kind,
        }));
    }
    for (request_id, approve) in std::mem::take(&mut books.client.cast) {
        books.client.ballots.retain(|b| b.request_id != request_id);
        out.push(Route::Host(Control::SeatVote {
            request_id,
            voter: me,
            approve,
        }));
    }

    // Host: votes past their deadline are denied.
    if is_host {
        for outcome in books.book.tick(now) {
            resolve(&mut traffic.pending, &books.view.seats.0, outcome, &mut out);
        }
    }

    // Process received messages plus our own routed ones (host loopback),
    // until nothing new is produced.
    loop {
        for route in out.drain(..) {
            match route {
                Route::Host(control) if is_host => queue.push_back((control, my_id)),
                Route::Host(control) => match traffic.net.host_id() {
                    Some(host) => traffic
                        .pending
                        .outgoing_targeted
                        .push((NetMsg::Control(control), host)),
                    None => warn!("no host to send a seat request to; dropped"),
                },
                Route::All(control) => {
                    traffic
                        .pending
                        .outgoing_broadcast
                        .push(NetMsg::Control(control.clone()));
                    queue.push_back((control, my_id));
                }
            }
        }
        let Some((control, from)) = queue.pop_front() else {
            break;
        };
        match control {
            Control::SeatRequest { .. } | Control::SeatVote { .. } if !is_host => {
                // Election disagreement: pass it on to whoever we think hosts.
                if from != my_id
                    && let Some(host) = traffic.net.host_id()
                    && host != from
                {
                    traffic
                        .pending
                        .outgoing_targeted
                        .push((NetMsg::Control(control), host));
                }
            }
            Control::SeatRequest {
                request_id,
                requester,
                kind,
            } => {
                let projected = seats::projected_seats(
                    &books.view.seats.0,
                    traffic.pending.unconfirmed.iter().map(|(_, e)| e),
                );
                let presence = &books.view.presence;
                match seats::decide_request(&projected, requester, &kind, |k| presence.abandoned(k))
                {
                    SeatDecision::Submit(event) => {
                        info!(request_id, ?event, "host: seat request granted");
                        traffic.pending.submit_game(event);
                        out.push(Route::All(closed(request_id, true, "Granted.")));
                    }
                    SeatDecision::Deny(reason) => {
                        info!(request_id, %reason, "host: seat request refused");
                        out.push(Route::All(closed(request_id, false, &reason)));
                    }
                    SeatDecision::Vote { event, summary } => {
                        let voters =
                            seats::vote_voters(&projected, requester, |k| presence.is_connected(k));
                        info!(request_id, ?voters, %summary, "host: seat vote opened");
                        match books.book.open(request_id, event, voters.clone(), now) {
                            Some(outcome) => resolve(
                                &mut traffic.pending,
                                &books.view.seats.0,
                                outcome,
                                &mut out,
                            ),
                            None if books.book.is_open(request_id) => {
                                out.push(Route::All(Control::SeatVoteOpen {
                                    request_id,
                                    requester,
                                    summary,
                                    voters,
                                    secs_left: SEAT_VOTE_SECS as f32,
                                }));
                            }
                            None => {}
                        }
                    }
                }
            }
            Control::SeatVote {
                request_id,
                voter,
                approve,
            } => {
                if let Some(outcome) = books.book.vote(request_id, voter, approve) {
                    resolve(&mut traffic.pending, &books.view.seats.0, outcome, &mut out);
                }
            }
            Control::SeatVoteOpen {
                request_id,
                requester,
                summary,
                voters,
                secs_left,
            } => {
                let deadline = now + f64::from(secs_left);
                if voters.contains(&me)
                    && !books
                        .client
                        .ballots
                        .iter()
                        .any(|b| b.request_id == request_id)
                {
                    books.client.ballots.push(Ballot {
                        request_id,
                        requester,
                        summary,
                        deadline,
                    });
                }
                if let Some(mine) = books
                    .client
                    .requests
                    .iter_mut()
                    .find(|r| r.request_id == request_id)
                {
                    mine.status = RequestStatus::Voting { deadline };
                }
            }
            Control::SeatVoteClosed {
                request_id,
                approved,
                reason,
            } => {
                books.client.ballots.retain(|b| b.request_id != request_id);
                if let Some(mine) = books
                    .client
                    .requests
                    .iter_mut()
                    .find(|r| r.request_id == request_id)
                    && matches!(
                        mine.status,
                        RequestStatus::Pending | RequestStatus::Voting { .. }
                    )
                {
                    mine.status = if approved {
                        RequestStatus::Approved
                    } else {
                        RequestStatus::Denied(reason.clone())
                    };
                    if let Some(d) = books.dispatches.as_deref_mut() {
                        d.push(
                            SEAT_HEADER,
                            if approved {
                                "Your request was approved.".to_string()
                            } else {
                                format!("Your request was refused: {reason}")
                            },
                        );
                    }
                }
            }
            // Only seat controls are buffered for this system.
            Control::RequestSnapshot | Control::SnapshotReceived | Control::GameHistory(_) => {}
        }
    }

    // -- expiry of ballots and unanswered requests --
    books.client.ballots.retain(|b| b.deadline > now);
    for request in &mut books.client.requests {
        let expired = match request.status {
            RequestStatus::Voting { deadline } => now > deadline + 5.0,
            RequestStatus::Pending => now - request.sent_at > NO_ANSWER_SECS,
            _ => false,
        };
        if expired {
            request.status = RequestStatus::Denied("No answer from the host.".into());
        }
    }
}

fn closed(request_id: u64, approved: bool, reason: &str) -> Control {
    Control::SeatVoteClosed {
        request_id,
        approved,
        reason: reason.to_string(),
    }
}

/// Host: act on a decided vote. An approved change is re-checked against the
/// seat table as it is *now* (plus our unconfirmed seat events) -- the seats
/// may have moved during the vote -- and submitted if it still applies.
fn resolve(
    pending: &mut PendingEdits,
    seats: &[omdurman_net::Seat],
    outcome: VoteOutcome,
    out: &mut Vec<Route>,
) {
    match outcome {
        VoteOutcome::Approved { request_id, event } => {
            let projected =
                seats::projected_seats(seats, pending.unconfirmed.iter().map(|(_, e)| e));
            if seats::apply_seat_event(&mut projected.clone(), &event) {
                info!(request_id, ?event, "host: seat vote passed");
                pending.submit_game(event);
                out.push(Route::All(closed(request_id, true, "Approved.")));
            } else {
                out.push(Route::All(closed(
                    request_id,
                    false,
                    "The seats changed during the vote.",
                )));
            }
        }
        VoteOutcome::Denied { request_id, reason } => {
            info!(request_id, %reason, "host: seat vote failed");
            out.push(Route::All(closed(request_id, false, &reason)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_net::{GameEvent, Seat, SeatHolder};
    use omdurman_types::{CommandScope, DervishTribe, Player};
    use std::collections::{BTreeSet, HashSet};

    const HOST: PlayerKey = PlayerKey(1);
    const AWAY: PlayerKey = PlayerKey(2);
    const NEWCOMER: PlayerKey = PlayerKey(3);

    fn table() -> Vec<Seat> {
        vec![
            Seat {
                faction: Player::Dervish,
                scope: Some(CommandScope::Tribes(BTreeSet::from([
                    DervishTribe::Baggara,
                    DervishTribe::Jaalin,
                ]))),
                holder: SeatHolder::Human(HOST),
            },
            Seat {
                faction: Player::AngloEgyptian,
                scope: None,
                holder: SeatHolder::Human(AWAY),
            },
        ]
    }

    /// A host app running only `seat_control`, with `AWAY` gone long enough
    /// to have abandoned their seat.
    fn host_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut net = NetState::default();
        net.is_host = true;
        net.set_my_id(Some(PeerId(uuid::Uuid::from_u128(1))));
        let mut presence = seats::SeatPresence::default();
        let connected: HashSet<PlayerKey> = [HOST, NEWCOMER].into_iter().collect();
        presence.update(&table(), &connected, 0.0, false);
        presence.update(&table(), &connected, seats::SEAT_ABANDON_SECS + 1.0, false);
        app.insert_resource(net)
            .insert_resource(PendingEdits::with_uid_base(100))
            .insert_resource(PendingIncoming::default())
            .insert_resource(seats::Seats(table()))
            .insert_resource(presence)
            .insert_resource(seats::LocalPlayerKey(HOST))
            .insert_resource(VoteBook::default())
            .insert_resource(SeatClient::default())
            .add_systems(Update, seat_control);
        app
    }

    fn guest_request(app: &mut App, request_id: u64, kind: SeatRequestKind) {
        app.world_mut()
            .resource_mut::<PendingIncoming>()
            .seat_control
            .push((
                Control::SeatRequest {
                    request_id,
                    requester: NEWCOMER,
                    kind,
                },
                PeerId(uuid::Uuid::from_u128(3)),
            ));
    }

    fn submitted(app: &App) -> Vec<GameEvent> {
        app.world()
            .resource::<PendingEdits>()
            .unconfirmed
            .iter()
            .map(|(_, e)| e.clone())
            .collect()
    }

    #[test]
    fn host_grants_an_abandoned_seat_without_a_vote() {
        let mut app = host_app();
        guest_request(&mut app, 9, SeatRequestKind::ClaimAbandoned { seat: 1 });
        app.update();
        assert_eq!(
            submitted(&app),
            vec![GameEvent::SeatAssigned {
                seat: 1,
                previous: SeatHolder::Human(AWAY),
                holder: SeatHolder::Human(NEWCOMER),
            }]
        );
        // A second claim racing the unconfirmed first is refused.
        guest_request(&mut app, 10, SeatRequestKind::ClaimAbandoned { seat: 1 });
        app.update();
        assert_eq!(submitted(&app).len(), 1);
    }

    #[test]
    fn a_takeover_waits_for_the_seated_host_to_approve() {
        let mut app = host_app();
        let scope = CommandScope::Tribes(BTreeSet::from([DervishTribe::Jaalin]));
        guest_request(
            &mut app,
            9,
            SeatRequestKind::TakeOver {
                faction: Player::Dervish,
                scope: scope.clone(),
            },
        );
        app.update();
        assert!(submitted(&app).is_empty(), "nothing before the vote");
        let ballots = app.world().resource::<SeatClient>().ballots.clone();
        assert_eq!(ballots.len(), 1, "the seated host is asked");
        assert_eq!(ballots[0].requester, NEWCOMER);
        app.world_mut()
            .resource_mut::<SeatClient>()
            .cast
            .push((9, true));
        app.update();
        assert_eq!(
            submitted(&app),
            vec![GameEvent::SeatCarved {
                faction: Player::Dervish,
                scope,
                holder: NEWCOMER,
            }]
        );
    }

    #[test]
    fn a_denied_takeover_submits_nothing_and_tells_the_requester() {
        let mut app = host_app();
        guest_request(
            &mut app,
            9,
            SeatRequestKind::TakeOver {
                faction: Player::Dervish,
                scope: CommandScope::Tribes(BTreeSet::from([DervishTribe::Jaalin])),
            },
        );
        app.update();
        app.world_mut()
            .resource_mut::<SeatClient>()
            .cast
            .push((9, false));
        app.update();
        assert!(submitted(&app).is_empty());
        let closed = app
            .world()
            .resource::<PendingEdits>()
            .outgoing_broadcast
            .iter()
            .any(|m| {
                matches!(
                    m,
                    NetMsg::Control(Control::SeatVoteClosed {
                        request_id: 9,
                        approved: false,
                        ..
                    })
                )
            });
        assert!(closed, "the denial is broadcast");
    }

    #[test]
    fn a_guest_sends_its_request_to_the_host() {
        let mut app = host_app();
        {
            let mut net = app.world_mut().resource_mut::<NetState>();
            net.is_host = false;
            net.add_peer(PeerId(uuid::Uuid::from_u128(0)));
        }
        app.world_mut()
            .resource_mut::<SeatClient>()
            .outbox
            .push(SeatRequestKind::ClaimFromAi { seat: 0 });
        app.update();
        let pending = app.world().resource::<PendingEdits>();
        assert!(matches!(
            pending.outgoing_targeted.as_slice(),
            [(NetMsg::Control(Control::SeatRequest { requester: HOST, .. }), to)]
                if *to == PeerId(uuid::Uuid::from_u128(0))
        ));
        assert!(
            pending.unconfirmed.is_empty(),
            "guests never submit seat events"
        );
        let client = app.world().resource::<SeatClient>();
        assert_eq!(client.requests[0].status, RequestStatus::Pending);
    }
}
