//! Splitting large wire messages into data-channel-sized chunks.
//!
//! A WebRTC data channel does not carry arbitrarily large messages: on
//! native, webrtc-rs refuses any message over its SCTP association's maximum
//! message size -- 64 KiB under matchbox's default settings
//! (`ErrOutboundPacketTooLarge`) -- and matchbox only logs that error, after
//! the app's `try_send` already returned `Ok`. Browsers interoperate safely
//! at 16 KiB. A game record outgrows 64 KiB after roughly a thousand events,
//! so every message whose encoding exceeds [`CHUNK_BYTES`] travels as a run
//! of [`Control::Chunk`] packets ([`enc_packets`]) and is reassembled on the
//! receiver ([`ChunkAssembler`]) before it enters the ordinary receive path.

use crate::{Control, NetMsg, decode, enc_msg};
use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;

/// Largest payload one [`Control::Chunk`] carries (the packet adds a few
/// bytes of framing). Well under webrtc-rs's 64 KiB limit and the 16 KiB
/// that browsers are known to interoperate at.
pub const CHUNK_BYTES: usize = 16 * 1024;

/// Upper bound on a reassembled message: a chunk claiming a larger total is
/// dropped instead of reserving room for it.
pub const MAX_CHUNKED_BYTES: usize = 64 * 1024 * 1024;

/// The most chunks one transfer may have.
const MAX_CHUNKS: u32 = (MAX_CHUNKED_BYTES / CHUNK_BYTES) as u32;

/// Incomplete transfers kept per sender; a further one evicts the oldest.
const MAX_PARTIAL_PER_PEER: usize = 4;

/// Encode `msg` into wire packets: a single packet when the encoding fits in
/// [`CHUNK_BYTES`], otherwise one [`Control::Chunk`] packet per slice of it,
/// all under one transfer id drawn from `next_transfer_id` (a per-sender
/// counter: ids must increase, see [`ChunkAssembler`]). `None` if encoding
/// fails.
pub fn enc_packets(msg: &NetMsg, next_transfer_id: &mut u64) -> Option<Vec<Box<[u8]>>> {
    let whole = enc_msg(msg)?;
    if whole.len() <= CHUNK_BYTES {
        return Some(vec![whole]);
    }
    let transfer_id = *next_transfer_id;
    *next_transfer_id = next_transfer_id.wrapping_add(1);
    let total = whole.len().div_ceil(CHUNK_BYTES) as u32;
    whole
        .chunks(CHUNK_BYTES)
        .enumerate()
        .map(|(index, bytes)| {
            enc_msg(&NetMsg::Control(Control::Chunk {
                transfer_id,
                index: index as u32,
                total,
                bytes: bytes.to_vec(),
            }))
        })
        .collect()
}

/// One incomplete transfer.
#[derive(Debug)]
struct Partial {
    from: PeerId,
    transfer_id: u64,
    parts: Vec<Option<Vec<u8>>>,
    received: u32,
}

/// Receive-side reassembly of [`Control::Chunk`] runs, per sender.
///
/// Tolerates chunks arriving out of order, duplicated, and interleaved with
/// other transfers. A sender's transfer ids increase, so once a transfer
/// completes, every *older* transfer of the same sender still incomplete
/// can never complete -- its missing chunks were lost with a failed send,
/// which the sender retries under a fresh id -- and is discarded
/// (superseded); late chunks of it are ignored.
#[derive(Debug, Default)]
pub struct ChunkAssembler {
    partial: Vec<Partial>,
    /// Per sender, the highest transfer id completed: chunks at or below it
    /// are late duplicates or superseded.
    completed: Vec<(PeerId, u64)>,
}

impl ChunkAssembler {
    /// Route one decoded message from `from`: anything but a chunk passes
    /// through; a chunk is stored, and the message it completes (decoded) is
    /// returned. A malformed chunk -- or a reassembled message that does not
    /// decode, or is itself a chunk -- yields nothing.
    pub fn receive(&mut self, from: PeerId, msg: NetMsg) -> Option<NetMsg> {
        let NetMsg::Control(Control::Chunk {
            transfer_id,
            index,
            total,
            bytes,
        }) = msg
        else {
            return Some(msg);
        };
        let whole = self.accept(from, transfer_id, index, total, bytes)?;
        match decode(&whole) {
            Some(NetMsg::Control(Control::Chunk { .. })) => {
                warn!(%from, transfer_id, "reassembled message is itself a chunk; dropping");
                None
            }
            Some(msg) => Some(msg),
            None => {
                warn!(%from, transfer_id, bytes = whole.len(), "reassembled message does not decode; dropping");
                None
            }
        }
    }

