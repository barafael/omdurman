# Architecture decisions

Every non-obvious choice in this scaffold, with sources.

---

## 1. P2P (no dedicated server)

**Decision:** Use WebRTC peer-to-peer via `bevy_matchbox`. No game server.

**Why:** You have two players who trust each other, no hidden information, and no anti-cheat requirement. A dedicated server would add cost and operational complexity for zero benefit. P2P also means zero ongoing hosting cost — the signaling server only handles the WebRTC handshake (a few hundred bytes), then drops out.

**When this would break:** If you later add hidden information (e.g. a hand of cards each player can't see), you'd need a server to be the authority. The easiest migration path is to add `bevy_replicon` + a `renet2` backend on top of this same RNG/turn-state model.

**Sources:**
- [bevy_matchbox on GitHub](https://github.com/johanhelsing/matchbox) — the only mature, production-tested WebRTC P2P library for Rust/WASM
- [Johan Helsing — Introducing Matchbox](https://johanhelsing.studio/posts/introducing-matchbox) — design rationale for the library
- Bevy networking discussion [#8675](https://github.com/bevyengine/bevy/discussions/8675) — "Players trust each other → P2P rollback or P2P message-passing is fine; no anti-cheat server needed"

---

## 2. No GGRS / no rollback netcode

**Decision:** Use simple reliable message-passing, not GGRS rollback.

**Why:** GGRS (and rollback netcode in general) exists to hide latency in *fast-paced* games where you cannot afford to wait for the remote player's input before advancing the simulation. In a turn-based game you *do* wait — the design already handles latency. Adding rollback buys nothing and imposes a heavy constraint: your entire simulation must be perfectly deterministic (no unregistered state, no `EventReader` outside `GgrsSchedule`, no float non-determinism, etc.).

**Sources:**
- [bevy_ggrs pitfalls.md](https://github.com/gschup/bevy_ggrs/blob/main/docs/pitfalls.md) — exhaustive list of determinism requirements
- [bevy_ggrs architecture.md](https://github.com/gschup/bevy_ggrs/blob/main/docs/architecture.md)
- [GGRS on GitHub](https://github.com/gschup/ggrs) — "P2P rollback networking for real-time games"

---

## 3. Reliable channel

**Decision:** `MatchboxSocket::new_reliable(url)` — every message is delivered exactly once, in order.

**Why:** Turn-based games care about correctness, not latency. An action message that arrives out of order or gets dropped would desync the game. Unreliable channels exist for fast-paced games that prefer dropping stale data to blocking on retransmit; that trade-off is wrong here.

**Sources:**
- [matchbox_socket README](https://github.com/johanhelsing/matchbox/tree/main/matchbox_socket) — reliable vs unreliable channel explanation
- [Announcing Matchbox 0.4](https://johanhelsing.studio/posts/matchbox-0-4) — reliable channels added in 0.4

---

## 4. Seeded shared RNG (ChaCha8Rng)

**Decision:** Host generates a `u64` seed, sends it once, both sides init `ChaCha8Rng::seed_from_u64(seed)`. Die rolls are never transmitted — only actions are.

**Why:** If you transmitted die rolls, you'd need to trust that the remote player's rolls are honest (they could re-roll until they get a good result and only then send). With a shared seed, *both players' dice come from the same RNG stream*, in the same order. Cheating a die roll would require knowing the seed in advance and predicting future outputs — impractical for a casual game.

It also simplifies the protocol: the only message you send per turn is the *action* (e.g. which piece moves), not the randomness behind it.

**Why ChaCha8:** Deterministic across platforms and architectures (no IEEE 754 float issues, no OS-level entropy differences), fast, and `rand_chacha` is the standard `rand`-ecosystem choice for reproducible simulations.

**Caveat:** This approach requires that both sides call `rng.next_u32()` in *exactly* the same order. If your game logic is branchy (different code paths call the RNG under different conditions), a bug can silently desync the two clients. Add a checksum or "state hash" message periodically if you want to detect this.

**Sources:**
- [`rand_chacha` docs](https://docs.rs/rand_chacha) — `ChaCha8Rng`, `SeedableRng`
- [`rand` WASM guide](https://docs.rs/rand/latest/rand/#wasm-support) — `getrandom` with `js` feature
- General game networking theory: sending actions rather than state is standard for lockstep/P2P netcode

---

## 5. PeerId comparison for host/guest assignment

**Decision:** `is_host = my_id.0 < peer.0` — whoever has the lexicographically smaller UUID is the host (sends seed, moves first).

**Why:** Both players already know both PeerIds from the signaling exchange. Comparing them is instantaneous and requires no additional message. The alternative — one player explicitly declaring themselves host — would need a round-trip and a tiebreaker for simultaneous declarations anyway.

`PeerId` wraps a `uuid::Uuid`; `Uuid` implements `PartialOrd`, so the comparison is stable.

**Sources:**
- [matchbox_socket `PeerId`](https://docs.rs/matchbox_socket/latest/matchbox_socket/struct.PeerId.html)
- Standard technique in P2P systems: use a pre-existing unique identifier rather than a coin-flip message

---

## 6. `update_peers()` called every frame

**Decision:** `handle_socket` runs unconditionally (not gated on `AppState`).

**Why:** Matchbox's `update_peers()` drives the internal WebRTC event loop. If you stop calling it (e.g. because you gated the system on a state that isn't active), the underlying data channel can stall. Keeping it unconditional is the safe default. The system does nothing expensive when idle.

**Sources:**
- [bevy_matchbox source](https://github.com/johanhelsing/matchbox/tree/main/bevy_matchbox) — `MatchboxSocket::update_peers` must be called to process queued events
- Common mistake reported in matchbox issues: socket appears to freeze after state transitions

---

## 7. Room code from URL hash

**Decision:** Room ID lives in `window.location.hash` (`#abc123`). If absent, generate one and write it back.

**Why the hash:** Fragment identifiers (`#...`) are never sent to the server — they exist only in the browser. Using a hash means the signaling server never sees the room code at all (it's part of the WebSocket *path*, which the server does see, but it's not in any HTTP log or referrer header from a link-click). It's also the simplest way to encode state in a shareable URL without a redirect.

**Why generate on first load:** Player 1 opens the bare URL, gets a code, copies it. Player 2 opens the URL with the hash. No separate "create room" UI needed.

**Sources:**
- [MDN: Location.hash](https://developer.mozilla.org/en-US/docs/Web/API/Location/hash)
- [Johan Helsing's Extreme Bevy tutorial](https://johanhelsing.studio/posts/extreme-bevy) — uses matchbox room codes in the same way

---

## 8. No `bevy_replicon` or `lightyear`

**Decision:** Raw matchbox messages, no high-level replication layer.

**Why:** `bevy_replicon` and `lightyear` solve ECS world synchronisation (automatically replicating components from server to clients). For a turn-based board game, the state you synchronise per turn is tiny — one action enum value. A full replication layer would add significant setup complexity (backends, plugins, channel configuration, `Replicated` markers) for no benefit. If the game grows to a point where you're syncing many entities every frame, reconsider.

**Sources:**
- [bevy_replicon on GitHub](https://github.com/simgine/bevy_replicon) — "server-authoritative replication"; overkill for P2P turn-based
- [lightyear on GitHub](https://github.com/cBournhonesque/lightyear) — targets fast-paced action games with prediction/lag-compensation

---

## 9. Version choices

| Crate | Version used | Notes |
|---|---|---|
| `bevy` | 0.15 | Stable at time of writing; `Text` API matches scaffold |
| `bevy_matchbox` | 0.11 | Targets Bevy 0.15; check [crates.io](https://crates.io/crates/bevy_matchbox) when upgrading Bevy |
| `rand` | 0.8 | Stable; 0.9 is in preview and changes some APIs |
| `rand_chacha` | 0.3 | Tracks rand 0.8 |
| `getrandom` | 0.2 | Must match what `rand` 0.8 uses internally |

`bevy_matchbox` follows Bevy's minor versions closely. After a Bevy upgrade, check the matchbox releases page for the corresponding version before `cargo update`.

**Sources:**
- [bevy_matchbox releases](https://github.com/johanhelsing/matchbox/releases)
- Bevy compatibility tables convention: major networking crates publish a release within 1–2 weeks of each Bevy release

---

## 10. Signaling server

**Decision:** Use `wss://match.helsing.studio` for development; self-host `matchbox_server` for production.

**Why:** `match.helsing.studio` is Johan Helsing's public demo server. It works fine for development and jam submissions but carries no uptime guarantee. The `matchbox_server` binary is a ~3 MB single-file WebSocket server; running it on any VPS (or a free-tier Fly.io instance) gives you a stable production endpoint.

**Self-hosting:**
```sh
cargo install matchbox_server
matchbox_server  # listens on 0.0.0.0:3536 by default
```
Then change the URL in `open_socket` to `wss://your-domain.com/`.

**Sources:**
- [matchbox_server README](https://github.com/johanhelsing/matchbox/tree/main/matchbox_server)
- [Announcing Matchbox 0.4](https://johanhelsing.studio/posts/matchbox-0-4) — signaling server design

---

## Building for the browser

Install [trunk](https://trunkrs.dev/):
```sh
cargo install trunk
rustup target add wasm32-unknown-unknown
```

Serve locally (auto-reloads on save):
```sh
trunk serve
```

Production build (outputs to `dist/`):
```sh
trunk build --release
```

For native testing (two terminals, same room name):
```sh
cargo run -- my-test-room   # terminal 1
cargo run -- my-test-room   # terminal 2
```
