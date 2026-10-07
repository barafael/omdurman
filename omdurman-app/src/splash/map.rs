//! The title screen's period maps: loaded once, then drawn every frame on
//! the GPU by the backdrop shader ([`super::backdrop`]), which projects,
//! blurs, dims and sepia-tones them. They show as a slow slideshow: one slide
//! at a time, each a map seen from its own random place and drifting slowly,
//! crossfading every so often into the next map at a new place
//! ([`MapShow`]).
//!
//! Each screen places the map with its own [`MapLayout`], whose docs give the
//! projection (it mirrors the reference design's CSS). This module holds the
//! show's clock and the geometry the shader is fed, plus (for the tests) the
//! per-point projection and compositing the shader computes per pixel; all
//! look-and-feel numbers live in [`super::params`].

use bevy::asset::{LoadState, RenderAssetUsages};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

use super::params::*;

/// Perspective divisor floor, as a fraction of the eye distance: a vertex that
/// would reach the eye plane is held just in front of it instead of flipping.
/// (Only the tests' [`project`] clamps; the shader inverts the projection
/// exactly, and the coverage test keeps every visible point well clear of the
/// plane.)
#[cfg(test)]
const NEAR_LIMIT: f32 = 0.05;

/// Most taps per axis of the shader's blur kernel (handed to `backdrop.wgsl`
/// as its `MAX_TAPS`).
pub(super) const MAX_TAPS: usize = 12;

/// The map images still loading, index-aligned with [`MAPS`]; an entry is
/// cleared once loaded (or failed), and the resource removed when all are.
#[derive(Resource)]
pub(super) struct SplashMapSources(Vec<Option<Handle<Image>>>);

/// The loaded map images (index-aligned with [`MAPS`], `None` until loaded or
/// if the image failed to load) and which of them is showing.
///
/// The show starts once the map it starts on has loaded (or, if that one
/// fails, once every load has resolved); the others join as they load.
#[derive(Resource)]
pub(crate) struct SplashMaps {
    pub(super) images: Vec<Option<MapImage>>,
    pub(super) show: MapShow,
    /// The showing slide's pan time (not wall-clock), shared by the title
    /// screen and the lobby so the map carries on from one to the other.
    pub(super) pan_time: f32,
    /// The pan time of the slide crossfading in, while one does.
    pub(super) incoming_pan_time: f32,
    /// Which of its map's views the showing slide starts from
    /// ([`SplashMapAsset::views`]), and the incoming slide's.
    pub(super) view: usize,
    pub(super) incoming_view: usize,
    /// The show has started.
    settled: bool,
    /// Dev affordance (`OMDURMAN_SPLASH_FREEZE=<map>,<pan time>`): hold one
    /// map at one pose, so screenshots of the screens compare across builds.
    freeze: Option<(usize, f32)>,
}

impl SplashMaps {
    /// Starts on a random map, from a random view, at a random point of the
    /// pan.
    pub(super) fn new() -> Self {
        use rand::RngExt;
        let current = rand::rng().random_range(0..MAPS.len());
        Self {
            images: MAPS.iter().map(|_| None).collect(),
            show: MapShow::new(current),
            pan_time: random_pan_start(),
            incoming_pan_time: 0.0,
            view: random_view(current),
            incoming_view: 0,
            settled: false,
            freeze: None,
        }
    }

    /// [`Self::new`], held still where `OMDURMAN_SPLASH_FREEZE` says
    /// (`<map index>,<pan seconds>`): the show starts on that map, whichever
    /// loads first, so screenshots compare across builds.
    pub(super) fn from_env() -> Self {
        let mut maps = Self::new();
        maps.freeze = std::env::var("OMDURMAN_SPLASH_FREEZE")
            .ok()
            .and_then(|spec| {
                let (index, time) = spec.split_once(',')?;
                Some((index.trim().parse().ok()?, time.trim().parse().ok()?))
            })
            .filter(|&(index, time): &(usize, f32)| index < MAPS.len() && time.is_finite());
        if let Some((index, _)) = maps.freeze {
            maps.show = MapShow::new(index);
        }
        maps
    }

    /// Whether map `index` may show: loaded, and the show started.
    pub(super) fn is_ready(&self, index: usize) -> bool {
        self.settled && self.images.get(index).is_some_and(Option::is_some)
    }

    /// The image of map `index`, if it may show.
    pub(super) fn image(&self, index: usize) -> Option<&MapImage> {
        if !self.settled {
            return None;
        }
        self.images.get(index).and_then(Option::as_ref)
    }

    /// Start the show, on another map if the random pick failed to load. A
    /// frozen show starts on its own map only, faded in (a failed load
    /// leaves the plain backdrop: still the same picture every run).
    fn settle(&mut self) {
        self.settled = true;
        if let Some((_, time)) = self.freeze {
            self.show.shown = MAP_FADE_IN_SECS;
            self.pan_time = time;
            self.view = 0;
            return;
        }
        let current = self.show.current;
        if !self.is_ready(current)
            && let Some(ready) = (0..MAPS.len()).find(|&index| self.is_ready(index))
        {
            self.show = MapShow::new(ready);
            self.view = random_view(ready);
        }
    }

    /// The view of the showing slide.
    pub(super) fn current_view(&self) -> MapView {
        view_of(self.show.current, self.view)
    }

    /// The view of the slide fading in (map `index`).
    pub(super) fn incoming_view(&self, index: usize) -> MapView {
        view_of(index, self.incoming_view)
    }

