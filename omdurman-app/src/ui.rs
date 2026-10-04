//! Small shared UI helpers (palette, card chrome, panel backgrounds, section
//! headers). Kept minimal so the rest of the app pulls common chrome from one
//! place.

use bevy::prelude::{Commands, Entity};
use bevy_egui::egui;

/// App-wide palette. Every value sits within a few luminance points of the
/// egui dark default it appears over, so contrast is unchanged -- only the hue
/// warms up (the same discipline as the theme pass in `ui_plugin`). Use these
/// instead of inlining `Color32::from_rgb` so the look is tweakable in one
/// file. Constants are named by *role*, not hue; near-duplicates that used to
/// be inlined per call site are collapsed onto one role each.
pub mod palette {
    use bevy_egui::egui::Color32;

    // -- Paper (dispatch slips, combat cards, tooltips) ----------------------

    /// Card / paper background (dispatch slips, combat cards, tooltips).
    pub const PAPER: Color32 = Color32::from_rgb(0xF6, 0xED, 0xC5);
    /// Dark text on paper.
    pub const INK: Color32 = Color32::from_rgb(0x1A, 0x16, 0x10);
    /// Dimmed ink for secondary text on paper.
    pub const FAINT_INK: Color32 = Color32::from_rgb(0x6B, 0x62, 0x50);
    /// Casualty ink on paper (combat-card losses).
    pub const INK_LOSS: Color32 = Color32::from_rgb(150, 40, 40);
    /// Disrupted-unit ink on paper (hover tooltip).
    pub const INK_DISRUPTED: Color32 = Color32::from_rgb(180, 90, 90);
    /// Outstanding-requirement ink in a popup (desertion picks left).
    pub const INK_WARN: Color32 = Color32::from_rgb(180, 80, 60);
    /// Requirement-met ink in a popup (desertion complete).
    pub const INK_DONE: Color32 = Color32::from_rgb(60, 140, 60);
    /// Bonus ink on paper (brigade-integrity fire bonus).
    pub const INK_BONUS: Color32 = Color32::from_rgb(0x6B, 0x8B, 0x40);

    // -- Accents ---------------------------------------------------------------

    /// Warm accent (turn indicators, ready marks, selection, picked faction).
    pub const GOLD: Color32 = Color32::from_rgb(230, 200, 110);
    /// Brightest accent: the active toggle in the overlay chip row.
    pub const HIGHLIGHT: Color32 = Color32::from_rgb(0xFF, 0xDD, 0x44);
    /// Fill behind an active [`HIGHLIGHT`] chip.
    pub const HIGHLIGHT_BG: Color32 = Color32::from_rgba_unmultiplied_const(80, 70, 30, 200);
    /// Fill behind an inactive chip.
    pub const CHIP_BG: Color32 = Color32::from_rgba_unmultiplied_const(40, 36, 28, 180);
    /// Muted brass: inactive chip text, newspaper sub-headlines.
    pub const BRASS_DIM: Color32 = Color32::from_rgb(170, 155, 110);
    /// Brass masthead / "spectating" tag.
    pub const BRASS: Color32 = Color32::from_rgb(200, 180, 120);
    /// Khaki used for sidebar section labels and command/scope lines.
    pub const HEADING: Color32 = Color32::from_rgb(200, 200, 150);
    /// Dimmer khaki: secondary informational lines (range line, chain rule).
    pub const HEADING_DIM: Color32 = Color32::from_rgb(185, 183, 155);
    /// Title of a rail section header ([`super::section_header`]).
    pub const SECTION_TITLE: Color32 = Color32::from_rgb(218, 204, 173);
    /// Large light title on HUD / modal chrome (phase banner, newspaper
    /// headline, text-selection stroke).
    pub const TITLE: Color32 = Color32::from_rgb(230, 210, 155);
    /// Cool informational text (night rules, AI commanders, moon glyph).
    pub const INFO: Color32 = Color32::from_rgb(160, 180, 220);
    /// Night-phase accent in the phase banner.
    pub const NIGHT_BLUE: Color32 = Color32::from_rgb(100, 130, 200);
    /// Night badge fill behind [`NIGHT_BLUE`] text.
    pub const NIGHT_BADGE_BG: Color32 = Color32::from_rgba_unmultiplied_const(40, 50, 80, 200);
    /// Fill of a night turn tile on the FoK turn strip.
    pub const NIGHT_TILE: Color32 = Color32::from_rgb(40, 45, 70);
    /// Fill of a day turn tile on the FoK turn strip.
    pub const DAY_TILE: Color32 = Color32::from_rgb(70, 65, 45);
    /// Search-hit highlight (rulebook / charts), drawn with a pulsing alpha.
    pub const SEARCH_HIT: Color32 = Color32::from_rgb(0x8f, 0xc5, 0xd7);
    /// Swatch next to an AI commander in the lobby.
    pub const AI_SWATCH: Color32 = Color32::from_rgb(120, 120, 140);

