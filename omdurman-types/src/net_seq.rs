//! Sequencing primitives of the host-relayed event stream (`omdurman-net`):
//! the identity dedup of applied submissions and the guest's reorder buffer.
//! They live here, without Bevy, so the Kani suite can prove them.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// How long a seq gap may persist in the [`ReorderBuffer`] before the guest
/// gives up waiting for the missing deliveries and requests the canonical
/// history instead.
pub const SEQ_GAP_TIMEOUT_SECS: f32 = 1.5;

/// Bounded ring of recently applied submission uids. Large enough to cover
/// every uid that could still be re-delivered (retransmit retries and echoes
/// are re-sent within seconds; stale post-failover streams within the churn
/// window), small enough to stay flat in memory over a long game. Ordered
/// (not hashed): no OS-seeded hasher, so it is deterministic and provable.
#[derive(Default, Debug, Clone)]
pub struct RecentUids {
    set: BTreeSet<u64>,
    order: VecDeque<u64>,
}

impl RecentUids {
    /// Uids remembered before the oldest is forgotten.
    pub const CAP: usize = 4096;

    /// Returns `true` if the uid was newly inserted (first application),
    /// `false` if it was already known (duplicate delivery).
    pub fn insert(&mut self, uid: u64) -> bool {
        if !self.set.insert(uid) {
            return false;
        }
        self.order.push_back(uid);
        while self.order.len() > Self::CAP {
            let evicted = self.order.pop_front().expect("non-empty");
            self.set.remove(&evicted);
        }
        true
    }

    pub fn contains(&self, uid: u64) -> bool {
        self.set.contains(&uid)
    }

    /// Replace the contents with `uids` (in application order, so the most
    /// recent ones survive the cap). Used when a history install replaces
    /// the local record: the dedup set must describe the *installed* line,
    /// not the discarded one.
    pub fn rebuild(&mut self, uids: impl IntoIterator<Item = u64>) {
        *self = Self::default();
        for uid in uids {
            self.insert(uid);
        }
    }
}

/// A delivery that carries its canonical sequence number.
pub trait Sequenced {
    fn seq(&self) -> u32;
}

/// Guest-side reorder buffer for sequenced deliveries that arrive past the
/// next expected seq. Applying such an event immediately would run it
/// against a state missing the events in between; instead it waits here
/// until the gap fills (contiguous runs are then applied in order) or the
/// gap outlives [`SEQ_GAP_TIMEOUT_SECS`], at which point the receive path
/// requests the canonical history (see [`ReorderBuffer::tick`]).
#[derive(Debug, Clone)]
pub struct ReorderBuffer<T> {
    pending: BTreeMap<u32, T>,
    /// Seconds the buffer has been continuously non-empty.
    stalled_secs: f32,
    /// Whether the current stall already triggered a history request.
    reported: bool,
}

impl<T> Default for ReorderBuffer<T> {
    fn default() -> Self {
        Self {
            pending: BTreeMap::new(),
            stalled_secs: 0.0,
            reported: false,
        }
    }
}

impl<T: Sequenced> ReorderBuffer<T> {
    /// Upper bound on buffered deliveries. Past it the history request is
    /// the recovery path anyway, so further deliveries are dropped.
    pub const CAP: usize = 1024;

    /// Buffer `delivery`. Returns `false` if it was not stored (buffer full).
    /// A later delivery at an already-buffered seq replaces the earlier one.
    pub fn insert(&mut self, delivery: T) -> bool {
        let seq = delivery.seq();
        if self.pending.len() >= Self::CAP && !self.pending.contains_key(&seq) {
            return false;
        }
        self.pending.insert(seq, delivery);
        true
    }

    /// Discard every buffered delivery below `expected` (already covered by
    /// an applied event or an installed history) and pop the one at
    /// `expected`, if buffered.
    pub fn pop_next(&mut self, expected: u32) -> Option<T> {
        self.pending = self.pending.split_off(&expected);
        let next = self.pending.remove(&expected);
        if self.pending.is_empty() {
            self.stalled_secs = 0.0;
            self.reported = false;
        }
        next
    }

