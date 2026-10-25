//! Integration test: two `WebRtcSocket` peers discover each other through a
//! local matchbox signalling server and reach `PeerState::Connected`.
//!
//! This isolates the matchbox + webrtc-rs stack from the app's bevy systems.
//! If this test fails, the lobby cannot work regardless of app-level code.

#![cfg(not(target_arch = "wasm32"))]

use std::net::SocketAddr;
use std::time::Duration;

use futures::FutureExt;
use matchbox_signaling::SignalingServer;
use matchbox_socket::{PeerState, WebRtcSocketBuilder};

/// How long to poll for both peers to see each other before giving up.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Bring up a local full-mesh signalling server on an ephemeral port, then
/// open two WebRTC sockets against it and poll until both report the other as
/// `PeerState::Connected`.
///
/// B joins the room only once the server has added A to it: the full-mesh
/// server tells a room's peers of a newcomer from a copy of the room taken
/// before the newcomer is added, so two peers joining at the same moment can
/// each miss the other and never connect (a race in `FullMeshState::add_peer`
/// of matchbox_signaling, not what this test is about).
#[tokio::test(flavor = "multi_thread")]
async fn two_peers_connect_via_local_signalling() {
    // RUST_LOG=matchbox_socket=debug,matchbox_signaling=debug shows a failure's handshake.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_test_writer()
        .try_init();
    let deadline = tokio::time::Instant::now() + CONNECT_TIMEOUT;

    // -- local signalling server; says when it has added a peer to its room --
    let (joined, mut joins) = futures::channel::mpsc::unbounded();
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let mut server = SignalingServer::full_mesh_builder(addr)
        .on_peer_connected(move |peer| {
            let _ = joined.unbounded_send(peer);
        })
        .build();
    server.bind().expect("bind signalling server");
    let bound = server.local_addr().expect("local_addr");
    let url = format!("ws://{bound}/test-room?next=2");

    let server_task = tokio::spawn(async move {
        server.serve().await.expect("signalling server serve");
    });

    // -- two sockets in the same room (single reliable channel) --
    let (mut sock_a, loop_a) = WebRtcSocketBuilder::new(&url)
        .add_reliable_channel()
        .build();
    let (mut sock_b, loop_b) = WebRtcSocketBuilder::new(&url)
        .add_reliable_channel()
        .build();

    let loop_a = loop_a.fuse();
    let loop_b = loop_b.fuse();
    futures::pin_mut!(loop_a, loop_b);

    // A's loop alone first (B's does nothing until polled), until A is in the room.
    futures::select! {
        _ = futures::FutureExt::fuse(&mut loop_a) => panic!("socket A message loop ended"),
        _ = futures::StreamExt::next(&mut joins).fuse() => {}
        _ = tokio::time::sleep_until(deadline).fuse() => {
            panic!("A did not join the room within {CONNECT_TIMEOUT:?}")
        }
    }

    let mut a_connected = false;
    let mut b_connected = false;

    loop {
        // Drain peer updates from both sockets.
        for (_peer, state) in sock_a.update_peers() {
            match state {
                PeerState::Connected => {
                    a_connected = true;
                    eprintln!("[A] peer connected");
                }
                PeerState::Disconnected => {
                    panic!("[A] peer disconnected before both connected");
                }
            }
        }
        for (_peer, state) in sock_b.update_peers() {
            match state {
                PeerState::Connected => {
                    b_connected = true;
                    eprintln!("[B] peer connected");
                }
                PeerState::Disconnected => {
                    panic!("[B] peer disconnected before both connected");
                }
            }
        }

        if a_connected && b_connected {
            eprintln!("both peers connected!");
            break;
        }

        if tokio::time::Instant::now() >= deadline {
            panic!(
                "peers did not connect within {CONNECT_TIMEOUT:?} \
                 (a_connected={a_connected}, b_connected={b_connected})"
            );
        }

        // Poll the message loops. A short timer yields back so we can call
        // `update_peers()` again — the socket does not push notifications.
        futures::select! {
            _ = futures::FutureExt::fuse(&mut loop_a) => panic!("socket A message loop ended"),
            _ = futures::FutureExt::fuse(&mut loop_b) => panic!("socket B message loop ended"),
            _ = tokio::time::sleep(Duration::from_millis(50)).fuse() => {}
        }
    }

    server_task.abort();
}
