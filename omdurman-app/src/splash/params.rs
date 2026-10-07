//! Every look-and-feel number of the title screen (design 2a: a menu column
//! on the left third, an animated period map on the right two thirds). Retune
//! the screen here; no logic change is needed for any value.
//!
//! Lengths ending in `_REL` are fractions of the screen width (`COL_*`) or of
//! a map box's height ([`MapLayout`]), so the composition scales with the
//! window. For the title screen the map box is the design's 900×800 map area
//! scaled to cover the actual map region, so at 1280×800 it *is* the region.
//! Plain pixel values are logical pixels (Bevy UI `Val::Px`).

use bevy::math::Vec2;

// -- Splash map (option 2a) --------------------------------------------------
/// One view of a map, a place a slide can start from: the point of the image
/// it centres on (fractions of the image), and how far it zooms in (1: the
/// shown square spans the image's shorter side). The square is held inside the
/// image, so a view near an edge stops at it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MapView {
    pub(super) centre: Vec2,
    pub(super) zoom: f32,
}

const fn view(x: f32, y: f32, zoom: f32) -> MapView {
    MapView {
        centre: Vec2::new(x, y),
        zoom,
    }
}

/// The whole map: every map's first view.
pub(super) const WHOLE: MapView = view(0.5, 0.5, 1.0);
/// How far the detail views zoom in: further, and the ~1200-pixel scans turn
/// soft.
const DETAIL: f32 = 1.3;

/// One title-screen map: its image under `omdurman-app/assets/`, the credit
/// drawn while it shows, and the views a slide of it can start from.
pub(super) struct SplashMapAsset {
    pub(super) file: &'static str,
    pub(super) credit: &'static str,
    pub(super) views: &'static [MapView],
}

