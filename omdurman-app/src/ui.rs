//! Small shared UI helpers (palette, card chrome, panel backgrounds, section
//! headers). Kept minimal so the rest of the app pulls common chrome from one
//! place.

use bevy::prelude::{Commands, Entity};
use bevy_egui::egui;

/// App-wide palette. Every value sits within a few luminance points of the
/// egui dark default it appears over, so contrast is unchanged -- only the hue
/// warms up (the same discipline as the theme pass in `ui_plugin`). Use these
/// instead of inlining `Color32::from_rgb` so the look is tweakable in one
/// file.
pub mod palette {
    use bevy_egui::egui::Color32;

    /// Card / paper background (dispatch slips, combat cards, tooltips).
    pub const PAPER: Color32 = Color32::from_rgb(0xF6, 0xED, 0xC5);
    /// Dark text on paper.
    pub const INK: Color32 = Color32::from_rgb(0x1A, 0x16, 0x10);
    /// Dimmed ink for secondary text on paper.
    pub const FAINT_INK: Color32 = Color32::from_rgb(0x6B, 0x62, 0x50);

    /// Warm accent (turn indicators, ready marks, selection).
    pub const GOLD: Color32 = Color32::from_rgb(230, 200, 110);
    /// Khaki used for sidebar section labels.
    pub const HEADING: Color32 = Color32::from_rgb(200, 200, 150);

    /// Anglo-Egyptian faction colour.
    pub const AE: Color32 = Color32::from_rgb(120, 180, 220);
    /// Dervish faction colour.
    pub const DERVISH: Color32 = Color32::from_rgb(220, 150, 100);

    /// Positive delta (VP gains, friendly status).
    pub const GOOD: Color32 = Color32::from_rgb(120, 200, 120);
    /// Negative delta (VP losses, hostile status).
    pub const BAD: Color32 = Color32::from_rgb(200, 120, 120);
    /// Warning / disrupted / danger text.
    pub const RED: Color32 = Color32::from_rgb(200, 100, 100);

    /// Primary text on the dark left-rail panels (`panel_bg`). The paper
    /// inks above are near-black and vanish on the rail.
    pub const RAIL_TEXT: Color32 = Color32::from_rgb(222, 214, 196);
    /// Secondary / hint text on the dark rail.
    pub const RAIL_DIM: Color32 = Color32::from_rgb(160, 152, 136);
    /// Amber caution text (ZOC notes, refusal reasons) on the dark rail.
    pub const CAUTION: Color32 = Color32::from_rgb(220, 180, 90);
    /// Soft green "you may" hint text on the dark rail.
    pub const HINT_GREEN: Color32 = Color32::from_rgb(0x80, 0xC0, 0x80);

    /// Fill of an affirmative action button (confirm, resolve, fire).
    pub const BTN_GO: Color32 = Color32::from_rgb(60, 80, 40);
    /// Fill of a neutral combat button (review allocations).
    pub const BTN_COMBAT: Color32 = Color32::from_rgb(55, 45, 45);
    /// Fill of a destructive button (remove, discard).
    pub const BTN_DANGER: Color32 = Color32::from_rgb(80, 30, 30);

    /// Light text on the dark combat panels (fire tray rows).
    pub const PANEL_TEXT: Color32 = Color32::from_rgb(210, 200, 180);
    /// Dim text on the dark combat panels.
    pub const PANEL_DIM: Color32 = Color32::from_rgb(170, 160, 140);
}

/// Full display name of a side ("Anglo-Egyptian" / "Dervish"). The derived
/// `Display` on `Player` prints the Rust identifier ("AngloEgyptian"), which
/// is not for players' eyes.
pub const fn faction_name(player: omdurman_types::Player) -> &'static str {
    match player {
        omdurman_types::Player::AngloEgyptian => "Anglo-Egyptian",
        omdurman_types::Player::Dervish => "Dervish",
    }
}

/// Compact side label for tight rail rows ("A-E" / "Dervish").
pub const fn faction_abbrev(player: omdurman_types::Player) -> &'static str {
    match player {
        omdurman_types::Player::AngloEgyptian => "A-E",
        omdurman_types::Player::Dervish => "Dervish",
    }
}

/// The "printed card" frame used by dispatch slips, combat cards, and the
/// hover tooltip: paper fill over an ink stroke. Margin is left to the caller
/// (the three surfaces use slightly different paddings).
pub fn paper_frame(stroke: egui::Stroke) -> egui::Frame {
    egui::Frame::new().fill(palette::PAPER).stroke(stroke)
}

/// Hover / pin state of a transient card (dispatch slip, combat card). A
/// card under the pointer stops ageing, and a click on it pins it until
/// clicked again, so a result can be read at leisure.
#[derive(Default, Clone, Copy, Debug)]
pub struct CardHold {
    /// Clicked to stay: never expires until unpinned.
    pub pinned: bool,
    /// Under the pointer last frame.
    pub hovered: bool,
    /// The card's rect last frame (the click target registered before the
    /// card's content, so the content's own links stay on top of it).
    pub rect: Option<egui::Rect>,
}

