//! The title screen and the lobby's backdrop as native Bevy UI: one root
//! node, spawned once from one BSN scene ([`splash_screen`]) and shown while
//! either screen is up, holding the backdrop (the [`BackdropMaterial`]
//! shader), the lobby's panel, the menu column (wide layout) or the centred
//! column (narrow windows), the map credits and the disabled "Game" button's
//! hint. The lobby's own widgets are egui, drawn over it.
//!
//! Layout (design 2a): the menu column on the dark left third, and on the
//! right two thirds the period map, fading into the backdrop. Every
//! look-and-feel number is in [`super::params`].

use bevy::asset::uuid_handle;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{FontSize, FontSourceTemplate, LetterSpacing, LineHeight};
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::Activate;
use bevy::window::PrimaryWindow;

use super::backdrop::{BackdropLayout, BackdropMaterial};
use super::map::SplashMaps;
use super::params::*;
use super::{Destination, Quote, SplashData, palette_color};
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

/// The screen's faces, the same embedded faces egui uses
/// (`ui_plugin::faces`), at fixed ids so a scene names them directly.
const SERIF: Handle<Font> = uuid_handle!("fce019ea-f33e-4939-a91b-45c8adaaa849");
const SERIF_ITALIC: Handle<Font> = uuid_handle!("26c7054c-1f33-45e9-93b3-83c2a6e9de1f");
const SERIF_BOLD: Handle<Font> = uuid_handle!("f544a4a4-2ce0-4bfb-a39f-b94cf48e90b7");
const SANS: Handle<Font> = uuid_handle!("3a4a2252-6355-447b-96e5-171cb9e6e7b0");

/// The backdrop material the root node draws with.
#[derive(Resource)]
pub(super) struct SplashBackdrop(Handle<BackdropMaterial>);

/// The root node: the backdrop.
#[derive(Component, Default, Clone)]
pub(super) struct SplashRoot;

/// The wide layout's menu column.
#[derive(Component, Default, Clone)]
pub(super) struct WideColumn;

/// The narrow layout's centred column.
#[derive(Component, Default, Clone)]
pub(super) struct NarrowColumn;

/// A narrow-layout text block that wraps at the narrow wrap width.
#[derive(Component, Default, Clone)]
pub(super) struct NarrowWrap;

/// "Loading…", standing in for the buttons until the board is ready.
#[derive(Component, Default, Clone)]
pub(super) struct LoadingLabel;

/// The buttons, shown once the board is ready.
#[derive(Component, Default, Clone)]
pub(super) struct ButtonGroup;