/// The period maps the title screen shows, all public domain (Wikimedia
/// Commons), each cropped inside its printed border; the River War plates
/// (Churchill, *The River War*, 1899) in the order of the battle. One is
/// picked at random at start; the others follow in this order.
pub(super) const MAPS: [SplashMapAsset; 10] = [
    // `File:River_War_2-2_Grand_Advance.jpg`: the panorama's Omdurman end,
    // 4:3 over its full height, 1600×1200.
    SplashMapAsset {
        file: "splash_map_grand_advance.webp",
        credit: "The River War (1899): the Grand Advance, 1898",
        views: &[
            WHOLE,
            view(0.62, 0.55, DETAIL), // Omdurman, the Kerreri hills
            view(0.8, 0.4, DETAIL),   // Khartoum and the confluence
            view(0.32, 0.4, DETAIL),  // the camps of 30 and 31 August
        ],
    },
    // `File:River_War_2-3_Omdurman_1st_September_1.30pm.jpg`, 1200×1253.
    SplashMapAsset {
        file: "splash_map_first_september.webp",
        credit: "The River War (1899): operations of the first of September, 1.30 p.m.",
        views: &[
            WHOLE,
            view(0.3, 0.6, DETAIL),   // the Dervish army forming
            view(0.65, 0.35, DETAIL), // the zariba at El Egeiga
            view(0.35, 0.8, DETAIL),  // towards Omdurman
        ],
    },
    // `File:River_War_2-4_Omdurman_Battle_6.45am.jpg`, 1600×1200.
    SplashMapAsset {
        file: "splash_map_river_war.webp",
        credit: "The River War (1899): Battle of Omdurman, the first attack, 6.45 a.m.",
        views: &[
            WHOLE,
            view(0.3, 0.45, DETAIL),  // Jebel Surgham, the Black Flag
            view(0.5, 0.7, DETAIL),   // the zariba
            view(0.75, 0.45, DETAIL), // the Kerreri hills
        ],
    },
    // `File:River_War_2-5_Charge_21st_Lancers.jpg`, 1269×1200.
    SplashMapAsset {
        file: "splash_map_lancers.webp",
        credit: "The River War (1899): the charge of the 21st Lancers",
        views: &[
            WHOLE,
            view(0.35, 0.6, DETAIL),  // the khor and the charge
            view(0.55, 0.25, DETAIL), // the advance from Surgham
            view(0.2, 0.25, DETAIL),  // the Dervish second position
        ],
    },
    // `File:River_War_2-6_Omdurman_Battle_9.40am.jpg`, 1606×1200.
    SplashMapAsset {
        file: "splash_map_battle_0940.webp",
        credit: "The River War (1899): Battle of Omdurman, the Khalifa's attack, 9.40 a.m.",
        views: &[
            WHOLE,
            view(0.35, 0.35, DETAIL), // the Khalifa's attack
            view(0.45, 0.6, DETAIL),  // the brigades wheeling
            view(0.78, 0.4, DETAIL),  // Ali Wad Helu's Green Flag
        ],
    },
    // `File:River_War_2-8_Omdurman_Battle_MacDonalds_Change.jpg`, 1366×1200.
    SplashMapAsset {
        file: "splash_map_macdonald.webp",
        credit: "The River War (1899): MacDonald's change of front, 9.50–10.15 a.m.",
        views: &[
            WHOLE,
            view(0.45, 0.4, DETAIL), // the line meeting the Khalifa
            view(0.3, 0.35, DETAIL), // the Khalifa's and Yakub's attack
            view(0.75, 0.3, DETAIL), // the line meeting Kerreri
        ],
    },
    // `File:River_War_2-7_Omdurman_Battle_10.15am.jpg`, 1607×1200.
    SplashMapAsset {
        file: "splash_map_battle_1015.webp",
        credit: "The River War (1899): Battle of Omdurman, the attack on MacDonald, 10.15 a.m.",
        views: &[
            WHOLE,
            view(0.3, 0.35, DETAIL),  // Surgham, the Black Flag routed
            view(0.62, 0.55, DETAIL), // MacDonald's brigade
            view(0.85, 0.45, DETAIL), // the Green Flag on the Kerreri hills
        ],
    },
    // `File:River_War_2-9_Omdurman_Battle_Noon.jpg`, 1200×1251.
    SplashMapAsset {
        file: "splash_map_noon.webp",
        credit: "The River War (1899): Battle of Omdurman, noon",
        views: &[
            WHOLE,
            view(0.6, 0.5, DETAIL),   // the army marching on Omdurman
            view(0.3, 0.82, DETAIL),  // the 21st Lancers, the Dervish retreat
            view(0.75, 0.35, DETAIL), // the Nile and Halfaya
        ],
    },
    // `File:River_War_2-10_Omdurman_Battle_After.jpg`, 1610×1200.
    SplashMapAsset {
        file: "splash_map_after.webp",
        credit: "The River War (1899): Battle of Omdurman, after the battle",
        views: &[
            WHOLE,
            view(0.3, 0.45, DETAIL),  // the Khalifa's attack, Surgham
            view(0.66, 0.45, DETAIL), // the Kerreri hills
            view(0.45, 0.7, DETAIL),  // the frontal attack
        ],
    },
    // `File:Plan_of_Omdurman_and_Khartum.png` (B. V. Darbishire), zoomed
    // out: the drawing spans 75% of a 1600×1600 sheet, on matching paper.
    SplashMapAsset {
        file: "splash_map_plan.webp",
        credit: "The Downfall of the Dervishes (1899): plan of Omdurman and Khartum",
        views: &[
            WHOLE,
            view(0.4, 0.5, DETAIL),   // the town and its wall
            view(0.65, 0.55, DETAIL), // the river forts
        ],
    },
];
/// Seconds each map gets, its crossfade into the next included.
pub(super) const MAP_SECS: f32 = 24.0;
/// Length of the crossfade in seconds (part of [`MAP_SECS`]); it eases in
/// and out (`map::ease`).
pub(super) const MAP_CROSSFADE_SECS: f32 = 6.0;
/// Seconds the first map takes to fade in from the plain backdrop.
pub(super) const MAP_FADE_IN_SECS: f32 = 2.0;
/// Longest step (seconds) the animation takes in one frame: after a stalled
/// frame the map carries on instead of jumping. Above the slowest regular
/// frame -- the ambient pace of an unfocused window -- so the show keeps its
/// speed there (the design's `Math.min(0.1, dt)` halved it at 5 fps).
pub(super) const MAP_MAX_STEP_SECS: f32 = 1.25 / crate::activity::AMBIENT_UNFOCUSED_FPS;

