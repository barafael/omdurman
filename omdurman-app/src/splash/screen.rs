//! The title screen and the lobby's backdrop as native Bevy UI: one root
//! node, spawned once and shown while either screen is up, holding the
//! backdrop (the [`BackdropMaterial`] shader), the lobby's panel, the menu
//! column (wide layout) or the centred column (narrow windows), the map
//! credits and the disabled "Game" button's hint. The lobby's own widgets are
//! egui, drawn over it.
//!
//! Layout (design 2a): the menu column on the dark left third, and on the
//! right two thirds the period map, fading into the backdrop. Every
//! look-and-feel number is in [`super::params`].

use bevy::picking::hover::Hovered;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::text::{FontSize, FontSource, LetterSpacing, LineHeight};
use bevy::window::PrimaryWindow;

use super::backdrop::{BackdropLayout, BackdropMaterial};
use super::map::SplashMaps;
use super::params::*;
use super::{Destination, SplashData, palette_color};
use crate::ui::palette;
use crate::ui_plugin::EguiPointerOverUi;
use crate::{AppMode, AppState, Screen};

/// The splash sits over every other Bevy UI node (egui draws over it).
const SPLASH_Z: i32 = 1_000;

/// The "Game" button's hover text while no game is in progress.
const NO_GAME_HINT: &str = "No game in progress — start one from the Lobby";

/// Seconds the pointer rests on the disabled "Game" button before its hint
/// shows (egui's tooltip delay).
const HINT_DELAY_SECS: f32 = 0.5;

/// The screen's fonts, the same embedded faces egui uses
/// (`ui_plugin::faces`).
#[derive(Resource)]
pub(super) struct SplashFonts {
    serif: Handle<Font>,
    serif_italic: Handle<Font>,
    serif_bold: Handle<Font>,
    sans: Handle<Font>,
}

/// The backdrop material the root node draws with.
#[derive(Resource)]
pub(super) struct SplashBackdrop(Handle<BackdropMaterial>);

/// The root node: the backdrop.
#[derive(Component)]
pub(super) struct SplashRoot;

/// The wide layout's menu column.
#[derive(Component)]
pub(super) struct WideColumn;

/// The narrow layout's centred column.
#[derive(Component)]
pub(super) struct NarrowColumn;

/// A narrow-layout text block that wraps at the narrow wrap width.
#[derive(Component)]
pub(super) struct NarrowWrap;

/// "Loading…", standing in for the buttons until the board is ready.
#[derive(Component)]
pub(super) struct LoadingLabel;

/// The buttons, shown once the board is ready.
#[derive(Component)]
pub(super) struct ButtonGroup;

/// A menu button: where it sends the player, which layout's it is, and
/// whether a press is held on it. Driven by picking events (a touch is a
/// pointer like the mouse) and gated, like the board, on
/// [`EguiPointerOverUi`]: egui draws over the native UI, and a pointer over
/// an egui surface (a seat vote, a tooltip) must not reach the button
/// beneath it.
#[derive(Component)]
pub(super) struct MenuButton {
    destination: Destination,
    narrow: bool,
    pressed: bool,
}

impl MenuButton {
    /// "Game" only leads somewhere while a game is in progress.
    fn enabled(&self, game_in_progress: bool) -> bool {
        self.destination == Destination::Lobby || game_in_progress
    }
}

/// How a button looks under the pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ButtonLook {
    Idle,
    Hovered,
    Pressed,
}

/// A menu button's label.
#[derive(Component)]
pub(super) struct MenuButtonLabel;

/// The lobby's floating panel.
#[derive(Component)]
pub(super) struct LobbyPanel;

/// The credit of the showing map (`incoming: false`) or of the one
/// crossfading in.
#[derive(Component)]
pub(super) struct Credit {
    incoming: bool,
}

/// One of a credit's texts: the credit itself, or a copy of its shadow.
#[derive(Component)]
pub(super) struct CreditText {
    shadow: bool,
}

