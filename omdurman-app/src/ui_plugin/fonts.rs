//! Startup appearance: window, egui fonts, touch config, and theme.
use super::*;

pub(crate) fn maximize_primary_window(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.set_maximized(true);
}

#[derive(Resource, Default)]
pub(crate) struct FontsInstalled(bool);

pub(crate) fn setup_egui_fonts(mut contexts: EguiContexts, mut installed: ResMut<FontsInstalled>) {
    if installed.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};

    // -- Inter: sans-serif UI font -------------------------------------------
    // Medium (500) is the primary weight for all UI text.
    ctx.add_font(FontInsert::new(
        "Inter-Medium",
        egui::FontData::from_static(include_bytes!("../../../assets/fonts/Inter-Medium.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Proportional,
            priority: FontPriority::Highest,
        }],
    ));

    // -- Merriweather: serif font for the splash screen ----------------------
    // Registered under "Garamond" family name so every existing reference
    // (splash screen, quoted titles) picks it up without code changes.
    ctx.add_font(FontInsert::new(
        "Merriweather-Regular",
        egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/Merriweather-Regular.ttf"
        )),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name("Garamond".into()),
            priority: FontPriority::Highest,
        }],
    ));
    // A real italic face, registered as its own family.  Italic text (the
    // splash quote, book titles) selects this family rather than egui's
    // synthetic italic -- epaint fakes italics by shearing the upright glyphs
    // without fixing advances, which left uneven gaps.  A genuine italic has
    // correct metrics.
    ctx.add_font(FontInsert::new(
        "Merriweather-Italic",
        egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/Merriweather-Italic.ttf"
        )),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name("GaramondItalic".into()),
            priority: FontPriority::Highest,
        }],
    ));
    // The bold face is a fallback of the regular family and, like the italic,
    // its own family: text that must really be bold (the splash title)
    // selects "GaramondBold".
    ctx.add_font(FontInsert::new(
        "Merriweather-Bold",
        egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/Merriweather-Bold.ttf"
        )),
        vec![
            InsertFontFamily {
                family: egui::FontFamily::Name("Garamond".into()),
                priority: FontPriority::Lowest,
            },
            InsertFontFamily {
                family: egui::FontFamily::Name("GaramondBold".into()),
                priority: FontPriority::Highest,
            },
        ],
    ));

    // -- Noto Sans Symbols 2: icon fallback ----------------------------------
    // Covers miscellaneous icons (arrows, checkmarks, warning signs, media
    // controls, emoji) that the text fonts lack.  Registered at lowest priority
    // so it only kicks in for missing glyphs.
    ctx.add_font(FontInsert::new(
        "NotoSansSymbols2",
        egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/NotoSansSymbols2-Regular.ttf"
        )),
        vec![
            InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: FontPriority::Lowest,
            },
            InsertFontFamily {
                family: egui::FontFamily::Monospace,
                priority: FontPriority::Lowest,
            },
        ],
    ));
    // NOTE: a full-app paper-skin override was tried and dropped UI contrast
    // too far, so egui keeps its default neutrals. What *is* applied is the
    // minimal accent pass below: luminance-matched warm shifts plus brass
    // selection/hyperlink accents. Per-surface colours are inlined where
    // needed (panel backgrounds via `crate::ui::palette::RAIL_BG`).

    // -- Period accent pass: brass instead of egui blue ----------------------
    // Every value here sits within a few luminance points of the egui dark
    // default it replaces, so contrast is unchanged -- only the hue warms up,
    // echoing the gold turn indicators and sepia chrome elsewhere in the game.
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        let v = &mut style.visuals;
        v.hyperlink_color = crate::ui::palette::theme::LINK;
        v.selection.bg_fill = crate::ui::palette::theme::SELECTION_BG;
        v.selection.stroke = egui::Stroke::new(1.0, crate::ui::palette::TITLE);
        v.faint_bg_color = crate::ui::palette::HUD_BG;
        v.extreme_bg_color = crate::ui::palette::theme::EXTREME_BG;
        v.panel_fill = crate::ui::palette::theme::WINDOW_BG;
        v.window_fill = crate::ui::palette::theme::WINDOW_BG;
        let w = &mut v.widgets;
        w.noninteractive.bg_stroke =
            egui::Stroke::new(1.0, crate::ui::palette::theme::WIDGET_HOVER);
        w.inactive.weak_bg_fill = crate::ui::palette::theme::WIDGET_FILL;
        w.inactive.bg_fill = crate::ui::palette::theme::WIDGET_FILL;
        w.hovered.weak_bg_fill = crate::ui::palette::theme::WIDGET_HOVER;
        w.hovered.bg_fill = crate::ui::palette::theme::WIDGET_HOVER;
        w.hovered.bg_stroke =
            egui::Stroke::new(1.0, crate::ui::palette::theme::WIDGET_HOVER_BORDER);
        w.active.weak_bg_fill = crate::ui::palette::theme::WIDGET_ACTIVE;
        w.active.bg_fill = crate::ui::palette::theme::WIDGET_ACTIVE;
    });
    installed.0 = true;
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut, unused_variables))]
pub(crate) fn configure_egui_touch(mut contexts: EguiContexts) {
    #[cfg(target_arch = "wasm32")]
    {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.spacing.interact_size = egui::vec2(40.0, 40.0);
            style.spacing.slider_width = 120.0;
        });
    }
}

pub(crate) fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(14.0),
                left: Val::Px(14.0),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            StatusPane,
        ))
        .with_child((
            StatusText,
            Text::new("Connecting..."),
            TextFont {
                font_size: FontSize::Px(22.0),
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
        ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(14.0),
                right: Val::Px(14.0),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            HexCoordPane,
        ))
        .with_child((
            HexCoordLabel,
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(16.0),
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
        ));
}