    // -- Factions ----------------------------------------------------------------

    /// Anglo-Egyptian faction colour (prefer [`super::faction_color`]).
    pub const AE: Color32 = Color32::from_rgb(120, 180, 220);
    /// Dervish faction colour (prefer [`super::faction_color`]).
    pub const DERVISH: Color32 = Color32::from_rgb(220, 150, 100);

    // -- Status ------------------------------------------------------------------

    /// Positive delta (VP gains, friendly status).
    pub const GOOD: Color32 = Color32::from_rgb(120, 200, 120);
    /// Negative delta (VP losses, hostile status).
    pub const BAD: Color32 = Color32::from_rgb(200, 120, 120);
    /// Warning / disrupted / danger text.
    pub const RED: Color32 = Color32::from_rgb(200, 100, 100);
    /// A positive outcome headline (Gordon holds the Palace).
    pub const SUCCESS: Color32 = Color32::from_rgb(150, 220, 150);
    /// A negative outcome headline (Gordon fallen, LOS-refusal reason).
    pub const ALERT: Color32 = Color32::from_rgb(230, 145, 135);
    /// "Clear" status line (LOS clear).
    pub const CLEAR: Color32 = Color32::from_rgb(140, 190, 140);
    /// A refused / impossible action note (LOS blocked, no wall in range).
    pub const REFUSED: Color32 = Color32::from_rgb(200, 130, 100);
    /// Net modifier in the attacker's favour.
    pub const FAVOURABLE: Color32 = Color32::from_rgb(170, 205, 170);
    /// Net modifier against the attacker; also destructive-action titles.
    pub const UNFAVOURABLE: Color32 = Color32::from_rgb(205, 160, 120);
    /// A pending choice the player still has to make ("Select a target").
    pub const AWAITING: Color32 = Color32::from_rgb(180, 140, 100);

    // -- Dark rail / panel text ------------------------------------------------

    /// Primary text on the dark left-rail panels ([`RAIL_BG`]). The paper
    /// inks above are near-black and vanish on the rail.
    pub const RAIL_TEXT: Color32 = Color32::from_rgb(222, 214, 196);
    /// Secondary / hint text on dark warm chrome (rail, HUD, action cards).
    pub const RAIL_DIM: Color32 = Color32::from_rgb(160, 152, 136);
    /// Amber caution text (ZOC notes, refusal reasons) on the dark rail.
    pub const CAUTION: Color32 = Color32::from_rgb(220, 180, 90);
    /// Soft green "you may" hint text on the dark rail.
    pub const HINT_GREEN: Color32 = Color32::from_rgb(0x80, 0xC0, 0x80);
    /// Light text on the dark combat panels (fire tray rows).
    pub const PANEL_TEXT: Color32 = Color32::from_rgb(210, 200, 180);
    /// Dim text on the dark combat panels (modifier lines, sub-notes).
    pub const PANEL_DIM: Color32 = Color32::from_rgb(170, 160, 140);

    // -- Neutral text tiers (lobby, picker, previews, event viewer) --------------