/// The disabled "Game" button's hint, and how long the pointer has rested on
/// the button.
#[derive(Component, Default)]
pub(super) struct GameHint {
    hovered_secs: f32,
}

/// Runs of `text` with `*...*` markdown emphasis: each run and whether it is
/// italic (the emphasized runs, and the rest when `base_italic`).
pub(super) fn emphasis_runs(text: &str, base_italic: bool) -> Vec<(&str, bool)> {
    text.split('*')
        .enumerate()
        .filter(|(_, segment)| !segment.is_empty())
        .map(|(i, segment)| (segment, i % 2 == 1 || base_italic))
        .collect()
}

pub(super) fn load_fonts(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    use crate::ui_plugin::faces;
    let mut add = |bytes: &'static [u8]| fonts.add(Font::from_bytes(bytes.to_vec()));
    commands.insert_resource(SplashFonts {
        serif: add(faces::MERRIWEATHER_REGULAR),
        serif_italic: add(faces::MERRIWEATHER_ITALIC),
        serif_bold: add(faces::MERRIWEATHER_BOLD),
        sans: add(faces::INTER_MEDIUM),
    });
}

fn font(handle: &Handle<Font>, size: f32) -> TextFont {
    TextFont {
        font: FontSource::from(handle),
        font_size: FontSize::Px(size),
        ..default()
    }
}

/// A text block of `text` with `*...*` runs in italic, `size` points,
/// `line_height` times the size apart.
fn emphasis_text(
    parent: &mut ChildSpawnerCommands,
    fonts: &SplashFonts,
    text: &str,
    style: (f32, f32, Color, bool, Justify),
    node: impl Bundle,
) {
    let (size, line_height, color, base_italic, justify) = style;
    parent
        .spawn((
            Text::default(),
            font(&fonts.serif, size),
            TextColor(color),
            LineHeight::RelativeToFont(line_height),
            TextLayout::justify(justify),
            node,
        ))
        .with_children(|text_block| {
            for (run, italic) in emphasis_runs(text, base_italic) {
                let face = if italic {
                    &fonts.serif_italic
                } else {
                    &fonts.serif
                };
                text_block.spawn((
                    TextSpan::new(run),
                    font(face, size),
                    TextColor(color),
                    LineHeight::RelativeToFont(line_height),
                ));
            }
        });
}

pub(super) fn spawn_splash_screen(
    mut commands: Commands,
    fonts: Res<SplashFonts>,
    splash: Res<SplashData>,
    mut materials: ResMut<Assets<BackdropMaterial>>,
) {
    let material = materials.add(BackdropMaterial::default());
    commands.insert_resource(SplashBackdrop(material.clone()));
    let full = Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    };
    commands
        .spawn((
            SplashRoot,
            full.clone(),
            MaterialNode(material),
            GlobalZIndex(SPLASH_Z),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                LobbyPanel,
                Node {
                    position_type: PositionType::Absolute,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(LOBBY_PANEL_RADIUS)),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(palette_color(palette::NEUTRAL_BG)),
                BorderColor::all(palette_color(palette::LOBBY_PANEL_BORDER)),
            ));
            spawn_wide_column(root, &fonts, &splash);
            spawn_narrow_column(root, &fonts, &splash);
            for incoming in [false, true] {
                spawn_credit(root, &fonts, incoming);
            }
            root.spawn((
                GameHint::default(),
                Node {
                    position_type: PositionType::Absolute,
                    padding: UiRect::axes(Val::Px(HINT_PAD.x), Val::Px(HINT_PAD.y)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(HINT_RADIUS)),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(palette_color(palette::theme::WINDOW_BG)),
                BorderColor::all(Color::srgb_u8(60, 60, 60)),
                BoxShadow::new(
                    Color::srgba(0.0, 0.0, 0.0, 0.38),
                    Val::Px(6.0),
                    Val::Px(10.0),
                    Val::Px(0.0),
                    Val::Px(8.0),
                ),
                children![(
                    Text::new(NO_GAME_HINT),
                    font(&fonts.sans, HINT_SIZE),
                    TextColor(Color::srgb_u8(140, 140, 140)),
                )],
            ));
        });
}

