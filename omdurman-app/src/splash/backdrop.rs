//! The backdrop material: one full-screen UI node whose shader
//! (`backdrop.wgsl`) draws the period map and every overlay over it, for the
//! title screen and for the lobby. This module turns the show's state and the
//! screen's layout into the shader's parameters.

use bevy::asset::uuid_handle;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderType};
use bevy::shader::{ShaderDefVal, ShaderRef};
use bevy::ui_render::prelude::{UiMaterial, UiMaterialKey};

use super::map::{self, MAX_TAPS, MapImage, MapVariant, SplashMaps};
use super::params::*;

/// The backdrop shader, embedded (see the plugin).
pub(super) const BACKDROP_SHADER: Handle<Shader> =
    uuid_handle!("6b0f5e8e-0c8f-4d9e-9a51-5a3f2b7c1d42");

/// Most stops of one gradient (the shader's `MAX_STOPS`).
const MAX_STOPS: usize = 8;

/// What the backdrop draws.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum BackdropLayout {
    /// The title screen: the map (posed as `map`, blurred `blur_px`) in the
    /// right two thirds faded into the menu column (`narrow`: full-bleed
    /// under a uniform scrim; without the `sidebar`: full-bleed and clear),
    /// the whole composition at group opacity `composite`.
    Title {
        narrow: bool,
        sidebar: bool,
        composite: f32,
        map: MapLayout,
        blur_px: f32,
    },
    /// The lobby: the map full-screen under a scrim, glowing out from the
    /// lobby's `panel`.
    Lobby { panel: Rect },
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug, Default, PartialEq)]
pub(super) struct BackdropMaterial {
    #[uniform(0)]
    pub(super) params: BackdropParams,
    #[texture(1)]
    #[sampler(2)]
    pub(super) current: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub(super) next: Handle<Image>,
}

impl UiMaterial for BackdropMaterial {
    fn fragment_shader() -> ShaderRef {
        BACKDROP_SHADER.into()
    }

    /// The shader's array sizes come from here, so the uniform layouts on
    /// both sides cannot drift apart.
    fn specialize(descriptor: &mut RenderPipelineDescriptor, _key: UiMaterialKey<Self>) {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader_defs.extend([
                ShaderDefVal::UInt("MAX_TAPS".into(), MAX_TAPS as u32),
                ShaderDefVal::UInt("MAX_STOPS".into(), MAX_STOPS as u32),
            ]);
        }
    }
}

/// One slide's map: see `MapSampling` in the shader.
#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct MapSampling {
    to_square: Mat3,
    uv_min: Vec2,
    uv_size: Vec2,
    texel: Vec2,
    tap_count: u32,
    taps: [Vec4; MAX_TAPS],
}

/// The shader's parameters: see `Params` in the shader.
#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct BackdropParams {
    screen: Vec2,
    composite: f32,
    mode: u32,
    region: Vec4,
    backdrop: Vec4,
    fade_in: f32,
    progress: f32,
    incoming: u32,
    has_map: u32,
    current: MapSampling,
    next: MapSampling,
    fade_stops: [Vec4; MAX_STOPS],
    scrim_stops: [Vec4; MAX_STOPS],
    glow_stops: [Vec4; MAX_STOPS],
    stop_counts: UVec4,
    sepia_matrix: Mat3,
    brightness: f32,
    sepia: f32,
    blend_gamma: f32,
    narrow_scrim: f32,
    panel: Vec4,
    panel_radius: f32,
    glow_width: f32,
    lobby_scrim: f32,
}

const MODE_WIDE: u32 = 0;
const MODE_NARROW: u32 = 1;
const MODE_LOBBY: u32 = 2;

fn stops(list: &[(f32, f32)]) -> [Vec4; MAX_STOPS] {
    assert!(
        list.len() <= MAX_STOPS,
        "at most {MAX_STOPS} gradient stops"
    );
    std::array::from_fn(|i| {
        list.get(i)
            .map_or(Vec4::ZERO, |&(at, value)| Vec4::new(at, value, 0.0, 0.0))
    })
}

/// How to draw `image` in `view` as `variant`, laid out as `layout` and
/// blurred `blur_px` screen points, at pan time `t` of this run's pan
/// `variation` in `region`.
fn sampling(
    (image, view): (&MapImage, MapView),
    variant: MapVariant,
    (layout, blur_px): (&MapLayout, f32),
    region: Rect,
    (t, variation): (f32, &map::PanVariation),
) -> MapSampling {
    let frame = map::map_frame(region, layout);
    let pose = map::map_pose(t, frame.box_height, layout, variation);
    let uv = map::cover_uv(image.size, view);
    let taps = map::blur_taps(map::blur_sigma_texels(
        image.size, variant, blur_px, view.zoom,
    ));
    MapSampling {
        to_square: map::map_homography(&frame, &pose).inverse(),
        uv_min: uv.min,
        uv_size: uv.size(),
        texel: image.size.recip(),
        tap_count: taps.len() as u32,
        taps: std::array::from_fn(|i| {
            taps.get(i).map_or(Vec4::ZERO, |&(offset, weight)| {
                Vec4::new(offset, weight, 0.0, 0.0)
            })
        }),
    }
}