    /// Store one chunk; returns the transfer's bytes once it is complete.
    pub fn accept(
        &mut self,
        from: PeerId,
        transfer_id: u64,
        index: u32,
        total: u32,
        bytes: Vec<u8>,
    ) -> Option<Vec<u8>> {
        if total == 0 || total > MAX_CHUNKS || index >= total || bytes.len() > CHUNK_BYTES {
            warn!(%from, transfer_id, index, total, len = bytes.len(), "malformed chunk; dropping");
            return None;
        }
        if self
            .completed
            .iter()
            .any(|&(peer, done)| peer == from && transfer_id <= done)
        {
            debug!(%from, transfer_id, index, "chunk of a completed or superseded transfer; ignoring");
            return None;
        }
        let pos = match self
            .partial
            .iter()
            .position(|p| p.from == from && p.transfer_id == transfer_id)
        {
            Some(pos) => pos,
            None => {
                let of_peer: Vec<usize> = (0..self.partial.len())
                    .filter(|&i| self.partial[i].from == from)
                    .collect();
                if of_peer.len() >= MAX_PARTIAL_PER_PEER {
                    let oldest = of_peer
                        .into_iter()
                        .min_by_key(|&i| self.partial[i].transfer_id)
                        .expect("at least one partial transfer of this peer");
                    let evicted = self.partial.remove(oldest);
                    warn!(%from, transfer_id = evicted.transfer_id, "too many incomplete transfers; evicting the oldest");
                }
                self.partial.push(Partial {
                    from,
                    transfer_id,
                    parts: vec![None; total as usize],
                    received: 0,
                });
                self.partial.len() - 1
            }
        };
        let partial = &mut self.partial[pos];
        if partial.parts.len() != total as usize {
            warn!(%from, transfer_id, total, "chunk disagrees with its transfer's total; dropping");
            return None;
        }
        let slot = &mut partial.parts[index as usize];
        if slot.is_some() {
            return None; // duplicate
        }
        *slot = Some(bytes);
        partial.received += 1;
        if partial.received < total {
            return None;
        }
        let done = self.partial.remove(pos);
        // Older incomplete transfers of this sender are superseded.
        self.partial
            .retain(|p| p.from != from || p.transfer_id > transfer_id);
        match self.completed.iter_mut().find(|(peer, _)| *peer == from) {
            Some((_, highest)) => *highest = (*highest).max(transfer_id),
            None => self.completed.push((from, transfer_id)),
        }
        Some(done.parts.into_iter().flatten().flatten().collect())
    }

    /// Forget everything about `peer` (it disconnected): its transfers can
    /// no longer complete.
    pub fn forget(&mut self, peer: PeerId) {
        self.partial.retain(|p| p.from != peer);
        self.completed.retain(|(p, _)| *p != peer);
    }

    /// Incomplete transfers held.
    pub fn pending_transfers(&self) -> usize {
        self.partial.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameRecord, InitialGameState, RecordedEvent};
    use omdurman_rules::effects::GameEffect;

    fn peer(n: u128) -> PeerId {
        PeerId(uuid::Uuid::from_u128(n))
    }

    /// A history large enough to need several chunks.
    fn big_history(events: u32) -> NetMsg {
        NetMsg::Control(Control::GameHistory(GameRecord {
            initial_state: InitialGameState { seed: 3 },
            events: (0..events)
                .map(|seq| RecordedEvent {
                    utc: chrono::Utc::now(),
                    sender_idx: Some(1),
                    seq,
                    uid: Some(u64::from(seq) * 7919),
                    payload: crate::GameEvent::Effect(GameEffect::AdvancePhase),
                })
                .collect(),
        }))
    }

    fn history_len(msg: &NetMsg) -> usize {
        match msg {
            NetMsg::Control(Control::GameHistory(record)) => record.events.len(),
            other => panic!("expected a history, got {other:?}"),
        }
    }

    fn decoded(packets: &[Box<[u8]>]) -> Vec<NetMsg> {
        packets
            .iter()
            .map(|p| decode(p).expect("packet decodes"))
            .collect()
    }

    #[test]
    fn small_messages_travel_whole() {
        let mut id = 5;
        let msg = NetMsg::Control(Control::RequestSnapshot);
        let packets = enc_packets(&msg, &mut id).unwrap();
        assert_eq!(packets.len(), 1);
        assert_eq!(id, 5, "no transfer id spent");
        let mut asm = ChunkAssembler::default();
        assert!(matches!(
            asm.receive(peer(1), decode(&packets[0]).unwrap()),
            Some(NetMsg::Control(Control::RequestSnapshot))
        ));
    }

    /// Every packet of a large message stays far below the native 64 KiB
    /// limit, and the run reassembles to the original message.
    #[test]
    fn a_large_history_is_chunked_and_reassembled() {
        let msg = big_history(5000);
        assert!(
            enc_msg(&msg).unwrap().len() > 64 * 1024,
            "the test needs a big record"
        );
        let mut id = 100;
        let packets = enc_packets(&msg, &mut id).unwrap();
        assert!(packets.len() > 4);
        assert_eq!(id, 101);
        assert!(packets.iter().all(|p| p.len() <= CHUNK_BYTES + 64));
        let mut asm = ChunkAssembler::default();
        let mut out = Vec::new();
        for m in decoded(&packets) {
            out.extend(asm.receive(peer(1), m));
        }
        assert_eq!(out.len(), 1);
        assert_eq!(history_len(&out[0]), 5000);
        assert_eq!(asm.pending_transfers(), 0);
    }