/// The wide layout's menu column: kicker, two-line title, quote, attribution
/// and the entry buttons, left-aligned in the left third and vertically
/// centred. The buttons' height is reserved while "Loading…" stands in for
/// them, so the column does not jump when they appear.
fn spawn_wide_column(root: &mut ChildSpawnerCommands, fonts: &SplashFonts, splash: &SplashData) {
    root.spawn((
        WideColumn,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(COL_LEFT_REL * 100.0),
            width: Val::Percent(COL_W_REL * 100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            ..default()
        },
    ))
    .with_children(|column| {
        column.spawn((
            Text::new(KICKER),
            font(&fonts.serif, KICKER_SIZE),
            TextColor(palette_color(palette::SPLASH_KICKER)),
            LetterSpacing::Px(KICKER_SIZE * KICKER_TRACKING),
            LineHeight::RelativeToFont(SERIF_LINE_HEIGHT),
        ));
        column.spawn((
            Text::new("REMEMBER\nGORDON!"),
            font(&fonts.serif_bold, TITLE_SIZE),
            TextColor(palette_color(palette::SPLASH_TITLE)),
            LineHeight::RelativeToFont(TITLE_LINE_HEIGHT),
            Node {
                margin: UiRect::top(Val::Px(GAP_KICKER_TITLE)),
                ..default()
            },
        ));
        if let Some(quote) = &splash.quote {
            let wrap = |gap: f32| Node {
                margin: UiRect::top(Val::Px(gap)),
                max_width: Val::Px(QUOTE_WRAP),
                ..default()
            };
            emphasis_text(
                column,
                fonts,
                &format!("\u{201c}{}\u{201d}", quote.text),
                (
                    QUOTE_SIZE,
                    QUOTE_LINE_HEIGHT,
                    palette_color(palette::TEXT_STRONG),
                    true,
                    Justify::Left,
                ),
                wrap(GAP_TITLE_QUOTE),
            );
            if !quote.attribution.is_empty() {
                // Upright; `*Title*` runs render italic.
                emphasis_text(
                    column,
                    fonts,
                    &format!("\u{2014} {}", quote.attribution),
                    (
                        ATTR_SIZE,
                        SERIF_LINE_HEIGHT,
                        palette_color(palette::TEXT_MUTED),
                        false,
                        Justify::Left,
                    ),
                    wrap(GAP_QUOTE_ATTR),
                );
            }
        }
        column
            .spawn(Node {
                margin: UiRect::top(Val::Px(GAP_ATTR_BUTTONS)),
                height: Val::Px(2.0 * BUTTON_SIZE.y + GAP_BUTTONS),
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|block| {
                block.spawn((
                    LoadingLabel,
                    Text::new("Loading\u{2026}"),
                    font(&fonts.serif, BUTTON_TEXT),
                    TextColor(palette_color(palette::TEXT_FAINT)),
                    LineHeight::RelativeToFont(SERIF_LINE_HEIGHT),
                ));
                block
                    .spawn((
                        ButtonGroup,
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(GAP_BUTTONS),
                            display: Display::None,
                            ..default()
                        },
                    ))
                    .with_children(|buttons| {
                        for destination in [Destination::Lobby, Destination::Game] {
                            spawn_button(buttons, fonts, destination, false);
                        }
                    });
            });
    });
}