    /// Brightest neutral text (screen titles, group headings).
    pub const TEXT_STRONG: Color32 = Color32::from_gray(225);
    /// Neutral body / section-label text.
    pub const TEXT: Color32 = Color32::from_gray(200);
    /// Neutral secondary text (detail rows, notes).
    pub const TEXT_SOFT: Color32 = Color32::from_gray(180);
    /// Neutral muted text (waiting lines, captions).
    pub const TEXT_MUTED: Color32 = Color32::from_gray(165);
    /// Neutral dim text (empty-state lines, the waiting phase banner).
    pub const TEXT_DIM: Color32 = Color32::from_gray(145);
    /// Neutral faint text (counts, "Loading...", punctuation).
    pub const TEXT_FAINT: Color32 = Color32::from_gray(120);
    /// Disabled / past text.
    pub const TEXT_DISABLED: Color32 = Color32::from_gray(105);

    // -- Action-card titles and sides --------------------------------------------

    /// Title line of a fire / melee preview card (and its net-modifier line).
    pub const CARD_TITLE: Color32 = Color32::from_rgb(235, 200, 170);
    /// Title of a construction / allocation card.
    pub const CARD_TITLE_TAN: Color32 = Color32::from_rgb(200, 180, 140);
    /// Title of an artillery-breach / river-mine card.
    pub const CARD_TITLE_RUST: Color32 = Color32::from_rgb(205, 170, 145);
    /// Attacker / friendly-side headline in a card.
    pub const ATTACKER: Color32 = Color32::from_rgb(180, 215, 180);
    /// Defender / hostile-side headline in a card.
    pub const DEFENDER: Color32 = Color32::from_rgb(225, 180, 170);
    /// Defender's CRT-band line.
    pub const DEFENDER_DIM: Color32 = Color32::from_rgb(200, 170, 170);

    // -- Action-card fills (translucent tints over the board) --------------------

    /// Fire preview card.
    pub const CARD_FIRE: Color32 = Color32::from_rgba_unmultiplied_const(40, 20, 20, 245);
    /// Declared-melee card. Near-opaque: it carries the retreat instructions
    /// and sits over the busiest part of the map (egui blends in linear
    /// light, so 220 read as roughly half transparent there).
    pub const CARD_MELEE_DECLARED: Color32 = Color32::from_rgba_unmultiplied_const(40, 30, 30, 248);
    /// Melee preview card.
    pub const CARD_MELEE: Color32 = Color32::from_rgba_unmultiplied_const(50, 30, 10, 245);
    /// Fire-allocation tray (long rows of firers and modifiers over the
    /// board: kept near-opaque, see `CARD_MELEE_DECLARED`).
    pub const CARD_ALLOCATION: Color32 = Color32::from_rgba_unmultiplied_const(30, 30, 40, 245);
    // The cards below carry rules text to read over the map: 210 blended in
    // linear light came out at about 60% (the Zariba card's text was lost
    // in the scan behind it), so they are near-opaque like the rest.
    /// Positive / friendly card (Friendlies transport, Gordon holds).
    pub const CARD_GOOD: Color32 = Color32::from_rgba_unmultiplied_const(35, 50, 30, 245);
    /// Negative card (Gordon fallen).
    pub const CARD_BAD: Color32 = Color32::from_rgba_unmultiplied_const(60, 25, 25, 245);
    /// Engineering / siege card (zariba, demolition, artillery breach).
    pub const CARD_ENGINEERING: Color32 = Color32::from_rgba_unmultiplied_const(50, 38, 30, 245);
    /// Setup-time optional-rule card (river mines / chain).
    pub const CARD_SETUP: Color32 = Color32::from_rgba_unmultiplied_const(40, 30, 40, 245);
    /// Board-anchored refusal tag (LOS blocked reason).
    pub const REFUSAL_TAG_BG: Color32 = Color32::from_rgba_premultiplied(40, 10, 10, 200);

    // -- Chrome backgrounds, borders, scrims -------------------------------------

