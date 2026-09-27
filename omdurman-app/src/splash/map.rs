//! The title screen's period maps: each loaded once, blurred, dimmed and
//! sepia-toned once, then drawn every frame as a CPU-projected perspective
//! mesh (egui has no 3D transforms and no blur). One map shows at a time; every
//! so often it crossfades, in place, into the next ([`MapShow`]).
//!
//! The projection mirrors the CSS of the reference design, `translate(pan)
//! rotateX(tilt) rotateZ(rot) scale(MAP_SCALE)` about
//! [`MAP_TRANSFORM_ORIGIN`], under a parent `perspective` whose eye sits at
//! [`MAP_PERSPECTIVE_ORIGIN`]. All look-and-feel numbers live in
//! [`super::params`].

use bevy::asset::{LoadState, RenderAssetUsages};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiTextureHandle, EguiUserTextures, egui};

use super::params::*;

/// Region height assumed for the blur radius when there is no window to
/// measure (the design's reference size is 1280×800).
const REFERENCE_HEIGHT: f32 = 800.0;

/// Perspective divisor floor, as a fraction of the eye distance: a vertex that
/// would reach the eye plane is held just in front of it instead of flipping.
const NEAR_LIMIT: f32 = 0.05;

/// The map images still loading, index-aligned with [`MAPS`]; an entry is
/// cleared once baked (or failed), and the resource removed when all are.
#[derive(Resource)]
pub(super) struct SplashMapSources(Vec<Option<Handle<Image>>>);

/// The baked map textures (index-aligned with [`MAPS`], `None` until ready or
/// if the image failed to load) and which of them is showing.
///
/// No map shows until every image has been baked (or has failed): the bakes
/// stall their frames, and they all happen while the screen is still static.
#[derive(Resource)]
pub(crate) struct SplashMaps {
    pub(super) textures: Vec<Option<SplashMap>>,
    pub(super) show: MapShow,
    /// Every image baked or failed.
    settled: bool,
}

impl SplashMaps {
    /// Starts on a random map.
    pub(super) fn new() -> Self {
        use rand::RngExt;
        Self {
            textures: MAPS.iter().map(|_| None).collect(),
            show: MapShow::new(rand::rng().random_range(0..MAPS.len())),
            settled: false,
        }
    }

    /// Whether map `index` may show: baked, and the bakes are all done.
    pub(super) fn is_ready(&self, index: usize) -> bool {
        self.settled && self.textures.get(index).is_some_and(Option::is_some)
    }

    /// The texture of map `index`, if it may show.
    pub(super) fn texture(&self, index: usize) -> Option<&SplashMap> {
        if !self.settled {
            return None;
        }
        self.textures.get(index).and_then(Option::as_ref)
    }

    /// All bakes are done: start the show, on another map if the random pick
    /// failed to load.
    fn settle(&mut self) {
        self.settled = true;
        let current = self.show.current;
        if !self.is_ready(current)
            && let Some(ready) = (0..MAPS.len()).find(|&index| self.is_ready(index))
        {
            self.show = MapShow::new(ready);
        }
    }

    /// Advance the show by `dt` seconds.
    pub(super) fn advance(&mut self, dt: f32) {
        let Self {
            textures,
            show,
            settled,
        } = self;
        show.advance(dt, |index| {
            *settled && textures.get(index).is_some_and(Option::is_some)
        });
    }
}

pub(crate) struct SplashMap {
    pub(super) id: egui::TextureId,
    /// Texture size in pixels.
    pub(super) size: egui::Vec2,
    /// Keeps the baked image alive.
    _image: Handle<Image>,
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
}

/// Seconds a map holds before its crossfade starts.
fn hold_secs() -> f32 {
    (MAP_SECS - MAP_CROSSFADE_SECS).max(0.0)
}

impl MapShow {
    pub(super) fn new(current: usize) -> Self {
        Self {
            current,
            clock: 0.0,
            target: None,
            shown: 0.0,
        }
    }

