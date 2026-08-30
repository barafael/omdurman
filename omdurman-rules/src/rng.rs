//! The shared deterministic PRNG (ChaCha8), seeded from the canonical game
//! record so every peer — live app and headless bot alike — draws the same
//! dice sequence for the same seed.
//!
//! This used to live as two hand-mirrored copies (`omdurman-app::state::GameRng`
//! and `omdurman-bot::rng::BotRng`); any edit to one silently desynchronised
//! bot traces from app replays. Owning it here (next to [`crate::DieRoll`],
//! which it produces) makes the engine the single source of the dice stream.

use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::DieRoll;

/// Deterministic PRNG resource shared by every peer.
#[derive(Clone)]
pub struct GameRng(ChaCha8Rng);

impl GameRng {
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }

    /// Roll a d10 (1..=10) as a validated [`DieRoll`]. The 1..=10 range is a
    /// closed subset of `DieRoll`'s valid domain, so the conversion never
    /// fails; consolidating it here keeps the modulo-10 + `unwrap` pattern in
    /// one place.
    pub fn roll_d10(&mut self) -> DieRoll {
        DieRoll::try_from((self.random_u32() % 10 + 1) as u16).unwrap()
    }

    /// Roll a d6 (1..=6) as a plain `u8` (used by the desertion roll's
    /// display table and by bot tooling).
    pub fn roll_d6(&mut self) -> u8 {
        (self.random_u32() % 6 + 1) as u8
    }

    /// Draw one raw `u32` from the shared stream. This is the primitive every
    /// derived roll funnels through, so strategy code (the bot's
    /// `choose`/`shuffle`) can consume the same sequence deterministically.
    pub fn random_u32(&mut self) -> u32 {
        self.0.random::<u32>()
    }
}
