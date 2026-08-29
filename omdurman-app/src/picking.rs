//! Board picking via `bevy_picking`'s mesh backend.
//!
//! One ray per pointer per frame, cast by the engine's mesh-picking backend
//! against the *marked* board plane (`MeshPickingSettings::require_markers`
//! keeps every other mesh — overlays, markers, counters — unpickable), with
//! the result fanned out to the app through a single [`PointerGroundHit`]
//! resource. This replaces the hand-rolled `raycast_ground` ray-vs-y=0
//! intersection that every hover/click system used to run itself, and moves
//! input coexistence with egui from per-system checks into one funnel:
//!
//! * over *interactive* egui, the `bevy_egui` `picking` feature wins the
//!   focus ordering (EguiPickingOrder > mesh hits), so the plane is simply
//!   not hovered and the hit is `None`;
//! * over the app's *background-layer panels* — which the egui API cannot
//!   see (see [`crate::ui_plugin::PanelRects`]) — the funnel consults the
//!   panel registry itself.
//!
//! Consumers read [`PointerGroundHit`] instead of owning camera/window
//! plumbing; camera scroll/drag/keyboard stay button-driven and keep their
//! existing gating (they are not hex-position based).

use bevy::picking::mesh_picking::ray_cast::RayCastVisibility;
use bevy::picking::mesh_picking::{MeshPickingCamera, MeshPickingSettings};
use bevy::picking::pointer::PointerId;
use bevy::picking::{Pickable, PickingPlugin, PickingSystems, hover::HoverMap};
use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;
use omdurman_hexmap::MapPlane;

/// World-space point where the local pointer's ray last hit the board plane,
/// or `None` when the pointer is over UI, off the board, or no camera sees
/// the hit. Written once per frame in `PreUpdate` (after picking's hover
/// systems); every `Update` consumer reads it without further ordering.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct PointerGroundHit(pub Option<Vec3>);

pub struct BoardPickingPlugin;

impl Plugin for BoardPickingPlugin {
    fn build(&self, app: &mut App) {
        // bevy enables `picking` (incl. mesh picking) by default and ships
        // `PickingPlugin` via DefaultPlugins; add it only if absent so the
        // plugin also works from a bare `App`.
        if !app.is_plugin_added::<PickingPlugin>() {
            app.add_plugins(PickingPlugin);
        }
        app.add_plugins(MeshPickingPlugin)
            // Only entities explicitly marked `Pickable` take part: the board
            // plane (marked where it spawns). Everything else — hex rings,
            // markers, arrows, unit counters — stays invisible to picking,
            // exactly like the old ground-only raycast.
            .insert_resource(MeshPickingSettings {
                require_markers: true,
                ray_cast_visibility: RayCastVisibility::VisibleInView,
            })
            .init_resource::<PointerGroundHit>()
            .add_systems(
                PreUpdate,
                update_pointer_ground_hit.after(PickingSystems::Last),
            );
    }
}

/// Copy the board-plane hover hit out of picking's [`HoverMap`] for the local
/// mouse pointer. Runs after all picking systems so the map reflects this
/// frame's ray against the current camera — including camera pans with a
/// stationary pointer, which the old cursor-event raycast also handled but a
/// naive event-based (Over/Move) port would not.
pub fn update_pointer_ground_hit(
    hover_map: Res<HoverMap>,
    plane: Query<Entity, With<MapPlane>>,
    wants: Res<EguiWantsInput>,
    panels: Res<crate::ui_plugin::PanelRects>,
    mut hit: ResMut<PointerGroundHit>,
) {
    hit.0 = None;
    // Interactive egui already starves the plane via backend ordering; the
    // background-layer panels need the explicit registry check. Either way
    // every consumer sees `None` without having to know why.
    if crate::ui_plugin::wants_pointer_core(&wants, &panels) {
        return;
    }
    let Ok(plane) = plane.single() else {
        return;
    };
    let Some(hits) = hover_map.get(&PointerId::Mouse) else {
        return;
    };
    if let Some(data) = hits.get(&plane) {
        hit.0 = data.position;
    }
}

/// Marker pair inserted on the board plane and the RTS camera so the
/// require-markers mesh backend sees them. Kept here so the picking
/// configuration has a single home.
pub(crate) fn plane_pickable() -> Pickable {
    Pickable::default()
}

pub(crate) fn picking_camera() -> MeshPickingCamera {
    MeshPickingCamera
}
