use crate::PendingEdits;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_types::{
    BrigadeId, DervishTribe, Faction, SectionName, SpriteColor, UnitKind,
};
use strum::IntoEnumIterator;

/// Browser-local sprite annotation (omdurman-types no longer carries this).
#[derive(Clone, Debug)]
pub struct SpriteAnnotation {
    pub color: SpriteColor,
    pub faction: Option<Faction>,
    pub text: String,
    pub kind: Option<UnitKind>,
}

impl SpriteAnnotation {
    /// Re-derive the `is_boat` flag from the kind.
    pub fn is_boat(&self) -> bool {
        self.kind.as_ref().is_some_and(|k| k.is_boat())
    }

    /// Whether this annotation represents a real playable unit (not a marker,
    /// breach marker, bare counter, or unclassified).
    pub fn is_unit(&self) -> bool {
        self.kind.as_ref().is_some_and(|k| {
            !matches!(k, UnitKind::Marker | UnitKind::Breech | UnitKind::BareCounter)
        })
    }
}

/// Map from section-name + grid position to browser-local annotation.
pub type SpriteAnnotations = std::collections::HashMap<
    SectionName,
    std::collections::HashMap<(u32, u32), SpriteAnnotation>,
>;

#[derive(Resource)]
pub struct SpriteBrowser {
    pub sections: Vec<UnitSection>,
    pub selected_sprite: Option<SpriteSelection>,
}

pub struct SpriteSelection {
    pub section: usize,
    pub sprite: usize,
    pub section_name: SectionName,
    pub unit_name: String,
    pub col: u32,
    pub row: u32,
}

pub struct UnitSection {
    pub name: SectionName,
    pub width: u32,
    pub height: u32,
    pub sprites: Vec<BrowserSprite>,
}

pub struct BrowserSprite {
    pub col: u32,
    pub row: u32,
    pub filename: String,
    pub handle: Handle<Image>,
}

#[derive(Component)]
pub struct SpriteBrowserRoot;

#[derive(Component)]
pub struct SpriteScroll(pub(crate) f32);

#[derive(Component)]
pub struct SpriteScrollContent;

#[derive(Component)]
pub struct SpriteButton {
    pub section: usize,
    pub sprite: usize,
}

#[derive(Component)]
pub struct SpriteSidebar;

#[derive(Resource, Default)]
pub struct SpriteAnnotationsResource(pub SpriteAnnotations);

#[derive(Resource)]
pub struct SpriteMetaClipboard {
    pub copied: Option<SpriteAnnotation>,
    pub last_selection: Option<(SectionName, u32, u32)>,
    pub cached_annotation: Option<SpriteAnnotation>,
    col_row_label: String,
    last_color_text: String,
    last_faction_text: String,
    last_color: SpriteColor,
    last_faction: Option<Faction>,
}

impl Default for SpriteMetaClipboard {
    fn default() -> Self {
        Self {
            copied: None,
            last_selection: None,
            cached_annotation: None,
            col_row_label: String::new(),
            last_color_text: String::new(),
            last_faction_text: String::new(),
            last_color: SpriteColor::SandBlack,
            last_faction: None,
        }
    }
}

mod generated {
    include!(concat!(env!("OUT_DIR"), "/sprites.rs"));
}

/// The canonical order in which counter sections appear, top to bottom.
///
/// Both the browser (this file) and the in-game picker ([`crate::picker`])
/// iterate sections in this order, so it lives here as the single source of
/// truth rather than being duplicated.
pub fn section_order() -> &'static [SectionName] {
    &[
        SectionName::Taiasha,
        SectionName::UpperGreen,
        SectionName::KhalifaAbdullah,
        SectionName::Sherif,
        SectionName::LowerGreen,
        SectionName::UpperJaalin,
        SectionName::Hadendowa,
        SectionName::LowerJaalin,
        SectionName::HadendowaForts,
        SectionName::Baggara,
        SectionName::BritishBoats,
        SectionName::AliWadHelu,
        SectionName::BritishArmy,
        SectionName::SheikElDin,
        SectionName::Kitchener,
        SectionName::Jehadia,
        SectionName::EgyptianArmy,
    ]
}