    /// Advance the stall clock by `dt`. Returns `true` exactly once per
    /// stall, when the buffer has been non-empty for longer than
    /// [`SEQ_GAP_TIMEOUT_SECS`].
    pub fn tick(&mut self, dt: f32) -> bool {
        if self.pending.is_empty() {
            self.stalled_secs = 0.0;
            self.reported = false;
            return false;
        }
        self.stalled_secs += dt;
        if self.stalled_secs > SEQ_GAP_TIMEOUT_SECS && !self.reported {
            self.reported = true;
            return true;
        }
        false
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// The lowest buffered seq, if any.
    pub fn first_seq(&self) -> Option<u32> {
        self.pending.keys().next().copied()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Kani proofs of the two primitives the receive path's exactly-once,
/// in-order delivery rests on (`omdurman-app`'s `receive_sequenced` and
/// `drain_contiguous`).
#[cfg(kani)]
mod verification {
    use super::*;

    /// A delivery reduced to what the buffer looks at, plus a tag telling
    /// two deliveries at the same seq apart.
    #[derive(Clone, Copy, PartialEq, Debug)]
    struct Delivery {
        seq: u32,
        tag: u8,
    }

    impl Sequenced for Delivery {
        fn seq(&self) -> u32 {
            self.seq
        }
    }

    /// Identity dedup is exact: a uid is new exactly once, and known ever
    /// after, whatever the other uids around it.
    #[cfg(feature = "kani-expensive")]
    #[kani::proof]
    #[kani::unwind(6)]
    fn recent_uids_accept_each_uid_exactly_once() {
        let uids: [u64; 4] = kani::any();
        let mut recent = RecentUids::default();
        let mut firsts = 0;
        let target = uids[0];
        for uid in uids {
            let new = recent.insert(uid);
            if uid == target && new {
                firsts += 1;
            }
            assert!(recent.contains(uid));
        }
        assert!(firsts == 1, "a uid was accepted twice, or never");
        core::mem::forget(recent);
    }

    /// Whatever order four deliveries arrive in (any seqs from a window,
    /// repeats allowed), popping from `expected` upward hands out exactly
    /// the buffered contiguous run starting at `expected`, in order, each
    /// seq once -- the last delivery at a seq winning -- and leaves nothing
    /// at or below the last seq popped.
    #[cfg(feature = "kani-expensive")]
    #[kani::proof]
    #[kani::unwind(7)]
    fn reorder_buffer_hands_out_the_contiguous_run_in_order() {
        let expected: u32 = kani::any();
        kani::assume(expected <= 8);
        let arrivals: [Delivery; 4] = [(); 4].map(|_| {
            let seq: u32 = kani::any();
            kani::assume(seq <= 12);
            Delivery {
                seq,
                tag: kani::any(),
            }
        });
        let mut buffer = ReorderBuffer::default();
        for d in arrivals {
            assert!(buffer.insert(d));
        }
        // The latest arrival at `seq`, if any.
        let latest = |seq: u32| arrivals.iter().rev().find(|d| d.seq == seq).copied();
        let mut next = expected;
        for _ in 0..5 {
            match buffer.pop_next(next) {
                Some(d) => {
                    assert!(d.seq == next, "popped out of order");
                    assert!(
                        Some(d) == latest(next),
                        "not the latest delivery at that seq"
                    );
                    next += 1;
                }
                None => {
                    assert!(latest(next).is_none(), "a buffered seq was skipped");
                    break;
                }
            }
        }
        // Everything below the watermark is gone.
        assert!(buffer.first_seq().is_none_or(|first| first > next));
        core::mem::forget(buffer);
    }

    /// The stall clock reports a gap exactly once per stall, and never for
    /// an empty buffer.
    #[kani::proof]
    #[kani::unwind(6)]
    fn reorder_buffer_reports_each_stall_once() {
        let mut buffer: ReorderBuffer<Delivery> = ReorderBuffer::default();
        let dt: f32 = kani::any();
        kani::assume(dt.is_finite() && dt >= 0.0 && dt <= 10.0);
        assert!(!buffer.tick(dt), "an empty buffer reported a gap");
        buffer.insert(Delivery { seq: 3, tag: 0 });
        let mut reports = 0;
        for _ in 0..4 {
            if buffer.tick(dt) {
                reports += 1;
            }
        }
        assert!(reports <= 1, "one stall reported twice");
        // 4 ticks of `dt` past the timeout: exactly one report.
        if dt * 4.0 > SEQ_GAP_TIMEOUT_SECS * 1.01 {
            assert!(reports == 1, "a stall past the timeout went unreported");
        }
        core::mem::forget(buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct Delivery {
        seq: u32,
        tag: u8,
    }

    impl Sequenced for Delivery {
        fn seq(&self) -> u32 {
            self.seq
        }
    }

    /// Exhaustive over every sequence of four uids from five values: a uid
    /// is new exactly once, and known ever after (the Kani proof's claim,
    /// over a bounded domain).
    #[test]
    fn recent_uids_accept_each_uid_exactly_once() {
        for code in 0..5u64.pow(4) {
            let uids: [u64; 4] = std::array::from_fn(|i| code / 5u64.pow(i as u32) % 5);
            let mut recent = RecentUids::default();
            let mut seen = Vec::new();
            for uid in uids {
                assert_eq!(recent.insert(uid), !seen.contains(&uid), "{uids:?}");
                seen.push(uid);
                assert!(seen.iter().all(|u| recent.contains(*u)), "{uids:?}");
            }
        }
    }

    /// Exhaustive over every three arrivals (seqs 0..=5, two tags) and every
    /// watermark 0..=3: popping upward hands out exactly the buffered
    /// contiguous run, in order, the latest delivery at each seq winning,
    /// and leaves nothing at or below the watermark.
    #[test]
    fn reorder_buffer_hands_out_the_contiguous_run_in_order() {
        let each: Vec<Delivery> = (0..=5)
            .flat_map(|seq| (0..2).map(move |tag| Delivery { seq, tag }))
            .collect();
        for &a in &each {
            for &b in &each {
                for &c in &each {
                    for expected in 0..=3 {
                        let arrivals = [a, b, c];
                        let mut buffer = ReorderBuffer::default();
                        for d in arrivals {
                            assert!(buffer.insert(d));
                        }
                        let latest =
                            |seq: u32| arrivals.iter().rev().find(|d| d.seq == seq).copied();
                        let mut next = expected;
                        while let Some(d) = buffer.pop_next(next) {
                            assert_eq!(Some(d), latest(next), "{arrivals:?} from {expected}");
                            next += 1;
                        }
                        assert_eq!(latest(next), None, "{arrivals:?} from {expected}");
                        assert!(buffer.first_seq().is_none_or(|first| first > next));
                    }
                }
            }
        }
    }
}