/// The narrow-window layout: one centred column (title, quote, attribution,
/// buttons) over the full-bleed map.
fn spawn_narrow_column(root: &mut ChildSpawnerCommands, fonts: &SplashFonts, splash: &SplashData) {
    root.spawn((
        NarrowColumn,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(NARROW_TOP_REL * 100.0),
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            display: Display::None,
            ..default()
        },
    ))
    .with_children(|column| {
        column.spawn((
            Text::new("REMEMBER GORDON!"),
            font(&fonts.serif, NARROW_TITLE_SIZE),
            TextColor(palette_color(palette::SPLASH_TITLE)),
            LineHeight::RelativeToFont(SERIF_LINE_HEIGHT),
        ));
        if let Some(quote) = &splash.quote {
            let wrap = |gap: f32| {
                (
                    NarrowWrap,
                    Node {
                        margin: UiRect::top(Val::Px(gap)),
                        ..default()
                    },
                )
            };
            // Italic throughout; `*...*` runs stay italic too.
            emphasis_text(
                column,
                fonts,
                &format!("\u{201c}{}\u{201d}", quote.text),
                (
                    NARROW_QUOTE_SIZE,
                    SERIF_LINE_HEIGHT,
                    palette_color(palette::TEXT_STRONG),
                    true,
                    Justify::Center,
                ),
                wrap(NARROW_GAP_LARGE),
            );
            if !quote.attribution.is_empty() {
                // Upright; `*Title*` runs render italic.
                emphasis_text(
                    column,
                    fonts,
                    &format!("\u{2014} {}", quote.attribution),
                    (
                        NARROW_SMALL_SIZE,
                        SERIF_LINE_HEIGHT,
                        palette_color(palette::TEXT_MUTED),
                        false,
                        Justify::Center,
                    ),
                    wrap(NARROW_GAP_ATTR),
                );
            }
        }
        column.spawn((
            LoadingLabel,
            Text::new("Loading\u{2026}"),
            font(&fonts.serif, NARROW_SMALL_SIZE),
            TextColor(palette_color(palette::TEXT_FAINT)),
            LineHeight::RelativeToFont(SERIF_LINE_HEIGHT),
            Node {
                margin: UiRect::top(Val::Px(NARROW_GAP_LARGE)),
                ..default()
            },
        ));
        column
            .spawn((
                ButtonGroup,
                Node {
                    margin: UiRect::top(Val::Px(NARROW_GAP_LARGE)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(NARROW_GAP_BUTTONS),
                    display: Display::None,
                    ..default()
                },
            ))
            .with_children(|buttons| {
                for destination in [Destination::Lobby, Destination::Game] {
                    spawn_button(buttons, fonts, destination, true);
                }
            });
    });
}

/// A menu button: the wide layout's label is left-aligned, the narrow
/// layout's centred.
fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &SplashFonts,
    destination: Destination,
    narrow: bool,
) {
    let label = match destination {
        Destination::Lobby => "Lobby",
        Destination::Game => "Game",
    };
    parent.spawn((
        MenuButton {
            destination,
            narrow,
            pressed: false,
        },
        Hovered::default(),
        Node {
            width: Val::Px(BUTTON_SIZE.x),
            height: Val::Px(BUTTON_SIZE.y),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(BUTTON_RADIUS)),
            padding: if narrow {
                UiRect::ZERO
            } else {
                UiRect::left(Val::Px(BUTTON_PAD_X - 1.0))
            },
            align_items: AlignItems::Center,
            justify_content: if narrow {
                JustifyContent::Center
            } else {
                JustifyContent::FlexStart
            },
            ..default()
        },
        children![(
            MenuButtonLabel,
            // The button takes the pointer, not its label.
            Pickable::IGNORE,
            Text::new(label),
            font(
                &fonts.serif,
                if narrow {
                    NARROW_SMALL_SIZE
                } else {
                    BUTTON_TEXT
                },
            ),
            LineHeight::RelativeToFont(SERIF_LINE_HEIGHT),
        )],
    ));
}

