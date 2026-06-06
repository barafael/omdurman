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

/// A queued edit to apply to every selected hex on the next frame. Multi-select
/// edits are *action-triggered* (set a terrain, press Delete, rotate the
/// current) rather than the old continuous diff against a single hex, so
/// applying never fights per-hex differences across the set.
#[derive(Clone, Debug)]
pub enum PendingApply {
    /// Set all selected hexes to this terrain (clearing Not-playable).
    Terrain(Terrain),
    /// Mark all selected hexes Not playable.
    NotPlayable,
    /// Rotate the Nile current on all selected Nile hexes by ±1 sixth.
    RotateFlow(i8),
    /// Set the anchor hex's name to the panel's text.
    Name,
    /// Set the road overlay on/off for all selected hexes.
    Road(bool),
}

#[derive(Resource, Default)]
pub struct HexEditor {
    /// The set of selected hexes. Edits apply to all of them.
    pub selection: std::collections::HashSet<HexCoord>,
    /// The "anchor" hex whose terrain/name/flow populate the side panel. Always
    /// a member of `selection` while a selection exists; `None` when empty.
    pub anchor: Option<HexCoord>,
    pub name: String,
    pub terrain: Terrain,
    /// Nile current of the anchor hex; `None` = no current. Only meaningful
    /// (and only shown) when `terrain.is_nile()`.
    pub nile_flow: Option<NileFlow>,
    /// Whether the anchor hex has a road overlay (Terrain Effects Chart).
    pub road: bool,
    pub show_terrain_overlay: bool,
    /// The hexside segment currently selected in the Hexside editor mode, if
    /// any. Clicking a segment selects it; the side panel then assigns a type.
    pub selected_hexside: Option<HexsideRef>,
    /// Whether the anchor hex's "type" dropdown is on the **Not playable**
    /// pseudo-terrain — i.e. board furniture (logo, turn track, …) excluded from
    /// the map. Applied via [`GameEvent::ExcludeHex`] rather than terrain.
    pub not_playable: bool,
    /// An edit queued by a key/dropdown action, consumed by `editor_ui` and
    /// applied to every hex in `selection` (§multi-select).
    pub pending_apply: Option<PendingApply>,
}

/// Whether `coord` is part of the grid (a playable hex or an excluded one).
fn in_grid(coord: HexCoord, game_map: &GameMap) -> bool {
    game_map.hexes.contains_key(&coord) || game_map.excluded.contains(&coord)
}

/// Make `coord` the anchor and load its terrain/name/flow into the side panel.
/// Assumes `coord` is in-grid.
fn load_anchor(coord: HexCoord, editor: &mut HexEditor, game_map: &GameMap) {
    editor.anchor = Some(coord);
    if let Some(data) = game_map.hexes.get(&coord) {
        editor.name = data.name.clone().unwrap_or_default();
        editor.terrain = data.terrain;
        editor.nile_flow = data.nile_flow;
        editor.road = data.road;
        editor.not_playable = false;
    } else {
        // Excluded hex: in-grid but Not playable; no terrain data while excluded.
        editor.name = String::new();
        editor.nile_flow = None;
        editor.road = false;
        editor.not_playable = true;
    }
}