impl CardHold {
    /// Whether the card must not age this frame.
    pub fn is_held(&self) -> bool {
        self.pinned || self.hovered
    }

    /// Advance `age` by `dt` unless held; a held card is also pulled back out
    /// of its fade so it reads at full strength.
    pub fn age(&self, age: &mut f32, dt: f32, ttl: f32, fade: f32) {
        if self.is_held() {
            *age = age.min(ttl - fade);
        } else {
            *age += dt;
        }
    }

    /// Register the card's click target (from last frame's rect) *before*
    /// its content is drawn; a click toggles the pin.
    pub fn begin(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        if let Some(rect) = self.rect
            && ui
                .interact(rect, id, egui::Sense::click())
                .on_hover_text(if self.pinned {
                    "Pinned — click to unpin"
                } else {
                    "Click to pin"
                })
                .clicked()
        {
            self.pinned = !self.pinned;
        }
    }

    /// Record this frame's card rect and hover state.
    pub fn end(&mut self, ui: &egui::Ui, rect: egui::Rect) {
        self.rect = Some(rect);
        self.hovered = ui.rect_contains_pointer(rect);
    }
}

/// Show `contents` in a foreground-ordered `egui::Area` pinned to a screen
/// edge, wrapped in `frame`. Collapses the Area+Frame boilerplate repeated by
/// every floating panel (preview cards, badges, modals); returns what
/// `contents` returned, or `None` if egui discarded the pass.
pub fn anchored_card<R>(
    ctx: &egui::Context,
    id: impl Into<egui::Id>,
    anchor: egui::Align2,
    offset: impl Into<egui::Vec2>,
    frame: egui::Frame,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    let mut inner = None;
    egui::Area::new(id.into())
        .anchor(anchor, offset.into())
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            frame.show(ui, |ui| {
                inner = Some(contents(ui));
            });
        });
    inner
}

/// Like [`anchored_card`] at `CENTER_TOP`, but anchored at the shared
/// [`ScreenLayout::center_stack_y`] cursor and advancing it by the card's
/// height, so simultaneous top-center cards (phase banner, fire/melee
/// previews, prompts, badges) stack downward instead of superimposing.
pub fn stacked_card<R>(
    ctx: &egui::Context,
    layout: &mut crate::ScreenLayout,
    id: impl Into<egui::Id>,
    frame: egui::Frame,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    let y = layout.center_stack_y;
    let mut inner = None;
    let mut height = 0.0;
    egui::Area::new(id.into())
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, y))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let response = frame.show(ui, |ui| {
                inner = Some(contents(ui));
            });
            height = response.response.rect.height();
        });
    if height > 0.0 {
        layout.center_stack_y = y + height + crate::layout::STACK_GAP;
    }
    inner
}

/// Despawn every entity in `entities` via deferred commands. Used by the
/// overlay systems that rebuild their meshes from scratch each change (despawn
/// all, then respawn). Centralising the loop here leaves a single seam for the
/// eventual pool-ification of these overlays.
pub fn despawn_all(commands: &mut Commands, entities: &[Entity]) {
    for &e in entities {
        commands.entity(e).despawn();
    }
}

/// Standard side-panel background colour used across the app.
/// Warm charcoal rather than neutral gray, at the default's luminance, so the
/// chrome sits with the game's sepia palette without costing contrast.
pub fn panel_bg() -> egui::Color32 {
    egui::Color32::from_rgb(44, 40, 35)
}

/// A bold section title followed by a separator. Shared by the side-panel
/// sections (overview, actions) so they read as one panel.
pub fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .font(egui::FontId::new(
                17.0,
                egui::FontFamily::Name("Garamond".into()),
            ))
            .color(egui::Color32::from_rgb(218, 204, 173)),
    );
    ui.separator();
    ui.add_space(4.0);
}

#[cfg(test)]
mod tests {
    use super::CardHold;

    #[test]
    fn held_cards_do_not_age_and_leave_their_fade() {
        let (ttl, fade) = (6.0, 1.0);
        let mut age = 5.5; // mid-fade
        let mut hold = CardHold::default();
        hold.age(&mut age, 0.25, ttl, fade);
        assert_eq!(age, 5.75, "an idle card ages");
        hold.hovered = true;
        hold.age(&mut age, 10.0, ttl, fade);
        assert_eq!(age, ttl - fade, "hover freezes it at full strength");
        hold.hovered = false;
        hold.pinned = true;
        hold.age(&mut age, 10.0, ttl, fade);
        assert_eq!(age, ttl - fade, "a pinned card never expires");
    }

    #[test]
    fn faction_names_are_for_humans() {
        use omdurman_types::Player;
        assert_eq!(super::faction_name(Player::AngloEgyptian), "Anglo-Egyptian");
        assert_eq!(super::faction_abbrev(Player::AngloEgyptian), "A-E");
        assert_eq!(super::faction_name(Player::Dervish), "Dervish");
    }
}