/// A blank annotation for a not-yet-edited counter: no stats, treated as a
/// unit, independent faction.
fn default_annotation() -> SpriteAnnotation {
    SpriteAnnotation {
        color: SpriteColor::SandBlack,
        faction: None,
        text: String::new(),
        kind: Some(UnitKind::Infantry { fire: 0, melee: 0, movement: 0 }),
    }
}

fn kind_display_name(kind: &UnitKind) -> &'static str {
    match kind {
        UnitKind::Infantry { .. } => "Infantry",
        UnitKind::Cavalry { .. } => "Cavalry",
        UnitKind::Camel { .. } => "Camel",
        UnitKind::Artillery { .. } => "Artillery",
        UnitKind::Maxim { .. } => "Maxim",
        UnitKind::Gunboat { .. } => "Gunboat",
        UnitKind::Fort { .. } => "Fort",
        UnitKind::DervishLeader { .. } => "Dervish Leader",
        UnitKind::BritishLeader { .. } => "British Leader",
        UnitKind::Marker => "Marker",
        UnitKind::Breech => "Breech",
        UnitKind::BareCounter => "Bare Counter",
    }
}

impl SpriteBrowser {
    pub fn new() -> Self {
        let section_order = section_order();

        let mut section_sprites: Vec<Vec<BrowserSprite>> =
            section_order.iter().map(|_| Vec::new()).collect();

        for &(filename, col, row) in generated::SPRITE_PATHS {
            for (idx, &unit) in section_order.iter().enumerate() {
                if format!("{}_{}_{}", unit, col, row) == filename {
                    section_sprites[idx].push(BrowserSprite {
                        col,
                        row,
                        filename: filename.to_string(),
                        handle: Handle::default(),
                    });
                    break;
                }
            }
        }

        let sections: Vec<UnitSection> = section_order
            .iter()
            .zip(section_sprites)
            .map(|(&name, mut sprites)| {
                let max_col = sprites.iter().map(|s| s.col).max().unwrap_or(0);
                let max_row = sprites.iter().map(|s| s.row).max().unwrap_or(0);
                let w = max_col + 1;
                let h = max_row + 1;
                sprites.sort_by_key(|s| (s.row, s.col));
                UnitSection {
                    name,
                    width: w,
                    height: h,
                    sprites,
                }
            })
            .collect();

        SpriteBrowser {
            sections,
            selected_sprite: None,
        }
    }
}

pub fn spawn_sprite_browser(
    mut commands: Commands,
    mut browser: ResMut<SpriteBrowser>,
    asset_server: Res<AssetServer>,
) {
    for section in &mut browser.sections {
        for sprite in &mut section.sprites {
            if sprite.handle.id() != AssetId::<Image>::default() {
                continue;
            }
            let path = format!("sprites/{}.webp", sprite.filename);
            sprite.handle = asset_server.load(&path);
        }
    }

    let header_color = Color::srgb_u8(200, 200, 200);
    let sprite_size = 76.0;
    let sprite_margin = 2.0;
    let item_width = sprite_size + 2.0 * sprite_margin;

    commands
        .spawn((
            SpriteBrowserRoot,
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(20, 20, 30)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                SpriteSidebar,
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    right: Val::Px(0.0),
                    width: Val::Px(280.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(30, 30, 40)),
                Visibility::Hidden,
            ));

            parent
                .spawn((
                    SpriteScrollContent,
                    SpriteScroll(0.0),
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        padding: UiRect {
                            left: Val::Percent(5.0),
                            top: Val::Px(20.0),
                            right: Val::Px(20.0),
                            bottom: Val::Px(20.0),
                        },
                        ..default()
                    },
                ))
                .with_children(|inner| {
                    for (section_idx, section) in browser.sections.iter().enumerate() {
                        inner.spawn((
                            Text::new(section.name.display_name()),
                            TextFont {
                                font_size: FontSize::Px(18.0),
                                ..default()
                            },
                            TextColor(header_color),
                            Node {
                                margin: UiRect {
                                    top: Val::Px(20.0),
                                    bottom: Val::Px(4.0),
                                    ..default()
                                },
                                ..default()
                            },
                        ));

                        let grid_width = section.width as f32 * item_width;

                        inner
                            .spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                width: Val::Px(grid_width),
                                margin: UiRect {
                                    bottom: Val::Px(16.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|grid| {
                                for (sprite_idx, sprite) in section.sprites.iter().enumerate() {
                                    grid.spawn((
                                        Node {
                                            width: Val::Px(sprite_size),
                                            height: Val::Px(sprite_size),
                                            margin: UiRect::all(Val::Px(sprite_margin)),
                                            ..default()
                                        },
                                        Button,
                                        SpriteButton {
                                            section: section_idx,
                                            sprite: sprite_idx,
                                        },
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            ImageNode {
                                                image: sprite.handle.clone(),
                                                ..default()
                                            },
                                            Node {
                                                width: Val::Px(sprite_size),
                                                height: Val::Px(sprite_size),
                                                ..default()
                                            },
                                        ));
                                    });
                                }
                            });
                    }
                });
        });
}