    /// Advance the show by `dt` seconds; `ready` tells which maps may show.
    pub(super) fn advance(&mut self, dt: f32, ready: impl Fn(usize) -> bool) {
        if !ready(self.current) {
            return;
        }
        self.shown += dt;
        self.clock += dt;
        if self.clock < hold_secs() {
            return;
        }
        if self.target.is_none() {
            // The next ready map after the current one, in list order.
            self.target = (1..MAPS.len())
                .map(|step| (self.current + step) % MAPS.len())
                .find(|&index| ready(index));
            if self.target.is_none() {
                self.clock = hold_secs(); // wait for another map
                return;
            }
        }
        let slot = hold_secs() + MAP_CROSSFADE_SECS;
        if self.clock >= slot
            && let Some(target) = self.target.take()
        {
            self.current = target;
            self.clock -= slot;
        }
    }

    /// The map fading in and how far (0..1), during a crossfade.
    pub(super) fn fade(&self) -> Option<(usize, f32)> {
        self.target.map(|target| {
            let progress = (self.clock - hold_secs()) / MAP_CROSSFADE_SECS.max(f32::EPSILON);
            (target, progress.clamp(0.0, 1.0))
        })
    }

    /// How far (0..1) the first map has faded in from the backdrop.
    pub(super) fn fade_in(&self) -> f32 {
        (self.shown / MAP_FADE_IN_SECS.max(f32::EPSILON)).clamp(0.0, 1.0)
    }
}

pub(super) fn load_splash_maps(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SplashMapSources(
        MAPS.iter()
            .map(|map| Some(asset_server.load(map.file)))
            .collect(),
    ));
}

/// Once a map image has loaded: blur, dim and sepia-tone it once and register
/// the result with egui -- at most one map per frame, so the one-off bakes
/// don't pile into a single hitch. Runs in `Update`, outside the egui pass
/// (the same split as the chart scans, to keep `EguiUserTextures` out of the
/// context pass).
pub(super) fn prepare_splash_maps(
    mut commands: Commands,
    sources: Option<ResMut<SplashMapSources>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut user_textures: ResMut<EguiUserTextures>,
    mut maps: ResMut<SplashMaps>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(mut sources) = sources else { return };
    for (index, slot) in sources.0.iter_mut().enumerate() {
        let Some(handle) = slot else { continue };
        if let LoadState::Failed(error) = asset_server.load_state(&*handle) {
            warn!(%error, file = MAPS[index].file, "splash: map image failed to load; skipping it");
            *slot = None;
            continue;
        }
        // Take the decoded image out of the asset store: only the baked copy
        // is kept, so the original's pixels are freed.
        let Some(image) = images.remove(&*handle) else {
            continue;
        };
        *slot = None;
        let box_height = window.single().map_or(REFERENCE_HEIGHT, |w| {
            let screen =
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w.width(), w.height()));
            map_box(map_region(screen)).height()
        });
        let started = bevy::platform::time::Instant::now();
        match bake_map(image, box_height) {
            Some(baked) => {
                debug!(
                    file = MAPS[index].file,
                    ms = started.elapsed().as_millis(),
                    "splash: baked map"
                );
                let size = egui::vec2(baked.width() as f32, baked.height() as f32);
                let handle = images.add(baked);
                let id = user_textures.add_image(EguiTextureHandle::Strong(handle.clone()));
                maps.textures[index] = Some(SplashMap {
                    id,
                    size,
                    _image: handle,
                });
            }
            None => warn!(
                file = MAPS[index].file,
                "splash: map image has no readable pixels; skipping it"
            ),
        }
        break; // one bake per frame
    }
    if sources.0.iter().all(Option::is_none) {
        commands.remove_resource::<SplashMapSources>();
        maps.settle();
    }
}

