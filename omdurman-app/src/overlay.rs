//! Shared board-overlay scaffolding: the despawn-then-respawn ring batch
//! every hex-overlay system builds. One constructor replaces the repeated
//! destructure/despawn/geometry prologue (and keeps the despawn happening
//! before any early return, as the hand-rolled version did).

use bevy::prelude::*;
use omdurman_hexmap::hex_world_pos;
use omdurman_types::{HexCoord, OverlayParams};

use crate::params::HexRender;
use crate::render::HexRingAssets;

/// A fresh overlay batch: the previous entities are queued for despawn, and
/// each [`ring`](RingBatch::ring) call spawns one hex ring (shared hex-ring
/// mesh) at this frame's board geometry.
pub(crate) struct RingBatch<'a, 'w, 's, 'r> {
    commands: &'a mut Commands<'w, 's>,
    assets: &'r HexRingAssets,
    origin: Vec2,
    params: &'r OverlayParams,
    size: f32,
}

impl<'a, 'w, 's, 'r> RingBatch<'a, 'w, 's, 'r> {
    /// Spawn one hex ring at `coord`, `height` above the plane, scaled by
    /// `scale` (1.0 = the full hex outline) and tinted with `material`.
    pub(crate) fn ring(
        &mut self,
        marker: impl Component,
        coord: HexCoord,
        height: f32,
        scale: f32,
        material: &Handle<StandardMaterial>,
    ) {
        let pos = hex_world_pos(coord, self.origin, self.params);
        self.commands.spawn((
            marker,
            Mesh3d(self.assets.mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(pos.x, height, pos.z).with_scale(Vec3::splat(self.size * scale)),
            Visibility::Visible,
        ));
    }

    /// The batch's adjusted board origin, for spawns outside the plain
    /// ring shape (bars, arrows, labels).
    pub(crate) fn origin(&self) -> Vec2 {
        self.origin
    }

    /// The batch's hex size.
    pub(crate) fn size(&self) -> f32 {
        self.size
    }

    /// The batch's overlay params, for spawns needing full layout math.
    pub(crate) fn params(&self) -> &OverlayParams {
        self.params
    }

    /// Direct access to the batch's command queue, for spawns outside the
    /// plain ring shape.
    pub(crate) fn commands(&mut self) -> &mut Commands<'w, 's> {
        self.commands
    }
}

/// Despawn the previous overlay batch and return a builder for the new one.
pub(crate) fn ring_batch<'a, 'w, 's, 'r, I>(
    commands: &'a mut Commands<'w, 's>,
    hex: &'r HexRender,
    existing: I,
) -> RingBatch<'a, 'w, 's, 'r>
where
    I: IntoIterator<Item = Entity>,
{
    for e in existing {
        commands.entity(e).despawn();
    }
    RingBatch {
        commands,
        assets: &hex.assets,
        origin: hex.layout.adjusted_origin(&hex.overlay.params),
        params: &hex.overlay.params,
        size: hex.overlay.params.hex_size,
    }
}
