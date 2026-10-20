//! The Zariba going up: hexsides built live (§5.3) are laid along their
//! length, one after another, instead of appearing all at once. The hexside
//! layer draws each growing side shortened to [`ZaribaGrowth::fraction`].

use bevy::prelude::*;
use omdurman_types::HexsideRef;

use super::MotionSettings;

/// Seconds one hexside takes to lay.
const LAY_SECS: f32 = 0.6;
/// Seconds between the starts of two sides built together.
const STAGGER_SECS: f32 = 0.25;

/// The Zariba hexsides still being laid, with their age in seconds
/// (negative while waiting their turn).
#[derive(Resource, Default, Debug)]
pub struct ZaribaGrowth {
    sides: Vec<(HexsideRef, f32)>,
}

impl ZaribaGrowth {
    /// Lay `sides`, after any still going up.
    pub fn start(&mut self, sides: &[HexsideRef]) {
        let queued = self.sides.len();
        for (i, side) in sides.iter().enumerate() {
            let age = -STAGGER_SECS * (queued + i) as f32;
            self.sides.push((*side, age));
        }
    }

    /// How much of `side` is drawn: `None` when it is not growing (drawn
    /// whole), else `0..1`, easing out.
    pub fn fraction(&self, side: &HexsideRef) -> Option<f32> {
        let (_, age) = self.sides.iter().find(|(s, _)| s == side)?;
        let p = (age / LAY_SECS).clamp(0.0, 1.0);
        Some(1.0 - (1.0 - p) * (1.0 - p))
    }

    pub fn is_empty(&self) -> bool {
        self.sides.is_empty()
    }

    pub fn clear(&mut self) {
        self.sides.clear();
    }

    /// Age every side by `dt`; drop the ones fully laid.
    fn tick(&mut self, dt: f32) {
        for (_, age) in &mut self.sides {
            *age += dt;
        }
        self.sides.retain(|(_, age)| *age < LAY_SECS);
    }
}

/// Lay the growing sides; ask for frames while any is going up. Without
/// motion a built side simply appears.
pub fn grow_zariba(
    time: Res<Time>,
    settings: Res<MotionSettings>,
    mut growth: ResMut<ZaribaGrowth>,
    mut activity: ResMut<crate::activity::Activity>,
) {
    if growth.is_empty() {
        return;
    }
    if !settings.motion() {
        growth.clear();
        return;
    }
    growth.tick(time.delta_secs());
    activity.keep_running();
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_types::HexCoord;

    fn side(q: i32) -> HexsideRef {
        HexsideRef::new(HexCoord::new(q, 0), HexCoord::new(q + 1, 0))
    }

    #[test]
    fn sides_built_together_are_laid_one_after_another() {
        let mut growth = ZaribaGrowth::default();
        growth.start(&[side(0), side(1)]);
        assert_eq!(growth.fraction(&side(0)), Some(0.0));
        growth.tick(STAGGER_SECS + 0.01);
        let first = growth.fraction(&side(0)).unwrap();
        assert!(first > 0.0 && first < 1.0);
        let second = growth.fraction(&side(1)).unwrap();
        assert!(second > 0.0 && second < first);
        assert_eq!(growth.fraction(&side(2)), None, "never built: drawn whole");
    }

    #[test]
    fn a_laid_side_is_drawn_whole_and_the_growth_ends() {
        let mut growth = ZaribaGrowth::default();
        growth.start(&[side(0), side(1), side(2)]);
        for _ in 0..40 {
            growth.tick(0.05);
        }
        assert!(growth.is_empty());
        assert_eq!(growth.fraction(&side(0)), None);
    }
}
