//! Rule-level types for "REMEMBER GORDON!" -- The Battle of Omdurman.
//!
//! Every fact stated in the printed rulebook (Phoenix Enterprises, Ltd., 1982)
//! that affects a legal move, a legal stack, a fire/melee resolution, or a
//! victory tally is encoded here as an enum, a tuple struct, or a struct so
//! that the rules engine can statically prove which states are reachable.
//!
//! Enums are used for every quantitative value that has a fixed, annotated set
//! of possible values so that match arms are exhaustive at compile time.
//! Tuple structs remain only for values with an unbounded range (movement
//! points, hex distances, victory points, game-turn indices).

use omdurman_types::{
    BrigadeId, BrigadeNationality, DayNight, DervishTribe, HexCoord, Player, UnitKind,
};

pub mod board;
pub mod board_data;
pub mod combat_results_table;
pub mod effects;
pub mod howitzer_scatter;
pub mod los_table;
pub mod newspaper;
pub mod range_effects;
pub mod reinforcements;
pub mod rng;
pub mod scenario_setup;
pub mod sprite_data;
pub mod tables_data;
pub mod tactics;
pub mod telegram_prompt;
pub mod terrain_chart;
pub mod turn_summary;
pub mod turn_track;
pub mod unit_profiles;
use crate::combat_results_table::FireFactorRow;

mod unit_id;
pub use unit_id::*;

/// Generates an enum for a quantitative value with a fixed, annotated set of
/// possibilities (see the crate docs), plus `ALL`, `value()` and
/// `TryFrom<u16>`. Defined before the type modules below so textual macro
/// scoping makes it visible inside them.
macro_rules! value_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident = $value:expr,)+
        }
    ) => {
        $(#[$meta])*
        pub enum $name {
            $($(#[$variant_meta])* $variant,)+
        }

        impl $name {
            /// Every variant, in declaration order. Generated so exhaustive
            /// callers (tests, Kani proofs) pick up new variants automatically
            /// instead of silently skipping them.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];

            pub fn value(self) -> u16 {
                match self {
                    $(Self::$variant => $value,)+
                }
            }
        }

        impl TryFrom<u16> for $name {
            type Error = ();
            fn try_from(v: u16) -> Result<Self, ()> {
                match v {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(()),
                }
            }
        }
    };
}

mod combat;
mod scalars;
mod transport;
mod turn;
mod unit;
mod victory;

pub use combat::*;
pub use scalars::*;
pub use transport::*;
pub use turn::*;
pub use unit::*;
pub use victory::*;

#[cfg(test)]
mod tests;

#[cfg(kani)]
mod verification;

#[cfg(all(kani, feature = "kani-quantifiers"))]
mod quantifier_experiment;