/// Letter/number keys set terrain; Delete/Backspace marks Not playable;
/// Ctrl+PgUp/PgDown rotate the Nile current; Ctrl+arrows extend the selection
/// from the anchor. All edits apply to **every** selected hex (§multi-select).
/// Plain arrows / PgUp/PgDown drive the camera (see `camera_control`).
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
    let Some(anchor) = editor.anchor else {
        return;
    };
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    // Ctrl+arrows extend the selection to a neighbour of the anchor (and move
    // the anchor there). Left/Right step ∓q, Up/Down step ∓r. Off-grid ignored.
    if ctrl {
        let target = match () {
            _ if keys.just_pressed(KeyCode::ArrowLeft) => {
                Some(HexCoord::new(anchor.q - 1, anchor.r))
            }
            _ if keys.just_pressed(KeyCode::ArrowRight) => {
                Some(HexCoord::new(anchor.q + 1, anchor.r))
            }
            _ if keys.just_pressed(KeyCode::ArrowUp) => Some(HexCoord::new(anchor.q, anchor.r - 1)),
            _ if keys.just_pressed(KeyCode::ArrowDown) => {
                Some(HexCoord::new(anchor.q, anchor.r + 1))
            }
            _ => None,
        };
        if let Some(target) = target {
            if in_grid(target, &game_map) {
                editor.selection.insert(target);
                load_anchor(target, &mut editor, &game_map);
            }
            return;
        }

        // Ctrl+PgUp/PgDown queue a Nile-current rotation for all selected
        // Nile hexes (plain PgUp/PgDown tilt the camera).
        if keys.just_pressed(KeyCode::PageUp) {
            editor.pending_apply = Some(PendingApply::RotateFlow(1));
            return;
        }
        if keys.just_pressed(KeyCode::PageDown) {
            editor.pending_apply = Some(PendingApply::RotateFlow(-1));
            return;
        }
    }

    // Delete/Backspace marks every selected hex Not playable.
    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        editor.not_playable = true;
        editor.pending_apply = Some(PendingApply::NotPlayable);
        return;
    }

    let t = match () {
        // Primary mnemonics requested: T trees, S swamp, R rough.
        _ if keys.just_pressed(KeyCode::KeyT) => Some(Terrain::Trees),
        _ if keys.just_pressed(KeyCode::KeyS) => Some(Terrain::Swamp),
        _ if keys.just_pressed(KeyCode::KeyR) => Some(Terrain::Rough),
        _ if keys.just_pressed(KeyCode::KeyD) => Some(Terrain::Desert),
        _ if keys.just_pressed(KeyCode::KeyP) => Some(Terrain::Palm),
        _ if keys.just_pressed(KeyCode::KeyE) => Some(Terrain::Shrubs), // scrub/scrEEn
        _ if keys.just_pressed(KeyCode::KeyB) => Some(Terrain::BlueNile),
        _ if keys.just_pressed(KeyCode::KeyW) => Some(Terrain::WhiteNile),
        _ if keys.just_pressed(KeyCode::KeyV) => Some(Terrain::RiverNile), // riVer
        _ if keys.just_pressed(KeyCode::KeyF) => Some(Terrain::Fortress),
        _ if keys.just_pressed(KeyCode::KeyK) => Some(Terrain::Khartoum),
        _ if keys.just_pressed(KeyCode::KeyI) => Some(Terrain::Tuti), // tutI
        _ if keys.just_pressed(KeyCode::KeyH) => Some(Terrain::Hogali),
        _ if keys.just_pressed(KeyCode::KeyU) => Some(Terrain::Buri),
        _ if keys.just_pressed(KeyCode::KeyM) => Some(Terrain::FortMakran),
        _ if keys.just_pressed(KeyCode::Digit1) => Some(Terrain::FortBuri),
        _ if keys.just_pressed(KeyCode::KeyN) => Some(Terrain::NorthFort),
        _ => None,
    };
    if let Some(t) = t {
        editor.terrain = t;
        editor.not_playable = false;
        editor.pending_apply = Some(PendingApply::Terrain(t));
    }
}