    /// Advance the show by `dt` seconds and, once a map shows, the pans by
    /// `dt · pan_speed`. A slide fading in starts at a random place, and keeps
    /// its pan when it takes over.
    pub(super) fn advance(&mut self, dt: f32, pan_speed: f32) {
        if self.freeze.is_some() {
            return;
        }
        let (current, fading) = (self.show.current, self.show.fade().is_some());
        if MAP_PAN && self.is_ready(current) {
            self.pan_time += dt * pan_speed;
            self.incoming_pan_time += dt * pan_speed;
        }
        let Self {
            images,
            show,
            settled,
            ..
        } = self;
        show.advance(dt, |index| {
            *settled && images.get(index).is_some_and(Option::is_some)
        });
        if !fading && let Some((index, _)) = self.show.fade() {
            self.incoming_pan_time = random_pan_start();
            self.incoming_view = random_view(index);
        }
        if self.show.current != current {
            self.pan_time = self.incoming_pan_time;
            self.view = self.incoming_view;
        }
    }

    /// Whether a map is on screen and moving, so the screen showing it must
    /// keep repainting.
    pub(crate) fn is_animating(&self) -> bool {
        self.is_ready(self.show.current) && self.freeze.is_none()
    }
}

/// A random view of map `index` for a slide to start from.
fn random_view(index: usize) -> usize {
    use rand::RngExt;
    let views = MAPS.get(index).map_or(1, |map| map.views.len().max(1));
    rand::rng().random_range(0..views)
}

/// View `slot` of map `index` (the whole map if there is none).
fn view_of(index: usize, slot: usize) -> MapView {
    MAPS.get(index)
        .and_then(|map| map.views.get(slot))
        .copied()
        .unwrap_or(WHOLE)
}

/// A random point of the pan's path for a slide to start at.
fn random_pan_start() -> f32 {
    use rand::RngExt;
    // (`max`: an empty range panics.)
    rand::rng().random_range(0.0..MAP_START_SPREAD_SECS.max(f32::EPSILON))
}

/// A loaded map image: its texture (its *encoded* sRGB values, sampled as
/// linear data so the shader blurs and tones them as the design's CSS
/// filters do) and its size in pixels.
#[derive(Clone, Debug)]
pub(crate) struct MapImage {
    pub(super) handle: Handle<Image>,
    pub(super) size: Vec2,
}

/// Which treatment of a map to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MapVariant {
    Title,
    Lobby,
}

impl MapVariant {
    /// The layout this variant is drawn with.
    pub(super) fn layout(self) -> &'static MapLayout {
        match self {
            MapVariant::Title => &TITLE_MAP,
            MapVariant::Lobby => &LOBBY_MAP,
        }
    }

    /// The blur sigma in screen points.
    pub(super) fn blur_px(self) -> f32 {
        match self {
            MapVariant::Title => MAP_BLUR_PX,
            MapVariant::Lobby => LOBBY_BLUR_PX,
        }
    }
}

/// Which map shows: the first fades in from the backdrop over
/// [`MAP_FADE_IN_SECS`]; each map then holds until the last
/// [`MAP_CROSSFADE_SECS`] of its [`MAP_SECS`], which crossfade into `target`,
/// which then becomes current. The clocks only run while the current map is
/// on screen, and a crossfade starts only once another map is ready; its
/// target is fixed when it starts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MapShow {
    pub(super) current: usize,
    clock: f32,
    target: Option<usize>,
    /// Seconds since the first map appeared, for its fade-in.
    shown: f32,
    /// How long each slide and its crossfade last.
    pub(super) timing: SlideTiming,
}

/// The slideshow's pace: the seconds each slide gets, its crossfade into the
/// next included, and the crossfade's length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SlideTiming {
    pub(super) slide_secs: f32,
    pub(super) crossfade_secs: f32,
}

impl Default for SlideTiming {
    fn default() -> Self {
        Self {
            slide_secs: MAP_SECS,
            crossfade_secs: MAP_CROSSFADE_SECS,
        }
    }
}

impl SlideTiming {
    /// Seconds a map holds before its crossfade starts.
    fn hold_secs(self) -> f32 {
        (self.slide_secs - self.crossfade_secs).max(0.0)
    }
}

impl MapShow {
    pub(super) fn new(current: usize) -> Self {
        Self {
            current,
            clock: 0.0,
            target: None,
            shown: 0.0,
            timing: SlideTiming::default(),
        }
    }

    /// Advance the show by `dt` seconds; `ready` tells which maps may show.
    pub(super) fn advance(&mut self, dt: f32, ready: impl Fn(usize) -> bool) {
        if !ready(self.current) {
            return;
        }
        self.shown += dt;
        self.clock += dt;
        let hold = self.timing.hold_secs();
        if self.clock < hold {
            return;
        }
        if self.target.is_none() {
            // The next ready map after the current one, in list order.
            self.target = (1..MAPS.len())
                .map(|step| (self.current + step) % MAPS.len())
                .find(|&index| ready(index));
            if self.target.is_none() {
                self.clock = hold; // wait for another map
                return;
            }
        }
        let slot = hold + self.timing.crossfade_secs;
        if self.clock >= slot
            && let Some(target) = self.target.take()
        {
            self.current = target;
            // At most to the next crossfade's start: a slide shortened under
            // a running clock (the tuning pane) moves on once, instead of
            // cascading through a slide a frame.
            self.clock = (self.clock - slot).min(hold);
        }
    }

