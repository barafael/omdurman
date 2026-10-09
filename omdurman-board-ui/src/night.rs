//! Day/night board tint (§8.1, §night tint). The
//! *source of truth* for the current time of day is injected via the
//! [`BoardDayNight`] resource, keeping this crate game-agnostic (the game
//! mirrors `GameState.day_night` into it with a tiny sync system).

use bevy::prelude::*;
use omdurman_hexmap::MapPlane;

/// How the board scan looks at full night, and how fast it eases there. The
/// scenario plays across day and night turns (§8.1); we tint the *board scan*
/// -- the map plane's material -- to make the current time of day legible at a
/// glance. Only the scan: counters, hex rings, path arrows and fire lines keep
/// their colours, since a camera-wide grade (the first version) desaturated
/// every green ring and red arrow on the board to the same mud at night, just
/// when movement and fire most need reading. Purely presentational and
/// derived from the replicated game state, so it needs no networking and
/// stays identical on every peer.
///
/// The map material is unlit, so its base colour multiplies the scan: about
/// 1.3 EV darker and cooler (the sepia scan has little blue, so a blue-heavy
/// tint lands on a grey-green night instead of olive).
const NIGHT_MAP_TINT: LinearRgba = LinearRgba::rgb(0.30, 0.37, 0.50);
const NIGHT_FADE_PER_SEC: f32 = 0.67; // ~1.5s day<->night crossfade (§night tint)

/// The board-wide time of day the night shading eases toward. `None` (or an
/// absent resource) means "day / unknown" — grading stays untouched. The game
/// writes this from the replicated rules state every frame.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct BoardDayNight(pub Option<omdurman_types::DayNight>);

/// Whether the day/night grading is still easing toward its target: a
/// reactive app keeps redrawing until it has settled, so the fade plays at
/// the display rate instead of at the idle wake-up rate. Published by
/// [`night_shading`] when the binary inserts it.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NightFading(pub bool);

/// Ease the board scan's tint toward the day/night target each frame: a
/// `night` factor of 0 is full daylight (the scan untouched), 1 is full night
/// ([`NIGHT_MAP_TINT`]). Interpolated so the transition fades rather than
/// snaps when a turn crosses dawn/dusk.
pub fn night_shading(
    time: Res<Time>,
    day_night: Option<Res<BoardDayNight>>,
    plane: Query<&MeshMaterial3d<StandardMaterial>, With<MapPlane>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    fading: Option<ResMut<NightFading>>,
    mut night: Local<f32>,
    // Dev: OMDURMAN_FORCE_NIGHT forces the night look for verification. Read
    // once (the environment doesn't change mid-run) instead of every frame.
    mut force_night: Local<Option<bool>>,
) {
    let Ok(material) = plane.single() else {
        return;
    };
    let force_night =
        *force_night.get_or_insert_with(|| std::env::var_os("OMDURMAN_FORCE_NIGHT").is_some());
    let target = match day_night.map(|d| d.0) {
        _ if force_night => 1.0,
        Some(Some(omdurman_types::DayNight::Night)) => 1.0,
        _ => 0.0,
    };
    // Frame-rate-independent ease toward the target, clamped so a long frame
    // can't overshoot past the endpoint. Snap once close so the ease settles
    // and the material stops being rewritten.
    let step = (NIGHT_FADE_PER_SEC * time.delta_secs()).min(1.0);
    let mut next = *night + (target - *night) * step;
    if (target - next).abs() < 1e-4 {
        next = target;
    }
    *night = next;
    if let Some(mut fading) = fading {
        fading.set_if_neq(NightFading(next != target));
    }

    let color = Color::LinearRgba(LinearRgba::WHITE.mix(&NIGHT_MAP_TINT, next));
    // Only touch the asset when the colour actually changes: `get_mut` marks
    // the material changed (and re-uploads it) every frame otherwise.
    if materials
        .get(&material.0)
        .is_some_and(|m| m.base_color != color)
        && let Some(mut m) = materials.get_mut(&material.0)
    {
        m.base_color = color;
    }
}