/// Fraction of the screen width covered by the map region (right-anchored).
pub(super) const MAP_REGION_W: f32 = 0.703; // 900 / 1280

/// How a map is placed and posed in the region it fills. Lengths are
/// fractions of the height of a box of `region_aspect` scaled to cover the
/// region, centred on it, so a wider or narrower window crops the composition
/// instead of exposing a map edge; `splash::map`'s coverage test checks both
/// layouts. The projection mirrors the design's CSS: `translate(pan)
/// rotateX(tilt) rotateZ(rot) scale(scale)` about `transform_origin`, under a
/// `perspective` whose eye is at `perspective_origin`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MapLayout {
    /// Width / height of the box the layout is tuned in.
    pub(super) region_aspect: f32,
    /// Side of the (square) map image, in box heights…
    pub(super) size_rel_h: f32,
    /// …and its top-left corner, from the box's top-left.
    pub(super) offset_rel: Vec2,
    /// Pivot of the scale and rotations, in fractions of the image square.
    pub(super) transform_origin: Vec2,
    /// Extra zoom about the pivot.
    pub(super) scale: f32,
    /// rotateX in degrees (0 = flat; the top recedes):
    /// tilt = base + swing + swing·sin(...), clamped ≥ 0.
    pub(super) tilt_deg: f32,
    pub(super) tilt_swing_deg: f32,
    /// rotateZ in degrees: rot = base + swing·sin(...).
    pub(super) rotate_deg: f32,
    pub(super) rotate_swing_deg: f32,
    /// Perspective distance in box heights, and the eye (vanishing point) in
    /// fractions of the box.
    pub(super) perspective_rel_h: f32,
    pub(super) perspective_origin: Vec2,
    /// Pan amplitudes (major, minor) in box heights; the periods and phases
    /// below are shared.
    pub(super) pan_amp: (f32, f32),
}

/// The title screen's map: design 2a's tilted map in its 900×800 area.
/// Values marked "design:" were retuned so no map edge ever shows; the pivot
/// stays where the design has it (450, 325 @ 900×800).
pub(super) const TITLE_MAP: MapLayout = MapLayout {
    region_aspect: 900.0 / 800.0,
    // design: 1.875 = 1500 / 800
    size_rel_h: 2.25,
    // design: (-0.375, -0.4375) = (-300, -350) / 800
    offset_rel: Vec2::new(-0.5625, -0.60625),
    transform_origin: Vec2::new(0.50, 0.45),
    scale: 1.12,
    tilt_deg: 37.0,
    // design: 10; any swing on top of the 37° base lets the receding top
    // edge into view
    tilt_swing_deg: 0.0,
    rotate_deg: -12.0,
    // design: 4
    rotate_swing_deg: 1.5,
    // CSS perspective 1100px / 800, origin 40% 40%
    perspective_rel_h: 1.375,
    perspective_origin: Vec2::new(0.40, 0.40),
    // 50px, 40px @ 800 (design: major 150px)
    pan_amp: (0.0625, 0.05),
};

/// The lobby's map: flat and full-screen, turning a little and drifting, and
/// less zoomed in than the title screen's.
pub(super) const LOBBY_MAP: MapLayout = MapLayout {
    region_aspect: 16.0 / 9.0,
    size_rel_h: 3.0,
    // centred on the box: ((16/9 - 3) / 2, (1 - 3) / 2)
    offset_rel: Vec2::new(-0.611_111, -1.0),
    transform_origin: Vec2::new(0.5, 0.5),
    scale: 1.0,
    tilt_deg: 0.0,
    tilt_swing_deg: 0.0,
    rotate_deg: -6.0,
    rotate_swing_deg: 1.0,
    perspective_rel_h: 1.375,
    perspective_origin: Vec2::new(0.5, 0.5),
    pan_amp: (0.05, 0.03),
};