/// Blur (sigma [`MAP_BLUR_PX`] screen points, converted to texture pixels for
/// a map box `box_height` points tall), dim and sepia-tone the map, in the
/// design's CSS filter order.
fn bake_map(image: Image, box_height: f32) -> Option<Image> {
    let rgb = image.try_into_dynamic().ok()?.to_rgb8();
    let (width, height) = rgb.dimensions();
    let rgb = if MAP_BLUR_PX > 0.0 {
        // The square spans the texture's shorter side (cover fit) and is
        // shown `MAP_SIZE_REL_H · MAP_SCALE` box heights wide.
        let shown = MAP_SIZE_REL_H * MAP_SCALE * box_height;
        image::imageops::blur(&rgb, MAP_BLUR_PX * width.min(height) as f32 / shown)
    } else {
        rgb
    };
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for pixel in rgb.pixels() {
        let [r, g, b] = tone(pixel.0);
        rgba.extend_from_slice(&[r, g, b, u8::MAX]);
    }
    let mut baked = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    baked.sampler = ImageSampler::linear();
    Some(baked)
}

/// Dim a pixel to [`MAP_BRIGHTNESS`], then mix it [`MAP_SEPIA`] of the way
/// toward its [`SEPIA_MATRIX`] tone (CSS `brightness() sepia()`; the order
/// matters where the sepia matrix clips at white).
fn tone(rgb: [u8; 3]) -> [u8; 3] {
    let dimmed = rgb.map(|c| f32::from(c) * MAP_BRIGHTNESS);
    let [r, g, b] = dimmed;
    std::array::from_fn(|i| {
        let [cr, cg, cb] = SEPIA_MATRIX[i];
        let toned = (cr * r + cg * g + cb * b).min(255.0);
        (dimmed[i] + (toned - dimmed[i]) * MAP_SEPIA).round() as u8
    })
}

/// The map region: the right [`MAP_REGION_W`] of the screen, full height.
pub(super) fn map_region(screen: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::pos2(screen.right() - MAP_REGION_W * screen.width(), screen.top()),
        screen.max,
    )
}

/// The box the map is laid out in: the design's map area
/// ([`MAP_REGION_ASPECT`]) scaled to cover `region`, centred on it. At the
/// design's own proportions it is the region itself.
pub(super) fn map_box(region: egui::Rect) -> egui::Rect {
    let height = region.height().max(region.width() / MAP_REGION_ASPECT);
    egui::Rect::from_center_size(
        region.center(),
        egui::vec2(height * MAP_REGION_ASPECT, height),
    )
}

/// The fixed geometry of the map in a region: the untransformed image square,
/// the transform origin, the perspective eye and distance, and the box height
/// every `_REL` length is a fraction of.
#[derive(Clone, Copy, Debug)]
pub(super) struct MapFrame {
    pub(super) image: egui::Rect,
    pub(super) origin: egui::Pos2,
    pub(super) eye: egui::Pos2,
    pub(super) distance: f32,
    pub(super) box_height: f32,
}

pub(super) fn map_frame(region: egui::Rect) -> MapFrame {
    let layout = map_box(region);
    let h = layout.height();
    let side = MAP_SIZE_REL_H * h;
    let image = egui::Rect::from_min_size(layout.min + MAP_OFFSET_REL * h, egui::Vec2::splat(side));
    MapFrame {
        image,
        origin: image.min + MAP_TRANSFORM_ORIGIN * side,
        eye: layout.min + MAP_PERSPECTIVE_ORIGIN * layout.size(),
        distance: MAP_PERSPECTIVE_REL_H * h,
        box_height: h,
    }
}

/// The animated pose at pan time `t`: translation in points, tilt and
/// rotation in radians.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MapPose {
    pub(super) pan: egui::Vec2,
    pub(super) tilt: f32,
    pub(super) rot: f32,
}

/// Broad Lissajous pan plus a slow tilt and rotation swing, for a map box
/// `box_height` points tall.
pub(super) fn map_pose(t: f32, box_height: f32) -> MapPose {
    use std::f32::consts::TAU;
    let wave = |period: f32, phase: f32| (TAU * t / period + phase).sin();
    let pan = box_height
        * egui::vec2(
            MAP_PAN_AMP_MAJOR * wave(MAP_PERIOD_X.0, 0.0)
                + MAP_PAN_AMP_MINOR * wave(MAP_PERIOD_X.1, MAP_PHASE_X_MINOR),
            MAP_PAN_AMP_MAJOR * wave(MAP_PERIOD_Y.0, MAP_PHASE_Y_MAJOR)
                + MAP_PAN_AMP_MINOR * (TAU * t / MAP_PERIOD_Y.1).cos(),
        );
    let tilt = (MAP_TILT_DEG
        + MAP_TILT_SWING_DEG
        + MAP_TILT_SWING_DEG * wave(MAP_PERIOD_TILT, MAP_PHASE_TILT))
    .max(0.0);
    let rot = MAP_ROTATE_DEG + MAP_ROTATE_SWING_DEG * wave(MAP_PERIOD_ROT, 0.0);
    MapPose {
        pan,
        tilt: tilt.to_radians(),
        rot: rot.to_radians(),
    }
}