pub fn handle_hex_editor_click(
    mode: Res<EditorMode>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
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

    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if ctrl && shift {
        // Ctrl+Shift+click: remove a hex from the multi-selection.
        editor.selection.remove(&coord);
        if editor.anchor == Some(coord) {
            // Re-anchor to any remaining member, or clear.
            let next = editor.selection.iter().next().copied();
            match next {
                Some(c) => load_anchor(c, &mut editor, &game_map),
                None => editor.anchor = None,
            }
        }
    } else if ctrl {
        // Ctrl+click: add a hex to the multi-selection (becomes the anchor).
        if in_grid(coord, &game_map) {
            editor.selection.insert(coord);
            load_anchor(coord, &mut editor, &game_map);
        }
    } else if in_grid(coord, &game_map) {
        // Plain click: replace the selection with this single hex.
        editor.selection.clear();
        editor.selection.insert(coord);
        load_anchor(coord, &mut editor, &game_map);
    } else {
        // Plain click on empty space: clear the selection.
        editor.selection.clear();
        editor.anchor = None;
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
        HexsideKind::KhorShambat => "S",
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
        _ if k(KeyCode::KeyS) => Some(Some(HexsideKind::KhorShambat)),
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

// ── Hexside rendering as ground-plane mesh quads ───────────────────────────
//
// Gizmo lines are sub-pixel-thin with no width control, so painted hexsides
// were nearly invisible. Instead draw each hexside as a flat coloured quad laid
// on the map: a real mesh bar with proper width that reads clearly from the
// top-down camera. A pool of reusable quad entities is repositioned each frame
// (grown on demand), mirroring how the selection marker works.

/// A pooled hexside bar (a flat quad on the ground plane).
#[derive(Component)]
pub struct HexsideQuad;

/// Reusable pool of hexside quad entities + the shared unit-square mesh they all
/// use (scaled per-bar via Transform). Materials are per-entity so each bar can
/// take its own colour.
#[derive(Resource, Default)]
pub struct HexsideQuads {
    mesh: Handle<Mesh>,
    pool: Vec<Entity>,
}

/// One-time setup: create the shared unit quad mesh used by every hexside bar.
pub fn setup_hexside_quads(mut quads: ResMut<HexsideQuads>, mut meshes: ResMut<Assets<Mesh>>) {
    quads.mesh = meshes.add(Rectangle::new(1.0, 1.0));
}

/// Bar width as a fraction of hex size — chunky enough to be obvious.
const HEXSIDE_WIDTH_FRAC: f32 = 0.16;

/// Place a flat coloured quad over the hexside `(p0, p1)` segment: centred on
/// the segment, rotated to lie along it, scaled to (length × width). `width`
/// and `y` (height above the map) and `color` are caller-chosen so selection /
/// hover bars can be wider, higher, and brighter than plain ones.
fn place_hexside_quad(
    transform: &mut Transform,
    material: &mut StandardMaterial,
    p0: Vec3,
    p1: Vec3,
    width: f32,
    y: f32,
    color: Color,
) {
    let mid = (p0 + p1) * 0.5;
    let len = p0.distance(p1).max(0.001);
    let dir = (p1 - p0) / len;
    // Lay the quad flat (rotate −90° about X: local +X→world +X, local +Y→world
    // +Z), then yaw about Y so the quad's local +X (its length axis) aligns with
    // the segment direction `dir`. Yaw θ sends local +X to (cosθ, 0, −sinθ),
    // so to match dir=(dx,0,dz) we need θ = atan2(−dz, dx).
    let angle = (-dir.z).atan2(dir.x);
    *transform = Transform::from_translation(Vec3::new(mid.x, y, mid.z))
        .with_rotation(
            Quat::from_rotation_y(angle) * Quat::from_rotation_x(-std::f32::consts::PI / 2.0),
        )
        .with_scale(Vec3::new(len, width, 1.0));
    material.base_color = color;
}

/// Rebuild the hexside quad pool each frame from the painted hexsides plus the
/// hover/selection bars, in the terrain Editor and Hexside editor modes.
/// Unused pooled quads are parked invisible.
#[allow(clippy::too_many_arguments)]
pub fn update_hexside_quads(
    mode: Res<EditorMode>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    editor: Res<HexEditor>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    mut contexts: EguiContexts,
    mut quads: ResMut<HexsideQuads>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q: Query<
        (
            &mut Transform,
            &mut Visibility,
            &MeshMaterial3d<StandardMaterial>,
        ),
        With<HexsideQuad>,
    >,
) {
    let active = mode.is_editor() || mode.is_hexside();

    // Gather the bars to draw this frame: (p0, p1, width, y, color).
    let mut bars: Vec<(Vec3, Vec3, f32, f32, Color)> = Vec::new();
    if active {
        let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
        let base_w = overlay.params.hex_size * HEXSIDE_WIDTH_FRAC;
        for (edge, kind) in &game_map.hexsides {
            let (p0, p1) = hexside_segment(edge, origin, &overlay);
            bars.push((p0, p1, base_w, 1.2, hexside_color(*kind)));
        }
        if mode.is_hexside() {
            // Hover preview (segment under the cursor), unless over the panel.
            let over_ui = contexts
                .ctx_mut()
                .map(|c| c.wants_pointer_input())
                .unwrap_or(false);
            if !over_ui && let Some(hit) = raycast_ground(&windows, &cameras) {
                let coord = hit_to_hex(hit, origin, &overlay.params);
                if game_map.hexes.contains_key(&coord)
                    && let Some(edge) = nearest_edge(coord, hit, origin, &overlay.params)
                    && editor.selected_hexside != Some(edge)
                {
                    let (p0, p1) = hexside_segment(&edge, origin, &overlay);
                    bars.push((p0, p1, base_w * 1.6, 1.4, Color::srgba(0.2, 0.9, 1.0, 0.6)));
                }
            }
            // Selected segment — widest, brightest, on top.
            if let Some(edge) = editor.selected_hexside {
                let (p0, p1) = hexside_segment(&edge, origin, &overlay);
                bars.push((p0, p1, base_w * 1.9, 1.6, Color::srgb(0.2, 0.9, 1.0)));
            }
        }
    }

    // Grow the pool to fit.
    while quads.pool.len() < bars.len() {
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let id = commands
            .spawn((
                HexsideQuad,
                Mesh3d(quads.mesh.clone()),
                MeshMaterial3d(material),
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        quads.pool.push(id);
    }

    // Position the needed quads; hide the rest.
    for (i, &entity) in quads.pool.iter().enumerate() {
        let Ok((mut transform, mut visibility, mat_handle)) = q.get_mut(entity) else {
            continue;
        };
        if let Some(&(p0, p1, width, y, color)) = bars.get(i) {
            if let Some(material) = materials.get_mut(&mat_handle.0) {
                place_hexside_quad(&mut transform, material, p0, p1, width, y, color);
            }
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn hexside_color(kind: HexsideKind) -> Color {
    match kind {
        HexsideKind::Wall => Color::srgb(0.25, 0.25, 0.25),
        HexsideKind::Gate => Color::srgb(0.9, 0.8, 0.2),
        HexsideKind::Breach => Color::srgb(0.9, 0.4, 0.1),
        HexsideKind::Khor => Color::srgb(0.4, 0.3, 0.15),
        HexsideKind::Crest => Color::srgb(0.6, 0.45, 0.3),
        HexsideKind::ZaribaThornHedge => Color::srgb(0.3, 0.55, 0.2),
        HexsideKind::ZaribaTrench => Color::srgb(0.5, 0.5, 0.6),
        // Khor Shambat: a brighter blue-tinted khor so the named one stands out.
        HexsideKind::KhorShambat => Color::srgb(0.2, 0.45, 0.55),
    }
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
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    // Every selected hex gets a green outline; the anchor (whose state the panel
    // shows) gets a brighter, slightly larger one to stand out.
    for &coord in &editor.selection {
        let pos = hex_world_pos(coord, origin, &overlay.params);
        let is_anchor = editor.anchor == Some(coord);
        let color = if is_anchor {
            Color::srgb(0.4, 1.0, 0.4)
        } else {
            Color::srgb(0.0, 0.7, 0.0)
        };
        let size = if is_anchor {
            overlay.params.hex_size
        } else {
            overlay.params.hex_size * 0.92
        };
        draw_hex_outline(&mut gizmos, pos, size, color);
    }
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
            if let Some(coord) = editor.anchor {
                let n = editor.selection.len();
                if n > 1 {
                    ui.label(format!(
                        "{n} hexes selected (anchor q {}  r {})",
                        coord.q, coord.r
                    ));
                } else {
                    ui.label(format!("hex  q {}  r {}", coord.q, coord.r));
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("name");
                    // Editing the name commits to the anchor on focus loss / Enter.
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut editor.name).desired_width(f32::INFINITY),
                    );
                    if resp.lost_focus() || resp.changed() {
                        editor.pending_apply = Some(PendingApply::Name);
                    }
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label("type");
                    // The dropdown lists every real terrain plus a "Not playable"
                    // pseudo-type that excludes the hex from the map (§dual-map).
                    // Picking one applies to every selected hex.
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
                                    editor.pending_apply = Some(PendingApply::Terrain(t));
                                }
                            }
                            ui.separator();
                            if ui
                                .selectable_label(editor.not_playable, "Not playable")
                                .clicked()
                            {
                                editor.not_playable = true;
                                editor.pending_apply = Some(PendingApply::NotPlayable);
                            }
                        });
                });

                // Nile current annotation: a single arrow per hex, pointing
                // downstream, rotated by the +/- buttons (rulebook §5.11,
                // §5.24). Rotating applies to every selected Nile hex.
                if !editor.not_playable && editor.terrain.is_nile() {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("Nile current").color(egui::Color32::from_gray(200)),
                    );
                    ui.add_space(2.0);
                    // Direction labels in HexCoord::neighbors order.
                    const DIR_LABELS: [&str; 6] = ["E", "SE", "SW", "W", "NW", "NE"];
                    let dir = editor.nile_flow.unwrap_or_default().dir;
                    ui.horizontal(|ui| {
                        if ui.button("⟲ -").clicked() {
                            editor.pending_apply = Some(PendingApply::RotateFlow(-1));
                        }
                        ui.label(
                            egui::RichText::new(format!(
                                "↦ {} ({})",
                                DIR_LABELS[dir as usize], dir
                            ))
                            .color(egui::Color32::from_rgb(255, 160, 60)),
                        );
                        if ui.button("+ ⟳").clicked() {
                            editor.pending_apply = Some(PendingApply::RotateFlow(1));
                        }
                    });
                }

                // Road overlay: a road costs 1 MP to move along but leaves the
                // hex's terrain (and its combat effect) intact. Hidden on the
                // Not-playable pseudo-type.
                if !editor.not_playable {
                    ui.add_space(4.0);
                    let mut road = editor.road;
                    if ui.checkbox(&mut road, "road").changed() {
                        editor.road = road;
                        editor.pending_apply = Some(PendingApply::Road(road));
                    }
                }
            } else {
                ui.label("click a hex to select · Ctrl+click adds · Ctrl+Shift+click removes");
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

    // Apply a queued action (terrain/Not-playable/flow/name) to the whole
    // selection. Edits are action-triggered, not a continuous diff, so a
    // multi-hex selection never re-writes every frame or fights per-hex
    // differences.
    let Some(action) = editor.pending_apply.take() else {
        return;
    };
    let targets: Vec<HexCoord> = match &action {
        // Name applies to the anchor only.
        PendingApply::Name => editor.anchor.into_iter().collect(),
        _ => editor.selection.iter().copied().collect(),
    };

    for coord in targets {
        let is_excluded = game_map.excluded.contains(&coord);
        match &action {
            PendingApply::NotPlayable => {
                // Exclude playable hexes; already-excluded ones are a no-op.
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
            }
            PendingApply::Terrain(t) => {
                if is_excluded {
                    // Restore an excluded hex first; it re-enters the map as
                    // Desert and the terrain can be set on a subsequent action.
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Game(GameEvent::ExcludeHex {
                            map: active.0,
                            q: coord.q,
                            r: coord.r,
                            excluded: false,
                        }));
                    dirty.mark();
                    continue;
                }
                let Some(d) = game_map.hexes.get(&coord) else {
                    continue;
                };
                // Preserve each hex's own name; set/clear flow per Nile-ness.
                let name = d.name.clone();
                let new_flow = t.is_nile().then(|| d.nile_flow.unwrap_or_default());
                let road = d.road;
                if d.terrain != *t || d.nile_flow != new_flow {
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Game(GameEvent::MapEdit {
                            map: active.0,
                            q: coord.q,
                            r: coord.r,
                            terrain: t.to_u8(),
                            name: name.clone().unwrap_or_default(),
                            nile_flow: new_flow,
                            road,
                        }));
                    if let Some(d) = game_map.hexes.get_mut(&coord) {
                        d.terrain = *t;
                        d.nile_flow = new_flow;
                    }
                    dirty.mark();
                }
            }
            PendingApply::RotateFlow(delta) => {
                let Some(d) = game_map.hexes.get(&coord) else {
                    continue;
                };
                if !d.terrain.is_nile() {
                    continue;
                }
                let new_flow = Some(d.nile_flow.unwrap_or_default().rotated(*delta));
                let road = d.road;
                pending
                    .outgoing_broadcast
                    .push(NetMsg::Game(GameEvent::MapEdit {
                        map: active.0,
                        q: coord.q,
                        r: coord.r,
                        terrain: d.terrain.to_u8(),
                        name: d.name.clone().unwrap_or_default(),
                        nile_flow: new_flow,
                        road,
                    }));
                if let Some(d) = game_map.hexes.get_mut(&coord) {
                    d.nile_flow = new_flow;
                }
                dirty.mark();
            }
            PendingApply::Name => {
                let Some(d) = game_map.hexes.get(&coord) else {
                    continue;
                };
                let new_name = (!editor.name.is_empty()).then(|| editor.name.clone());
                if d.name != new_name {
                    let road = d.road;
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Game(GameEvent::MapEdit {
                            map: active.0,
                            q: coord.q,
                            r: coord.r,
                            terrain: d.terrain.to_u8(),
                            name: editor.name.clone(),
                            nile_flow: d.nile_flow,
                            road,
                        }));
                    if let Some(d) = game_map.hexes.get_mut(&coord) {
                        d.name = new_name;
                    }
                    dirty.mark();
                }
            }
            PendingApply::Road(on) => {
                // Toggle the road overlay, preserving terrain/name/flow. Skip
                // excluded hexes (no terrain to overlay).
                let Some(d) = game_map.hexes.get(&coord) else {
                    continue;
                };
                if d.road != *on {
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Game(GameEvent::MapEdit {
                            map: active.0,
                            q: coord.q,
                            r: coord.r,
                            terrain: d.terrain.to_u8(),
                            name: d.name.clone().unwrap_or_default(),
                            nile_flow: d.nile_flow,
                            road: *on,
                        }));
                    if let Some(d) = game_map.hexes.get_mut(&coord) {
                        d.road = *on;
                    }
                    dirty.mark();
                }
            }
        }
    }
    // Keep the panel's anchor state in sync after applying (e.g. flow rotation).
    if let Some(a) = editor.anchor
        && let Some(d) = game_map.hexes.get(&a)
    {
        editor.terrain = d.terrain;
        editor.nile_flow = d.nile_flow;
        editor.road = d.road;
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