    /// The map fading in and how far (0..1, eased), during a crossfade --
    /// not before it starts (a slide lengthened under a running crossfade
    /// holds again, without drawing the incoming map unseen).
    pub(super) fn fade(&self) -> Option<(usize, f32)> {
        let hold = self.timing.hold_secs();
        self.target.filter(|_| self.clock >= hold).map(|target| {
            let progress = (self.clock - hold) / self.timing.crossfade_secs.max(f32::EPSILON);
            (target, ease(progress))
        })
    }

    /// How far (0..1, eased) the first map has faded in from the backdrop.
    pub(super) fn fade_in(&self) -> f32 {
        ease(self.shown / MAP_FADE_IN_SECS.max(f32::EPSILON))
    }
}

/// The fades' curve over linear time `t` (clamped to 0..1): smootherstep,
/// whose rate and acceleration are both zero at the ends, so a slide eases
/// out of the one before and settles into place instead of starting and
/// stopping abruptly.
pub(super) fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (6.0 * t - 15.0) + 10.0)
}

/// Run the map show and pan on the screens that show the maps: the title
/// screen and menu, and (slower) the lobby. Clocks only move while one of them
/// is up; a stalled frame steps at most [`MAP_MAX_STEP_SECS`].
pub(super) fn animate_splash_maps(
    time: Res<Time>,
    screen: Option<Res<State<crate::Screen>>>,
    tuning: Res<super::tuning::SplashTuning>,
    mut maps: ResMut<SplashMaps>,
    mut activity: ResMut<crate::activity::Activity>,
) {
    let pan_speed = match screen.as_deref().map(State::get) {
        Some(crate::Screen::Title) => tuning.pan_speed,
        Some(crate::Screen::Lobby) => LOBBY_PAN_SPEED,
        _ => return,
    };
    maps.show.timing = tuning.timing;
    maps.advance(time.delta_secs().min(MAP_MAX_STEP_SECS), pan_speed);
    // The pan is slow continuous motion (the lobby's backdrop too): ambient
    // frames are enough.
    if maps.is_animating() {
        activity.keep_ambient();
    }
}

pub(super) fn load_splash_maps(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SplashMapSources(
        MAPS.iter()
            .map(|map| {
                Some(
                    asset_server
                        .load_builder()
                        .with_settings(|settings: &mut ImageLoaderSettings| {
                            // Encoded values, as data: blur and tone apply to them.
                            settings.is_srgb = false;
                            settings.sampler = ImageSampler::linear();
                            // Only the GPU needs the pixels.
                            settings.asset_usage = RenderAssetUsages::RENDER_WORLD;
                        })
                        .load(map.file),
                )
            })
            .collect(),
    ));
}

/// Record each map image as it finishes loading, and start the show once all
/// have loaded (or failed).
pub(super) fn prepare_splash_maps(
    mut commands: Commands,
    sources: Option<ResMut<SplashMapSources>>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    mut maps: ResMut<SplashMaps>,
) {
    let Some(mut sources) = sources else { return };
    for (index, slot) in sources.0.iter_mut().enumerate() {
        let Some(handle) = slot else { continue };
        match asset_server.load_state(&*handle) {
            LoadState::Failed(error) => {
                warn!(%error, file = MAPS[index].file, "splash: map image failed to load; skipping it");
                *slot = None;
            }
            LoadState::Loaded => {
                match images.get(&*handle) {
                    Some(image) => {
                        maps.images[index] = Some(MapImage {
                            handle: handle.clone(),
                            size: image.size_f32(),
                        });
                    }
                    None => warn!(
                        file = MAPS[index].file,
                        "splash: map image has no asset; skipping it"
                    ),
                }
                *slot = None;
            }
            _ => {}
        }
    }
    let resolved = sources.0.iter().all(Option::is_none);
    if resolved {
        commands.remove_resource::<SplashMapSources>();
    }
    // Start as soon as the first map is in (the show waits for the others
    // as they load), or, if it failed, on whichever loaded.
    let first_in = maps
        .images
        .get(maps.show.current)
        .is_some_and(Option::is_some);
    if !maps.settled && (first_in || resolved) {
        maps.settle();
    }
}

/// The map region: the right [`MAP_REGION_W`] of the screen, full height.
pub(super) fn map_region(screen: Rect) -> Rect {
    Rect::from_corners(
        Vec2::new(screen.max.x - MAP_REGION_W * screen.width(), screen.min.y),
        screen.max,
    )
}

/// The box `layout` places the map in: a box of its `region_aspect` scaled to
/// cover `region`, centred on it. At those proportions it is the region itself.
pub(super) fn map_box(region: Rect, layout: &MapLayout) -> Rect {
    let height = region.height().max(region.width() / layout.region_aspect);
    Rect::from_center_size(
        region.center(),
        Vec2::new(height * layout.region_aspect, height),
    )
}

/// The fixed geometry of a map in a region: the untransformed image square,
/// the transform origin and extra zoom, the perspective eye and distance, and
/// the box height every layout length is a fraction of.
#[derive(Clone, Copy, Debug)]
pub(super) struct MapFrame {
    pub(super) image: Rect,
    pub(super) origin: Vec2,
    pub(super) scale: f32,
    pub(super) eye: Vec2,
    pub(super) distance: f32,
    pub(super) box_height: f32,
}