/// A map credit, bottom-right, over a soft shadow: dark copies on two rings
/// plus the centre, each faint enough that the overlapping core reaches the
/// shadow's peak opacity.
fn spawn_credit(root: &mut ChildSpawnerCommands, fonts: &SplashFonts, incoming: bool) {
    let text = || {
        (
            Text::default(),
            font(&fonts.serif_italic, CREDIT_SIZE),
            LineHeight::RelativeToFont(SERIF_LINE_HEIGHT),
        )
    };
    root.spawn((
        Credit { incoming },
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(CREDIT_MARGIN.x),
            bottom: Val::Px(CREDIT_MARGIN.y),
            ..default()
        },
        Visibility::Hidden,
    ))
    .with_children(|credit| {
        for offset in shadow_offsets() {
            credit.spawn((
                CreditText { shadow: true },
                text(),
                TextColor(Color::NONE),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(offset.x),
                    top: Val::Px(offset.y),
                    ..default()
                },
            ));
        }
        credit.spawn((CreditText { shadow: false }, text(), TextColor(Color::NONE)));
    });
}

/// Where the credit shadow's copies sit: the centre and two rings of eight.
fn shadow_offsets() -> Vec<Vec2> {
    [0.0, 0.5, 1.0]
        .into_iter()
        .enumerate()
        .flat_map(|(ring, f)| {
            let steps = if ring == 0 { 1 } else { 8 };
            (0..steps).map(move |step| {
                let angle = std::f32::consts::TAU * step as f32 / steps as f32;
                CREDIT_SHADOW_OFFSET + f * CREDIT_SHADOW_RADIUS * Vec2::from_angle(angle)
            })
        })
        .collect()
}

/// Show the screen that is up, lay it out for the window, and point the
/// backdrop at the show's current pose.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn sync_splash_screen(
    app_state: Res<State<AppState>>,
    screen_up: Option<Res<State<Screen>>>,
    splash: Res<SplashData>,
    maps: Res<SplashMaps>,
    tuning: Res<super::tuning::SplashTuning>,
    progress: (Res<crate::TurnState>, Res<crate::game_record::GameRecorder>),
    backdrop: Res<SplashBackdrop>,
    mut materials: ResMut<Assets<BackdropMaterial>>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut root: Query<&mut Visibility, With<SplashRoot>>,
    mut nodes: ParamSet<(
        Query<&mut Node, With<WideColumn>>,
        Query<&mut Node, With<NarrowColumn>>,
        Query<&mut Node, With<LobbyPanel>>,
        Query<&mut Node, With<NarrowWrap>>,
        Query<&mut Node, With<LoadingLabel>>,
        Query<&mut Node, With<ButtonGroup>>,
    )>,
) {
    let Ok(mut visibility) = root.single_mut() else {
        return;
    };
    let shown = screen_up
        .map(|screen| *screen.get())
        .filter(|screen| matches!(screen, Screen::Title | Screen::Lobby));
    let Some(shown) = shown else {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    };
    visibility.set_if_neq(Visibility::Inherited);
    let Ok(window) = window.single() else { return };
    let screen = Vec2::new(window.width(), window.height());
    let narrow = screen.x < NARROW_BREAKPOINT;
    let title = shown == Screen::Title;
    let column = title && tuning.sidebar;
    let display = |on: bool| if on { Display::Flex } else { Display::None };
    let set_display = |node: &mut Node, on: bool| {
        if node.display != display(on) {
            node.display = display(on);
        }
    };
    for mut node in &mut nodes.p0() {
        set_display(&mut node, column && !narrow);
    }
    for mut node in &mut nodes.p1() {
        set_display(&mut node, column && narrow);
    }
    let panel = super::lobby_panel_rect(screen, crate::lobby::lobby_column_width(screen.x));
    for mut node in &mut nodes.p2() {
        set_display(&mut node, !title);
        let place = (
            Val::Px(panel.min.x),
            Val::Px(panel.min.y),
            Val::Px(panel.width()),
            Val::Px(panel.height()),
        );
        if (node.left, node.top, node.width, node.height) != place {
            (node.left, node.top, node.width, node.height) = place;
        }
    }
    // The quote wraps at a fraction of the window, at most a fixed width.
    let wrap = Val::Px((screen.x * NARROW_WRAP_REL).min(NARROW_WRAP_MAX));
    for mut node in &mut nodes.p3() {
        if node.max_width != wrap {
            node.max_width = wrap;
        }
    }
    for mut node in &mut nodes.p4() {
        set_display(&mut node, !splash.loaded);
    }
    for mut node in &mut nodes.p5() {
        set_display(&mut node, splash.loaded);
    }

    let layout = if title {
        BackdropLayout::Title {
            narrow,
            sidebar: tuning.sidebar,
            composite: title_composite(&app_state, &progress),
            map: tuning.title_map,
            blur_px: tuning.blur_px,
        }
    } else {
        BackdropLayout::Lobby { panel }
    };
    // Written back only when it changed: a modified material is re-prepared
    // (its uniform buffer and bind group rebuilt), so an untouched write
    // every frame cost that every frame for nothing (a frozen show, a show
    // still loading, an idle lobby between pan steps).
    if let Some(current) = materials.get(&backdrop.0) {
        let mut next = current.clone();
        next.update(screen, &maps, layout);
        if next != *current
            && let Some(mut material) = materials.get_mut(&backdrop.0)
        {
            *material = next;
        }
    }
}

