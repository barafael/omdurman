use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_map::GameMap;
use omdurman_types::SpriteAnnotations as Annotations;
use omdurman_types::{Faction, IntoEnumIterator, SpriteAnnotation, SpriteColor};
use crate::PendingEdits;
#[cfg(not(target_arch = "wasm32"))]
use omdurman_map::save_annotations_to_file;
#[cfg(not(target_arch = "wasm32"))]
use crate::editor::ANNOTATIONS_SAVE_PATH;
use omdurman_net::NetMsg;

#[derive(Resource)]
pub struct SpriteBrowser {
    pub sections: Vec<UnitSection>,
    pub selected_sprite: Option<SpriteSelection>,
}

pub struct SpriteSelection {
    pub section: usize,
    pub sprite: usize,
    pub section_name: String,
    pub unit_name: String,
    pub col: u32,
    pub row: u32,
}

#[allow(dead_code)]
pub struct UnitSection {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub sprites: Vec<BrowserSprite>,
}

#[allow(dead_code)]
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

#[derive(Resource)]
pub struct SpriteAnnotationsResource(pub Annotations);

#[derive(Resource)]
pub struct SpriteMetaClipboard {
    pub copied: Option<SpriteAnnotation>,
    pub last_selection: Option<(String, u32, u32)>,
    pub cached_annotation: Option<SpriteAnnotation>,
    col_row_label: String,
    last_color_text: String,
    last_faction_text: String,
    last_color: SpriteColor,
    last_faction: Faction,
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
            last_faction: Faction::Independent,
        }
    }
}

mod generated {
    include!(concat!(env!("OUT_DIR"), "/sprites.rs"));
}

impl SpriteBrowser {
    pub fn new() -> Self {
        let section_order: &[&str] = &[
            "Talasha",
            "upper_green",
            "Khalifa_Abdullah",
            "Sherif",
            "lower_green",
            "upper_Jaalin",
            "Hadendowa",
            "lower_Jaalin",
            "Hadendowa_Guns",
            "Baggara",
            "British_Boats",
            "Ali_Wad_Helu",
            "British_Army",
            "Sheik_El_Din",
            "Kitchener",
            "Jehadia",
            "Egyptian_Army",
        ];

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
                    name: name.to_string(),
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
            let path = format!("sprites/{}.png", sprite.filename);
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
                            Text::new(section.name.replace('_', " ")),
                            TextFont {
                                font_size: 18.0,
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
            section_name: section.name.clone(),
            unit_name: section.name.replace('_', " "),
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

#[cfg(not(target_arch = "wasm32"))]
fn save_annotations(annotations: &Annotations, game_map: &GameMap) {
    save_annotations_to_file(game_map, annotations, ANNOTATIONS_SAVE_PATH);
}

#[cfg(target_arch = "wasm32")]
fn save_annotations(_annotations: &Annotations, _game_map: &GameMap) {}

pub fn navigate_sprite_selection(
    keys: Res<ButtonInput<KeyCode>>,
    mut browser: ResMut<SpriteBrowser>,
    root_q: Query<&Visibility, With<SpriteBrowserRoot>>,
    mut contexts: EguiContexts,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_keyboard_input() {
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
            section_name: section.name.clone(),
            unit_name: section.name.replace('_', " "),
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
    game_map: Res<GameMap>,
    mut pending: ResMut<PendingEdits>,
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

    let selection_changed = clipboard.last_selection.as_ref().map_or(
        true,
        |(name, col, row)| name != &sel.section_name || *col != sel.col || *row != sel.row,
    );
    if selection_changed {
        let default_meta = SpriteAnnotation {
            color: SpriteColor::SandBlack,
            faction: Faction::Independent,
            text: String::new(),
            a: 0,
            b: 0,
            c: 0,
            is_boat: false,
            is_unit: true,
        };
        let entry = annotations
            .0
            .units
            .get(&sel.section_name)
            .and_then(|m| m.get(&(sel.col, sel.row)));
        let m = entry.cloned().unwrap_or(default_meta);
        clipboard.last_color = m.color;
        clipboard.last_faction = m.faction;
        clipboard.last_color_text = format!("{:?}", m.color);
        clipboard.last_faction_text = format!("{:?}", m.faction);
        clipboard.cached_annotation = Some(m);
        clipboard.last_selection = Some((sel.section_name.clone(), sel.col, sel.row));
        clipboard.col_row_label = format!("Col: {}, Row: {}", sel.col, sel.row);
    }
    // take cached annotation out (avoids per-frame clone when unchanged)
    let mut meta = clipboard.cached_annotation.take().unwrap_or(SpriteAnnotation {
        color: SpriteColor::SandBlack,
        faction: Faction::Independent,
        text: String::new(),
        a: 0,
        b: 0,
        c: 0,
        is_boat: false,
        is_unit: true,
    });

    let mut changed = false;

    egui::SidePanel::right("sprite_meta_panel")
        .resizable(true)
        .default_width(280.0)
        .width_range(200.0..=500.0)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(45))
                .inner_margin(egui::Margin::symmetric(16, 16)),
        )
        .show(ctx, |ui| {
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
                                .selectable_value(&mut meta.color, c, format!("{:?}", c))
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
                        for f in Faction::iter() {
                            if ui
                                .selectable_value(&mut meta.faction, f, format!("{:?}", f))
                                .clicked()
                            {
                                changed = true;
                            }
                        }
                    });
            });

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

            // a / b / c
            ui.horizontal(|ui| {
                ui.label("a");
                if ui
                    .add(egui::DragValue::new(&mut meta.a).speed(1).range(0.0..=15.0))
                    .changed()
                {
                    changed = true;
                }
                ui.label("b");
                if ui
                    .add(egui::DragValue::new(&mut meta.b).speed(1).range(0.0..=15.0))
                    .changed()
                {
                    changed = true;
                }
                ui.label("c");
                if ui
                    .add(egui::DragValue::new(&mut meta.c).speed(1).range(0.0..=15.0))
                    .changed()
                {
                    changed = true;
                }
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let before = meta.is_boat;
                ui.checkbox(&mut meta.is_boat, "boat");
                if before != meta.is_boat {
                    changed = true;
                }
                let before_u = meta.is_unit;
                ui.checkbox(&mut meta.is_unit, "unit");
                if before_u != meta.is_unit {
                    changed = true;
                }
            });

            ui.add_space(16.0);

            // copy / paste buttons
            ui.horizontal(|ui| {
                if ui.button("[Copy Meta]").clicked() {
                    clipboard.copied = clipboard.cached_annotation.clone();
                }
                if ui.button("[Paste Meta]").clicked()
                    && let Some(ref data) = clipboard.copied
                {
                    meta = data.clone();
                    changed = true;
                }
            });
        });

    if changed {
        clipboard.cached_annotation = Some(meta.clone());
        clipboard.last_color = meta.color;
        clipboard.last_faction = meta.faction;
        clipboard.last_color_text = format!("{:?}", meta.color);
        clipboard.last_faction_text = format!("{:?}", meta.faction);
        annotations
            .0
            .units
            .entry(sel.section_name.clone())
            .or_default()
            .insert((sel.col, sel.row), meta.clone());
        pending.items.push(NetMsg::AnnotateSprite {
            section_name: sel.section_name.clone(),
            col: sel.col,
            row: sel.row,
            annotation: meta,
        });
        save_annotations(&annotations.0, &game_map);
    } else {
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
