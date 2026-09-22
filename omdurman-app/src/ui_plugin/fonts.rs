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
    ctx.add_font(FontInsert::new(
        "Merriweather-Bold",
        egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/Merriweather-Bold.ttf"
        )),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name("Garamond".into()),
            priority: FontPriority::Lowest,
        }],
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
    // needed (panel backgrounds via `crate::ui::panel_bg`).

    // -- Period accent pass: brass instead of egui blue ----------------------
    // Every value here sits within a few luminance points of the egui dark
    // default it replaces, so contrast is unchanged -- only the hue warms up,
    // echoing the gold turn indicators and sepia chrome elsewhere in the game.
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        let v = &mut style.visuals;
        v.hyperlink_color = egui::Color32::from_rgb(196, 158, 90);
        v.selection.bg_fill = egui::Color32::from_rgb(110, 84, 30);
        v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(235, 210, 150));
        v.faint_bg_color = egui::Color32::from_rgb(35, 31, 26);
        v.extreme_bg_color = egui::Color32::from_rgb(14, 13, 11);
        v.panel_fill = egui::Color32::from_rgb(29, 27, 24);
        v.window_fill = egui::Color32::from_rgb(29, 27, 24);
        let w = &mut v.widgets;
        w.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(74, 66, 54));
        w.inactive.weak_bg_fill = egui::Color32::from_rgb(62, 56, 47);
        w.inactive.bg_fill = egui::Color32::from_rgb(62, 56, 47);
        w.hovered.weak_bg_fill = egui::Color32::from_rgb(74, 67, 56);
        w.hovered.bg_fill = egui::Color32::from_rgb(74, 67, 56);
        w.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 132, 100));
        w.active.weak_bg_fill = egui::Color32::from_rgb(58, 52, 42);
        w.active.bg_fill = egui::Color32::from_rgb(58, 52, 42);
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
