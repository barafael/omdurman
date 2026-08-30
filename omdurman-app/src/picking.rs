//! Board picking via `bevy_picking`'s mesh backend.
//!
//! One ray per pointer per frame, cast by the engine's mesh-picking backend
//! against the *marked* board plane (`MeshPickingSettings::require_markers`
//! keeps every other mesh — overlays, markers, counters — unpickable), with
//! the result fanned out to the app through a single [`PointerGroundHit`]
//! resource. This replaces the hand-rolled `raycast_ground` ray-vs-y=0
//! intersection that every hover/click system used to run itself.
//!
//! Input coexistence with egui is handled by one funnel check: while
//! [`crate::ui_plugin::EguiPointerOverUi`] is set (see
//! `ui_plugin::egui_wants_pointer_input` — the panels self-register their
//! rects with egui), the plane hit is discarded and every consumer sees
//! `None`. bevy_egui's own egui-picking capture is switched off: it gates on
//! `egui_wants_pointer_input`, which is *always true* in the single-pass
//! mode we run (see the `ui_plugin` module docs), so its `PointerHits`
//! would starve the board plane everywhere.
//!
//! Consumers read [`PointerGroundHit`] instead of owning camera/window
//! plumbing; camera scroll/drag/keyboard stay button-driven and keep their
//! existing gating (they are not hex-position based).

use bevy::picking::mesh_picking::ray_cast::RayCastVisibility;
use bevy::picking::mesh_picking::{MeshPickingCamera, MeshPickingSettings};
use bevy::picking::pointer::PointerId;
use bevy::picking::{Pickable, PickingPlugin, PickingSystems, hover::HoverMap};
use bevy::prelude::*;
use bevy_egui::EguiContextSettings;
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
            .init_resource::<EguiCaptureDisabled>()
            .add_systems(
                PreUpdate,
                update_pointer_ground_hit.after(PickingSystems::Last),
            )
            // bevy_egui's egui-picking capture reports `PointerHits` for the
            // egui context whenever `egui_wants_pointer_input` — which is
            // always true in our single-pass mode (see `ui_plugin` docs) —
            // and would therefore outrank the board plane everywhere. Our own
            // funnel check (`EguiPointerOverUi`) does this job instead.
            .add_systems(
                Update,
                disable_egui_picking_capture.run_if(not(egui_capture_disabled)),
            );
    }
}

#[derive(Resource, Default)]
struct EguiCaptureDisabled(bool);

fn egui_capture_disabled(done: Res<EguiCaptureDisabled>) -> bool {
    done.0
}

/// Retries until the egui context entity exists, then flips the setting once.
fn disable_egui_picking_capture(
    mut contexts: Query<&mut EguiContextSettings>,
    mut done: ResMut<EguiCaptureDisabled>,
) {
    let Some(mut settings) = contexts.iter_mut().next() else {
        return;
    };
    settings.capture_pointer_input = false;
    done.0 = true;
}

/// Copy the board-plane hover hit out of picking's [`HoverMap`] for the local
/// mouse pointer. Runs after all picking systems so the map reflects this
/// frame's ray against the current camera — including camera pans with a
/// stationary pointer, which the old cursor-event raycast also handled but a
/// naive event-based (Over/Move) port would not. `None` whenever egui holds
/// the pointer interest (panels and widgets self-register with egui's
/// hit-testing; bevy_egui's picking backend then outranks the mesh hits), so
/// every consumer is UI-gated without knowing why.
pub fn update_pointer_ground_hit(
    hover_map: Res<HoverMap>,
    plane: Query<Entity, With<MapPlane>>,
    over_ui: Res<crate::ui_plugin::EguiPointerOverUi>,
    mut hit: ResMut<PointerGroundHit>,
) {
    hit.0 = None;
    // Egui owns the pointer (widgets, panel blockers, overlays): every
    // consumer sees `None` without having to know why.
    if over_ui.0 {
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