    /// Standard side-panel (left rail) background. Warm charcoal rather than
    /// neutral gray, at the default's luminance, so the chrome sits with the
    /// game's sepia palette without costing contrast.
    pub const RAIL_BG: Color32 = Color32::from_rgb(44, 40, 35);
    /// HUD strip background (phase banner); also the theme's faint bg.
    pub const HUD_BG: Color32 = Color32::from_rgb(35, 30, 25);
    /// Modal background (victory newspaper, "Your turn" popup).
    pub const MODAL_BG: Color32 = Color32::from_rgb(42, 36, 28);
    /// Brass border of HUD / modal chrome.
    pub const CHROME_BORDER: Color32 = Color32::from_rgb(180, 160, 110);
    /// Full-width top toolbar.
    pub const TOOLBAR_BG: Color32 = Color32::from_rgba_unmultiplied_const(40, 40, 50, 220);
    /// Neutral near-black screen / sheet background (lobby, charts, timeline,
    /// event-viewer panes).
    pub const NEUTRAL_BG: Color32 = Color32::from_gray(26);
    /// Neutral fill of a resting tile / button.
    pub const NEUTRAL_FILL: Color32 = Color32::from_gray(34);
    /// Neutral fill of a raised (or hovered) tile / button.
    pub const NEUTRAL_FILL_RAISED: Color32 = Color32::from_gray(50);
    /// Neutral fill of a pressed (or hovered raised) tile / button.
    pub const NEUTRAL_FILL_PRESSED: Color32 = Color32::from_gray(67);
    /// Neutral border (chart cards, resting buttons, inactive turn tiles).
    pub const NEUTRAL_BORDER: Color32 = Color32::from_gray(90);
    /// Neutral border of a hovered button (splash).
    pub const NEUTRAL_BORDER_HOVER: Color32 = Color32::from_gray(150);
    /// Neutral border of a pressed button (splash).
    pub const NEUTRAL_BORDER_ACTIVE: Color32 = Color32::from_gray(180);
    /// Translucent black behind small HUD labels over the board.
    pub const HUD_SCRIM: Color32 = Color32::from_black_alpha(180);
    /// Scrim over a chart image around a highlighted cell.
    pub const IMAGE_SCRIM: Color32 = Color32::from_black_alpha(150);
    /// Drop shadow under floating cards.
    pub const DROP_SHADOW: Color32 = Color32::from_black_alpha(46);
    /// Tint of the drag ghost (sprite at reduced opacity; premultiplied
    /// white at alpha 180, i.e. `Color32::from_white_alpha(180)`).
    pub const GHOST_TINT: Color32 = Color32::from_rgba_premultiplied(180, 180, 180, 180);
    /// Splash / title-screen backdrop, drawn at a variable alpha.
    pub const SPLASH_BACKDROP: Color32 = Color32::from_gray(16);
    /// Splash title.
    pub const SPLASH_TITLE: Color32 = Color32::from_rgb(214, 178, 106);
    /// Splash kicker line above the title (the dim brass).
    pub const SPLASH_KICKER: Color32 = BRASS_DIM;
    /// Splash map credit, bottom-right.
    pub const SPLASH_CREDIT: Color32 = Color32::from_rgb(180, 171, 152);
    /// Fill of a disabled splash menu button.
    pub const SPLASH_BUTTON_DISABLED_FILL: Color32 = Color32::from_gray(24);
    /// Border of a disabled splash menu button.
    pub const SPLASH_BUTTON_DISABLED_BORDER: Color32 = Color32::from_gray(51);
    /// Border of the lobby's floating panel.
    pub const LOBBY_PANEL_BORDER: Color32 = Color32::from_gray(52);

    /// Current-turn marker on the board's turn track.
    pub const TURN_MARKER: Color32 = Color32::from_rgba_premultiplied(255, 100, 80, 240);

    // -- Buttons -----------------------------------------------------------------

    /// Fill of an affirmative action button (confirm, resolve, fire).
    pub const BTN_GO: Color32 = Color32::from_rgb(60, 80, 40);
    /// Fill of a neutral combat button (review allocations).
    pub const BTN_COMBAT: Color32 = Color32::from_rgb(55, 45, 45);
    /// Fill of a destructive button (remove, discard).
    pub const BTN_DANGER: Color32 = Color32::from_rgb(80, 30, 30);

