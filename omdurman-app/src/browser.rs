use std::collections::BTreeMap;

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

#[derive(Resource)]
pub struct SpriteBrowser {
    pub visible: bool,
    pub sections: Vec<UnitSection>,
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
pub struct SpriteScrollContent;

#[derive(Component)]
pub(crate) struct SpriteScroll(pub(crate) f32);

impl SpriteBrowser {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut by_unit: BTreeMap<String, Vec<(u32, u32, String)>> = BTreeMap::new();
            let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join("sprites");

            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) != Some("png") {
                        continue;
                    }
                    let filename = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    let parts: Vec<&str> = filename.rsplitn(3, '_').collect();
                    if parts.len() == 3 {
                        if let (Ok(col), Ok(row)) =
                            (parts[1].parse::<u32>(), parts[0].parse::<u32>())
                        {
                            let unit_name = parts[2].to_string();
                            by_unit
                                .entry(unit_name)
                                .or_default()
                                .push((col, row, filename));
                        }
                    }
                }
            }

            let sections: Vec<UnitSection> = by_unit
                .into_iter()
                .map(|(name, mut raw)| {
                    let max_col = raw.iter().map(|s| s.0).max().unwrap_or(0);
                    let max_row = raw.iter().map(|s| s.1).max().unwrap_or(0);
                    let w = max_col + 1;
                    let h = max_row + 1;
                    raw.sort_by_key(|s| (s.1, s.0));
                    let sprites: Vec<BrowserSprite> = raw
                        .into_iter()
                        .map(|(col, row, filename)| BrowserSprite {
                            col,
                            row,
                            filename,
                            handle: Handle::default(),
                        })
                        .collect();
                    UnitSection {
                        name,
                        width: w,
                        height: h,
                        sprites,
                    }
                })
                .collect();

            return SpriteBrowser {
                visible: false,
                sections,
            };
        }

        #[cfg(target_arch = "wasm32")]
        SpriteBrowser {
            visible: false,
            sections: vec![],
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
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                ))
                .with_children(|inner| {
                    for section in &browser.sections {
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
                                    left: Val::Px(8.0),
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
                                    left: Val::Px(8.0),
                                    bottom: Val::Px(16.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|grid| {
                                for sprite in &section.sprites {
                                    grid.spawn((
                                        ImageNode {
                                            image: sprite.handle.clone(),
                                            ..default()
                                        },
                                        Node {
                                            width: Val::Px(sprite_size),
                                            height: Val::Px(sprite_size),
                                            margin: UiRect::all(Val::Px(
                                                sprite_margin,
                                            )),
                                            ..default()
                                        },
                                    ));
                                }
                            });
                    }
                });
        });
}

pub fn scroll_sprite_browser(
    mut scroll_events: MessageReader<MouseWheel>,
    mut content_q: Query<(&mut SpriteScroll, &mut Node), With<SpriteScrollContent>>,
    root_q: Query<&Visibility, (With<SpriteBrowserRoot>, Without<SpriteScrollContent>)>,
) {
    let Ok(vis) = root_q.single() else { return };
    if *vis != Visibility::Visible {
        return;
    }

    let total: f32 = scroll_events.read().map(|e| e.y).sum();
    if total == 0.0 {
        return;
    }

    let Ok((mut scroll, mut node)) = content_q.single_mut() else { return };
    scroll.0 = (scroll.0 - total * 30.0).max(0.0);
    node.top = Val::Px(-scroll.0);
}