/// A palette colour as the shader's linear RGBA.
fn linear(color: bevy_egui::egui::Color32) -> Vec4 {
    Vec4::from_array(super::palette_color(color).to_linear().to_f32_array())
}

impl BackdropMaterial {
    /// Point the material at the show's maps and pose for a screen `screen`
    /// logical pixels large drawn as `layout`.
    pub(super) fn update(&mut self, screen: Vec2, maps: &SplashMaps, layout: BackdropLayout) {
        let screen_rect = Rect::from_corners(Vec2::ZERO, screen);
        let (variant, look, region) = match layout {
            BackdropLayout::Title {
                narrow: false,
                sidebar: true,
                map,
                blur_px,
                ..
            } => (
                MapVariant::Title,
                (map, blur_px),
                map::map_region(screen_rect),
            ),
            BackdropLayout::Title { map, blur_px, .. } => {
                (MapVariant::Title, (map, blur_px), screen_rect)
            }
            BackdropLayout::Lobby { .. } => (
                MapVariant::Lobby,
                (LOBBY_MAP, MapVariant::Lobby.blur_px()),
                screen_rect,
            ),
        };
        let look = (&look.0, look.1);
        let current = maps.image(maps.show.current);
        let fade = maps.show.fade();
        let incoming_index = fade.map_or(0, |(index, _)| index);
        let incoming = fade.and_then(|(index, progress)| Some((maps.image(index)?, progress)));
        let (mode, composite, backdrop, panel) = match layout {
            BackdropLayout::Title {
                narrow,
                sidebar,
                composite,
                ..
            } => (
                if narrow || !sidebar {
                    MODE_NARROW
                } else {
                    MODE_WIDE
                },
                composite,
                crate::ui::palette::SPLASH_BACKDROP,
                Rect::default(),
            ),
            BackdropLayout::Lobby { panel } => {
                (MODE_LOBBY, 1.0, crate::ui::palette::NEUTRAL_BG, panel)
            }
        };
        let sepia = Mat3::from_cols_array_2d(&SEPIA_MATRIX).transpose();
        self.params = BackdropParams {
            screen,
            composite,
            mode,
            region: Vec4::new(region.min.x, region.min.y, region.max.x, region.max.y),
            backdrop: linear(backdrop),
            fade_in: maps.show.fade_in(),
            progress: incoming.map_or(0.0, |(_, progress)| progress),
            incoming: u32::from(incoming.is_some()),
            has_map: u32::from(current.is_some()),
            current: current
                .map(|image| {
                    let shown = (image, maps.current_view());
                    sampling(
                        shown,
                        variant,
                        look,
                        region,
                        (maps.pan_time, &maps.variation),
                    )
                })
                .unwrap_or_default(),
            next: incoming
                .map(|(image, _)| {
                    let shown = (image, maps.incoming_view(incoming_index));
                    sampling(
                        shown,
                        variant,
                        look,
                        region,
                        (maps.incoming_pan_time, &maps.variation),
                    )
                })
                .unwrap_or_default(),
            fade_stops: stops(&FADE_STOPS),
            scrim_stops: stops(&SCRIM_STOPS),
            glow_stops: stops(&LOBBY_GLOW_STOPS),
            stop_counts: UVec4::new(
                FADE_STOPS.len() as u32,
                SCRIM_STOPS.len() as u32,
                LOBBY_GLOW_STOPS.len() as u32,
                0,
            ),
            sepia_matrix: sepia,
            brightness: MAP_BRIGHTNESS,
            sepia: MAP_SEPIA,
            blend_gamma: CSS_BLEND_GAMMA,
            // Without the sidebar there is no text to keep legible: the map
            // shows clear.
            narrow_scrim: match layout {
                BackdropLayout::Title { sidebar: false, .. } => 0.0,
                _ => NARROW_SCRIM_ALPHA,
            },
            panel: Vec4::new(panel.min.x, panel.min.y, panel.max.x, panel.max.y),
            panel_radius: LOBBY_PANEL_RADIUS,
            glow_width: LOBBY_GLOW_REL * screen.y,
            lobby_scrim: LOBBY_SCRIM_ALPHA,
        };
        let image = |image: Option<&MapImage>| image.map(|image| image.handle.clone());
        // An unused slot samples the current map (any valid texture will do).
        let current = image(current).unwrap_or_default();
        self.next = image(incoming.map(|(image, _)| image)).unwrap_or_else(|| current.clone());
        self.current = current;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The sepia matrix reaches the shader with its rows producing r, g, b.
    #[test]
    fn the_sepia_matrix_keeps_its_rows() {
        let sepia = Mat3::from_cols_array_2d(&SEPIA_MATRIX).transpose();
        let white = sepia * Vec3::ONE;
        for (i, row) in SEPIA_MATRIX.iter().enumerate() {
            assert!((white[i] - row.iter().sum::<f32>()).abs() < 1e-6);
        }
    }

    #[test]
    fn every_gradient_fits_the_shader() {
        for list in [&FADE_STOPS[..], &SCRIM_STOPS, &LOBBY_GLOW_STOPS] {
            assert!(list.len() <= MAX_STOPS);
        }
    }
}
