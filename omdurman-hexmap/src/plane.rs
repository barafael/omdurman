//! The board plane: the textured ground quad a board's scan is drawn on, the
//! texture cache that keeps each board image resident across switches, the
//! hex-grid [`HexOverlay`] calibration the conversions read, and the hex
//! ring mesh.

use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};
use omdurman_types::OverlayParams;

// -- Map plane -----------------------------------------------------------------

/// Marker for the single ground quad the board scan is drawn on.
#[derive(Component)]
pub struct MapPlane;

/// Holds a handle per map-texture path so each board image is decoded and
/// uploaded once, kept resident across board switches, and re-used instantly
/// when switching back. Keyed by asset path; [`texture`](Self::texture) loads
/// on first request and returns the cached handle thereafter.
#[derive(Resource, Default)]
pub struct MapTextureCache(pub std::collections::HashMap<String, Handle<Image>>);

impl MapTextureCache {
    /// The handle for `image`, loading (and caching) it on first request.
    /// `AssetServer::load` already dedupes by path, so the win here is avoiding
    /// the repeated `load` call churn and giving us an explicit place to
    /// preload from.
    pub fn texture(&mut self, asset_server: &AssetServer, image: &str) -> Handle<Image> {
        self.0
            .entry(image.to_string())
            .or_insert_with(|| asset_server.load(image.to_string()))
            .clone()
    }
}

/// Bundle of the mutable asset stores + the asset server + the texture cache
/// so [`apply_map_data_to_plane`] stays under clippy's argument limit.
pub struct PlaneTextureStores<'a> {
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<StandardMaterial>,
    pub cache: &'a mut MapTextureCache,
    pub asset_server: &'a AssetServer,
}

/// Re-size and re-texture the existing map plane to a board's image and
/// dimensions (§dual-map). Used when a scenario selects a board or the map
/// editor switches the active map.
pub fn apply_map_data_to_plane(
    plane: &Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>), With<MapPlane>>,
    stores: &mut PlaneTextureStores<'_>,
    image: &str,
    img_w: f32,
    img_h: f32,
) {
    let PlaneTextureStores {
        meshes,
        materials,
        cache,
        asset_server,
    } = stores;
    let Ok((mesh, material)) = plane.single() else {
        return;
    };
    if let Some(mut m) = meshes.get_mut(&mesh.0) {
        *m = Rectangle::new(img_w, img_h).into();
    }
    if let Some(mut mat) = materials.get_mut(&material.0) {
        // Re-use the already-decoded handle when switching back to a board.
        mat.base_color_texture = Some(cache.texture(asset_server, image));
    }
}

// -- Hex overlay resource ------------------------------------------------------

/// The active board's hex-grid calibration ([`OverlayParams`], loaded with
/// the board); every consumer of `HexLayout`'s warp-aware conversions reads
/// it.
#[derive(Resource, Default)]
pub struct HexOverlay {
    pub params: OverlayParams,
}

// -- Hex ring mesh -------------------------------------------------------------

/// A flat hexagonal ring (outer radius 1.0, inner 0.96) lying in the XZ plane,
/// built once and scaled per use for hex outlines and highlights.
pub fn hex_ring_mesh() -> Mesh {
    let outer = 1.0;
    let inner = 0.96;
    let mut positions = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for i in 0..6 {
        let a0 = std::f32::consts::FRAC_PI_6 + i as f32 * std::f32::consts::PI / 3.0;
        let a1 = std::f32::consts::FRAC_PI_6 + (i + 1) as f32 * std::f32::consts::PI / 3.0;

        let o0 = Vec3::new(outer * a0.cos(), 0.0, outer * a0.sin());
        let o1 = Vec3::new(outer * a1.cos(), 0.0, outer * a1.sin());
        let i0 = Vec3::new(inner * a0.cos(), 0.0, inner * a0.sin());
        let i1 = Vec3::new(inner * a1.cos(), 0.0, inner * a1.sin());

        let base = positions.len() as u32;
        positions.extend([o0, o1, i0, i1]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
    }

    let normals = vec![Vec3::Y; positions.len()];
    let uvs = vec![Vec2::ZERO; positions.len()];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}
