//! Every look-and-feel number of the title screen (design 2a: a menu column
//! on the left third, an animated period map on the right two thirds). Retune
//! the screen here; no logic change is needed for any value.
//!
//! Lengths ending in `_REL` are fractions of the screen width (`COL_*`) or of
//! the map box's height (`MAP_*`), so the composition scales with the window.
//! The map box is the design's 900×800 map area ([`MAP_REGION_ASPECT`]) scaled
//! to cover the actual map region, so at 1280×800 it *is* the region. Plain
//! pixel values are egui points.
//!
//! Values marked "design:" were retuned from the reference design so that no
//! map edge is ever visible; `splash::map` has the test that checks it.

use bevy_egui::egui;

// -- Splash map (option 2a) --------------------------------------------------
/// One title-screen map: its image under `omdurman-app/assets/` and the
/// credit drawn while it shows.
pub(super) struct SplashMapAsset {
    pub(super) file: &'static str,
    pub(super) credit: &'static str,
}

/// The period maps the title screen shows, all public domain (Wikimedia
/// Commons), each cropped inside its printed border. One is picked at random
/// at start; the others follow in this order.
pub(super) const MAPS: [SplashMapAsset; 3] = [
    // `File:River_War_2-4_Omdurman_Battle_6.45am.jpg`, 1600×1200.
    SplashMapAsset {
        file: "splash_map_river_war.webp",
        credit: "The River War (1899): Battle of Omdurman, the first attack, 6.45 a.m.",
    },
    // `File:River_War_2-2_Grand_Advance.jpg`: the panorama's Omdurman end,
    // 4:3 over its full height, 1600×1200.
    SplashMapAsset {
        file: "splash_map_grand_advance.webp",
        credit: "The River War (1899): the Grand Advance, 1898",
    },
    // `File:Plan_of_Omdurman_and_Khartum.png` (B. V. Darbishire), zoomed
    // out: the drawing spans 75% of a 1600×1600 sheet, on matching paper.
    SplashMapAsset {
        file: "splash_map_plan.webp",
        credit: "The Downfall of the Dervishes (1899): plan of Omdurman and Khartum",
    },
];
/// Seconds each map gets, its crossfade into the next included.
pub(super) const MAP_SECS: f32 = 30.0;
/// Length of the crossfade in seconds (part of [`MAP_SECS`]).
pub(super) const MAP_CROSSFADE_SECS: f32 = 10.0;
/// Seconds the first map takes to fade in from the plain backdrop.
pub(super) const MAP_FADE_IN_SECS: f32 = 2.0;
/// Longest step (seconds) the animation takes in one frame, like the design's
/// `Math.min(0.1, dt)`: after a stalled frame the map carries on instead of
/// jumping.
pub(super) const MAP_MAX_STEP_SECS: f32 = 0.1;

/// Fraction of the screen width covered by the map region (right-anchored).
pub(super) const MAP_REGION_W: f32 = 0.703; // 900 / 1280
/// Width / height of the design's map area (900×800). The map is laid out in
/// a box of this shape covering the region, centred, so a wider or narrower
/// window crops the composition instead of exposing a map edge.
pub(super) const MAP_REGION_ASPECT: f32 = 900.0 / 800.0;
/// Map image is drawn as a square this many times the box's height…
/// (design: 1.875 = 1500 / 800)
pub(super) const MAP_SIZE_REL_H: f32 = 2.25; // 1800 / 800
/// …offset from the box's top-left by these fractions of its height; this
/// keeps the transform origin where the design has it (450, 325 @ 900×800).
/// (design: (-0.375, -0.4375) = (-300, -350) / 800)
pub(super) const MAP_OFFSET_REL: egui::Vec2 = egui::vec2(-0.5625, -0.60625); // (-450, -485) / 800
/// Which part of the (portrait) image the square shows, like CSS
/// `background-position`: the square is filled edge to edge ("cover") and
/// this picks the crop's centre, as fractions of the image.
pub(super) const MAP_FOCUS: egui::Vec2 = egui::vec2(0.5, 0.5);
/// Extra zoom so tilted edges never show.
pub(super) const MAP_SCALE: f32 = 1.12;

