use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hex::HexLayout;
use omdurman_map::GameMap;
use omdurman_types::{HexCoord, HexsideKind, HexsideRef, IntoEnumIterator, NileFlow, Terrain};

use omdurman_net::{GameEvent, NetMsg};

use crate::{
    EditorMode, PendingEdits, SidebarClip,
    camera::RtsCamera,
    render::{HexOverlay, draw_hex_outline},
    util::{adjusted_origin, hex_world_pos, hit_to_hex, raycast_ground},
};

pub const ANNOTATIONS_SAVE_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/annotations.ron");

#[derive(Resource, Default)]
pub struct HexEditor {
    pub selected: Option<HexCoord>,
    pub name: String,
    pub terrain: Terrain,
    /// Nile current of the selected hex; `None` = no current. Only meaningful
    /// (and only shown) when `terrain.is_nile()`.
    pub nile_flow: Option<NileFlow>,
    pub show_terrain_overlay: bool,
    /// The hexside segment currently selected in the Hexside editor mode, if
    /// any. Clicking a segment selects it; the side panel then assigns a type.
    pub selected_hexside: Option<HexsideRef>,
    /// Whether the selected hex's "type" dropdown is on the **Not playable**
    /// pseudo-terrain — i.e. the hex is board furniture (logo, turn track, …)
    /// excluded from the map. Mirrors the selected hex's exclusion state and is
    /// applied via [`GameEvent::ExcludeHex`] rather than terrain.
    pub not_playable: bool,
}

/// Letter/number keys set terrain on the selected hex; Delete/Backspace marks
/// it Not playable; Ctrl+arrow keys move the selection between hexes (plain
/// arrows pan the viewport); Ctrl+PgUp/PgDown rotate the Nile current on
/// `is_nile` hexes (plain PgUp/PgDown tilt the camera).
pub fn editor_terrain_keys(
    mode: Res<EditorMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut editor: ResMut<HexEditor>,
    game_map: Res<GameMap>,
) {
    if !mode.is_editor() {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_keyboard_input()
    {
        return;
    }
    let Some(coord) = editor.selected else {
        return;
    };

    // Ctrl+arrows move the selection to a neighbouring hex (and load its state,
    // like a click). Plain arrows pan the viewport (see `camera_control`), so
    // selection movement is gated behind Ctrl. Left/Right step along the q-axis
    // (W/E), Up/Down along the r-axis. Off-grid steps are ignored.
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let step = ctrl
        .then(|| match () {
            _ if keys.just_pressed(KeyCode::ArrowLeft) => Some(HexCoord::new(coord.q - 1, coord.r)),
            _ if keys.just_pressed(KeyCode::ArrowRight) => {
                Some(HexCoord::new(coord.q + 1, coord.r))
            }
            _ if keys.just_pressed(KeyCode::ArrowUp) => Some(HexCoord::new(coord.q, coord.r - 1)),
            _ if keys.just_pressed(KeyCode::ArrowDown) => Some(HexCoord::new(coord.q, coord.r + 1)),
            _ => None,
        })
        .flatten();
    if let Some(target) = step {
        select_hex(target, &mut editor, &game_map);
        return;
    }

    // Ctrl+PgUp/PgDown rotate the Nile current direction on Nile hexes (plain
    // PgUp/PgDown tilt the camera, see `camera_control`).
    if ctrl && editor.terrain.is_nile() && !editor.not_playable {
        let rotate = if keys.just_pressed(KeyCode::PageUp) {
            Some(1)
        } else if keys.just_pressed(KeyCode::PageDown) {
            Some(-1)
        } else {
            None
        };
        if let Some(delta) = rotate {
            let flow = editor.nile_flow.get_or_insert_with(NileFlow::default);
            *flow = flow.rotated(delta);
            return;
        }
    }

    // Delete/Backspace marks the selected hex Not playable (the apply path then
    // emits ExcludeHex); mirrors the dropdown's "Not playable" pseudo-type.
    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        editor.not_playable = true;
        return;
    }
    let t = match () {
        _ if keys.just_pressed(KeyCode::KeyB) => Some(Terrain::BlueNile),
        _ if keys.just_pressed(KeyCode::KeyD) => Some(Terrain::Desert),
        _ if keys.just_pressed(KeyCode::KeyF) => Some(Terrain::Fortress),
        _ if keys.just_pressed(KeyCode::KeyP) => Some(Terrain::Palm),
        _ if keys.just_pressed(KeyCode::KeyS) => Some(Terrain::Shrubs),
        _ if keys.just_pressed(KeyCode::KeyW) => Some(Terrain::WhiteNile),
        _ if keys.just_pressed(KeyCode::KeyR) => Some(Terrain::RiverNile),
        _ if keys.just_pressed(KeyCode::KeyK) => Some(Terrain::Khartoum),
        _ if keys.just_pressed(KeyCode::KeyT) => Some(Terrain::Tuti),
        _ if keys.just_pressed(KeyCode::KeyH) => Some(Terrain::Hogali),
        _ if keys.just_pressed(KeyCode::KeyU) => Some(Terrain::Buri),
        _ if keys.just_pressed(KeyCode::KeyM) => Some(Terrain::FortMakran),
        _ if keys.just_pressed(KeyCode::Digit1) => Some(Terrain::FortBuri),
        _ if keys.just_pressed(KeyCode::KeyN) => Some(Terrain::NorthFort),
        _ if keys.just_pressed(KeyCode::KeyO) => Some(Terrain::Trees), // wOods
        _ if keys.just_pressed(KeyCode::KeyA) => Some(Terrain::Swamp), // mArsh
        _ => None,
    };
    if let Some(t) = t {
        editor.terrain = t;
        // A terrain hotkey also takes the hex off the "Not playable" type.
        editor.not_playable = false;
    }
}

