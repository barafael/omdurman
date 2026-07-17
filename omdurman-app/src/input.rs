//! Shared input/camera/raycast plumbing for combat click handlers.
//!
//! The four combat click handlers (fire, melee, advance-after-combat, and
//! retreat) each pull in the same half-dozen system parameters just to answer
//! one question: "did the player left-click on a hex this frame, and if so
//! which one?". [`CombatClickCtx`] bundles those parameters and exposes a
//! single [`CombatClickCtx::clicked_hex`] helper that runs the full
//! egui-consume + ground-raycast + hex-pick pipeline.

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use omdurman_hexmap::{HexLayout, hit_to_hex};
use omdurman_types::HexCoord;

use crate::camera::RtsCamera;
use crate::render::HexOverlay;
use crate::util::raycast_ground;

/// Bundles the common input / camera / raycast plumbing shared by the four
/// combat click handlers (`handle_fire_combat`, `handle_melee_combat`,
/// `handle_advance_after_combat`, `handle_retreat`). Each handler still takes
/// its own rules-engine / networking parameters -- this only collects the
/// boilerplate needed to answer "which hex did the player just left-click on?".
#[derive(bevy::ecs::system::SystemParam)]
pub struct CombatClickCtx<'w, 's> {
    pub buttons: Res<'w, ButtonInput<MouseButton>>,
    pub contexts: EguiContexts<'w, 's>,
    pub layout: Res<'w, HexLayout>,
    pub overlay: Res<'w, HexOverlay>,
    pub windows: Query<'w, 's, &'static Window>,
    pub cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<RtsCamera>>,
}

impl CombatClickCtx<'_, '_> {
    /// Returns the hex coord under the cursor on left-click, or `None` if there
    /// was no left-click this frame, the click is consumed by an egui area, the
    /// ground raycast misses, or no camera/window is available.
    pub fn clicked_hex(&mut self) -> Option<HexCoord> {
        if !self.buttons.just_released(MouseButton::Left) {
            return None;
        }
        let ctx = self.contexts.ctx_mut().ok()?;
        if ctx.egui_wants_pointer_input() {
            return None;
        }
        let hit = raycast_ground(&self.windows, &self.cameras)?;
        let origin = self.layout.adjusted_origin(&self.overlay.params);
        Some(hit_to_hex(hit, origin, &self.overlay.params))
    }
}