/// Project a point of the untransformed image square to the screen: scale,
/// rotate about the transform origin, tilt (the top recedes, as under CSS
/// `rotateX(+θ)`), translate, then apply perspective toward the eye.
pub(super) fn project(
    frame: &MapFrame,
    pose: &MapPose,
    scale: f32,
    point: egui::Pos2,
) -> egui::Pos2 {
    let rel = (point - frame.origin) * scale;
    let (sin_r, cos_r) = pose.rot.sin_cos();
    let rotated = egui::vec2(rel.x * cos_r - rel.y * sin_r, rel.x * sin_r + rel.y * cos_r);
    let (sin_t, cos_t) = pose.tilt.sin_cos();
    // Distance *away* from the viewer: points above the origin (y < 0) tip back.
    let depth = -rotated.y * sin_t;
    let flat = frame.origin + egui::vec2(rotated.x, rotated.y * cos_t) + pose.pan;
    let k = frame.distance / (frame.distance + depth).max(frame.distance * NEAR_LIMIT);
    frame.eye + (flat - frame.eye) * k
}

/// The UV rect a square shows of an image `size` pixels large when it covers
/// the square edge to edge, cropped around [`MAP_FOCUS`].
pub(super) fn cover_uv(size: egui::Vec2) -> egui::Rect {
    let aspect = size.x / size.y;
    let span = if aspect < 1.0 {
        egui::vec2(1.0, aspect)
    } else {
        egui::vec2(1.0 / aspect, 1.0)
    };
    let min = egui::pos2((1.0 - span.x) * MAP_FOCUS.x, (1.0 - span.y) * MAP_FOCUS.y);
    egui::Rect::from_min_size(min, span)
}

/// The map as a [`MAP_GRID`]² textured mesh. `alpha_at` gives each vertex's
/// opacity from its projected screen position (the folded-in fade, see
/// [`map_alpha`]); the tint is white at that opacity (the brightness is baked
/// into the texture).
pub(super) fn map_mesh(
    texture: egui::TextureId,
    texture_size: egui::Vec2,
    frame: &MapFrame,
    pose: &MapPose,
    alpha_at: impl Fn(egui::Pos2) -> f32,
) -> egui::Mesh {
    let uv = cover_uv(texture_size);
    let n = MAP_GRID.max(1);
    let mut mesh = egui::Mesh::with_texture(texture);
    for j in 0..=n {
        for i in 0..=n {
            let f = egui::vec2(i as f32, j as f32) / n as f32;
            let pos = project(
                frame,
                pose,
                MAP_SCALE,
                frame.image.min + f * frame.image.size(),
            );
            let alpha = (alpha_at(pos).clamp(0.0, 1.0) * 255.0).round() as u8;
            mesh.vertices.push(egui::epaint::Vertex {
                pos,
                uv: uv.min + f * uv.size(),
                color: egui::Color32::from_rgba_premultiplied(alpha, alpha, alpha, alpha),
            });
        }
    }
    let row = n as u32 + 1;
    for j in 0..n as u32 {
        for i in 0..n as u32 {
            let a = j * row + i;
            let c = a + row;
            mesh.add_triangle(a, a + 1, c + 1);
            mesh.add_triangle(a, c + 1, c);
        }
    }
    mesh
}