    #[test]
    fn out_of_order_and_duplicate_chunks_reassemble_once() {
        let msg = big_history(4000);
        let mut id = 0;
        let mut chunks = decoded(&enc_packets(&msg, &mut id).unwrap());
        chunks.reverse();
        let dup = chunks[1].clone();
        chunks.insert(2, dup);
        let mut asm = ChunkAssembler::default();
        let mut out = Vec::new();
        for m in chunks.iter().cloned() {
            out.extend(asm.receive(peer(1), m));
        }
        assert_eq!(out.len(), 1, "reassembled exactly once");
        assert_eq!(history_len(&out[0]), 4000);
        // A late duplicate of the completed transfer is ignored.
        assert!(asm.receive(peer(1), chunks[0].clone()).is_none());
        assert_eq!(asm.pending_transfers(), 0);
    }

    /// Two senders' transfers -- and two transfers of one sender --
    /// interleaved chunk by chunk all complete.
    #[test]
    fn interleaved_transfers_complete() {
        let mut id_a = 10;
        let mut id_b = 10; // the same id from another sender is distinct
        let a1 = decoded(&enc_packets(&big_history(3000), &mut id_a).unwrap());
        let a2 = decoded(&enc_packets(&big_history(3500), &mut id_a).unwrap());
        let b = decoded(&enc_packets(&big_history(4000), &mut id_b).unwrap());
        let mut asm = ChunkAssembler::default();
        let mut out = Vec::new();
        let longest = a1.len().max(a2.len()).max(b.len());
        for i in 0..longest {
            // a2's chunks run one behind a1's, so a1 completes first.
            if let Some(m) = a1.get(i) {
                out.extend(asm.receive(peer(1), m.clone()).map(|m| (1, m)));
            }
            if i > 0
                && let Some(m) = a2.get(i - 1)
            {
                out.extend(asm.receive(peer(1), m.clone()).map(|m| (1, m)));
            }
            if let Some(m) = b.get(i) {
                out.extend(asm.receive(peer(2), m.clone()).map(|m| (2, m)));
            }
        }
        if let Some(m) = a2.last() {
            out.extend(asm.receive(peer(1), m.clone()).map(|m| (1, m)));
        }
        let mut got: Vec<(u8, usize)> = out.iter().map(|(p, m)| (*p, history_len(m))).collect();
        got.sort();
        assert_eq!(got, vec![(1, 3000), (1, 3500), (2, 4000)]);
        assert_eq!(asm.pending_transfers(), 0);
    }

    /// A newer transfer completing first supersedes an older incomplete one
    /// of the same sender: the old one is dropped, and its late chunks are
    /// ignored rather than starting it afresh.
    #[test]
    fn a_completed_newer_transfer_supersedes_an_older_one() {
        let mut id = 0;
        let old = decoded(&enc_packets(&big_history(3000), &mut id).unwrap());
        let new = decoded(&enc_packets(&big_history(3100), &mut id).unwrap());
        let mut asm = ChunkAssembler::default();
        assert!(asm.receive(peer(1), old[0].clone()).is_none());
        let mut out = Vec::new();
        for m in &new {
            out.extend(asm.receive(peer(1), m.clone()));
        }
        assert_eq!(out.len(), 1);
        assert_eq!(history_len(&out[0]), 3100);
        assert_eq!(asm.pending_transfers(), 0, "the old transfer is gone");
        for m in &old[1..] {
            assert!(asm.receive(peer(1), m.clone()).is_none());
        }
        assert_eq!(asm.pending_transfers(), 0, "late chunks start nothing");
    }

    #[test]
    fn malformed_chunks_and_disconnects_are_bounded() {
        let mut asm = ChunkAssembler::default();
        assert!(
            asm.accept(peer(1), 1, 3, 3, vec![1]).is_none(),
            "index past total"
        );
        assert!(
            asm.accept(peer(1), 1, 0, u32::MAX, vec![1]).is_none(),
            "absurd total"
        );
        assert!(
            asm.accept(peer(1), 1, 0, 2, vec![0; CHUNK_BYTES + 1])
                .is_none(),
            "oversized chunk"
        );
        assert_eq!(asm.pending_transfers(), 0);
        for transfer in 0..10 {
            asm.accept(peer(1), transfer, 0, 2, vec![1]);
        }
        assert_eq!(asm.pending_transfers(), MAX_PARTIAL_PER_PEER);
        asm.forget(peer(1));
        assert_eq!(asm.pending_transfers(), 0);
    }
}