/// Base rotateX in degrees (0 = flat). The top of the map recedes.
pub(super) const MAP_TILT_DEG: f32 = 37.0;
/// tilt = base + swing + swing·sin(...)  (clamped ≥ 0). (design: 10; any
/// swing on top of the 37° base lets the receding top edge into view)
pub(super) const MAP_TILT_SWING_DEG: f32 = 0.0;
/// Base rotateZ in degrees.
pub(super) const MAP_ROTATE_DEG: f32 = -12.0;
/// rot = base + swing·sin(...).
pub(super) const MAP_ROTATE_SWING_DEG: f32 = 4.0;
/// Mesh subdivisions per side (egui interpolates UVs affinely per triangle,
/// so a finer grid is closer to perspective-correct texturing).
pub(super) const MAP_GRID: usize = 24;
/// Perspective distance as a fraction of the box's height (CSS perspective
/// 1100px / 800).
pub(super) const MAP_PERSPECTIVE_REL_H: f32 = 1.375;
/// Perspective eye (vanishing point), in fractions of the box.
pub(super) const MAP_PERSPECTIVE_ORIGIN: egui::Vec2 = egui::vec2(0.40, 0.40);
/// Pivot of the scale/rotations, in fractions of the image square.
pub(super) const MAP_TRANSFORM_ORIGIN: egui::Vec2 = egui::vec2(0.50, 0.45);

/// Animate the map. Off: it rests at its t = 0 pose.
pub(super) const MAP_PAN: bool = true;
/// Time multiplier of the animation.
pub(super) const MAP_PAN_SPEED: f32 = 0.5;
/// Pan amplitudes as fractions of the box's height (50px, 40px @ 800).
/// (design: major 0.1875 = 150px)
pub(super) const MAP_PAN_AMP_MAJOR: f32 = 0.0625;
pub(super) const MAP_PAN_AMP_MINOR: f32 = 0.05;
/// Periods in seconds; keep them mutually incommensurate so the path never
/// repeats. `(major, minor)` for each pan axis.
pub(super) const MAP_PERIOD_X: (f32, f32) = (53.0, 19.0);
pub(super) const MAP_PERIOD_Y: (f32, f32) = (37.0, 23.0);
pub(super) const MAP_PERIOD_TILT: f32 = 29.0;
pub(super) const MAP_PERIOD_ROT: f32 = 61.0;
/// Phase offsets (radians) of the minor x term, the major y term and the tilt
/// swing, so the curves don't all start at their zero crossing.
pub(super) const MAP_PHASE_X_MINOR: f32 = 1.3;
pub(super) const MAP_PHASE_Y_MAJOR: f32 = 0.7;
pub(super) const MAP_PHASE_TILT: f32 = 2.1;

/// Gaussian blur sigma in screen points, baked into the texture once at load
/// (converted to texture pixels for the size the map is shown at). 0 = none.
pub(super) const MAP_BLUR_PX: f32 = 1.5;
/// Brightness multiplier, baked into the texture between the blur and the
/// sepia (the design's CSS filter order: blur, brightness, sepia).
pub(super) const MAP_BRIGHTNESS: f32 = 0.85;
/// 0..1 mix toward sepia, baked into the texture once at load.
pub(super) const MAP_SEPIA: f32 = 0.35;
/// The standard sepia matrix (rows produce r, g, b), as CSS `sepia(1)`.
pub(super) const SEPIA_MATRIX: [[f32; 3]; 3] = [
    [0.393, 0.769, 0.189],
    [0.349, 0.686, 0.168],
    [0.272, 0.534, 0.131],
];

/// Horizontal fade: (x as fraction of screen width, backdrop alpha 0..1).
pub(super) const FADE_STOPS: [(f32, f32); 5] = [
    (0.0, 1.0),
    (0.37, 1.0),
    (0.46, 0.8),
    (0.60, 0.3),
    (0.75, 0.0),
];
/// Vertical scrims: (y fraction, alpha).
pub(super) const SCRIM_STOPS: [(f32, f32); 4] = [(0.0, 0.5), (0.18, 0.0), (0.82, 0.0), (1.0, 0.6)];