/// A menu button: where it sends the player and which layout's it is. The
/// press, release and click handling is Bevy's headless
/// [`Button`](bevy::ui_widgets::Button) widget (a touch is a pointer like
/// the mouse); "Game" is [`InteractionDisabled`] while it leads nowhere.
/// Its activation is gated, like the board, on [`EguiPointerOverUi`]: egui
/// draws over the native UI, and a pointer over an egui surface (a seat
/// vote, a tooltip) must not reach the button beneath it.
#[derive(Component, Clone)]
pub(super) struct MenuButton {
    destination: Destination,
    narrow: bool,
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
#[derive(Component, Default, Clone)]
pub(super) struct MenuButtonLabel;

/// The lobby's floating panel.
#[derive(Component, Default, Clone)]
pub(super) struct LobbyPanel;

/// The credit of the showing map (`incoming: false`) or of the one
/// crossfading in.
#[derive(Component, Default, Clone)]
pub(super) struct Credit {
    incoming: bool,
}

/// One of a credit's texts: the credit itself, or a copy of its shadow.
#[derive(Component, Default, Clone)]
pub(super) struct CreditText {
    shadow: bool,
}

/// The disabled "Game" button's hint, and how long the pointer has rested on
/// the button.
#[derive(Component, Default, Clone)]
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

/// Register the faces under their fixed ids.
pub(super) fn load_fonts(mut fonts: ResMut<Assets<Font>>) {
    use crate::ui_plugin::faces;
    for (face, bytes) in [
        (SERIF, faces::MERRIWEATHER_REGULAR),
        (SERIF_ITALIC, faces::MERRIWEATHER_ITALIC),
        (SERIF_BOLD, faces::MERRIWEATHER_BOLD),
        (SANS, faces::INTER_MEDIUM),
    ] {
        fonts
            .insert(face.id(), Font::from_bytes(bytes.to_vec()))
            .expect("a uuid asset id has no generation to be stale");
    }
}

/// Text in `face` at `size` points.
fn font(face: Handle<Font>, size: f32) -> impl Scene {
    bsn! {
        TextFont { font: FontSourceTemplate::Handle(face), font_size: FontSize::Px(size) }
    }
}

/// Text in `face` at `size` points, `line_height` times the size apart.
fn typeface(face: Handle<Font>, size: f32, line_height: f32) -> impl Scene {
    bsn! {
        @font(face, size)
        LineHeight::RelativeToFont(line_height)
    }
}

/// A text block of `text` with `*...*` runs in italic, `size` points,
/// `line_height` times the size apart.
fn emphasis_text(text: &str, style: (f32, f32, Color, bool, Justify)) -> impl Scene + use<> {
    let (size, line_height, color, base_italic, justify) = style;
    let runs: Vec<_> = emphasis_runs(text, base_italic)
        .into_iter()
        .map(|(run, italic)| {
            let run = run.to_string();
            let face = if italic { SERIF_ITALIC } else { SERIF };
            bsn! {
                TextSpan(run)
                @typeface(face, size, line_height)
                TextColor(color)
            }
        })
        .collect();
    bsn! {
        Text
        @typeface(SERIF, size, line_height)
        TextColor(color)
        TextLayout { justify }
        Children [ {runs} ]
    }
}

/// The epigraph's lines: the quote in curly quotes, and its attribution
/// after a dash if it has one.
fn epigraph(quote: Option<&Quote>) -> (Option<String>, Option<String>) {
    let text = quote.map(|quote| format!("\u{201c}{}\u{201d}", quote.text));
    let attribution = quote
        .filter(|quote| !quote.attribution.is_empty())
        .map(|quote| format!("\u{2014} {}", quote.attribution));
    (text, attribution)
}

pub(super) fn spawn_splash_screen(
    mut commands: Commands,
    splash: Res<SplashData>,
    mut materials: ResMut<Assets<BackdropMaterial>>,
) {
    let material = materials.add(BackdropMaterial::default());
    commands.insert_resource(SplashBackdrop(material.clone()));
    commands.spawn_scene(splash_screen(material, splash.quote.as_ref()));
}

/// The whole screen: the backdrop, and over it the lobby's panel, both
/// layouts' columns, the two credits and the hint.
fn splash_screen(material: Handle<BackdropMaterial>, quote: Option<&Quote>) -> impl Scene + use<> {
    bsn! {
        SplashRoot
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
        }
        MaterialNode<BackdropMaterial>(material)
        GlobalZIndex(SPLASH_Z)
        Visibility::Hidden
        Children [
            LobbyPanel
            Node {
                position_type: PositionType::Absolute,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(LOBBY_PANEL_RADIUS)),
                display: Display::None,
            }
            BackgroundColor(palette_color(palette::NEUTRAL_BG))
            BorderColor::all(palette_color(palette::LOBBY_PANEL_BORDER))
            --
            @wide_column(quote)
            --
            @narrow_column(quote)
            --
            @credit(false)
            --
            @credit(true)
            --
            @game_hint()
        ]
    }
}