pub(super) fn map_frame(region: Rect, layout: &MapLayout) -> MapFrame {
    let area = map_box(region, layout);
    let h = area.height();
    let side = layout.size_rel_h * h;
    let min = area.min + layout.offset_rel * h;
    let image = Rect::from_corners(min, min + Vec2::splat(side));
    MapFrame {
        image,
        origin: image.min + layout.transform_origin * side,
        scale: layout.scale,
        eye: area.min + layout.perspective_origin * area.size(),
        distance: layout.perspective_rel_h * h,
        box_height: h,
    }
}

/// The animated pose at pan time `t`: translation in points, tilt and
/// rotation in radians.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MapPose {
    pub(super) pan: Vec2,
    pub(super) tilt: f32,
    pub(super) rot: f32,
}

/// Broad Lissajous pan plus a slow tilt and rotation swing, for `layout` in a
/// map box `box_height` points tall.
pub(super) fn map_pose(t: f32, box_height: f32, layout: &MapLayout) -> MapPose {
    use std::f32::consts::TAU;
    let wave = |period: f32, phase: f32| (TAU * t / period + phase).sin();
    let (major, minor) = layout.pan_amp;
    let pan = box_height
        * Vec2::new(
            major * wave(MAP_PERIOD_X.0, 0.0) + minor * wave(MAP_PERIOD_X.1, MAP_PHASE_X_MINOR),
            major * wave(MAP_PERIOD_Y.0, MAP_PHASE_Y_MAJOR)
                + minor * (TAU * t / MAP_PERIOD_Y.1).cos(),
        );
    let tilt = (layout.tilt_deg
        + layout.tilt_swing_deg
        + layout.tilt_swing_deg * wave(MAP_PERIOD_TILT, MAP_PHASE_TILT))
    .max(0.0);
    let rot = layout.rotate_deg + layout.rotate_swing_deg * wave(MAP_PERIOD_ROT, 0.0);
    MapPose {
        pan,
        tilt: tilt.to_radians(),
        rot: rot.to_radians(),
    }
}

/// A point of the untransformed image square, scaled, rotated about the
/// transform origin, tilted (the top recedes, as under CSS `rotateX(+θ)`) and
/// translated: its flat position and its distance *away* from the viewer.
fn posed(frame: &MapFrame, pose: &MapPose, point: Vec2) -> (Vec2, f32) {
    let rotated = Vec2::from_angle(pose.rot).rotate((point - frame.origin) * frame.scale);
    let (sin_t, cos_t) = pose.tilt.sin_cos();
    // Points above the origin (y < 0) tip back.
    let depth = -rotated.y * sin_t;
    let flat = frame.origin + Vec2::new(rotated.x, rotated.y * cos_t) + pose.pan;
    (flat, depth)
}

/// Project a point of the untransformed image square to the screen: pose it
/// ([`posed`]), then apply perspective toward the eye.
#[cfg(test)]
pub(super) fn project(frame: &MapFrame, pose: &MapPose, point: Vec2) -> Vec2 {
    let (flat, depth) = posed(frame, pose, point);
    let k = frame.distance / (frame.distance + depth).max(frame.distance * NEAR_LIMIT);
    frame.eye + (flat - frame.eye) * k
}

/// The projection of the image square as a homography: it takes a point of the
/// square as fractions of its side, `(fx, fy, 1)`, to homogeneous screen
/// coordinates `(x·w, y·w, w)`. Exactly [`project`] wherever the square is in
/// front of the eye (the posing is affine in the point and the perspective
/// divides by an affine depth). The shader draws with its inverse.
pub(super) fn map_homography(frame: &MapFrame, pose: &MapPose) -> Mat3 {
    let at = |f: Vec2| {
        let (flat, depth) = posed(frame, pose, frame.image.min + f * frame.image.size());
        let w = (frame.distance + depth) / frame.distance;
        (frame.eye * w + flat - frame.eye).extend(w)
    };
    let origin = at(Vec2::ZERO);
    Mat3::from_cols(at(Vec2::X) - origin, at(Vec2::Y) - origin, origin)
}

/// The UV rect the square shows of an image `size` pixels large in `view`:
/// the shorter side over its zoom, centred on its point, held inside the
/// image.
pub(super) fn cover_uv(size: Vec2, view: MapView) -> Rect {
    let span = Vec2::splat(size.min_element() / view.zoom.max(1.0)) / size;
    let min = (view.centre - span / 2.0).clamp(Vec2::ZERO, Vec2::ONE - span);
    Rect::from_corners(min, min + span)
}

/// The blur sigma, in texture pixels, of a map `size` pixels large drawn as
/// `variant`, zoomed in `zoom` times, and blurred `blur_px` screen points
/// (normally [`MapVariant::blur_px`]) at the size the layout shows it in a
/// [`REFERENCE_HEIGHT`] box (the square spans the texture's shorter side over
/// the zoom and is shown `size_rel_h · scale` box heights wide).
pub(super) fn blur_sigma_texels(size: Vec2, variant: MapVariant, blur_px: f32, zoom: f32) -> f32 {
    let layout = variant.layout();
    let shown = layout.size_rel_h * layout.scale * REFERENCE_HEIGHT;
    blur_px * size.min_element() / zoom.max(1.0) / shown
}