pub fn handle_hex_editor_click(
    mode: Res<EditorMode>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut editor: ResMut<HexEditor>,
) {
    if !mode.is_editor() || !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_pointer_input()
    {
        return;
    }
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    if !select_hex(coord, &mut editor, &game_map) && editor.selected == Some(coord) {
        // Clicking empty space (off-grid) where the current selection is
        // deselects it.
        editor.selected = None;
    }
}

/// Select `coord` and load its state into the editor panel, mirroring a click.
/// Returns `true` if `coord` is an in-grid hex (playable or excluded), `false`
/// if it's off the grid (in which case nothing is selected).
fn select_hex(coord: HexCoord, editor: &mut HexEditor, game_map: &GameMap) -> bool {
    if let Some(data) = game_map.hexes.get(&coord) {
        // A playable hex: load its terrain/name/flow into the panel.
        editor.selected = Some(coord);
        editor.name = data.name.clone().unwrap_or_default();
        editor.terrain = data.terrain;
        editor.nile_flow = data.nile_flow;
        editor.not_playable = false;
        true
    } else if game_map.excluded.contains(&coord) {
        // An excluded hex (in-grid but Not playable): selectable so its type can
        // be switched back. It carries no terrain data while excluded.
        editor.selected = Some(coord);
        editor.name = String::new();
        editor.nile_flow = None;
        editor.not_playable = true;
        true
    } else {
        false
    }
}

/// The edge of `coord` nearest the world point `hit` — i.e. the neighbour
/// whose shared border the click is closest to. Returns the `[coord, neighbour]`
/// pair as a canonical [`HexsideRef`].
///
/// All six edges are candidates, including those toward off-map or excluded
/// neighbours: a wall/khor can sit on the board's outer border, so the editor
/// must be able to select any of a hex's sides.
fn nearest_edge(
    coord: HexCoord,
    hit: Vec3,
    origin: bevy::math::Vec2,
    overlay: &omdurman_types::OverlayParams,
) -> Option<HexsideRef> {
    let center = hex_world_pos(coord, origin, overlay);
    // The clicked point's offset from the hex centre points toward an edge;
    // the nearest neighbour is the one whose centre direction best matches it.
    let off = Vec3::new(hit.x - center.x, 0.0, hit.z - center.z);
    if off.length() < 1e-3 {
        return None;
    }
    let neighbour = coord.neighbors().into_iter().max_by(|a, b| {
        let da = edge_alignment(center, *a, off, origin, overlay);
        let db = edge_alignment(center, *b, off, origin, overlay);
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })?;
    Some(HexsideRef::new(coord, neighbour))
}

