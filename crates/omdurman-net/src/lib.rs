//! Internet multiplayer — dumb pipe, no game logic.
//!
//! Syncs [`GameEvent`] streams between peers via matchbox (WebRTC signaling)
//! and lightyear (rollback netcode). Peer-to-peer, no dedicated server.