/// One axis of a Gaussian blur of `sigma` texels as (offset in texels,
/// weight) taps for bilinear sampling. The kernel is the normalised one the
/// title screen's earlier CPU bake used (`image::imageops::blur`: OpenCV's
/// size for the sigma); neighbouring pairs of its texels merge into one tap
/// between them, so a radius of `r` texels costs `r + 1` taps (one more for
/// odd `r`). At most [`MAX_TAPS`]; an empty kernel (`sigma <= 0`) is the
/// single centre tap.
pub(super) fn blur_taps(sigma: f32) -> Vec<(f32, f32)> {
    if sigma <= 0.0 || !sigma.is_finite() {
        return vec![(0.0, 1.0)];
    }
    let size = ((((sigma - 0.8) / 0.3) + 1.0) * 2.0 + 1.0).max(3.0) as usize;
    let size = if size.is_multiple_of(2) {
        size + 1
    } else {
        size
    };
    // 1 + 2·ceil(r/2) taps must fit in MAX_TAPS.
    let radius = (size / 2).min(2 * ((MAX_TAPS - 1) / 2));
    let weight = |x: usize| (-0.5 * (x as f32 / sigma).powi(2)).exp();
    let total: f32 = weight(0) + 2.0 * (1..=radius).map(weight).sum::<f32>();
    let mut taps = vec![(0.0, weight(0) / total)];
    for i in (1..=radius).step_by(2) {
        let (a, b) = (weight(i), if i < radius { weight(i + 1) } else { 0.0 });
        let w = a + b;
        let offset = (i as f32 * a + (i + 1) as f32 * b) / w;
        taps.push((offset, w / total));
        taps.push((-offset, w / total));
    }
    taps
}

/// Opacity to draw the map at under a backdrop layer of opacity
/// `fade · composite`, so the pair composites like the opaque composition
/// (map faded into the backdrop by `fade`) seen through one group opacity
/// `composite` -- the opacity of the returning-via-M overlay. Drawing the map
/// at plain `composite` instead would let less of the board through behind the
/// map than behind the column, leaving a seam at the region edge. (The shader
/// computes the same, per pixel.)
#[cfg(test)]
pub(super) fn map_alpha(fade: f32, composite: f32) -> f32 {
    let covered = 1.0 - fade * composite;
    if covered <= f32::EPSILON {
        0.0 // an opaque backdrop hides the map entirely
    } else {
        composite * (1.0 - fade) / covered
    }
}

/// Opacities for map `a` (drawn first) and map `b` (drawn over it) while `b`
/// crossfades in by `progress` (0..1), both under the group opacity `alpha`:
/// the pair composites like `alpha · lerp(a, b, progress)`, so what shows
/// through behind the maps stays exactly `1 - alpha` during the fade. (The
/// shader computes the same, per pixel.)
#[cfg(test)]
pub(super) fn crossfade_alphas(alpha: f32, progress: f32) -> (f32, f32) {
    let over = alpha * progress;
    let under = if 1.0 - over <= f32::EPSILON {
        0.0 // the incoming map alone covers it
    } else {
        alpha * (1.0 - progress) / (1.0 - over)
    };
    (under, over)
}

/// Piecewise-linear value of `stops` ((position, value) pairs, ascending) at
/// `at`, held flat beyond the first and last stop. (The shader's `stops_at`
/// is the same.)
#[cfg(test)]
pub(super) fn stops_at(stops: &[(f32, f32)], at: f32) -> f32 {
    let Some(&(first_at, first)) = stops.first() else {
        return 0.0;
    };
    if at <= first_at {
        return first;
    }
    for pair in stops.windows(2) {
        let [(a_at, a), (b_at, b)] = [pair[0], pair[1]];
        if at <= b_at {
            let t = if b_at > a_at {
                (at - a_at) / (b_at - a_at)
            } else {
                1.0
            };
            return a + (b - a) * t;
        }
    }
    stops.last().map_or(0.0, |&(_, last)| last)
}

