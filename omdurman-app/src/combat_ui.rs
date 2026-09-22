//! Shared combat-presentation helpers: modifier descriptions, result
//! labels, and the fire/melee direction arrow. One home for the mappings
//! and geometry the combat panels must not let drift apart.

use bevy::prelude::*;
use bevy_egui::egui;
use omdurman_rules::{CombatResult, FireModifier, MeleeModifier};
use omdurman_types::HexCoord;

use crate::DirectionArrowCtx;

/// One rulebook modifier rendered as a label + the § paragraph that
/// justifies it.
#[derive(Clone)]
pub(crate) struct ModifierLine {
    pub label: String,
    pub paragraph: String,
}

/// Canonical fire-modifier description (§6.24/§5.54/§6.23/§9.231/§9.232).
pub(crate) fn describe_fire_modifier(m: FireModifier) -> ModifierLine {
    let (label, paragraph) = match m {
        FireModifier::AngloEgyptianDirectFire => {
            ("+1 Anglo-Egyptian direct fire".to_string(), "6.24")
        }
        FireModifier::BrigadeIntegrity => ("+1 brigade integrity".to_string(), "5.54"),
        FireModifier::Terrain(n) => (format!("{n:+} terrain defence"), "6.23"),
        FireModifier::ZaribaThornHedge => ("-2 zariba thorn hedge".to_string(), "9.231"),
        FireModifier::ZaribaTrenchEntrenched => {
            ("-4 zariba trench (entrenched)".to_string(), "9.232")
        }
    };
    ModifierLine {
        label,
        paragraph: paragraph.into(),
    }
}

/// Canonical melee-modifier description (§7.7/§9.232).
pub(crate) fn describe_melee_modifier(m: MeleeModifier) -> ModifierLine {
    let (label, paragraph) = match m {
        MeleeModifier::DervishStandard => ("+2 Dervish standard".to_string(), "7.7"),
        MeleeModifier::AngloEgyptianStandard => ("+1 Anglo-Egyptian standard".to_string(), "7.7"),
        MeleeModifier::DervishVsTrenchedDefender => {
            ("-2 vs. entrenched defender".to_string(), "9.232")
        }
    };
    ModifierLine {
        label,
        paragraph: paragraph.into(),
    }
}

/// Canonical combat-result label for the combat card.
pub(crate) fn describe_result(result: CombatResult) -> String {
    match result {
        CombatResult::NoEffect => "No effect".to_string(),
        CombatResult::Disrupt => "Disrupt".to_string(),
        CombatResult::Eliminate(n) => format!("Eliminate {n}"),
    }
}

/// The bold-orange direction arrow from an acting stack to its hovered
/// target — the shared geometry behind the fire and melee aim arrows
/// (inset 18% of the hex, drawn between the two rims).
pub(crate) fn direction_arrow(
    commands: &mut Commands,
    ctx: &DirectionArrowCtx,
    from: HexCoord,
    to: HexCoord,
    marker: impl Component,
) {
    use omdurman_hexmap::hex_world_pos;

    let hex = &ctx.hex;
    let origin = hex.layout.adjusted_origin(&hex.overlay.params);
    let size = hex.overlay.params.hex_size;
    let from = hex_world_pos(from, origin, &hex.overlay.params);
    let to = hex_world_pos(to, origin, &hex.overlay.params);
    let delta = Vec3::new(to.x - from.x, 0.0, to.z - from.z);
    let len = delta.length();
    if len < f32::EPSILON {
        return;
    }
    let dir = delta / len;
    let inset = size * 0.18;
    let draw_len = (len - inset).max(len * 0.4);
    let tail = from + dir * ((len - draw_len) * 0.5);
    commands.spawn((
        marker,
        Mesh3d(ctx.arrow_assets.mesh.clone()),
        MeshMaterial3d(hex.assets.orange.clone()),
        Transform::from_xyz(tail.x, 1.55, tail.z)
            .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
            .with_scale(Vec3::new(size * 0.5, 1.0, draw_len)),
        Visibility::Visible,
    ));
}

/// Dark panel frame shared by the combat panels (`fill` is each panel's
/// tint over the same rounded, inset base).
pub(crate) fn combat_frame(fill: egui::Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(10, 6))
}