/// Animate the map. Off: it rests at its t = 0 pose.
pub(super) const MAP_PAN: bool = true;
/// Time multiplier of the animation: a slow drift, so the show reads as a
/// sequence of still views rather than motion.
pub(super) const MAP_PAN_SPEED: f32 = 0.25;
/// Each slide of the show starts at a random point of the pan's path: a pan
/// time drawn from `0..MAP_START_SPREAD_SECS`, so every slide (and every
/// launch) shows the map from a different place (the periods below never
/// repeat, so any spread gives fresh poses).
pub(super) const MAP_START_SPREAD_SECS: f32 = 600.0;
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
/// Per-run variety ([`super::map::PanVariation`]): each launch scales every
/// period by up to this fraction either way, shifts the phases by up to
/// [`PAN_PHASE_JITTER`] radians, and scales the pan's reach and the swings
/// *down* by up to [`PAN_AMP_JITTER`] and [`PAN_SWING_JITTER`] -- down only,
/// so the map covers its region in every run as it does with the values
/// above.
pub(super) const PAN_PERIOD_JITTER: f32 = 0.15;
pub(super) const PAN_PHASE_JITTER: f32 = 0.6;
pub(super) const PAN_AMP_JITTER: f32 = 0.15;
pub(super) const PAN_SWING_JITTER: f32 = 0.2;

/// Gaussian blur sigma in screen points, converted to texture pixels for the
/// size the map is shown at in a [`REFERENCE_HEIGHT`] box; the shader blurs
/// in texture space, so the blur scales with the map. 0 = none.
pub(super) const MAP_BLUR_PX: f32 = 1.5;
/// Map box height (points) the blur radii are converted for: the design's
/// 1280×800 reference. Fixed rather than read from the window, so the blur
/// does not change with the window size.
pub(super) const REFERENCE_HEIGHT: f32 = 800.0;
/// Brightness multiplier, applied between the blur and the sepia (the
/// design's CSS filter order: blur, brightness, sepia).
pub(super) const MAP_BRIGHTNESS: f32 = 0.85;
/// 0..1 mix toward sepia.
pub(super) const MAP_SEPIA: f32 = 0.35;
/// The standard sepia matrix (rows produce r, g, b), as CSS `sepia(1)`.
pub(super) const SEPIA_MATRIX: [[f32; 3]; 3] = [
    [0.393, 0.769, 0.189],
    [0.349, 0.686, 0.168],
    [0.272, 0.534, 0.131],
];

/// Horizontal fade: (x as fraction of screen width, backdrop alpha 0..1, as in
/// the design's CSS -- see [`CSS_BLEND_GAMMA`]). The solid part must reach
/// past the map region's left edge (`1 - MAP_REGION_W`), which a test checks.
/// (design: solid to 0.37, 0.8 @ 0.46, 0.3 @ 0.60, clear @ 0.75)
pub(super) const FADE_STOPS: [(f32, f32); 5] = [
    (0.0, 1.0),
    (0.30, 1.0),
    (0.41, 0.8),
    (0.60, 0.3),
    (0.80, 0.0),
];
/// Vertical scrims: (y fraction, alpha, as in the design's CSS).
pub(super) const SCRIM_STOPS: [(f32, f32); 4] = [(0.0, 0.5), (0.18, 0.0), (0.82, 0.0), (1.0, 0.6)];

/// The GPU blends in linear light, the design's CSS in sRGB, so a dark
/// overlay darkens much less than its CSS opacity would. The fade and scrims
/// above take CSS opacities `a` and are drawn at `1 - (1 - a)^γ`, which
/// darkens (near-black over the map) as the design does.
pub(super) const CSS_BLEND_GAMMA: f32 = 2.2;

