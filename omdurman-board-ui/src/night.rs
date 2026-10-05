//! Day/night board colour grading (§8.1, §night tint). The
//! *source of truth* for the current time of day is injected via the
//! [`BoardDayNight`] resource, keeping this crate game-agnostic (the game
//! mirrors `GameState.day_night` into it with a tiny sync system).

use bevy::prelude::*;
use bevy::render::view::ColorGrading;

use crate::camera::RtsCamera;

/// How dark and desaturated the board looks at full night, and how fast it
/// eases there. The scenario plays across day and night turns (§8.1); we shade
/// the *rendered board* (not the egui UI, which renders in its own pass) to
/// make the current time of day legible at a glance. Purely presentational and
/// derived from the replicated game state, so it needs no networking and stays
/// identical on every peer.
const NIGHT_EXPOSURE: f32 = -1.3; // EV stops darker at full night
const NIGHT_SATURATION: f32 = 0.35; // post_saturation at full night (1.0 = unchanged)
const NIGHT_FADE_PER_SEC: f32 = 0.67; // ~1.5s day<->night crossfade (§night tint)
// Push colour toward the printed NIGHT-cell green at full night: cooler
// temperature + green tint. Bevy's convention: negative temperature = cooler,
// negative tint = toward green. Scaled by the eased `night` factor so day is
// untouched. UI chrome is unaffected (ColorGrading applies to the camera view
// only, not egui).
const NIGHT_TEMPERATURE: f32 = -0.25;
const NIGHT_TINT: f32 = -0.30;

/// The board-wide time of day the night shading eases toward. `None` (or an
/// absent resource) means "day / unknown" — grading stays untouched. The game
/// writes this from the replicated rules state every frame.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct BoardDayNight(pub Option<omdurman_types::DayNight>);

/// Ease the camera's colour grading toward the day/night target each frame: a
/// `night` factor of 0 is full daylight (grading untouched), 1 is full night
/// (darker + desaturated). Interpolated so the transition fades rather than
/// snaps when a turn crosses dawn/dusk.
pub fn night_shading(
    time: Res<Time>,
    day_night: Option<Res<BoardDayNight>>,
    mut grading: Query<&mut ColorGrading, With<RtsCamera>>,
    mut night: Local<f32>,
    // Dev: OMDURMAN_FORCE_NIGHT forces the night look for verification. Read
    // once (the environment doesn't change mid-run) instead of every frame.
    mut force_night: Local<Option<bool>>,
) {
    let Ok(mut grading) = grading.single_mut() else {
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
    // and the grading stops being rewritten.
    let step = (NIGHT_FADE_PER_SEC * time.delta_secs()).min(1.0);
    let mut next = *night + (target - *night) * step;
    if (target - next).abs() < 1e-4 {
        next = target;
    }
    *night = next;

    let exposure = NIGHT_EXPOSURE * next;
    let post_saturation = 1.0 + (NIGHT_SATURATION - 1.0) * next;
    // Tint toward the night-cell green as night deepens.
    let temperature = NIGHT_TEMPERATURE * next;
    let tint = NIGHT_TINT * next;
    // Only touch the component when a value actually changes: a `Mut` write
    // marks `ColorGrading` changed (and re-extracts it) every frame otherwise.
    let g = &grading.global;
    if g.exposure != exposure
        || g.post_saturation != post_saturation
        || g.temperature != temperature
        || g.tint != tint
    {
        let g = &mut grading.global;
        g.exposure = exposure;
        g.post_saturation = post_saturation;
        g.temperature = temperature;
        g.tint = tint;
    }
}