/// The title screen's group opacity: opaque during the initial load and
/// whenever there is no game behind it; semi-transparent when returning to the
/// menu from a game, so the board shows through. It applies to the whole
/// composition, map included.
fn title_composite(
    app_state: &State<AppState>,
    progress: &(Res<crate::TurnState>, Res<crate::game_record::GameRecorder>),
) -> f32 {
    let game_enabled = crate::game_in_progress(&progress.0, &progress.1);
    if *app_state.get() == AppState::Splash || !game_enabled {
        1.0
    } else {
        200.0 / 255.0
    }
}

/// The credits: the showing map's, and during a crossfade the incoming one's.
/// They hand over in sequence -- the old one out in the first half, the new
/// one in over the second -- so two lines of different lengths never overlap.
#[allow(clippy::type_complexity)]
pub(super) fn sync_credits(
    app_state: Res<State<AppState>>,
    screen: Option<Res<State<Screen>>>,
    maps: Res<SplashMaps>,
    progress: (Res<crate::TurnState>, Res<crate::game_record::GameRecorder>),
    mut credits: Query<(&Credit, &mut Visibility, &Children)>,
    mut texts: Query<(&CreditText, &mut Text, &mut TextColor)>,
) {
    let composite = match screen.as_deref().map(State::get) {
        Some(Screen::Title) => title_composite(&app_state, &progress),
        Some(Screen::Lobby) => 1.0,
        _ => return,
    };
    let opacity = composite * maps.show.fade_in();
    let fade = maps.show.fade().filter(|&(index, _)| maps.is_ready(index));
    let fade_progress = fade.map_or(0.0, |(_, progress)| progress);
    let handover = |from: f32| ((fade_progress - from) * 2.0).clamp(0.0, 1.0);
    for (credit, mut visibility, children) in &mut credits {
        let line = if credit.incoming {
            fade.map(|(index, _)| (MAPS[index].credit, opacity * handover(0.5)))
        } else {
            maps.is_ready(maps.show.current).then(|| {
                (
                    MAPS[maps.show.current].credit,
                    opacity * (1.0 - handover(0.0)),
                )
            })
        };
        let Some((line, opacity)) = line.filter(|&(_, opacity)| opacity > 0.0) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        visibility.set_if_neq(Visibility::Inherited);
        let copies = shadow_offsets().len() as f32;
        let per_copy = 1.0 - (1.0 - CREDIT_SHADOW_ALPHA * opacity).powf(1.0 / copies);
        for &child in children {
            let Ok((text, mut content, mut color)) = texts.get_mut(child) else {
                continue;
            };
            if content.0 != line {
                content.0 = line.to_string();
            }
            let wanted = if text.shadow {
                Color::BLACK.with_alpha(per_copy)
            } else {
                palette_color(palette::SPLASH_CREDIT).with_alpha(opacity)
            };
            color.set_if_neq(TextColor(wanted));
        }
    }
}