/// Map credit: size (italic) and distance from the bottom-right corner.
pub(super) const CREDIT_SIZE: f32 = 12.0;
pub(super) const CREDIT_MARGIN: Vec2 = Vec2::new(20.0, 16.0);
/// The credit's soft shadow (CSS `0 1px 6px rgba(0,0,0,.9)`): offset, blur
/// radius and peak opacity, approximated by stacked offset copies.
pub(super) const CREDIT_SHADOW_OFFSET: Vec2 = Vec2::new(0.0, 1.0);
pub(super) const CREDIT_SHADOW_RADIUS: f32 = 3.0;
pub(super) const CREDIT_SHADOW_ALPHA: f32 = 0.9;

// -- Menu column -------------------------------------------------------------
pub(super) const COL_LEFT_REL: f32 = 0.05625; // 72 / 1280
pub(super) const COL_W_REL: f32 = 0.367; // 470 / 1280
pub(super) const KICKER: &str = "KHARTOUM 1885 · OMDURMAN 1898";
pub(super) const KICKER_SIZE: f32 = 14.0;
/// Letter spacing in em.
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
pub(super) const BUTTON_SIZE: Vec2 = Vec2::new(300.0, 44.0);
pub(super) const BUTTON_TEXT: f32 = 21.0;
/// Left inset of the (left-aligned) button label.
pub(super) const BUTTON_PAD_X: f32 = 18.0;
pub(super) const BUTTON_RADIUS: f32 = 2.0;
pub(super) const GAP_KICKER_TITLE: f32 = 16.0;
pub(super) const GAP_TITLE_QUOTE: f32 = 36.0;
pub(super) const GAP_QUOTE_ATTR: f32 = 12.0;
pub(super) const GAP_ATTR_BUTTONS: f32 = 44.0;
pub(super) const GAP_BUTTONS: f32 = 10.0;

/// Line height (multiple of the size) of serif text that sets none of its
/// own: Merriweather's ascent − descent (+ line gap 0), as its `hhea` table
/// gives it and the egui screen it replaces laid it out.
pub(super) const SERIF_LINE_HEIGHT: f32 = 1.257;
/// Opacity of the disabled buttons (their fill, border and label).
pub(super) const DISABLED_ALPHA: f32 = 0.5;
/// The disabled "Game" button's hint: size, offset from the pointer, padding,
/// corner radius.
pub(super) const HINT_SIZE: f32 = 12.5;
pub(super) const HINT_OFFSET: Vec2 = Vec2::new(12.0, 18.0);
pub(super) const HINT_PAD: Vec2 = Vec2::new(8.0, 6.0);
pub(super) const HINT_RADIUS: f32 = 6.0;

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

// -- Lobby background --------------------------------------------------------
/// The lobby shows the same maps as the title screen, full-screen, more
/// blurred and moving more slowly, behind a floating dark panel that holds its
/// UI.
/// Blur sigma in screen points (converted like [`MAP_BLUR_PX`]).
pub(super) const LOBBY_BLUR_PX: f32 = 3.0;
/// Pan speed in the lobby (the title screen's is [`MAP_PAN_SPEED`]).
pub(super) const LOBBY_PAN_SPEED: f32 = 0.125;
/// A light backdrop scrim over the whole lobby map, so it stays a background.
pub(super) const LOBBY_SCRIM_ALPHA: f32 = 0.35;
/// The lobby's floating panel: its height as a fraction of the screen's, the
/// padding (points) between its edge and the lobby's UI column, and its
/// corner radius.
pub(super) const LOBBY_PANEL_H_REL: f32 = 0.88;
pub(super) const LOBBY_PANEL_PAD: f32 = 28.0;
pub(super) const LOBBY_PANEL_RADIUS: f32 = 6.0;
/// The soft glow around the panel that fades it into the map: its width as a
/// fraction of the screen height, and its falloff from the panel's edge
/// outward (fraction of the width, opacity of the panel colour 0..1).
pub(super) const LOBBY_GLOW_REL: f32 = 0.22;
pub(super) const LOBBY_GLOW_STOPS: [(f32, f32); 4] =
    [(0.0, 1.0), (0.2, 0.8), (0.5, 0.4), (1.0, 0.0)];