pub fn handle_sprite_clicks(
    mut browser: ResMut<SpriteBrowser>,
    buttons: Query<(&SpriteButton, &Interaction), Changed<Interaction>>,
) {
    for (button, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(section) = browser.sections.get(button.section) else {
            continue;
        };
        let Some(sprite) = section.sprites.get(button.sprite) else {
            continue;
        };
        browser.selected_sprite = Some(SpriteSelection {
            section: button.section,
            sprite: button.sprite,
            section_name: section.name,
            unit_name: section.name.display_name().to_string(),
            col: sprite.col,
            row: sprite.row,
        });
    }
}

pub fn update_sprite_selection_marker(
    mut commands: Commands,
    browser: Res<SpriteBrowser>,
    buttons: Query<(Entity, &SpriteButton)>,
    marked: Query<Entity, (With<SpriteButton>, With<Outline>)>,
) {
    for e in &marked {
        commands.entity(e).remove::<Outline>();
    }
    if let Some(ref sel) = browser.selected_sprite {
        for (entity, button) in &buttons {
            if button.section == sel.section && button.sprite == sel.sprite {
                commands.entity(entity).insert(Outline {
                    width: Val::Px(2.0),
                    offset: Val::Px(0.0),
                    color: Color::srgb_u8(230, 50, 50),
                });
                break;
            }
        }
    }
}

pub fn navigate_sprite_selection(
    keys: Res<ButtonInput<KeyCode>>,
    mut browser: ResMut<SpriteBrowser>,
    root_q: Query<&Visibility, With<SpriteBrowserRoot>>,
    mut contexts: EguiContexts,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    let Ok(vis) = root_q.single() else { return };
    if *vis != Visibility::Visible {
        return;
    }
    let sel = match browser.selected_sprite.as_ref() {
        Some(s) => (s.section, s.col, s.row),
        None => return,
    };
    let Some(section) = browser.sections.get(sel.0) else {
        return;
    };
    if section.sprites.is_empty() {
        return;
    }

    let (new_col, new_row) = if keys.just_pressed(KeyCode::ArrowLeft) {
        if sel.1 > 0 {
            (sel.1 - 1, sel.2)
        } else {
            return;
        }
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        if sel.1 + 1 < section.width {
            (sel.1 + 1, sel.2)
        } else {
            return;
        }
    } else if keys.just_pressed(KeyCode::ArrowUp) {
        if sel.2 > 0 {
            (sel.1, sel.2 - 1)
        } else {
            return;
        }
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        if sel.2 + 1 < section.height {
            (sel.1, sel.2 + 1)
        } else {
            return;
        }
    } else {
        return;
    };

    if let Some(sprite_idx) = section
        .sprites
        .iter()
        .position(|s| s.col == new_col && s.row == new_row)
    {
        let sprite = &section.sprites[sprite_idx];
        browser.selected_sprite = Some(SpriteSelection {
            section: sel.0,
            sprite: sprite_idx,
            section_name: section.name,
            unit_name: section.name.display_name().to_string(),
            col: sprite.col,
            row: sprite.row,
        });
    }
}