    /// The global egui visuals (`ui_plugin::fonts`): warm dark widget skin.
    pub mod theme {
        use bevy_egui::egui::Color32;
        /// Hyperlinks.
        pub const LINK: Color32 = Color32::from_rgb(196, 158, 90);
        /// Text-selection / selected-tile fill.
        pub const SELECTION_BG: Color32 = Color32::from_rgb(110, 84, 30);
        /// Deepest background (text edits, scroll wells, debug overlays).
        pub const EXTREME_BG: Color32 = Color32::from_rgb(14, 13, 11);
        /// Panel and window fill.
        pub const WINDOW_BG: Color32 = Color32::from_rgb(29, 27, 24);
        /// Resting widget fill.
        pub const WIDGET_FILL: Color32 = Color32::from_rgb(62, 56, 47);
        /// Hovered widget fill / non-interactive widget border.
        pub const WIDGET_HOVER: Color32 = Color32::from_rgb(74, 67, 55);
        /// Pressed widget fill.
        pub const WIDGET_ACTIVE: Color32 = Color32::from_rgb(58, 52, 42);
        /// Hovered widget border.
        pub const WIDGET_HOVER_BORDER: Color32 = Color32::from_rgb(150, 132, 100);
    }

    /// RON syntax highlighting in the event viewer.
    pub mod syntax {
        use bevy_egui::egui::Color32;
        /// Selected event row.
        pub const SELECTION_BG: Color32 = Color32::from_rgb(90, 60, 30);
        /// String literals.
        pub const STRING: Color32 = Color32::from_rgb(206, 145, 120);
        /// Numbers.
        pub const NUMBER: Color32 = Color32::from_rgb(220, 190, 120);
        /// Keywords (`true`, `None`, ...).
        pub const KEYWORD: Color32 = Color32::from_rgb(224, 130, 60);
        /// Struct field names.
        pub const FIELD: Color32 = Color32::from_rgb(235, 200, 140);
        /// Enum variant names.
        pub const VARIANT: Color32 = Color32::from_rgb(210, 120, 90);
        /// Comments.
        pub const COMMENT: Color32 = Color32::from_rgb(150, 130, 90);
    }

    /// `color` with its alpha replaced by `alpha` (unmultiplied). For
    /// data-driven fades of an opaque palette colour.
    pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
    }

    /// An opaque `color`'s channels reused verbatim as *premultiplied* RGB
    /// with `alpha`. Not a physically correct fade (RGB is not scaled): it
    /// reproduces the additive-looking glow the splash backdrop and the
    /// "Your turn" popup were designed with.
    pub const fn with_alpha_premultiplied(color: Color32, alpha: u8) -> Color32 {
        Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), alpha)
    }
}

/// A side's faction colour -- the single source for AE / Dervish tints.
pub const fn faction_color(player: omdurman_types::Player) -> egui::Color32 {
    match player {
        omdurman_types::Player::AngloEgyptian => palette::AE,
        omdurman_types::Player::Dervish => palette::DERVISH,
    }
}

/// Shared `egui::Frame` chrome. Floating panels pick one of these instead of
/// assembling fill / radius / margin inline.
pub mod frames {
    use super::palette;
    use bevy_egui::egui;

    /// The "printed card" frame used by dispatch slips, combat cards, and the
    /// hover tooltip: paper fill over an ink stroke. Margin is left to the
    /// caller (the three surfaces use slightly different paddings).
    pub fn paper(stroke: egui::Stroke) -> egui::Frame {
        egui::Frame::new().fill(palette::PAPER).stroke(stroke)
    }

    /// A translucent action card over the board (fire / melee previews,
    /// special actions, badges), tinted by `fill` (one of the `CARD_*`
    /// palette entries).
    pub fn card(fill: egui::Color32) -> egui::Frame {
        egui::Frame::new()
            .fill(fill)
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(10, 6))
    }

    /// The HUD strip (phase banner): dark fill, brass border.
    pub fn hud() -> egui::Frame {
        egui::Frame::new()
            .fill(palette::HUD_BG)
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(20, 10))
            .stroke(egui::Stroke::new(1.0, palette::CHROME_BORDER))
    }

    /// A centred modal (victory newspaper): dark fill, heavy brass border.
    pub fn modal() -> egui::Frame {
        egui::Frame::new()
            .fill(palette::MODAL_BG)
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(32, 24))
            .stroke(egui::Stroke::new(2.0, palette::CHROME_BORDER))
    }

    /// A docked rail panel (overview, unit picker).
    pub fn rail() -> egui::Frame {
        egui::Frame::default()
            .fill(palette::RAIL_BG)
            .inner_margin(egui::Margin::symmetric(8, 8))
    }

    /// A small rail-coloured chip (e.g. the "show result" re-open button).
    pub fn chip() -> egui::Frame {
        egui::Frame::new()
            .fill(palette::RAIL_BG)
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(8, 4))
    }

    /// A tiny label tag over the board or inline in a HUD (movement cost,
    /// LOS refusal reason, night badge).
    pub fn tag(fill: egui::Color32, margin_x: i8) -> egui::Frame {
        egui::Frame::new()
            .fill(fill)
            .corner_radius(3.0)
            .inner_margin(egui::Margin::symmetric(margin_x, 2))
    }
}

