//! Bot strategy helpers over the engine's shared [`GameRng`].
//!
//! The deterministic dice stream itself lives in
//! `omdurman_rules::rng::GameRng` (the app draws from the same type); this
//! wrapper only adds the bot's selection helpers (`choose`, `shuffle`), which
//! consume the same stream via [`GameRng::random_u32`].

use omdurman_rules::rng::GameRng;

pub struct BotRng(GameRng);

impl BotRng {
    pub fn from_seed(seed: u64) -> Self {
        Self(GameRng::from_seed(seed))
    }

    pub fn roll_d10(&mut self) -> omdurman_rules::DieRoll {
        self.0.roll_d10()
    }

    /// Roll a d6 (1..=6) for desertion (§8.2).
    pub fn roll_d6(&mut self) -> u8 {
        self.0.roll_d6()
    }

    /// Pick a uniform-random element, or `None` if the slice is empty.
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            None
        } else {
            let idx = (self.0.random_u32() as usize) % slice.len();
            Some(&slice[idx])
        }
    }

    /// Shuffle a slice in place (Fisher–Yates).
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let n = slice.len();
        for i in (1..n).rev() {
            let j = (self.0.random_u32() as usize) % (i + 1);
            slice.swap(i, j);
        }
    }
}