pub fn sprite_meta_editor_ui(
    mut contexts: EguiContexts,
    browser: Res<SpriteBrowser>,
    mut annotations: Option<ResMut<SpriteAnnotationsResource>>,
    mut clipboard: ResMut<SpriteMetaClipboard>,
    root_q: Query<&Visibility, With<SpriteBrowserRoot>>,
    _pending: ResMut<PendingEdits>,
) {
    let Ok(vis) = root_q.single() else { return };
    let browser_visible = *vis == Visibility::Visible;
    if !browser_visible {
        return;
    }
    let Some(ref sel) = browser.selected_sprite else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let Some(ref mut annotations) = annotations else {
        return;
    };

    let selection_changed = clipboard
        .last_selection
        .as_ref()
        .is_none_or(|(name, col, row)| {
            name != &sel.section_name || *col != sel.col || *row != sel.row
        });
    if selection_changed {
        let entry = annotations
            .0
            .get(&sel.section_name)
            .and_then(|m| m.get(&(sel.col, sel.row)));
        let m = entry.cloned().unwrap_or_else(default_annotation);
        clipboard.last_color = m.color;
        clipboard.last_faction = m.faction;
        clipboard.last_color_text = m.color.to_string();
        clipboard.last_faction_text = m.faction.map(|f| f.to_string()).unwrap_or_default();
        clipboard.cached_annotation = Some(m);
        clipboard.last_selection = Some((sel.section_name, sel.col, sel.row));
        clipboard.col_row_label = format!("Col: {}, Row: {}", sel.col, sel.row);
    }
    // take cached annotation out (avoids per-frame clone when unchanged)
    let mut meta = clipboard
        .cached_annotation
        .take()
        .unwrap_or_else(default_annotation);

    let mut changed = false;
    // Track whether only the numeric stat fields (fire/melee/movement)
    // changed, so we can defer remote updates until the drag is finished.
    let mut stats_changed = false;

    let mut __ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("browser_panel"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    egui::Panel::right("sprite_meta_panel")
        .resizable(true)
        .default_size(280.0)
        .size_range(200.0..=500.0)
        .frame(
            egui::Frame::default()
                .fill(crate::ui::panel_bg())
                .inner_margin(egui::Margin::symmetric(16, 16)),
        )
        .show(&mut __ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));

            // unit name
            ui.label(
                egui::RichText::new(&sel.unit_name)
                    .size(18.0)
                    .color(egui::Color32::from_gray(220)),
            );
            ui.add_space(4.0);

            // grid info
            ui.label(
                egui::RichText::new(&clipboard.col_row_label)
                    .size(14.0)
                    .color(egui::Color32::from_gray(180)),
            );
            ui.add_space(8.0);

            // color
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("color").color(egui::Color32::from_gray(200)));
                egui::ComboBox::from_id_salt("sprite_color")
                    .selected_text(&clipboard.last_color_text)
                    .width(160.0)
                    .show_ui(ui, |ui| {
                        for c in SpriteColor::iter() {
                            if ui
                                .selectable_value(&mut meta.color, c, c.to_string())
                                .clicked()
                            {
                                changed = true;
                            }
                        }
                    });
            });

            // faction
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("faction").color(egui::Color32::from_gray(200)));
                egui::ComboBox::from_id_salt("sprite_faction")
                    .selected_text(&clipboard.last_faction_text)
                    .width(160.0)
                    .show_ui(ui, |ui| {
                        let is_dervish = matches!(meta.faction, Some(Faction::Dervish { .. }));
                        if ui.selectable_label(is_dervish, "Dervish").clicked() && !is_dervish {
                            meta.faction = Some(Faction::Dervish {
                                tribe: DervishTribe::Baggara,
                            });
                            changed = true;
                        }
                        let is_be = matches!(meta.faction, Some(Faction::BritishEgyptian { .. }));
                        if ui.selectable_label(is_be, "BritishEgyptian").clicked() && !is_be {
                            meta.faction = Some(Faction::BritishEgyptian {
                                brigade: None,
                            });
                            changed = true;
                        }
                    });
            });

            // tribe picker (Dervish only)
            if let Some(Faction::Dervish { tribe }) = &mut meta.faction {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("tribe").color(egui::Color32::from_gray(200)));
                    egui::ComboBox::from_id_salt("sprite_tribe")
                        .selected_text(tribe.to_string())
                        .width(120.0)
                        .show_ui(ui, |ui| {
                            for t in DervishTribe::iter() {
                                if ui.selectable_value(tribe, t, t.to_string()).clicked() {
                                    changed = true;
                                }
                            }
                        });
                });
            }

            // brigade picker (BritishEgyptian infantry only)
            if let Some(Faction::BritishEgyptian { brigade }) = &mut meta.faction
                && matches!(meta.kind, Some(UnitKind::Infantry { .. }))
            {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("brigade").color(egui::Color32::from_gray(200)));
                    egui::ComboBox::from_id_salt("sprite_brigade")
                        .selected_text(
                            brigade
                                .map(|b| b.to_string())
                                .unwrap_or_else(|| "--".to_string()),
                        )
                        .width(60.0)
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(brigade, None, "--").clicked() {
                                changed = true;
                            }
                            for b in BrigadeId::ALL {
                                if ui
                                    .selectable_value(brigade, Some(b), b.to_string())
                                    .clicked()
                                {
                                    changed = true;
                                }
                            }
                            // Friendlies -- separately, since §5.54 does not
                            // enumerate it as an integrity-eligible brigade.
                            let friendlies = BrigadeId::friendlies();
                            if ui
                                .selectable_value(brigade, Some(friendlies), "F")
                                .clicked()
                            {
                                changed = true;
                            }
                        });
                });
            }

            // text
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("text").color(egui::Color32::from_gray(200)));
                if ui
                    .add(egui::TextEdit::singleline(&mut meta.text).desired_width(f32::INFINITY))
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("kind").color(egui::Color32::from_gray(200)));
                let kind_name = meta.kind.as_ref().map(kind_display_name).unwrap_or("");
                egui::ComboBox::from_id_salt("sprite_kind")
                    .selected_text(kind_name)
                    .width(160.0)
                    .show_ui(ui, |ui| {
                        let kind_options: &[(&str, Option<UnitKind>, &str)] = &[
                            ("", None, "Unclassified counter"),
                            ("Infantry", Some(UnitKind::Infantry { fire: 0, melee: 0, movement: 0 }), "§2.3 — fire / melee / movement.\nRifles or Spears weapon depending on tribe."),
                            ("Cavalry", Some(UnitKind::Cavalry { fire: 0, melee: 0, movement: 0 }), "§2.3 — fire / melee / movement.\nMay retreat before melee (§7.5)."),
                            ("Camel", Some(UnitKind::Camel { fire: 0, melee: 0, movement: 0 }), "§2.3 — fire / melee / movement.\nMay retreat before melee (§7.5)."),
                            ("Artillery", Some(UnitKind::Artillery { fire: 0, melee: 0, movement: 0 }), "§2.31 — fire / melee / movement.\nFires on the Artillery CRT line."),
                            ("Maxim", Some(UnitKind::Maxim { fire: 0, melee: 0, movement: 0 }), "§6.42 — fire / melee / movement.\nFires TWICE per turn (x2).\nFires on the Maxims CRT line."),
                            ("Gunboat", Some(UnitKind::Gunboat { fire: 0, upstream: 0, downstream: 0 }), "§2.32 — gunboat (old & named share this kind).\nfire / upstream / downstream (§5.24).\nNo melee (§7.1). Fires on Artillery line.\nNamed (new-type) boats also fire howitzer\n(§6.64); that is derived from identity."),
                            ("Fort", Some(UnitKind::Fort { fire: 0, melee: 0 }), "§6.54 — permanent emplacement.\nfire (artillery) / melee (defensive, -3).\nMay not move once placed (§5.25)."),
                            ("Dervish Leader", Some(UnitKind::DervishLeader { fire: 0, melee: 0, movement: 0 }), "§6.51 — fire / melee / movement.\nMay melee attack (§7.4)."),
                            ("British Leader", Some(UnitKind::BritishLeader { movement: 0 }), "§6.51 — movement factor only.\nNo fire or melee. Exerts no ZOC (§5.41)."),
                            ("Marker", Some(UnitKind::Marker), "Non-unit marker (objective token, etc.).\nNo stats, not placed as a unit."),
                            ("Breech", Some(UnitKind::Breech), "§6.63 — wall-breach marker placed\nby artillery fire. Not a combat unit."),
                            ("Bare Counter", Some(UnitKind::BareCounter), "Non-playable bare print-run duplicate.\nHidden from the unit picker."),
                        ];
                        for (name, default_kind, help) in kind_options {
                            let is_selected = match (&meta.kind, default_kind) {
                                (Some(k), Some(d)) => std::mem::discriminant(k) == std::mem::discriminant(d),
                                (None, None) => true,
                                _ => false,
                            };
                            let mut response = ui.selectable_label(is_selected, *name);
                            if !help.is_empty() {
                                response = response.on_hover_text(*help);
                            }
                            if response.clicked() {
                                meta.kind = *default_kind;
                                changed = true;
                            }
                        }
                    });
            });

            let is_maxim = matches!(meta.kind, Some(UnitKind::Maxim { .. }));
            // Named-ness is an identity trait (GunboatId::Named), not a kind --
            // derive it from the selected cell's resolved profile so the
            // howitzer help text still shows for the 5 named-boat cells.
            let is_named_gunboat =
                omdurman_rules::unit_id_for_section_pos(sel.section_name, sel.col as u8, sel.row as u8)
                    .and_then(omdurman_rules::unit_profiles::profile_for_unit)
                    .is_some_and(|p| {
                        matches!(
                            p.identity,
                            omdurman_rules::UnitIdentity::AngloEgyptianGunboat(
                                omdurman_rules::GunboatId::Named(_)
                            )
                        )
                    });

            match &mut meta.kind {
                Some(UnitKind::Infantry { fire, melee, movement })
                | Some(UnitKind::Cavalry { fire, melee, movement })
                | Some(UnitKind::Camel { fire, melee, movement })
                | Some(UnitKind::Artillery { fire, melee, movement })
                | Some(UnitKind::Maxim { fire, melee, movement })
                | Some(UnitKind::DervishLeader { fire, melee, movement }) => {
                    ui.horizontal(|ui| {
                        ui.label("fire");
                        ui.label(egui::RichText::new("?").small().color(egui::Color32::from_gray(150)))
                            .on_hover_text("§6.11 — fire combat factor.\nPrinted on the counter.\nFor Maxim: fires x2 per turn (§6.42).");
                        if ui.add(egui::DragValue::new(fire).speed(1).range(0..=15)).changed() {
                            changed = true;
                            stats_changed = true;
                        }
                        if is_maxim {
                            ui.label(egui::RichText::new("x2").strong());
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("melee");
                        ui.label(egui::RichText::new("?").small().color(egui::Color32::from_gray(150)))
                            .on_hover_text("§7.1 — melee combat factor.\nInfantry, cavalry, camel and\nDervish leaders may melee attack (§7.4).\nCavalry/camel may retreat before melee (§7.5).");
                        if ui.add(egui::DragValue::new(melee).speed(1).range(0..=15)).changed() {
                            changed = true;
                            stats_changed = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("movement");
                        ui.label(egui::RichText::new("?").small().color(egui::Color32::from_gray(150)))
                            .on_hover_text("§5.11 — movement allowance (MP).\nPrinted on the counter. Terrain costs\nare deducted from this on entry (§5.11).");
                        if ui.add(egui::DragValue::new(movement).speed(1).range(0..=99)).changed() {
                            changed = true;
                            stats_changed = true;
                        }
                    });
                }
                Some(UnitKind::Gunboat { fire, upstream, downstream }) => {
                    ui.horizontal(|ui| {
                        ui.label("fire");
                        ui.label(egui::RichText::new("?").small().color(egui::Color32::from_gray(150)))
                            .on_hover_text(if is_named_gunboat {
                                "§6.64 — artillery fire factor.\nFires as direct fire (Artillery line),\nthen fires the same factor again as\nhowitzer in the Maxim Second Fire\nsubphase."
                            } else {
                                "§2.32 — fire factor (Artillery CRT line).\nOld gunboats fire on the Artillery line.\nCannot fire as howitzer."
                            });
                        if ui.add(egui::DragValue::new(fire).speed(1).range(0..=15)).changed() {
                            changed = true;
                            stats_changed = true;
                        }
                        if is_named_gunboat {
                            ui.label(egui::RichText::new("howitzer").strong());
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("upstream");
                        ui.label(egui::RichText::new("?").small().color(egui::Color32::from_gray(150)))
                            .on_hover_text("§5.24 — movement allowance when\nmoving upstream (smaller number).\ne.g. counter shows '10/16' → upstream = 10.");
                        if ui.add(egui::DragValue::new(upstream).speed(1).range(0..=99)).changed() {
                            changed = true;
                            stats_changed = true;
                        }
                        ui.label("downstream");
                        ui.label(egui::RichText::new("?").small().color(egui::Color32::from_gray(150)))
                            .on_hover_text("§5.24 — movement allowance when\nmoving downstream (larger number).\ne.g. counter shows '10/16' → downstream = 16.");
                        if ui.add(egui::DragValue::new(downstream).speed(1).range(0..=99)).changed() {
                            changed = true;
                            stats_changed = true;
                        }
                    });
                }
                Some(UnitKind::Fort { fire, melee }) => {
                    ui.horizontal(|ui| {
                        ui.label("fire");
                        ui.label(egui::RichText::new("?").small().color(egui::Color32::from_gray(150)))
                            .on_hover_text("§6.54 — artillery fire factor.\nFires on the Artillery CRT line.\nMay fire normally each turn.");
                        if ui.add(egui::DragValue::new(fire).speed(1).range(0..=15)).changed() {
                            changed = true;
                            stats_changed = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("melee");
                        ui.label(egui::RichText::new("?").small().color(egui::Color32::from_gray(150)))
                            .on_hover_text("§6.54 — defensive melee value only.\nAttacker applies -3 modifier in melee\nagainst forts. Forts may NOT melee attack.");
                        if ui.add(egui::DragValue::new(melee).speed(1).range(0..=15)).changed() {
                            changed = true;
                            stats_changed = true;
                        }
                    });
                }
                Some(UnitKind::BritishLeader { movement }) => {
                    ui.horizontal(|ui| {
                        ui.label("movement");
                        ui.label(egui::RichText::new("?").small().color(egui::Color32::from_gray(150)))
                            .on_hover_text("§6.51 — movement factor only.\nBritish leaders print no fire or melee.\nExerts no ZOC (§5.41).");
                        if ui.add(egui::DragValue::new(movement).speed(1).range(0..=99)).changed() {
                            changed = true;
                            stats_changed = true;
                        }
                    });
                }
                Some(UnitKind::Marker) | Some(UnitKind::Breech) | Some(UnitKind::BareCounter) | None => {}
            }

            ui.add_space(16.0);

            // copy / paste buttons
            ui.horizontal(|ui| {
                if ui.button("[Copy Meta]").clicked() {
                    // Capture the live in-form annotation (including any
                    // just-typed fire/melee/movement/upstream/downstream),
                    // not the last-committed snapshot in `cached_annotation`.
                    clipboard.copied = Some(meta.clone());
                }
                if ui.button("[Paste Meta]").clicked()
                    && let Some(ref data) = clipboard.copied
                {
                    meta = data.clone();
                    changed = true;
                }
            });
        });

    // For local state (UI + annotations resource), apply changes immediately so
    // the editor reacts in real time while dragging.
    if changed {
        clipboard.cached_annotation = Some(meta.clone());
        clipboard.last_color = meta.color;
        clipboard.last_faction = meta.faction;
        clipboard.last_color_text = meta.color.to_string();
        clipboard.last_faction_text = meta.faction.map(|f| f.to_string()).unwrap_or_default();
    }
    annotations
        .0
        .entry(sel.section_name)
        .or_default()
        .insert((sel.col, sel.row), meta.clone());

    // Remote/net + on-disk persistence:
    // - non-stat edits (color, faction, text, flags) are committed immediately
    // - stat edits (fire/melee/movement) only emit once the drag is finished
    //   (pointer released) to avoid spamming remote peers.
    // Remote broadcast of sprite annotations is no longer supported
    // (AnnotateSprite was removed from GameEvent). The local resource
    // is updated above; stats edits are committed locally on every change.

    if !changed {
        // Preserve cached annotation when no fields changed this frame.
        clipboard.cached_annotation = Some(meta);
    }
}

pub fn scroll_sprite_browser(
    mut scroll_events: MessageReader<MouseWheel>,
    mut content_q: Query<(&mut SpriteScroll, &mut Node, &ComputedNode), With<SpriteScrollContent>>,
    root_q: Query<&Visibility, With<SpriteBrowserRoot>>,
) {
    let Ok(visibility) = root_q.single() else {
        return;
    };
    if *visibility != Visibility::Visible {
        return;
    }
    let Ok((mut scroll, mut node, _)) = content_q.single_mut() else {
        return;
    };
    let mut total = 0.0;
    let mut is_pixel = false;
    for ev in scroll_events.read() {
        if ev.unit == MouseScrollUnit::Pixel {
            is_pixel = true;
        }
        total += ev.y;
    }
    if total == 0.0 {
        return;
    }
    let scale = if is_pixel { 1.0 } else { 30.0 };
    scroll.0 = (scroll.0 - total * scale).max(0.0);
    node.top = Val::Px(-scroll.0);
}