/// Map credit: size (italic) and distance from the bottom-right corner.
pub(super) const CREDIT_SIZE: f32 = 12.0;
pub(super) const CREDIT_MARGIN: egui::Vec2 = egui::vec2(20.0, 16.0);
/// The credit's soft shadow (CSS `0 1px 6px rgba(0,0,0,.9)`): offset, blur
/// radius and peak opacity, approximated by stacked offset copies.
pub(super) const CREDIT_SHADOW_OFFSET: egui::Vec2 = egui::vec2(0.0, 1.0);
pub(super) const CREDIT_SHADOW_RADIUS: f32 = 3.0;
pub(super) const CREDIT_SHADOW_ALPHA: f32 = 0.9;

// -- Menu column -------------------------------------------------------------
pub(super) const COL_LEFT_REL: f32 = 0.05625; // 72 / 1280
pub(super) const COL_W_REL: f32 = 0.367; // 470 / 1280
pub(super) const KICKER: &str = "KHARTOUM 1885 · OMDURMAN 1898";
pub(super) const KICKER_SIZE: f32 = 14.0;
/// Letter spacing in em (egui: `extra_letter_spacing = size·tracking`).
pub(super) const KICKER_TRACKING: f32 = 0.28;
/// Bold, two lines: "REMEMBER" / "GORDON!".
pub(super) const TITLE_SIZE: f32 = 56.0;
/// Title line height, as a multiple of its size.
pub(super) const TITLE_LINE_HEIGHT: f32 = 1.08;
pub(super) const QUOTE_SIZE: f32 = 22.0;
/// Quote line height, as a multiple of its size.
pub(super) const QUOTE_LINE_HEIGHT: f32 = 1.5;
pub(super) const QUOTE_WRAP: f32 = 360.0;
pub(super) const ATTR_SIZE: f32 = 16.0;
pub(super) const BUTTON_SIZE: egui::Vec2 = egui::vec2(300.0, 44.0);
pub(super) const BUTTON_TEXT: f32 = 21.0;
/// Left inset of the (left-aligned) button label.
pub(super) const BUTTON_PAD_X: f32 = 18.0;
pub(super) const BUTTON_RADIUS: u8 = 2;
pub(super) const GAP_KICKER_TITLE: f32 = 16.0;
pub(super) const GAP_TITLE_QUOTE: f32 = 36.0;
pub(super) const GAP_QUOTE_ATTR: f32 = 12.0;
pub(super) const GAP_ATTR_BUTTONS: f32 = 44.0;
pub(super) const GAP_BUTTONS: f32 = 10.0;

// -- Narrow windows ----------------------------------------------------------
/// Below this window width the screen falls back to one centred column, with
/// the map full-bleed behind it.
pub(super) const NARROW_BREAKPOINT: f32 = 900.0;
/// The uniform scrim over the full-bleed map (replaces the horizontal fade).
pub(super) const NARROW_SCRIM_ALPHA: f32 = 0.72;
/// Where the centred column starts, as a fraction of screen height.
pub(super) const NARROW_TOP_REL: f32 = 0.26;
pub(super) const NARROW_TITLE_SIZE: f32 = 52.0;
pub(super) const NARROW_QUOTE_SIZE: f32 = 34.0;
pub(super) const NARROW_SMALL_SIZE: f32 = 22.0;
/// Quote wrap: this fraction of the screen width, at most `NARROW_WRAP_MAX`.
pub(super) const NARROW_WRAP_REL: f32 = 0.7;
pub(super) const NARROW_WRAP_MAX: f32 = 820.0;
pub(super) const NARROW_GAP_LARGE: f32 = 56.0;
pub(super) const NARROW_GAP_ATTR: f32 = 16.0;
pub(super) const NARROW_GAP_BUTTONS: f32 = 12.0;