/// Dot product of the click offset with the (normalised) direction from the
/// hex centre toward `neighbour` — higher means the click is more toward that
/// neighbour's shared edge.
fn edge_alignment(
    center: Vec3,
    neighbour: HexCoord,
    off: Vec3,
    origin: bevy::math::Vec2,
    overlay: &omdurman_types::OverlayParams,
) -> f32 {
    let n = hex_world_pos(neighbour, origin, overlay);
    let dir = Vec3::new(n.x - center.x, 0.0, n.z - center.z);
    let len = dir.length();
    if len < 1e-3 {
        return f32::MIN;
    }
    off.dot(dir / len)
}

/// In the Hexside editor mode, left-click selects the hexside segment nearest
/// the cursor (so the side panel can assign it a type); right-click clears the
/// selection. No edit is broadcast here — that happens when a type is chosen in
/// [`hexside_editor_ui`].
#[allow(clippy::too_many_arguments)]
pub fn handle_hexside_select(
    mode: Res<EditorMode>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut editor: ResMut<HexEditor>,
) {
    if !mode.is_hexside() {
        return;
    }
    let select = buttons.just_pressed(MouseButton::Left);
    let clear = buttons.just_pressed(MouseButton::Right);
    if !select && !clear {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_pointer_input()
    {
        return;
    }
    if clear {
        editor.selected_hexside = None;
        return;
    }
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let coord = hit_to_hex(hit, origin, &overlay.params);
    if !game_map.hexes.contains_key(&coord) {
        return;
    }
    if let Some(edge) = nearest_edge(coord, hit, origin, &overlay.params) {
        editor.selected_hexside = Some(edge);
    }
}

/// The hotkey letter shown for a hexside kind in the panel. Kept in sync with
/// [`hexside_hotkey`] (the single source of truth for the key→action mapping).
fn hexside_hotkey_label(kind: HexsideKind) -> &'static str {
    match kind {
        HexsideKind::Wall => "W",
        HexsideKind::Gate => "G",
        HexsideKind::Breach => "B",
        HexsideKind::Khor => "K",
        HexsideKind::Crest => "C",
        HexsideKind::ZaribaThornHedge => "T",
        HexsideKind::ZaribaTrench => "R",
    }
}

/// The hexside action a hotkey maps to in the Hexside editor:
/// `Some(Some(kind))` sets that feature, `Some(None)` clears it, `None` is no
/// binding. Mnemonic where possible; these only fire in hexside mode, so they
/// don't clash with the terrain-editor terrain keys.
fn hexside_hotkey(keys: &ButtonInput<KeyCode>) -> Option<Option<HexsideKind>> {
    let k = |code| keys.just_pressed(code);
    match () {
        _ if k(KeyCode::KeyW) => Some(Some(HexsideKind::Wall)),
        _ if k(KeyCode::KeyG) => Some(Some(HexsideKind::Gate)),
        _ if k(KeyCode::KeyB) => Some(Some(HexsideKind::Breach)),
        _ if k(KeyCode::KeyK) => Some(Some(HexsideKind::Khor)),
        _ if k(KeyCode::KeyC) => Some(Some(HexsideKind::Crest)),
        _ if k(KeyCode::KeyT) => Some(Some(HexsideKind::ZaribaThornHedge)),
        _ if k(KeyCode::KeyR) => Some(Some(HexsideKind::ZaribaTrench)),
        // Clear the feature.
        _ if k(KeyCode::KeyN) || k(KeyCode::Delete) || k(KeyCode::Backspace) => Some(None),
        _ => None,
    }
}

/// Mutate the live hexside set and broadcast a [`GameEvent::HexsideEdit`]; used
/// by both the side-panel buttons and the hotkeys.
fn apply_hexside_edit(
    edge: HexsideRef,
    kind: Option<HexsideKind>,
    map: omdurman_types::MapKind,
    game_map: &mut GameMap,
    pending: &mut PendingEdits,
    dirty: &mut crate::AnnotationsDirty,
) {
    match kind {
        Some(k) => {
            game_map.hexsides.insert(edge, k);
        }
        None => {
            game_map.hexsides.remove(&edge);
        }
    }
    pending
        .outgoing_broadcast
        .push(NetMsg::Game(GameEvent::HexsideEdit { map, edge, kind }));
    dirty.mark();
}