/// The buttons' visuals for the pointer, and the disabled "Game" button's
/// hint after the pointer rests on it. (The clicks are observers:
/// [`press_menu_button`], [`release_menu_button`], [`click_menu_button`].)
#[allow(clippy::type_complexity)]
pub(super) fn menu_buttons(
    (time, over_egui): (Res<Time>, Res<EguiPointerOverUi>),
    progress: (Res<crate::TurnState>, Res<crate::game_record::GameRecorder>),
    window: Query<&Window, With<PrimaryWindow>>,
    mut buttons: Query<(
        &Hovered,
        &mut MenuButton,
        &mut BackgroundColor,
        &mut BorderColor,
        &Children,
        &ComputedNode,
    )>,
    mut labels: Query<&mut TextColor, With<MenuButtonLabel>>,
    mut hint: Query<(&mut GameHint, &mut Node)>,
    mut activity: ResMut<crate::activity::Activity>,
) {
    let game_enabled = crate::game_in_progress(&progress.0, &progress.1);
    let mut hint_hovered = false;
    for (hovered, mut button, mut fill, mut border, children, node) in &mut buttons {
        if node.size().x <= 0.0 {
            // Laid out away (the other layout's, or the screen is down).
            if button.pressed {
                button.pressed = false;
            }
            continue;
        }
        let enabled = button.enabled(game_enabled);
        let hovered = hovered.get() && !over_egui.0;
        let look = match (hovered, button.pressed) {
            (true, true) => ButtonLook::Pressed,
            (true, false) => ButtonLook::Hovered,
            (false, _) => ButtonLook::Idle,
        };
        hint_hovered |= !enabled && hovered;
        let (fill_color, border_color, text) = button_colors(enabled, button.narrow, look);
        fill.set_if_neq(BackgroundColor(fill_color));
        border.set_if_neq(BorderColor::all(border_color));
        for &child in children {
            if let Ok(mut label) = labels.get_mut(child) {
                label.set_if_neq(TextColor(text));
            }
        }
    }

    if let Ok((mut hint, mut node)) = hint.single_mut() {
        hint.hovered_secs = if hint_hovered {
            hint.hovered_secs + time.delta_secs()
        } else {
            0.0
        };
        let cursor = window.single().ok().and_then(Window::cursor_position);
        let show = hint.hovered_secs >= HINT_DELAY_SECS && cursor.is_some();
        if hint_hovered && !show {
            // Frames until the hint is due.
            activity.keep_ambient();
        }
        let display = if show { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
        if let (true, Some(cursor)) = (show, cursor) {
            let at = cursor + HINT_OFFSET;
            node.left = Val::Px(at.x);
            node.top = Val::Px(at.y);
        }
    }
}

/// A primary press on a menu button holds it down (its pressed look).
pub(super) fn press_menu_button(
    mut press: On<Pointer<Press>>,
    mut buttons: Query<&mut MenuButton>,
    over_egui: Res<EguiPointerOverUi>,
) {
    if let Ok(mut button) = buttons.get_mut(press.entity) {
        press.propagate(false);
        if press.button == PointerButton::Primary && !over_egui.0 {
            button.pressed = true;
        }
    }
}

/// The press ends: released over the button, dragged off it, or cancelled.
pub(super) fn release_menu_button(
    mut release: On<Pointer<Release>>,
    mut buttons: Query<&mut MenuButton>,
) {
    if let Ok(mut button) = buttons.get_mut(release.entity) {
        release.propagate(false);
        button.pressed = false;
    }
}

/// See [`release_menu_button`].
pub(super) fn end_menu_button_drag(
    mut drag_end: On<Pointer<DragEnd>>,
    mut buttons: Query<&mut MenuButton>,
) {
    if let Ok(mut button) = buttons.get_mut(drag_end.entity) {
        drag_end.propagate(false);
        button.pressed = false;
    }
}

/// See [`release_menu_button`].
pub(super) fn cancel_menu_button(
    mut cancel: On<Pointer<Cancel>>,
    mut buttons: Query<&mut MenuButton>,
) {
    if let Ok(mut button) = buttons.get_mut(cancel.entity) {
        cancel.propagate(false);
        button.pressed = false;
    }
}

/// A primary click (a press and a release on the button) sends the player
/// on, if the button leads somewhere.
pub(super) fn click_menu_button(
    mut click: On<Pointer<Click>>,
    buttons: Query<&MenuButton>,
    progress: (Res<crate::TurnState>, Res<crate::game_record::GameRecorder>),
    over_egui: Res<EguiPointerOverUi>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_app_mode: ResMut<NextState<AppMode>>,
) {
    let Ok(button) = buttons.get(click.entity) else {
        return;
    };
    click.propagate(false);
    let game_enabled = crate::game_in_progress(&progress.0, &progress.1);
    if click.button != PointerButton::Primary || over_egui.0 || !button.enabled(game_enabled) {
        return;
    }
    match button.destination {
        Destination::Lobby => {
            info!("menu: entering lobby");
            next_app_state.set(AppState::Lobby);
            next_app_mode.set(AppMode::Lobby);
        }
        Destination::Game => {
            info!("menu: entering game");
            crate::enter_game_view(&mut next_app_mode, &mut next_app_state);
        }
    }
}

/// A button's fill, border and label colours: the dark title-card visuals (the
/// splash is art-directed, not paper chrome) for the pointer, or faded when
/// disabled.
fn button_colors(enabled: bool, narrow: bool, look: ButtonLook) -> (Color, Color, Color) {
    if !enabled {
        // The wide column paints its own disabled visuals; the narrow one's
        // are the normal ones, faded.
        let (fill, border) = if narrow {
            (palette::NEUTRAL_FILL, palette::NEUTRAL_BORDER)
        } else {
            (
                palette::SPLASH_BUTTON_DISABLED_FILL,
                palette::SPLASH_BUTTON_DISABLED_BORDER,
            )
        };
        let faded = |color| palette_color(color).with_alpha(DISABLED_ALPHA);
        return (faded(fill), faded(border), faded(palette::TEXT_DISABLED));
    }
    let (fill, border) = match look {
        ButtonLook::Idle => (palette::NEUTRAL_FILL, palette::NEUTRAL_BORDER),
        ButtonLook::Hovered => (palette::NEUTRAL_FILL_RAISED, palette::NEUTRAL_BORDER_HOVER),
        ButtonLook::Pressed => (
            palette::NEUTRAL_FILL_PRESSED,
            palette::NEUTRAL_BORDER_ACTIVE,
        ),
    };
    (
        palette_color(fill),
        palette_color(border),
        palette_color(palette::TEXT_STRONG),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emphasis_marks_star_runs_italic() {
        let runs = emphasis_runs("plain *italic* plain", false);
        assert_eq!(
            runs,
            vec![("plain ", false), ("italic", true), (" plain", false)]
        );
        // An italic base keeps the starred runs italic too.
        assert!(emphasis_runs("a *b* c", true).iter().all(|&(_, i)| i));
        assert!(runs.iter().all(|(run, _)| !run.contains('*')));
    }

    #[test]
    fn the_shadow_has_a_centre_and_two_rings() {
        let offsets = shadow_offsets();
        assert_eq!(offsets.len(), 17);
        assert_eq!(offsets[0], CREDIT_SHADOW_OFFSET);
        let reach = offsets
            .iter()
            .map(|o| (*o - CREDIT_SHADOW_OFFSET).length())
            .fold(0.0, f32::max);
        assert!((reach - CREDIT_SHADOW_RADIUS).abs() < 1e-5);
    }
}