/// The drawn opacity of a dark overlay the design gives CSS opacity `alpha`
/// (see [`CSS_BLEND_GAMMA`]).
#[cfg(test)]
pub(super) fn css_overlay_alpha(alpha: f32) -> f32 {
    1.0 - (1.0 - alpha.clamp(0.0, 1.0)).powf(CSS_BLEND_GAMMA)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec2, b: Vec2) -> bool {
        (a - b).length() < 1e-3
    }

    fn rect(min: Vec2, size: Vec2) -> Rect {
        Rect::from_corners(min, min + size)
    }

    #[test]
    fn a_flat_unscaled_pose_is_the_identity() {
        let mut frame = map_frame(
            rect(Vec2::new(380.0, 0.0), Vec2::new(900.0, 800.0)),
            &TITLE_MAP,
        );
        frame.scale = 1.0;
        let rest = MapPose {
            pan: Vec2::ZERO,
            tilt: 0.0,
            rot: 0.0,
        };
        for p in [
            frame.image.min,
            frame.image.max,
            frame.origin,
            Vec2::new(12.0, 34.0),
        ] {
            assert!(close(project(&frame, &rest, p), p));
        }
    }

    #[test]
    fn the_top_of_a_tilted_map_recedes() {
        let mut frame = map_frame(rect(Vec2::ZERO, Vec2::new(900.0, 800.0)), &TITLE_MAP);
        frame.scale = 1.0;
        let tilted = MapPose {
            pan: Vec2::ZERO,
            tilt: 30f32.to_radians(),
            rot: 0.0,
        };
        let edge = |y: f32| {
            let a = project(&frame, &tilted, Vec2::new(frame.image.min.x, y));
            let b = project(&frame, &tilted, Vec2::new(frame.image.max.x, y));
            (b - a).length()
        };
        assert!(edge(frame.image.min.y) < frame.image.width());
        assert!(edge(frame.image.max.y) > frame.image.width());
    }

    // The shader draws the map through the inverse of `map_homography`: it
    // must be the per-point projection, and invert back to the square.
    #[test]
    fn the_homography_is_the_projection() {
        for (layout, region) in [
            (
                &TITLE_MAP,
                rect(Vec2::new(608.0, 0.0), Vec2::new(1440.0, 1136.0)),
            ),
            (&LOBBY_MAP, rect(Vec2::ZERO, Vec2::new(2048.0, 1136.0))),
        ] {
            let frame = map_frame(region, layout);
            for t in [0.0, 17.5, 133.0] {
                let pose = map_pose(t, frame.box_height, layout);
                let h = map_homography(&frame, &pose);
                let inverse = h.inverse();
                for f in [
                    Vec2::ZERO,
                    Vec2::ONE,
                    Vec2::new(0.25, 0.8),
                    Vec2::new(0.6, 0.1),
                ] {
                    let point = frame.image.min + f * frame.image.size();
                    let q = h * f.extend(1.0);
                    let screen = q.truncate() / q.z;
                    let expected = project(&frame, &pose, point);
                    assert!((screen - expected).length() < 1e-2, "{screen} {expected}");
                    let back = inverse * screen.extend(1.0);
                    assert!(back.z > 0.0, "in front of the eye");
                    assert!((back.truncate() / back.z - f).length() < 1e-4);
                }
            }
        }
    }

    #[test]
    fn the_pose_follows_the_layout() {
        for layout in [&TITLE_MAP, &LOBBY_MAP] {
            let pose = map_pose(0.0, 800.0, layout);
            let tilt = layout.tilt_deg + layout.tilt_swing_deg * (1.0 + MAP_PHASE_TILT.sin());
            assert!((pose.tilt - tilt.max(0.0).to_radians()).abs() < 1e-5);
            assert!((pose.rot - layout.rotate_deg.to_radians()).abs() < 1e-5);
            let (major, minor) = layout.pan_amp;
            let pan_y = 800.0 * (major * MAP_PHASE_Y_MAJOR.sin() + minor);
            assert!((pose.pan.y - pan_y).abs() < 1e-3);
        }
    }

    #[test]
    fn a_portrait_image_covers_the_square_from_its_middle() {
        let uv = cover_uv(Vec2::new(1583.0, 2820.0), WHOLE);
        let aspect = 1583.0 / 2820.0;
        assert!((uv.width() - 1.0).abs() < 1e-6);
        assert!((uv.height() - aspect).abs() < 1e-6);
        assert!((uv.min.y - (1.0 - aspect) * 0.5).abs() < 1e-6);
        assert!(uv.min.y >= 0.0 && uv.max.y <= 1.0);
    }

    // The taps are the Gaussian the CPU bake used: normalised, centred, and
    // with its spread -- for every map the screens can show.
    #[test]
    fn the_blur_taps_are_the_bakes_gaussian() {
        assert_eq!(blur_taps(0.0), vec![(0.0, 1.0)]);
        for size in [Vec2::new(1600.0, 1200.0), Vec2::splat(1600.0)] {
            for variant in [MapVariant::Title, MapVariant::Lobby] {
                let sigma = blur_sigma_texels(size, variant, variant.blur_px(), 1.0);
                let taps = blur_taps(sigma);
                assert!(taps.len() <= MAX_TAPS && taps.len() % 2 == 1, "{taps:?}");
                let total: f32 = taps.iter().map(|&(_, w)| w).sum();
                assert!((total - 1.0).abs() < 1e-5);
                let mean: f32 = taps.iter().map(|&(x, w)| x * w).sum();
                assert!(mean.abs() < 1e-5);
                // Merging two texels into one tap between them keeps the
                // weight and the mean, and loses a little of the variance.
                let variance: f32 = taps.iter().map(|&(x, w)| x * x * w).sum();
                assert!(
                    variance > 0.6 * sigma * sigma && variance < 1.2 * sigma * sigma,
                    "{variant:?} {sigma}: {variance}"
                );
            }
        }
    }

    #[test]
    fn stops_interpolate_and_hold_at_the_ends() {
        let stops = [(0.2, 1.0), (0.6, 0.0)];
        assert_eq!(stops_at(&stops, 0.0), 1.0);
        assert!((stops_at(&stops, 0.4) - 0.5).abs() < 1e-6);
        assert_eq!(stops_at(&stops, 0.9), 0.0);
        assert_eq!(stops_at(&[], 0.5), 0.0);
    }

    // Map at `map_alpha`, then the backdrop at `fade · composite` on top: the
    // board behind shows through by exactly `1 - composite` everywhere, and
    // the map/backdrop mix is the opaque composition's.
    #[test]
    fn the_folded_fade_composites_as_one_group() {
        for composite in [1.0, 200.0 / 255.0, 0.5] {
            for fade in [0.0, 0.3, 0.8, 1.0] {
                let map = map_alpha(fade, composite);
                let backdrop = fade * composite;
                let board = (1.0 - backdrop) * (1.0 - map);
                assert!(
                    (board - (1.0 - composite)).abs() < 1e-5,
                    "{composite} {fade}"
                );
                let map_share = (1.0 - backdrop) * map;
                assert!((map_share - composite * (1.0 - fade)).abs() < 1e-5);
            }
        }
    }

    /// How far (points) the projected image square reaches past `region`'s
    /// corners at pan time `t`: the smallest distance of a region corner
    /// inside the square's projected edges, negative if a map edge shows.
    /// Exact while every corner stays in front of the eye, because a
    /// perspective projection keeps the square's edges straight.
    fn coverage_margin(region: Rect, t: f32, layout: &MapLayout) -> f32 {
        let frame = map_frame(region, layout);
        let pose = map_pose(t, frame.box_height, layout);
        let r = frame.image;
        let corners = [
            r.min,
            Vec2::new(r.max.x, r.min.y),
            r.max,
            Vec2::new(r.min.x, r.max.y),
        ];
        for corner in corners {
            // In front of the eye, clear of the `NEAR_LIMIT` clamp.
            let (_, depth) = posed(&frame, &pose, corner);
            assert!(frame.distance + depth > frame.distance * NEAR_LIMIT);
        }
        let quad = corners.map(|p| project(&frame, &pose, p));
        let orientation = (0..4)
            .map(|i| quad[i].perp_dot(quad[(i + 1) % 4]))
            .sum::<f32>()
            .signum();
        [
            region.min,
            Vec2::new(region.max.x, region.min.y),
            region.max,
            Vec2::new(region.min.x, region.max.y),
        ]
        .into_iter()
        .flat_map(|p| {
            (0..4).map(move |i| {
                let (a, b) = (quad[i], quad[(i + 1) % 4]);
                let edge = b - a;
                orientation * edge.perp_dot(p - a) / edge.length()
            })
        })
        .fold(f32::INFINITY, f32::min)
    }

    // The handoff's acceptance: "No map edge is ever visible during a 10-minute
    // run at defaults" -- for the title screen at the design's 1280×800 and at
    // wider, taller and narrow (full-bleed) windows, and for the lobby's
    // full-screen map at the same window sizes, from any starting point of the
    // pan. 600 s of pan time past the latest start is 20 minutes at the title
    // screen's speed (40 in the lobby).
    #[test]
    fn no_map_edge_shows_in_a_ten_minute_run() {
        let wide = [
            (1280.0, 800.0),
            (1440.0, 900.0),
            (1920.0, 1080.0),
            (1920.0, 1017.0),
            (2560.0, 1600.0),
            (3440.0, 1440.0),
        ];
        let narrow = [(800.0, 800.0), (600.0, 900.0), (850.0, 1100.0)];
        let screen = |(w, h): (f32, f32)| rect(Vec2::ZERO, Vec2::new(w, h));
        let regions = wide
            .map(|size| map_region(screen(size)))
            .into_iter()
            .chain(narrow.map(screen));
        let screens = wide.into_iter().chain(narrow).map(screen);
        let cases = regions
            .map(|region| (region, &TITLE_MAP))
            .chain(screens.map(|screen| (screen, &LOBBY_MAP)));
        for (region, layout) in cases {
            let samples = ((MAP_START_SPREAD_SECS + 600.0) / 0.05) as usize;
            let (worst, at) = (0..samples)
                .map(|i| i as f32 * 0.05)
                .map(|t| (coverage_margin(region, t, layout), t))
                .fold((f32::INFINITY, 0.0), |a, b| if b.0 < a.0 { b } else { a });
            assert!(
                worst > 0.0,
                "map edge visible in region {region:?} at pan time {at}: {worst:.1} points"
            );
        }
    }

    #[test]
    fn the_crossfade_composites_as_one_group() {
        for alpha in [1.0, 200.0 / 255.0, 0.4] {
            for progress in [0.0, 0.25, 0.5, 1.0] {
                let (under, over) = crossfade_alphas(alpha, progress);
                let behind = (1.0 - under) * (1.0 - over);
                assert!((behind - (1.0 - alpha)).abs() < 1e-5, "{alpha} {progress}");
                assert!((under * (1.0 - over) - alpha * (1.0 - progress)).abs() < 1e-5);
                assert!((over - alpha * progress).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn each_map_gets_its_seconds_crossfade_included() {
        let all = |_: usize| true;
        let mut show = MapShow::new(MAPS.len() - 1);
        show.advance(MAP_SECS - MAP_CROSSFADE_SECS - 1.0, all);
        assert_eq!(show.fade(), None);
        show.advance(1.0 + MAP_CROSSFADE_SECS / 2.0, all);
        let (target, progress) = show.fade().expect("fading");
        assert_eq!(target, 0, "wraps to the first map");
        assert!((progress - 0.5).abs() < 1e-4);
        // The next map takes over exactly `MAP_SECS` after the first appeared.
        show.advance(MAP_CROSSFADE_SECS / 2.0, all);
        assert_eq!((show.current, show.fade()), (0, None));
    }

    // The curve runs 0 to 1, symmetric about its middle, never backwards,
    // and flat at both ends.
    #[test]
    fn the_fades_ease_in_and_out() {
        assert_eq!(
            (ease(-1.0), ease(0.0), ease(1.0), ease(2.0)),
            (0.0, 0.0, 1.0, 1.0)
        );
        assert!((ease(0.5) - 0.5).abs() < 1e-6);
        let samples: Vec<f32> = (0..=100).map(|i| ease(i as f32 / 100.0)).collect();
        assert!(samples.windows(2).all(|w| w[1] >= w[0]));
        for i in 1..50 {
            let t = i as f32 / 100.0;
            assert!((ease(t) + ease(1.0 - t) - 1.0).abs() < 1e-5);
        }
        let h = 1e-3;
        assert!(ease(h) / h < 1e-3 && (1.0 - ease(1.0 - h)) / h < 1e-3);
    }

    #[test]
    fn the_first_map_fades_in() {
        let mut show = MapShow::new(0);
        assert_eq!(show.fade_in(), 0.0);
        show.advance(MAP_FADE_IN_SECS / 2.0, |_| true);
        assert!((show.fade_in() - 0.5).abs() < 1e-4);
        show.advance(MAP_FADE_IN_SECS, |_| true);
        assert_eq!(show.fade_in(), 1.0);
    }

    #[test]
    fn the_show_waits_for_its_maps() {
        let mut show = MapShow::new(0);
        // Nothing ready yet: the clocks do not run.
        show.advance(MAP_SECS * 3.0, |_| false);
        assert_eq!(show, MapShow::new(0));
        // Only the current map is ready: it holds, and a fade starts the
        // moment another map is.
        show.advance(MAP_SECS * 3.0, |i| i == 0);
        assert_eq!(show.fade(), None);
        show.advance(0.0, |_| true);
        assert_eq!(show.fade(), Some((1, 0.0)));
    }

    // Every view is a zoom in on a point of its map, and its square stays
    // inside the image whatever the map's shape.
    #[test]
    fn every_view_shows_only_its_map() {
        for map in &MAPS {
            assert_eq!(map.views.first(), Some(&WHOLE), "{}", map.file);
            for view in map.views {
                assert!(view.zoom >= 1.0 && view.zoom <= 2.0, "{}", map.file);
                assert!(view.centre.cmpge(Vec2::ZERO).all() && view.centre.cmple(Vec2::ONE).all());
                for size in [Vec2::new(1600.0, 1200.0), Vec2::new(1200.0, 1253.0)] {
                    let uv = cover_uv(size, *view);
                    assert!(uv.min.cmpge(Vec2::ZERO).all(), "{} {view:?}", map.file);
                    assert!(
                        uv.max.cmple(Vec2::splat(1.0 + 1e-6)).all(),
                        "{} {view:?}",
                        map.file
                    );
                    // A square on screen: as many pixels across as down.
                    let pixels = uv.size() * size;
                    assert!((pixels.x - pixels.y).abs() < 1e-2);
                }
            }
        }
    }

    // A zoomed-in view blurs fewer texels, the same on screen.
    #[test]
    fn the_blur_follows_the_zoom() {
        let size = Vec2::new(1600.0, 1200.0);
        let whole = blur_sigma_texels(size, MapVariant::Title, MAP_BLUR_PX, 1.0);
        let detail = blur_sigma_texels(size, MapVariant::Title, MAP_BLUR_PX, 1.3);
        assert!((whole / detail - 1.3).abs() < 1e-5);
    }

    #[test]
    fn the_show_waits_for_its_first_map() {
        let mut maps = SplashMaps::new();
        maps.show = MapShow::new(0);
        // Only map 1 loaded, map 0 (the random pick) still missing: the
        // show has not started.
        maps.images[1] = Some(MapImage {
            handle: Handle::default(),
            size: Vec2::ONE,
        });
        assert!(!maps.is_ready(1) && maps.image(1).is_none());
        let start = maps.pan_time;
        assert!((0.0..MAP_START_SPREAD_SECS).contains(&start));
        maps.advance(MAP_SECS, MAP_PAN_SPEED);
        assert_eq!(maps.pan_time, start);
        assert_eq!(maps.show, MapShow::new(0));
        // Settled with the pick failed: the show starts on a map that loaded.
        maps.settle();
        assert_eq!(maps.show.current, 1);
        assert!(maps.image(1).is_some());
    }

    // Each slide fading in starts somewhere new, drifts through the
    // crossfade, and keeps its place when it takes over.
    #[test]
    fn each_slide_starts_at_its_own_place() {
        let mut maps = SplashMaps::new();
        maps.images = MAPS
            .iter()
            .map(|_| {
                Some(MapImage {
                    handle: Handle::default(),
                    size: Vec2::ONE,
                })
            })
            .collect();
        maps.settle();
        let step = 0.05;
        while maps.show.fade().is_none() {
            maps.advance(step, 1.0);
        }
        let start = maps.incoming_pan_time;
        assert!((0.0..MAP_START_SPREAD_SECS).contains(&start));
        let current = maps.show.current;
        while maps.show.current == current {
            maps.advance(step, 1.0);
        }
        // (The steps land the switch within one of the crossfade's end.)
        let drift = maps.pan_time - start;
        assert!(
            (0.0..=MAP_CROSSFADE_SECS + 2.0 * step).contains(&drift),
            "{drift}"
        );
    }

    // The title screen's fade is solid where the map region starts, so the
    // map's left edge never shows.
    #[test]
    fn the_fade_hides_the_map_regions_left_edge() {
        assert_eq!(
            css_overlay_alpha(stops_at(&FADE_STOPS, 1.0 - MAP_REGION_W)),
            1.0
        );
    }
}