/// In the Hexside editor mode, number/letter keys assign a feature type to the
/// currently selected segment (see [`hexside_hotkey`]). No-op when no segment is
/// selected or a text field has keyboard focus.
#[allow(clippy::too_many_arguments)]
pub fn handle_hexside_keys(
    mode: Res<EditorMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    editor: Res<HexEditor>,
    mut game_map: ResMut<GameMap>,
    mut pending: ResMut<PendingEdits>,
    mut dirty: ResMut<crate::AnnotationsDirty>,
    active: Res<crate::ActiveEditMap>,
) {
    if !mode.is_hexside() {
        return;
    }
    let Some(edge) = editor.selected_hexside else {
        return;
    };
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_keyboard_input()
    {
        return;
    }
    if let Some(kind) = hexside_hotkey(&keys) {
        apply_hexside_edit(
            edge,
            kind,
            active.0,
            &mut game_map,
            &mut pending,
            &mut dirty,
        );
    }
}

/// Draw excluded hexes with a red outline while in Editor mode, so the holes in
/// the map (board furniture) are visible during terrain editing.
pub fn draw_excluded_hexes(
    mode: Res<EditorMode>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut gizmos: Gizmos,
) {
    if !mode.is_editor() {
        return;
    }
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    for coord in &game_map.excluded {
        let pos = hex_world_pos(*coord, origin, &overlay.params);
        draw_hex_outline(
            &mut gizmos,
            pos,
            overlay.params.hex_size,
            Color::srgb(1.0, 0.2, 0.2),
        );
    }
}

/// The endpoints of the short bar drawn along the shared border of `edge`
/// (the perpendicular-bisector segment at the midpoint of the two hex centres).
fn hexside_segment(edge: &HexsideRef, origin: Vec2, overlay: &HexOverlay) -> (Vec3, Vec3) {
    let a = hex_world_pos(edge.a, origin, &overlay.params);
    let b = hex_world_pos(edge.b, origin, &overlay.params);
    let mid = (a + b) * 0.5;
    let along = (b - a).normalize_or_zero();
    let perp = Vec3::new(-along.z, 0.0, along.x); // perpendicular in the ground plane
    let half = overlay.params.hex_size * 0.5;
    (
        Vec3::new(mid.x, 1.0, mid.z) - perp * half,
        Vec3::new(mid.x, 1.0, mid.z) + perp * half,
    )
}

/// Draw all hexsides as a coloured bar along the shared edge, in both the
/// terrain Editor and the dedicated Hexside editor modes, so painted
/// walls/khors/etc. are visible. In Hexside mode the selected segment is
/// highlighted (drawn thicker, in cyan) so it's clear what a type applies to.
pub fn draw_hexsides(
    mode: Res<EditorMode>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    editor: Res<HexEditor>,
    mut gizmos: Gizmos,
) {
    if !mode.is_editor() && !mode.is_hexside() {
        return;
    }
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    for (edge, kind) in &game_map.hexsides {
        let (p0, p1) = hexside_segment(edge, origin, &overlay);
        gizmos.line(p0, p1, hexside_color(*kind));
    }
    // Highlight the selected segment (Hexside mode), even if it currently has
    // no feature, so the user sees what they're about to set.
    if mode.is_hexside()
        && let Some(edge) = editor.selected_hexside
    {
        let (p0, p1) = hexside_segment(&edge, origin, &overlay);
        let sel = Color::srgb(0.2, 0.9, 1.0);
        // A few offset lines to fake a thicker, more visible highlight.
        for d in [-1.0_f32, 0.0, 1.0] {
            let off = Vec3::new(0.0, 0.0, 0.0) + Vec3::Y * d;
            gizmos.line(p0 + off, p1 + off, sel);
        }
    }
}

