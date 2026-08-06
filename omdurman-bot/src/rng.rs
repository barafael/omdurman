//! Deterministic PRNG mirroring the app's [`GameRng`] so the same seed produces
//! the same dice sequence in both the headless bot and the live game.

use omdurman_rules::DieRoll;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct BotRng(ChaCha8Rng);

impl BotRng {
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }

    /// Roll a d10 (1..=10) as a validated [`DieRoll`]. Mirrors
    /// `omdurman_app::state::GameRng::roll_d10` exactly so a replayed seed
    /// reproduces the same rolls.
    pub fn roll_d10(&mut self) -> DieRoll {
        DieRoll::try_from(((self.0.random::<u32>() % 10) + 1) as u16).unwrap()
    }

    /// Roll a d6 (1..=6) for desertion (§8.2).
    pub fn roll_d6(&mut self) -> u8 {
        ((self.0.random::<u32>() % 6) + 1) as u8
    }

    /// Pick a uniform-random element, or `None` if the slice is empty.
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            None
        } else {
            let idx = (self.0.random::<u32>() as usize) % slice.len();
            Some(&slice[idx])
        }
    }

    /// Shuffle a slice in place (Fisher–Yates).
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let n = slice.len();
        for i in (1..n).rev() {
            let j = (self.0.random::<u32>() as usize) % (i + 1);
            slice.swap(i, j);
        }
    }
}