/// Small `RichText` helpers for patterns repeated across panels.
pub mod text {
    use super::palette;
    use bevy_egui::egui;

    /// A bold neutral sub-heading inside a panel (lobby sections).
    pub fn subheading(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text).strong().color(palette::TEXT)
    }

    /// A small dim explanatory note under an action-card title.
    pub fn note(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text)
            .size(11.0)
            .color(palette::RAIL_DIM)
    }
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
            with_room(ui, |ui| {
                frame.show(ui, |ui| {
                    inner = Some(contents(ui));
                });
            });
        });
    inner
}

/// The width a card's text may wrap at unless the card sets its own.
pub const CARD_ROOM_WIDTH: f32 = 460.0;

/// Run `contents` with room to lay out in: down to the bottom of the screen
/// and at least [`CARD_ROOM_WIDTH`] wide. egui hands an area *last frame's*
/// size as its room, so a card could never outgrow what it once was: a
/// scroll area inside was cut off at the bottom for good (a volley of combat
/// results), and wrapped text only ever got narrower (a fire preview that
/// once said "No effect" wrapped its firers a word to a line). The area
/// re-centres / re-anchors on the grown size from the next frame on.
pub fn with_room<R>(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let screen = ui.ctx().content_rect();
    let top_left = ui.max_rect().min;
    let width = ui.max_rect().width().max(CARD_ROOM_WIDTH);
    let room = egui::Rect::from_min_max(
        top_left,
        egui::pos2(top_left.x + width, screen.max.y.max(top_left.y)),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(room), contents)
        .inner
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
    stacked_card_impl(ctx, layout, id, frame, true, contents)
}

/// A [`stacked_card`] for hover-only information (fire / melee previews):
/// pointer-transparent, so it never blocks the board. Such a card follows the
/// hovered hex, and the hex can lie *under* it -- an interactive card there
/// would swallow the very click (declare the melee, allocate the fire) the
/// preview describes. Its widgets are inert (drawn at full opacity).
pub fn passive_stacked_card<R>(
    ctx: &egui::Context,
    layout: &mut crate::ScreenLayout,
    id: impl Into<egui::Id>,
    frame: egui::Frame,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    stacked_card_impl(ctx, layout, id, frame, false, contents)
}