/// The wide layout's menu column: kicker, two-line title, quote, attribution
/// and the entry buttons, left-aligned in the left third and vertically
/// centred. The buttons' height is reserved while "Loading…" stands in for
/// them, so the column does not jump when they appear.
fn wide_column(quote: Option<&Quote>) -> impl Scene + use<> {
    let (text, attribution) = epigraph(quote);
    let wrap = |gap: f32| {
        bsn! {
            Node { margin: UiRect::top(px(gap)), max_width: px(QUOTE_WRAP) }
        }
    };
    let text = text.map(|text| {
        bsn_list! {
            @emphasis_text(&text, (
                QUOTE_SIZE,
                QUOTE_LINE_HEIGHT,
                palette_color(palette::TEXT_STRONG),
                true,
                Justify::Left,
            ))
            @wrap(GAP_TITLE_QUOTE)
        }
    });
    // Upright; `*Title*` runs render italic.
    let attribution = attribution.map(|attribution| {
        bsn_list! {
            @emphasis_text(&attribution, (
                ATTR_SIZE,
                SERIF_LINE_HEIGHT,
                palette_color(palette::TEXT_MUTED),
                false,
                Justify::Left,
            ))
            @wrap(GAP_QUOTE_ATTR)
        }
    });
    bsn! {
        WideColumn
        Node {
            position_type: PositionType::Absolute,
            left: percent(COL_LEFT_REL * 100.0),
            width: percent(COL_W_REL * 100.0),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
        }
        Children [
            Text(KICKER)
            @typeface(SERIF, KICKER_SIZE, SERIF_LINE_HEIGHT)
            TextColor(palette_color(palette::SPLASH_KICKER))
            LetterSpacing::Px({KICKER_SIZE * KICKER_TRACKING})
            --
            Text("REMEMBER\nGORDON!")
            @typeface(SERIF_BOLD, TITLE_SIZE, TITLE_LINE_HEIGHT)
            TextColor(palette_color(palette::SPLASH_TITLE))
            Node { margin: UiRect::top(px(GAP_KICKER_TITLE)) }
            --
            {text}
            --
            {attribution}
            --
            Node {
                margin: UiRect::top(px(GAP_ATTR_BUTTONS)),
                height: px(2.0 * BUTTON_SIZE.y + GAP_BUTTONS),
                flex_direction: FlexDirection::Column,
            }
            Children [
                LoadingLabel
                Text("Loading\u{2026}")
                @typeface(SERIF, BUTTON_TEXT, SERIF_LINE_HEIGHT)
                TextColor(palette_color(palette::TEXT_FAINT))
                --
                ButtonGroup
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(GAP_BUTTONS),
                    display: Display::None,
                }
                Children [
                    @wide_button(Destination::Lobby)
                    --
                    @wide_button(Destination::Game)
                ]
            ]
        ]
    }
}

/// The narrow-window layout: one centred column (title, quote, attribution,
/// buttons) over the full-bleed map.
fn narrow_column(quote: Option<&Quote>) -> impl Scene + use<> {
    let (text, attribution) = epigraph(quote);
    let wrap = |gap: f32| {
        bsn! {
            NarrowWrap
            Node { margin: UiRect::top(px(gap)) }
        }
    };
    // Italic throughout; `*...*` runs stay italic too.
    let text = text.map(|text| {
        bsn_list! {
            @emphasis_text(&text, (
                NARROW_QUOTE_SIZE,
                SERIF_LINE_HEIGHT,
                palette_color(palette::TEXT_STRONG),
                true,
                Justify::Center,
            ))
            @wrap(NARROW_GAP_LARGE)
        }
    });
    // Upright; `*Title*` runs render italic.
    let attribution = attribution.map(|attribution| {
        bsn_list! {
            @emphasis_text(&attribution, (
                NARROW_SMALL_SIZE,
                SERIF_LINE_HEIGHT,
                palette_color(palette::TEXT_MUTED),
                false,
                Justify::Center,
            ))
            @wrap(NARROW_GAP_ATTR)
        }
    });
    bsn! {
        NarrowColumn
        Node {
            position_type: PositionType::Absolute,
            top: percent(NARROW_TOP_REL * 100.0),
            width: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            display: Display::None,
        }
        Children [
            Text("REMEMBER GORDON!")
            @typeface(SERIF, NARROW_TITLE_SIZE, SERIF_LINE_HEIGHT)
            TextColor(palette_color(palette::SPLASH_TITLE))
            --
            {text}
            --
            {attribution}
            --
            LoadingLabel
            Text("Loading\u{2026}")
            @typeface(SERIF, NARROW_SMALL_SIZE, SERIF_LINE_HEIGHT)
            TextColor(palette_color(palette::TEXT_FAINT))
            Node { margin: UiRect::top(px(NARROW_GAP_LARGE)) }
            --
            ButtonGroup
            Node {
                margin: UiRect::top(px(NARROW_GAP_LARGE)),
                flex_direction: FlexDirection::Column,
                row_gap: px(NARROW_GAP_BUTTONS),
                display: Display::None,
            }
            Children [
                @narrow_button(Destination::Lobby)
                --
                @narrow_button(Destination::Game)
            ]
        ]
    }
}