fn hexside_color(kind: HexsideKind) -> Color {
    match kind {
        HexsideKind::Wall => Color::srgb(0.85, 0.85, 0.85),
        HexsideKind::Gate => Color::srgb(0.9, 0.8, 0.2),
        HexsideKind::Breach => Color::srgb(0.9, 0.4, 0.1),
        HexsideKind::Khor => Color::srgb(0.4, 0.3, 0.15),
        HexsideKind::Crest => Color::srgb(0.6, 0.45, 0.3),
        HexsideKind::ZaribaThornHedge => Color::srgb(0.3, 0.55, 0.2),
        HexsideKind::ZaribaTrench => Color::srgb(0.5, 0.5, 0.6),
    }
}

/// In Hexside mode, preview the segment under the cursor — the one a click would
/// select — as a dim bar, so it's clear which side will be picked. Skipped while
/// the pointer is over egui (so it doesn't fight the side panel).
pub fn draw_hexside_hover(
    mode: Res<EditorMode>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    editor: Res<HexEditor>,
    mut gizmos: Gizmos,
) {
    if !mode.is_hexside() {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_pointer_input()
    {
        return;
    }
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let coord = hit_to_hex(hit, origin, &overlay.params);
    if !game_map.hexes.contains_key(&coord) {
        return;
    }
    let Some(edge) = nearest_edge(coord, hit, origin, &overlay.params) else {
        return;
    };
    // Don't double-draw the already-selected segment (it has its own bright
    // highlight from `draw_hexsides`).
    if editor.selected_hexside == Some(edge) {
        return;
    }
    let (p0, p1) = hexside_segment(&edge, origin, &overlay);
    gizmos.line(p0, p1, Color::srgba(0.2, 0.9, 1.0, 0.5));
}

pub fn draw_editor_highlight(
    mode: Res<EditorMode>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    editor: Res<HexEditor>,
    mut gizmos: Gizmos,
) {
    if !mode.is_editor() {
        return;
    }
    let Some(coord) = editor.selected else { return };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let pos = hex_world_pos(coord, origin, &overlay.params);
    draw_hex_outline(
        &mut gizmos,
        pos,
        overlay.params.hex_size,
        Color::srgb(0.0, 1.0, 0.0),
    );
}

/// Direction in the ground plane (XZ) the Nile current flows for a hex with
/// `flow.dir == dir`, derived from the hex's world centre and the centre of
/// its `dir`-th neighbour so it stays correct under any orientation / stagger.
/// `None` when the neighbour and hex coincide (degenerate overlay).
fn flow_world_dir(
    coord: HexCoord,
    flow: NileFlow,
    origin: bevy::math::Vec2,
    overlay: &omdurman_types::OverlayParams,
) -> Option<Vec3> {
    let c = hex_world_pos(coord, origin, overlay);
    let n = hex_world_pos(coord.neighbors()[flow.dir as usize], origin, overlay);
    let v = Vec3::new(n.x - c.x, 0.0, n.z - c.z);
    let len = v.length();
    (len > 1e-3).then(|| v / len)
}

/// Draw the single Nile-current arrow in the centre of every `is_nile` hex
/// that has a current annotated, while in Editor mode. The arrow points
/// **downstream** (the direction the current flows / the direction a gunboat
/// moves to go downstream — §5.11, §5.24).
pub fn draw_nile_flow_indicators(
    mode: Res<EditorMode>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut gizmos: Gizmos,
) {
    if !mode.is_editor() {
        return;
    }
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let size = overlay.params.hex_size;
    let arrow_len = size * 0.7;
    // Stroke width of the arrow, in world units (gizmo lines are 1px, so the
    // helper stacks parallel strands to fake this).
    let arrow_thickness = size * 0.14;

    for (coord, data) in &game_map.hexes {
        if !data.terrain.is_nile() {
            continue;
        }
        let Some(flow) = data.nile_flow else {
            continue;
        };
        let Some(dir) = flow_world_dir(*coord, flow, origin, &overlay.params) else {
            continue;
        };
        let center = hex_world_pos(*coord, origin, &overlay.params);
        let center = Vec3::new(center.x, 1.5, center.z);
        let tail = center - dir * (arrow_len * 0.5);
        let tip = center + dir * (arrow_len * 0.5);
        crate::render::draw_ground_arrow(
            &mut gizmos,
            tail,
            tip,
            arrow_thickness,
            Color::srgb(1.0, 0.55, 0.0),
        );
    }
}

pub fn editor_ui(
    mut contexts: EguiContexts,
    mode: Res<EditorMode>,
    mut editor: ResMut<HexEditor>,
    mut game_map: ResMut<GameMap>,
    mut pending: ResMut<PendingEdits>,
    mut clip: ResMut<SidebarClip>,
    mut dirty: ResMut<crate::AnnotationsDirty>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    active: Res<crate::ActiveEditMap>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_editor() {
        clip.right_sidebar = None;
        return;
    }

    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Some(vp_size) = camera.logical_viewport_size() else {
        return;
    };

    // hex labels & optional terrain colour overlay (single pass over hexes)
    {
        // Clip to the canvas area, excluding the sidebar from the previous frame so
        // background-order painters don't bleed over the panel.
        let canvas_rect = {
            let screen = ctx.viewport_rect();
            match clip.right_sidebar {
                Some(sidebar) => {
                    egui::Rect::from_min_max(screen.min, egui::pos2(sidebar.left(), screen.max.y))
                }
                None => screen,
            }
        };
        // Paint into the shared background layer so shapes append in call-order with
        // panels that share LayerId::background() (CentralPanel, SidePanel). The
        // SidePanel adds its shapes later, so they paint on top — which is what we want.
        let mut label_painter = ctx.layer_painter(egui::LayerId::background());
        label_painter.set_clip_rect(canvas_rect);
        // Tile terrain/name labels at 0.75× the former 10pt.
        let font_size = 7.5;
        let char_w = font_size * 0.6;
        let line_h = font_size * 1.4;
        let padding = 3.0;
        let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
        let size = overlay.params.hex_size;
        let overlay_painter = editor.show_terrain_overlay.then(|| {
            let mut p = ctx.layer_painter(egui::LayerId::background());
            p.set_clip_rect(canvas_rect);
            p
        });
        // First pass: terrain colour overlays (so labels paint on top of them).
        if let Some(ref overlay_painter) = overlay_painter {
            for (coord, data) in &game_map.hexes {
                let center = hex_world_pos(*coord, origin, &overlay.params);
                let corners = crate::render::hex_corners(Vec3::new(center.x, 1.5, center.z), size);
                let mut screen_verts = Vec::with_capacity(6);
                for world in corners {
                    if let Ok(screen) = camera.world_to_viewport(cam_transform, world) {
                        screen_verts.push(egui::pos2(screen.x, screen.y));
                    }
                }
                if screen_verts.len() == 6 {
                    let [r, g, b, a] = data.terrain.overlay_color();
                    let color = egui::Color32::from_rgba_unmultiplied(
                        (r * 255.0) as u8,
                        (g * 255.0) as u8,
                        (b * 255.0) as u8,
                        (a * 255.0) as u8,
                    );
                    overlay_painter.add(egui::Shape::convex_polygon(
                        screen_verts,
                        color,
                        egui::Stroke::NONE,
                    ));
                }
            }
        }
        // Second pass: hex labels on top of the overlay.
        for (coord, data) in &game_map.hexes {
            let center = hex_world_pos(*coord, origin, &overlay.params);
            let pos = Vec3::new(center.x, 0.1, center.z);
            let Ok(screen) = camera.world_to_viewport(cam_transform, pos) else {
                continue;
            };
            if screen.x < 0.0 || screen.x > vp_size.x || screen.y < 0.0 || screen.y > vp_size.y {
                continue;
            }
            let text = match &data.name {
                Some(n) => format!("{}\n{}", data.terrain, n),
                None => format!("{}", data.terrain),
            };
            let lines: Vec<&str> = text.lines().collect();
            let max_line = lines.iter().map(|l| l.len()).max().unwrap_or(0) as f32;
            let rect = egui::Rect::from_center_size(
                egui::pos2(screen.x, screen.y),
                egui::vec2(
                    max_line * char_w + 2.0 * padding,
                    lines.len() as f32 * line_h + 2.0 * padding,
                ),
            );
            label_painter.rect_filled(rect, 3.0, egui::Color32::from_black_alpha(160));
            label_painter.text(
                egui::pos2(screen.x, screen.y),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::monospace(font_size),
                egui::Color32::WHITE,
            );
        }
    }

    // ---- sidebar panel (Order::Middle, on top of background) ----
    let response = egui::SidePanel::right("editor_panel")
        .resizable(true)
        .default_width(200.0)
        .width_range(150.0..=500.0)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(45))
                .inner_margin(egui::Margin::symmetric(12, 12)),
        )
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
            if let Some(coord) = editor.selected {
                ui.label(format!("hex  q {}  r {}", coord.q, coord.r));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("name");
                    ui.add(
                        egui::TextEdit::singleline(&mut editor.name).desired_width(f32::INFINITY),
                    );
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label("type");
                    // The dropdown lists every real terrain plus a "Not playable"
                    // pseudo-type that excludes the hex from the map (§dual-map).
                    let selected_text = if editor.not_playable {
                        "Not playable".to_string()
                    } else {
                        format!("{}", editor.terrain)
                    };
                    egui::ComboBox::from_id_salt("terrain")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for t in Terrain::iter() {
                                if ui
                                    .selectable_label(
                                        !editor.not_playable && editor.terrain == t,
                                        format!("{}", t),
                                    )
                                    .clicked()
                                {
                                    editor.terrain = t;
                                    editor.not_playable = false;
                                }
                            }
                            ui.separator();
                            if ui
                                .selectable_label(editor.not_playable, "Not playable")
                                .clicked()
                            {
                                editor.not_playable = true;
                            }
                        });
                });

                // Nile current annotation: a single arrow per hex, pointing
                // downstream, rotated by the +/- buttons (rulebook §5.11,
                // §5.24). Every Nile hex always carries a current, so the only
                // choice is its direction. Hidden for the Not-playable type.
                if !editor.not_playable && editor.terrain.is_nile() {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("Nile current").color(egui::Color32::from_gray(200)),
                    );
                    ui.add_space(2.0);
                    // Direction labels in HexCoord::neighbors order.
                    const DIR_LABELS: [&str; 6] = ["E", "SE", "SW", "W", "NW", "NE"];
                    let flow = editor.nile_flow.get_or_insert_with(NileFlow::default);
                    ui.horizontal(|ui| {
                        if ui.button("⟲ -").clicked() {
                            *flow = flow.rotated(-1);
                        }
                        ui.label(
                            egui::RichText::new(format!(
                                "↦ {} ({})",
                                DIR_LABELS[flow.dir as usize], flow.dir
                            ))
                            .color(egui::Color32::from_rgb(255, 160, 60)),
                        );
                        if ui.button("+ ⟳").clicked() {
                            *flow = flow.rotated(1);
                        }
                    });
                }
            } else {
                ui.label("click a hex to select");
            }
            ui.add_space(8.0);
            {
                let prev = editor.show_terrain_overlay;
                ui.checkbox(&mut editor.show_terrain_overlay, "terrain overlay");
                if prev != editor.show_terrain_overlay {
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Game(GameEvent::ShowTerrainOverlay(
                            editor.show_terrain_overlay,
                        )));
                }
            }

            // "Not playable" is a type in the dropdown above (board furniture:
            // logos, turn track, …). Hexside/wall editing lives in its own mode.
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!("{} hexes not playable", game_map.excluded.len()))
                    .size(11.0)
                    .color(egui::Color32::from_gray(160)),
            );
        });
    clip.right_sidebar = Some(response.response.rect);

    let Some(coord) = editor.selected else { return };
    let is_excluded = game_map.excluded.contains(&coord);

    if editor.not_playable {
        // "Not playable" picked: exclude the hex if it isn't already. The actual
        // reclip (removing it from `hexes`) happens when the event echoes back
        // through `game_apply`.
        if !is_excluded && game_map.hexes.contains_key(&coord) {
            pending
                .outgoing_broadcast
                .push(NetMsg::Game(GameEvent::ExcludeHex {
                    map: active.0,
                    q: coord.q,
                    r: coord.r,
                    excluded: true,
                }));
            dirty.mark();
        }
        return;
    }

    // A real terrain is selected. If the hex was excluded, restore it first;
    // the terrain edit then lands once it's back in the playable set.
    if is_excluded {
        pending
            .outgoing_broadcast
            .push(NetMsg::Game(GameEvent::ExcludeHex {
                map: active.0,
                q: coord.q,
                r: coord.r,
                excluded: false,
            }));
        dirty.mark();
        return;
    }

    // Normal terrain/name/flow edit on a playable hex.
    if let Some(d) = game_map.hexes.get(&coord) {
        let terrain = editor.terrain;
        let editor_name = editor.name.clone();
        // Flow is only carried by Nile hexes; on any other terrain it's dropped.
        let new_flow = if terrain.is_nile() {
            editor.nile_flow
        } else {
            None
        };
        let new_name = (!editor_name.is_empty()).then(|| editor_name.clone());
        let changed = d.terrain != terrain || d.name != new_name || d.nile_flow != new_flow;
        if changed {
            pending
                .outgoing_broadcast
                .push(NetMsg::Game(GameEvent::MapEdit {
                    map: active.0,
                    q: coord.q,
                    r: coord.r,
                    terrain: terrain.to_u8(),
                    name: editor_name,
                    nile_flow: new_flow,
                }));
            if let Some(d) = game_map.hexes.get_mut(&coord) {
                d.terrain = terrain;
                d.name = new_name;
                d.nile_flow = new_flow;
            }
            // Map edits mutate in-memory state and are recorded in the event
            // log. Mark annotations.ron dirty; the flush system debounces.
            dirty.mark();
        }
    }
}