fn stacked_card_impl<R>(
    ctx: &egui::Context,
    layout: &mut crate::ScreenLayout,
    id: impl Into<egui::Id>,
    frame: egui::Frame,
    interactive: bool,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    let y = layout.center_stack_y;
    let mut inner = None;
    let mut height = 0.0;
    egui::Area::new(id.into())
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, y))
        .order(egui::Order::Foreground)
        .interactable(interactive)
        .show(ctx, |ui| {
            if !interactive {
                // Disabled widgets sense nothing (so egui reports no
                // interactive rect here); keep them at full opacity.
                ui.style_mut().visuals.disabled_alpha = 1.0;
                ui.disable();
            }
            let response = with_room(ui, |ui| {
                frame.show(ui, |ui| {
                    inner = Some(contents(ui));
                })
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

/// A bold section title followed by a separator. Shared by the side-panel
/// sections (overview, actions) so they read as one panel.
pub fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .font(egui::FontId::new(
                17.0,
                egui::FontFamily::Name("Garamond".into()),
            ))
            .color(palette::SECTION_TITLE),
    );
    ui.separator();
    ui.add_space(4.0);
}

#[cfg(test)]
mod tests {
    use super::CardHold;
    use bevy_egui::egui;

    /// Run `frames` egui passes on a 1200x800 screen, calling `draw` with
    /// the frame number; returns the context for inspection.
    fn headless(frames: usize, mut draw: impl FnMut(&egui::Context, usize)) -> egui::Context {
        let ctx = egui::Context::default();
        for frame in 0..frames {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200.0, 800.0),
                )),
                ..Default::default()
            };
            ctx.begin_pass(input);
            draw(&ctx, frame);
            ctx.end_pass().drop_without_applying_deltas();
        }
        ctx
    }

    /// An anchored card whose scrolling column grows (a volley of combat
    /// cards arriving) shows all of it: egui hands an area last frame's size
    /// as its room, and a scroll area inside it never grew past that -- the
    /// newest cards were cut off at the bottom.
    #[test]
    fn an_anchored_card_grows_with_its_content() {
        let lines = |frame: usize| if frame < 3 { 1 } else { 12 };
        let ctx = headless(8, |ctx, frame| {
            super::anchored_card(
                ctx,
                "growing",
                egui::Align2::RIGHT_TOP,
                egui::vec2(-10.0, 40.0),
                egui::Frame::NONE,
                |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(600.0)
                        .show(ui, |ui| {
                            for i in 0..lines(frame) {
                                ui.label(format!("line {i}"));
                            }
                        });
                },
            );
        });
        let rect = ctx
            .memory(|m| m.area_rect(egui::Id::new("growing")))
            .expect("the card was shown");
        assert!(rect.height() > 12.0 * 14.0, "card height {}", rect.height());
        assert!(rect.max.x <= 1200.0, "card runs off screen: {rect:?}");
    }

    /// A centred card (the fire / melee previews) whose text gets longer
    /// widens to fit it, rather than wrapping at the width its earlier,
    /// shorter text needed.
    #[test]
    fn a_stacked_card_widens_for_longer_text() {
        let text = |frame: usize| {
            if frame < 3 {
                "No effect".to_string()
            } else {
                "Firers: 1B First Btn, 1B Second Btn, 1B Third Btn, 1B Fourth Btn, Maxim, Maxim"
                    .to_string()
            }
        };
        let ctx = headless(8, |ctx, frame| {
            let mut layout = crate::ScreenLayout::default();
            super::stacked_card(ctx, &mut layout, "preview", egui::Frame::NONE, |ui| {
                ui.label(text(frame));
            });
        });
        let rect = ctx
            .memory(|m| m.area_rect(egui::Id::new("preview")))
            .expect("the card was shown");
        assert!(rect.width() > 300.0, "wrapped narrow: {rect:?}");
    }

    /// The room is a ceiling, not a size: a short card stays short.
    #[test]
    fn a_short_card_stays_narrow() {
        let ctx = headless(4, |ctx, _| {
            let mut layout = crate::ScreenLayout::default();
            super::stacked_card(ctx, &mut layout, "short", egui::Frame::NONE, |ui| {
                ui.label("No effect");
            });
        });
        let rect = ctx
            .memory(|m| m.area_rect(egui::Id::new("short")))
            .expect("the card was shown");
        assert!(rect.width() < 120.0, "inflated: {rect:?}");
    }

    /// The same for width: a right-anchored card that widens stays on
    /// screen and wraps at its own width, not at last frame's.
    #[test]
    fn an_anchored_card_widens_on_screen() {
        let text = |frame: usize| {
            if frame < 3 {
                "short".to_string()
            } else {
                "Fire refused -- target (29, 14) out of range from (33, 12).".repeat(2)
            }
        };
        let ctx = headless(8, |ctx, frame| {
            super::anchored_card(
                ctx,
                "widening",
                egui::Align2::RIGHT_TOP,
                egui::vec2(-10.0, 40.0),
                egui::Frame::NONE,
                |ui| {
                    ui.set_max_width(300.0);
                    ui.label(text(frame));
                },
            );
        });
        let rect = ctx
            .memory(|m| m.area_rect(egui::Id::new("widening")))
            .expect("the card was shown");
        assert!(rect.width() > 250.0, "wrapped short: {rect:?}");
        assert!(rect.max.x <= 1200.0, "card runs off screen: {rect:?}");
    }

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