/// Opacity to draw the map at under a backdrop layer of opacity
/// `fade · composite`, so the pair composites like the opaque composition
/// (map faded into the backdrop by `fade`) seen through one group opacity
/// `composite` -- the `bg_alpha` of the returning-via-M overlay. Drawing the
/// map at plain `composite` instead would let less of the board through
/// behind the map than behind the column, leaving a seam at the region edge.
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
/// through behind the maps stays exactly `1 - alpha` during the fade.
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
/// `at`, held flat beyond the first and last stop.
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

/// `color` as a layer at `alpha` (0..1), correctly premultiplied.
pub(super) fn solid(color: egui::Color32, alpha: f32) -> egui::Color32 {
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}

/// A gradient of `color` over `span` along one axis: `stops` are (position as
/// a fraction of `span` along the axis, alpha), scaled by `composite`, and
/// held flat out to both ends of the span.
pub(super) fn gradient_mesh(
    span: egui::Rect,
    stops: &[(f32, f32)],
    axis: egui::Direction,
    color: egui::Color32,
    composite: f32,
) -> egui::Mesh {
    let mut ends: Vec<(f32, f32)> = Vec::with_capacity(stops.len() + 2);
    if let Some(&(at, alpha)) = stops.first()
        && at > 0.0
    {
        ends.push((0.0, alpha));
    }
    ends.extend_from_slice(stops);
    if let Some(&(at, alpha)) = stops.last()
        && at < 1.0
    {
        ends.push((1.0, alpha));
    }
    let mut mesh = egui::Mesh::default();
    for &(at, alpha) in &ends {
        let fill = solid(color, alpha * composite);
        let (a, b) = match axis {
            egui::Direction::LeftToRight | egui::Direction::RightToLeft => {
                let x = span.left() + at * span.width();
                (egui::pos2(x, span.top()), egui::pos2(x, span.bottom()))
            }
            egui::Direction::TopDown | egui::Direction::BottomUp => {
                let y = span.top() + at * span.height();
                (egui::pos2(span.left(), y), egui::pos2(span.right(), y))
            }
        };
        mesh.colored_vertex(a, fill);
        mesh.colored_vertex(b, fill);
    }
    for i in 0..ends.len().saturating_sub(1) as u32 {
        let (a, b, c, d) = (2 * i, 2 * i + 1, 2 * i + 2, 2 * i + 3);
        mesh.add_triangle(a, b, d);
        mesh.add_triangle(a, d, c);
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: egui::Pos2, b: egui::Pos2) -> bool {
        (a - b).length() < 1e-3
    }

    #[test]
    fn a_flat_unscaled_pose_is_the_identity() {
        let frame = map_frame(egui::Rect::from_min_size(
            egui::pos2(380.0, 0.0),
            egui::vec2(900.0, 800.0),
        ));
        let rest = MapPose {
            pan: egui::Vec2::ZERO,
            tilt: 0.0,
            rot: 0.0,
        };
        for p in [
            frame.image.min,
            frame.image.max,
            frame.origin,
            egui::pos2(12.0, 34.0),
        ] {
            assert!(close(project(&frame, &rest, 1.0, p), p));
        }
    }

    #[test]
    fn the_top_of_a_tilted_map_recedes() {
        let frame = map_frame(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(900.0, 800.0),
        ));
        let tilted = MapPose {
            pan: egui::Vec2::ZERO,
            tilt: 30f32.to_radians(),
            rot: 0.0,
        };
        let edge = |y: f32| {
            let a = project(&frame, &tilted, 1.0, egui::pos2(frame.image.left(), y));
            let b = project(&frame, &tilted, 1.0, egui::pos2(frame.image.right(), y));
            (b - a).length()
        };
        assert!(edge(frame.image.top()) < frame.image.width());
        assert!(edge(frame.image.bottom()) > frame.image.width());
    }

    #[test]
    fn the_pose_follows_the_constants() {
        let pose = map_pose(0.0, 800.0);
        let tilt = MAP_TILT_DEG + MAP_TILT_SWING_DEG * (1.0 + MAP_PHASE_TILT.sin());
        assert!((pose.tilt - tilt.max(0.0).to_radians()).abs() < 1e-5);
        assert!((pose.rot - MAP_ROTATE_DEG.to_radians()).abs() < 1e-5);
        let pan_y = 800.0 * (MAP_PAN_AMP_MAJOR * MAP_PHASE_Y_MAJOR.sin() + MAP_PAN_AMP_MINOR);
        assert!((pose.pan.y - pan_y).abs() < 1e-3);
    }

    #[test]
    fn a_portrait_image_covers_the_square_from_its_middle() {
        let uv = cover_uv(egui::vec2(1583.0, 2820.0));
        let aspect = 1583.0 / 2820.0;
        assert!((uv.width() - 1.0).abs() < 1e-6);
        assert!((uv.height() - aspect).abs() < 1e-6);
        assert!((uv.min.y - (1.0 - aspect) * MAP_FOCUS.y).abs() < 1e-6);
        assert!(uv.min.y >= 0.0 && uv.max.y <= 1.0);
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
    fn coverage_margin(region: egui::Rect, t: f32) -> f32 {
        let frame = map_frame(region);
        let pose = map_pose(t, frame.box_height);
        let r = frame.image;
        let corners = [
            r.left_top(),
            r.right_top(),
            r.right_bottom(),
            r.left_bottom(),
        ];
        for corner in corners {
            // In front of the eye, clear of the `NEAR_LIMIT` clamp.
            let rel = (corner - frame.origin) * MAP_SCALE;
            let y = rel.x * pose.rot.sin() + rel.y * pose.rot.cos();
            assert!(frame.distance - y * pose.tilt.sin() > frame.distance * NEAR_LIMIT);
        }
        let quad = corners.map(|p| project(&frame, &pose, MAP_SCALE, p));
        let orientation = (0..4)
            .map(|i| {
                let (a, b) = (quad[i], quad[(i + 1) % 4]);
                a.x * b.y - b.x * a.y
            })
            .sum::<f32>()
            .signum();
        [
            region.left_top(),
            region.right_top(),
            region.right_bottom(),
            region.left_bottom(),
        ]
        .into_iter()
        .flat_map(|p| {
            (0..4).map(move |i| {
                let (a, b) = (quad[i], quad[(i + 1) % 4]);
                let edge = b - a;
                orientation * (edge.x * (p.y - a.y) - edge.y * (p.x - a.x)) / edge.length()
            })
        })
        .fold(f32::INFINITY, f32::min)
    }

    // The handoff's acceptance: "No map edge is ever visible during a 10-minute
    // run at defaults" -- at the design's 1280×800 and at wider, taller and
    // narrow (full-bleed) windows. 600 s of pan time is 20 minutes at the
    // default speed.
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
        let regions = wide
            .map(|(w, h)| {
                map_region(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(w, h),
                ))
            })
            .into_iter()
            .chain(
                narrow.map(|(w, h)| egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w, h))),
            );
        for region in regions {
            let (worst, at) = (0..12_000)
                .map(|i| i as f32 * 0.05)
                .map(|t| (coverage_margin(region, t), t))
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

    #[test]
    fn no_map_shows_before_every_bake_is_done() {
        let mut maps = SplashMaps::new();
        maps.show = MapShow::new(0);
        // Only map 1 baked, map 0 (the random pick) still missing.
        maps.textures[1] = Some(SplashMap {
            id: egui::TextureId::Managed(1),
            size: egui::Vec2::splat(1.0),
            _image: Handle::default(),
        });
        assert!(!maps.is_ready(1) && maps.texture(1).is_none());
        maps.advance(MAP_SECS);
        assert_eq!(maps.show, MapShow::new(0));
        // Settled with the pick failed: the show starts on a map that loaded.
        maps.settle();
        assert_eq!(maps.show.current, 1);
        assert!(maps.texture(1).is_some());
    }

    #[test]
    fn toning_dims_then_warms() {
        let [r, g, b] = tone([200, 200, 200]);
        assert!(r >= g && g >= b && r > b);
        assert_eq!(tone([0, 0, 0]), [0, 0, 0]);
        // Dimmed before the sepia clips: white keeps a warm cast instead of
        // staying neutral.
        let [r, _, b] = tone([255, 255, 255]);
        assert!(r > b);
        assert!(f32::from(r) > 255.0 * MAP_BRIGHTNESS);
    }
}
