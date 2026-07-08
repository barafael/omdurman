//! The paper theme: one module owns every UI color so the chrome reads as if it
//! were printed on the same 1982 sheet as the map and charts. Nothing else in
//! the app should hardcode an egui `Color32` -- pull from here or derive a shade
//! from these tokens (alpha / darken), never introduce a new hue.
//!
//! Two accents only, with fixed meanings:
//!   * [`TEAL`] (the Nile wash) = "you may / yours" -- selection, hover, legal
//!     moves/targets, active tab, current turn cell, own-side highlights.
//!   * [`ROSE`] (the rough-terrain wash) = "beware / theirs" -- ZOC warnings,
//!     rejected moves, enemy highlights, warn/error text.
//! Never both on one element; never egui's default blue/green/red anywhere.

// Some tokens are consumed only by later phases of the paper-theme work (the
// map-space overlay sweep, night tint, and chart sheet). Keep them defined here
// -- one module owns every color -- and drop this allow once they are wired.
#![allow(dead_code)]

use bevy_egui::egui;
use egui::{Color32, CornerRadius, Shadow, Stroke, Visuals};

// -- Papers ----------------------------------------------------------------
// The map board scans greyer; the manual charts are a warmer butter.

/// Chart cream (`#FEF3C1` raw scan, dialed down). Primary panel/window fill.
pub const PAPER_CHART: Color32 = Color32::from_rgb(0xF6, 0xED, 0xC5);
/// Map-board grey-cream.
pub const PAPER_MAP: Color32 = Color32::from_rgb(0xEA, 0xE6, 0xE0);
/// Pressed / recessed cells; text-edit and slider troughs.
pub const PAPER_DIM: Color32 = Color32::from_rgb(0xE7, 0xDD, 0xB4);

// -- Ink -------------------------------------------------------------------

/// Near-black print ink -- primary text and the heavy 2px print rules.
pub const INK: Color32 = Color32::from_rgb(0x1A, 0x16, 0x10);
/// Secondary text and hairlines.
pub const INK_FAINT: Color32 = Color32::from_rgb(0x6B, 0x62, 0x50);

// -- Accents (exactly two, both sampled from the printed sheets) -----------

/// Nile wash -- interactive / selection / "yours".
pub const TEAL: Color32 = Color32::from_rgb(0x8F, 0xC5, 0xD7);
/// Faint Nile -- table striping / hover fills.
pub const TEAL_FAINT: Color32 = Color32::from_rgb(0xDC, 0xEA, 0xE2);
/// Rough-terrain wash -- warnings / hostile / "theirs".
pub const ROSE: Color32 = Color32::from_rgb(0xBA, 0xA7, 0xA6);
/// Soft warning fills.
pub const ROSE_FAINT: Color32 = Color32::from_rgb(0xE3, 0xD6, 0xD4);

// -- Night (sampled from the NIGHT turn-track cells) -----------------------

/// The printed NIGHT-cell green; the world tints toward this at night.
pub const NIGHT_CELL: Color32 = Color32::from_rgb(0xCD, 0xD5, 0xB0);

// -- Derived shades --------------------------------------------------------
// warn/error read as a dark rose, never red-red (decision: two accents only).

/// Dark-rose warning text -- `ROSE` darkened ~40%.
pub const WARN_INK: Color32 = Color32::from_rgb(0x70, 0x64, 0x63);
/// Error text: same rose family, a touch deeper. Never a pure red.
pub const ERROR_INK: Color32 = Color32::from_rgb(0x5C, 0x4E, 0x4D);

/// Alpha used for map-space overlays so the board and chrome read as one
/// printed system (§3: highlights at ~35% alpha).
pub const OVERLAY_ALPHA: u8 = 90; // ~0.35 * 255

/// Apply the paper skin to an egui context. Idempotent and cheap; call once
/// after the context exists (the egui context is not ready at Bevy `Startup`
/// on native, so this is driven from an idempotent `Update` system alongside
/// font setup, not from `Startup`).
pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::light();

    visuals.dark_mode = false;
    visuals.override_text_color = Some(INK);

    // Surfaces.
    visuals.panel_fill = PAPER_CHART;
    visuals.window_fill = PAPER_CHART;
    visuals.faint_bg_color = TEAL_FAINT; // Grid::striped -> the CRT's alternating columns.
    visuals.extreme_bg_color = PAPER_DIM; // text edits, slider troughs.
    visuals.code_bg_color = PAPER_DIM;

    // Heavy print rules; no radii, no blur -- depth comes from the 2px rule.
    let rule = Stroke::new(2.0, INK);
    visuals.window_stroke = rule;
    visuals.window_corner_radius = CornerRadius::ZERO;
    visuals.menu_corner_radius = CornerRadius::ZERO;
    visuals.window_shadow = Shadow::NONE;
    visuals.popup_shadow = Shadow::NONE;

    // Selection: kill egui's default blue.
    visuals.selection.bg_fill = TEAL;
    visuals.selection.stroke = Stroke::new(1.0, INK);

    // Links read as printed cross-references, not web links.
    visuals.hyperlink_color = INK;

    // Warn/error stay in the rose family (decision: never red-red).
    visuals.warn_fg_color = WARN_INK;
    visuals.error_fg_color = ERROR_INK;

    // Widget states.
    let w = &mut visuals.widgets;
    for state in [&mut w.noninteractive, &mut w.inactive] {
        state.bg_fill = PAPER_CHART;
        state.weak_bg_fill = PAPER_CHART;
        state.bg_stroke = Stroke::new(1.0, INK_FAINT);
        state.fg_stroke = Stroke::new(1.0, INK);
        state.corner_radius = CornerRadius::ZERO;
        state.expansion = 0.0;
    }
    // noninteractive text is the plain ink; its "frame" (window outline) is the
    // heavy rule set on `window_stroke`, so keep its own bg_stroke faint.
    w.hovered.bg_fill = TEAL_FAINT;
    w.hovered.weak_bg_fill = TEAL_FAINT;
    w.hovered.bg_stroke = Stroke::new(1.5, INK);
    w.hovered.fg_stroke = Stroke::new(1.0, INK);
    w.hovered.corner_radius = CornerRadius::ZERO;
    w.hovered.expansion = 0.0;
    for state in [&mut w.active, &mut w.open] {
        state.bg_fill = TEAL;
        state.weak_bg_fill = TEAL;
        state.bg_stroke = rule;
        state.fg_stroke = Stroke::new(1.0, INK);
        state.corner_radius = CornerRadius::ZERO;
        state.expansion = 0.0;
    }

    ctx.set_visuals(visuals);

    // Spacing + striping: print tables breathe, and stripe by default so the
    // CRT's alternating-column signature carries across the whole app.
    ctx.style_mut(|style| {
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        style.visuals.striped = true;
    });
}
