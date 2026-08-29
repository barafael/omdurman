//! Shared input plumbing for combat click handlers.
//!
//! The four combat click handlers (fire, melee, advance-after-combat, and
//! retreat) each pull in the same half-dozen system parameters just to answer
//! one question: "did the player left-click on a hex this frame, and if so
//! which one?". [`CombatClickCtx`] bundles those parameters and exposes a
//! single [`CombatClickCtx::clicked_hex`] helper. The world-space hit comes
//! from bevy_picking's board-plane funnel ([`PointerGroundHit`]), which is
//! already `None` when the pointer is over UI or off the board.

use bevy::prelude::*;
use omdurman_hexmap::{HexLayout, hit_to_hex};
use omdurman_types::HexCoord;

use crate::render::HexOverlay;

/// Bundles the common input plumbing shared by the four combat click handlers
/// (`handle_fire_combat`, `handle_melee_combat`, `handle_advance_after_combat`,
/// `handle_retreat`). Each handler still takes its own rules-engine /
/// networking parameters -- this only collects the boilerplate needed to
/// answer "which hex did the player just left-click on?".
#[derive(bevy::ecs::system::SystemParam)]
pub struct CombatClickCtx<'w> {
    pub buttons: Res<'w, ButtonInput<MouseButton>>,
    pub ground: Res<'w, crate::picking::PointerGroundHit>,
    pub layout: Res<'w, HexLayout>,
    pub overlay: Res<'w, HexOverlay>,
}

impl CombatClickCtx<'_> {
    /// Returns the hex coord under the cursor on left-click, or `None` if there
    /// was no left-click this frame, the pointer is over UI or off the board,
    /// or the picking funnel has no hit.
    pub fn clicked_hex(&mut self) -> Option<HexCoord> {
        if !self.buttons.just_released(MouseButton::Left) {
            return None;
        }
        let hit = (**self.ground)?;
        let origin = self.layout.adjusted_origin(&self.overlay.params);
        Some(hit_to_hex(hit, origin, &self.overlay.params))
    }
}