/// A menu button with its label at `label_size` points; the layouts place
/// the label ([`wide_button`], [`narrow_button`]).
fn menu_button(destination: Destination, narrow: bool, label_size: f32) -> impl Scene {
    let label = match destination {
        Destination::Lobby => "Lobby",
        Destination::Game => "Game",
    };
    let button = MenuButton {
        destination,
        narrow,
    };
    // The button takes the pointer, not its label.
    let label_ignores_pointer = Pickable::IGNORE;
    bsn! {
        button
        bevy::ui_widgets::Button
        Hovered
        Node {
            width: px(BUTTON_SIZE.x),
            height: px(BUTTON_SIZE.y),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(BUTTON_RADIUS)),
            align_items: AlignItems::Center,
        }
        on(activate_menu_button)
        Children [
            MenuButtonLabel
            label_ignores_pointer
            Text(label)
            @typeface(SERIF, label_size, SERIF_LINE_HEIGHT)
        ]
    }
}

/// The wide column's buttons: the label left-aligned.
fn wide_button(destination: Destination) -> impl Scene {
    bsn! {
        @menu_button(destination, false, BUTTON_TEXT)
        Node {
            padding: UiRect::left(px(BUTTON_PAD_X - 1.0)),
            justify_content: JustifyContent::FlexStart,
        }
    }
}

/// The narrow column's buttons: the label centred.
fn narrow_button(destination: Destination) -> impl Scene {
    bsn! {
        @menu_button(destination, true, NARROW_SMALL_SIZE)
        Node { justify_content: JustifyContent::Center }
    }
}

/// A map credit, bottom-right, over a soft shadow: dark copies on two rings
/// plus the centre, each faint enough that the overlapping core reaches the
/// shadow's peak opacity.
fn credit(incoming: bool) -> impl Scene {
    let text = || {
        bsn! {
            Text
            @typeface(SERIF_ITALIC, CREDIT_SIZE, SERIF_LINE_HEIGHT)
            TextColor(Color::NONE)
        }
    };
    let shadow: Vec<_> = shadow_offsets()
        .into_iter()
        .map(|offset| {
            bsn! {
                CreditText { shadow: true }
                @text()
                Node {
                    position_type: PositionType::Absolute,
                    left: px(offset.x),
                    top: px(offset.y),
                }
            }
        })
        .collect();
    bsn! {
        Credit { incoming }
        Node {
            position_type: PositionType::Absolute,
            right: px(CREDIT_MARGIN.x),
            bottom: px(CREDIT_MARGIN.y),
        }
        Visibility::Hidden
        Children [
            {shadow}
            --
            CreditText { shadow: false }
            @text()
        ]
    }
}

/// The disabled "Game" button's hint, placed by the pointer.
fn game_hint() -> impl Scene {
    bsn! {
        GameHint
        Node {
            position_type: PositionType::Absolute,
            padding: UiRect::axes(px(HINT_PAD.x), px(HINT_PAD.y)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(HINT_RADIUS)),
            display: Display::None,
        }
        BackgroundColor(palette_color(palette::theme::WINDOW_BG))
        BorderColor::all(Color::srgb_u8(60, 60, 60))
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.38),
            px(6),
            px(10),
            px(0),
            px(8),
        )
        Children [
            Text(NO_GAME_HINT)
            @font(SANS, HINT_SIZE)
            TextColor(Color::srgb_u8(140, 140, 140))
        ]
    }
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