/// Side panel for the Hexside editor mode: shows the selected segment's current
/// feature and a button per type (plus "none") to assign it. Applying a type
/// updates the live map and broadcasts a [`GameEvent::HexsideEdit`].
#[allow(clippy::too_many_arguments)]
pub fn hexside_editor_ui(
    mut contexts: EguiContexts,
    mode: Res<EditorMode>,
    editor: Res<HexEditor>,
    mut game_map: ResMut<GameMap>,
    mut pending: ResMut<PendingEdits>,
    mut clip: ResMut<SidebarClip>,
    mut dirty: ResMut<crate::AnnotationsDirty>,
    active: Res<crate::ActiveEditMap>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_hexside() {
        clip.right_sidebar = None;
        return;
    }

    let mut apply: Option<(HexsideRef, Option<HexsideKind>)> = None;

    let response = egui::SidePanel::right("hexside_editor_panel")
        .resizable(true)
        .default_width(180.0)
        .width_range(140.0..=320.0)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(45))
                .inner_margin(egui::Margin::symmetric(12, 12)),
        )
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
            ui.label(
                egui::RichText::new("Hexside editor")
                    .size(15.0)
                    .color(egui::Color32::from_gray(220)),
            );
            ui.label(
                egui::RichText::new("L-click a segment to select · R-click to deselect")
                    .size(11.0)
                    .color(egui::Color32::from_gray(160)),
            );
            ui.label(
                egui::RichText::new("then press a key (or click) to set its type")
                    .size(11.0)
                    .color(egui::Color32::from_gray(160)),
            );
            ui.separator();

            let Some(edge) = editor.selected_hexside else {
                ui.label(
                    egui::RichText::new("no segment selected").color(egui::Color32::from_gray(140)),
                );
                return;
            };

            let current = game_map.hexsides.get(&edge).copied();
            ui.label(format!(
                "({}, {}) — ({}, {})",
                edge.a.q, edge.a.r, edge.b.q, edge.b.r
            ));
            ui.label(format!(
                "type: {}",
                current
                    .map(|k| k.to_string())
                    .unwrap_or_else(|| "none".into())
            ));
            ui.add_space(4.0);

            // "none" clears the feature.
            if ui
                .add(egui::Button::selectable(current.is_none(), "none  [N]"))
                .clicked()
            {
                apply = Some((edge, None));
            }
            for k in HexsideKind::iter() {
                let label = format!("{}  [{}]", k, hexside_hotkey_label(k));
                if ui
                    .add(egui::Button::selectable(current == Some(k), label))
                    .clicked()
                {
                    apply = Some((edge, Some(k)));
                }
            }
        });
    clip.right_sidebar = Some(response.response.rect);

    if let Some((edge, kind)) = apply {
        apply_hexside_edit(
            edge,
            kind,
            active.0,
            &mut game_map,
            &mut pending,
            &mut dirty,
        );
    }
}
