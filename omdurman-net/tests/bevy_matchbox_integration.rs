//! Integration test: two `MatchboxSocket` Bevy resources discover each other
//! through a local signalling server and reach `PeerState::Connected`.
//!
//! This verifies the `bevy_matchbox` → `matchbox_socket` → `webrtc-rs` stack
//! works correctly when driven by Bevy's `Update` loop (the same path the app
//! uses). On native this requires a real tokio runtime for webrtc-rs's DTLS
//! handshake; the fix lives in `bevy_matchbox::socket::spawn_message_loop`.

#![cfg(not(target_arch = "wasm32"))]

use std::net::SocketAddr;
use std::time::Duration;

use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use matchbox_signaling::SignalingServer;

/// Timeout for peers to discover each other.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Resource, Default)]
struct PeerSeen {
    connected: bool,
}

fn open_socket(room_url: String) -> impl FnMut(Commands) {
    move |mut commands: Commands| {
        let builder = WebRtcSocketBuilder::new(&room_url)
            .add_reliable_channel()
            .add_unreliable_channel();
        commands.insert_resource(MatchboxSocket::from(builder));
    }
}

fn check_for_peers(socket: Option<ResMut<MatchboxSocket>>, mut seen: ResMut<PeerSeen>) {
    let Some(mut socket) = socket else {
        return;
    };
    for (_peer, state) in socket.try_update_peers().unwrap() {
        if matches!(state, PeerState::Connected) {
            seen.connected = true;
        }
    }
}

/// Run a minimal Bevy app that opens a matchbox socket and polls it until a
/// peer connects (or the timeout expires). Returns `true` if a peer connected.
fn run_bevy_peer(room_url: &str) -> bool {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(PeerSeen::default())
        .add_systems(Startup, open_socket(room_url.to_string()))
        .add_systems(Update, check_for_peers);

    let start = std::time::Instant::now();
    while start.elapsed() < CONNECT_TIMEOUT {
        app.update();
        if app.world().resource::<PeerSeen>().connected {
            return true;
        }
        std::thread::sleep(Duration::from_millis(16));
    }
    false
}

#[test]
fn two_bevy_peers_connect_via_local_signalling() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let mut server = SignalingServer::full_mesh_builder(addr).build();
    server.bind().expect("bind signalling server");
    let bound = server.local_addr().expect("local_addr");
    let url = format!("ws://{bound}/bevy-test-room?next=2");

    let server_handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            server.serve().await.expect("signalling server serve");
        });
    });

    let url_a = url.clone();
    let url_b = url;

    let app_a = std::thread::spawn(move || run_bevy_peer(&url_a));

    // Give app A a head start so it connects to the signalling server first.
    std::thread::sleep(Duration::from_millis(500));

    let app_b = std::thread::spawn(move || run_bevy_peer(&url_b));

    let a_connected = app_a.join().expect("app A thread panicked");
    let b_connected = app_b.join().expect("app B thread panicked");

    assert!(a_connected, "peer A never saw a connection");
    assert!(b_connected, "peer B never saw a connection");

    drop(server_handle);
}