/// The buttons' visuals for the pointer, "Game"'s [`InteractionDisabled`]
/// while it leads nowhere, and the disabled "Game" button's hint after the
/// pointer rests on it. (A click is the [`Button`](bevy::ui_widgets::Button)
/// widget's [`Activate`], observed per button: [`activate_menu_button`].)
#[allow(clippy::type_complexity)]
pub(super) fn menu_buttons(
    (mut commands, mut activity): (Commands, ResMut<crate::activity::Activity>),
    (time, over_egui): (Res<Time>, Res<EguiPointerOverUi>),
    progress: (Res<crate::TurnState>, Res<crate::game_record::GameRecorder>),
    window: Query<&Window, With<PrimaryWindow>>,
    mut buttons: Query<(
        Entity,
        (
            &Hovered,
            &MenuButton,
            Has<Pressed>,
            Has<InteractionDisabled>,
        ),
        &mut BackgroundColor,
        &mut BorderColor,
        &Children,
        &ComputedNode,
    )>,
    mut labels: Query<&mut TextColor, With<MenuButtonLabel>>,
    mut hint: Query<(&mut GameHint, &mut Node)>,
) {
    let game_enabled = crate::game_in_progress(&progress.0, &progress.1);
    let mut hint_hovered = false;
    for (entity, (hovered, button, pressed, disabled), mut fill, mut border, children, node) in
        &mut buttons
    {
        let enabled = button.enabled(game_enabled);
        if enabled == disabled {
            if enabled {
                commands.entity(entity).remove::<InteractionDisabled>();
            } else {
                commands.entity(entity).insert(InteractionDisabled);
            }
        }
        if node.size().x <= 0.0 {
            // Laid out away (the other layout's, or the screen is down).
            if pressed {
                commands.entity(entity).remove::<Pressed>();
            }
            continue;
        }
        let hovered = hovered.get() && !over_egui.0;
        let look = match (hovered, pressed) {
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

/// An activated menu button (a press and a release on it, unless it is
/// [`InteractionDisabled`]) sends the player on -- unless the pointer is
/// over an egui surface drawn above it.
fn activate_menu_button(
    activate: On<Activate>,
    buttons: Query<&MenuButton>,
    over_egui: Res<EguiPointerOverUi>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_app_mode: ResMut<NextState<AppMode>>,
) {
    let Ok(button) = buttons.get(activate.entity) else {
        return;
    };
    if over_egui.0 {
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

    /// The screen's scene spawned into a bare world (no window, no
    /// renderer): the scene alone, without the systems that lay it out.
    fn spawned(quote: Option<&Quote>) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin,
        ))
        .insert_resource(EguiPointerOverUi(false))
        .init_resource::<NextState<AppState>>()
        .init_resource::<NextState<AppMode>>();
        app.world_mut()
            .spawn_scene(splash_screen(Handle::default(), quote))
            .expect("the scene has no assets to wait for");
        app
    }

    fn children_of<C: Component>(app: &mut App) -> usize {
        let world = app.world_mut();
        let column = world
            .query_filtered::<Entity, With<C>>()
            .single(world)
            .expect("one column");
        world.get::<Children>(column).map_or(0, |c| c.len())
    }

    /// The epigraph's blocks are list entries that may be absent: an absent
    /// one spawns no entity (no empty node in the column's flow).
    #[test]
    fn the_columns_hold_only_the_epigraph_lines_there_are() {
        let quote = |attribution: &str| Quote {
            text: "War is *the* father.".into(),
            attribution: attribution.into(),
        };
        for (quote, lines) in [
            (Some(quote("Heraclitus, *Fragments*")), 2),
            (Some(quote("")), 1),
            (None, 0),
        ] {
            let mut app = spawned(quote.as_ref());
            // Kicker, title and the buttons' block; title, "Loading…" and
            // the buttons.
            assert_eq!(children_of::<WideColumn>(&mut app), 3 + lines);
            assert_eq!(children_of::<NarrowColumn>(&mut app), 3 + lines);
        }
    }

    /// Each menu button observes its own activation: "Lobby" enters the
    /// lobby, unless the pointer is over an egui surface above it.
    #[test]
    fn a_menu_button_activation_enters_its_destination() {
        for over_egui in [false, true] {
            let mut app = spawned(None);
            app.world_mut().resource_mut::<EguiPointerOverUi>().0 = over_egui;
            let world = app.world_mut();
            let buttons: Vec<(Entity, Destination)> = world
                .query::<(Entity, &MenuButton)>()
                .iter(world)
                .map(|(entity, button)| (entity, button.destination))
                .collect();
            assert_eq!(buttons.len(), 4, "two buttons in each layout");
            let lobby = buttons
                .iter()
                .find(|(_, destination)| *destination == Destination::Lobby)
                .expect("a Lobby button")
                .0;
            world.trigger(Activate { entity: lobby });
            let entered = matches!(
                *world.resource::<NextState<AppMode>>(),
                NextState::Pending(AppMode::Lobby)
            );
            assert_eq!(entered, !over_egui);
        }
    }
}
