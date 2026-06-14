use bevy::{
    asset::RenderAssetUsages,
    gizmos::config::{DefaultGizmoConfigGroup, GizmoConfigStore},
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::{GameMap, HexLayout, MapDims, SQRT_3, load_annotations_from_str, load_map_data};
use omdurman_hexmap::{adjusted_origin, hex_world_pos, hit_to_hex};
use omdurman_types::{
    HexCoord, HexsideKind, HexsideRef, IntoEnumIterator, NileFlow, Orientation, Terrain,
};

use omdurman_net::{GameEvent, NetMsg};

use crate::{
    ActiveEditMap, AnnotationsDirty, EditorMode, GameStateResource, LoadedAnnotations,
    MapStateStore, PendingEdits, PendingMapLoad, SidebarClip,
    browser::SpriteAnnotationsResource,
    browser::SpriteBrowserRoot,
    camera::RtsCamera,
    picker::{PlacedUnit, UnitPicker},
    render::{HexOverlay, HexRingAssets, MapPlane, MapTextureCache, apply_map_data_to_plane},
    units::UnitsPlane,
    ui_plugin::StatusPane,
    util::{ctrl_held, raycast_ground, shift_held},
};

pub const ANNOTATIONS_SAVE_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/annotations.ron");

/// Debounce interval for annotation file writes: wait this many seconds of
/// inactivity after the last edit before persisting to disk.
pub(crate) const ANNOTATIONS_FLUSH_SECS: f32 = 0.5;

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
    /// Toggle a road connection between two adjacent hexes.
    RoadToggle(HexsideRef),
    /// Set the crossroad flag on all selected hexes.
    Crossroad(bool),
}

#[derive(Resource, Default)]
pub struct HexEditor {
    /// The set of selected hexes. Edits apply to all of them.
    pub selection: std::collections::HashSet<HexCoord>,
    /// The "anchor" hex whose state populates the side panel. Always a member of
    /// `selection` while a selection exists; `None` when empty.
    pub anchor: Option<HexCoord>,
    /// The name text-edit buffer for the anchor. Editor-owned: it can differ
    /// from the committed name while the user is typing, so unlike the
    /// terrain/flow/road display it is *not* derived from the map on demand.
    pub name: String,
    pub show_terrain_overlay: bool,
    /// The hexside segment currently selected in the Hexside editor mode, if
    /// any. Clicking a segment selects it; the side panel then assigns a type.
    pub selected_hexside: Option<HexsideRef>,
    /// An edit queued by a key/dropdown action, consumed by the apply system and
    /// applied to every hex in `selection` (§multi-select).
    pub pending_apply: Option<PendingApply>,
}

/// The anchor hex's display state, resolved from the live map on demand rather
/// than mirrored into [`HexEditor`]. Keeping this derived (not cached) means the
/// panel always shows the map's truth with no re-sync step after an edit.
pub struct AnchorView {
    pub terrain: Terrain,
    /// Nile current; `None` = no current. Only meaningful when `terrain.is_nile()`.
    pub nile_flow: Option<NileFlow>,
    /// Whether roads converge at this hex's centre (`true`) or stop at the edge.
    pub is_crossroad: bool,
    /// Whether the hex is the **Not playable** pseudo-type — board furniture
    /// (logo, turn track, …) excluded from the map via [`GameEvent::ExcludeHex`].
    pub not_playable: bool,
}

impl HexEditor {
    /// Resolve the anchor's display state from `game_map`. `None` when there is
    /// no anchor or it is off-grid. An excluded (in-grid) anchor reads back as
    /// the Not-playable pseudo-type.
    pub fn anchor_view(&self, game_map: &GameMap) -> Option<AnchorView> {
        let coord = self.anchor?;
        if let Some(d) = game_map.hexes.get(&coord) {
            Some(AnchorView {
                terrain: d.terrain,
                nile_flow: d.nile_flow,
                is_crossroad: d.is_crossroad,
                not_playable: false,
            })
        } else if game_map.excluded.contains(&coord) {
            Some(AnchorView {
                terrain: Terrain::default(),
                nile_flow: None,
                is_crossroad: false,
                not_playable: true,
            })
        } else {
            None
        }
    }
}

/// Apply a [`GameEvent::MapEdit`] to the playable hex at `coord`: `edit` takes
/// the hex's current data and returns the desired
/// `(terrain, name, nile_flow, is_crossroad)`; if anything changed, broadcast
/// the edit, mutate the live hex, and mark the annotations dirty. No-op for
/// excluded / off-map hexes. The terrain-side edits (set terrain, rotate flow,
/// rename, toggle crossroad) all funnel through here so the `MapEdit`
/// construction lives in one place.
fn apply_map_edit(
    coord: HexCoord,
    map: omdurman_types::MapKind,
    game_map: &mut GameMap,
    pending: &mut PendingEdits,
    dirty: &mut crate::AnnotationsDirty,
    edit: impl FnOnce(&omdurman_types::HexData) -> (Terrain, Option<String>, Option<NileFlow>, bool),
) {
    let Some(d) = game_map.hexes.get(&coord) else {
        return;
    };
    let (terrain, name, nile_flow, is_crossroad) = edit(d);
    if d.terrain == terrain
        && d.name == name
        && d.nile_flow == nile_flow
        && d.is_crossroad == is_crossroad
    {
        return;
    }
    pending
        .outgoing_broadcast
        .push(NetMsg::Game(GameEvent::MapEdit {
            map,
            q: coord.q,
            r: coord.r,
            terrain: terrain.to_u8(),
            name: name.clone().unwrap_or_default(),
            nile_flow,
            is_crossroad,
        }));
    if let Some(d) = game_map.hexes.get_mut(&coord) {
        d.terrain = terrain;
        d.name = name;
        d.nile_flow = nile_flow;
        d.is_crossroad = is_crossroad;
    }
    dirty.mark();
}

/// Toggle a road connection between two adjacent hexes: set or clear the road
/// edge, broadcast a [`GameEvent::RoadEdit`], and mark annotations dirty.
fn apply_road_edit(
    edge: HexsideRef,
    present: bool,
    map: omdurman_types::MapKind,
    game_map: &mut GameMap,
    pending: &mut PendingEdits,
    dirty: &mut crate::AnnotationsDirty,
) {
    if present {
        game_map.roads.insert(edge);
    } else {
        game_map.roads.remove(&edge);
    }
    pending
        .outgoing_broadcast
        .push(NetMsg::Game(GameEvent::RoadEdit { map, edge, present }));
    dirty.mark();
}

/// Build the right-hand editor side panel with the shared dark frame and
/// monospace font used by both the terrain and hexside editors, run `content`,
/// and return the panel's screen rect (for `SidebarClip`). The two editors
/// differ only in id and width, so the chrome lives here once.
fn editor_side_panel(
    ctx: &egui::Context,
    id: &str,
    default_width: f32,
    width_range: std::ops::RangeInclusive<f32>,
    content: impl FnOnce(&mut egui::Ui),
) -> egui::Rect {
    egui::SidePanel::right(id.to_string())
        .resizable(true)
        .default_width(default_width)
        .width_range(width_range)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(45))
                .inner_margin(egui::Margin::symmetric(12, 12)),
        )
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
            content(ui);
        })
        .response
        .rect
}

/// Whether `coord` is part of the grid (a playable hex or an excluded one).
fn in_grid(coord: HexCoord, game_map: &GameMap) -> bool {
    game_map.hexes.contains_key(&coord) || game_map.excluded.contains(&coord)
}

/// Make `coord` the anchor and load its name into the panel's edit buffer. The
/// terrain/flow/road/not-playable display is derived on demand via
/// [`HexEditor::anchor_view`], so only the editor-owned `name` is cached here.
/// Assumes `coord` is in-grid.
pub(crate) fn load_anchor(coord: HexCoord, editor: &mut HexEditor, game_map: &GameMap) {
    editor.anchor = Some(coord);
    editor.name = game_map
        .hexes
        .get(&coord)
        .and_then(|d| d.name.clone())
        .unwrap_or_default();
}

/// Letter/number keys set terrain; Delete/Backspace marks Not playable;
/// Ctrl+PgUp/PgDown rotate the Nile current; Ctrl+arrows extend the selection
/// from the anchor. All edits apply to **every** selected hex (§multi-select).
/// Plain arrows / PgUp/PgDown drive the camera (see `camera_control`).
pub fn editor_terrain_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut editor: ResMut<HexEditor>,
    game_map: Res<GameMap>,
) {
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_keyboard_input()
    {
        return;
    }
    let Some(anchor) = editor.anchor else {
        return;
    };
    let ctrl = ctrl_held(&keys);

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
        editor.pending_apply = Some(PendingApply::NotPlayable);
        return;
    }

    let t = match () {
        _ if keys.just_pressed(KeyCode::KeyC) => Some(Terrain::Clear),
        _ if keys.just_pressed(KeyCode::KeyR) => Some(Terrain::Rough),
        _ if keys.just_pressed(KeyCode::KeyT) => Some(Terrain::Trees),
        _ if keys.just_pressed(KeyCode::KeyS) => Some(Terrain::Swamp),
        _ if keys.just_pressed(KeyCode::KeyN) => Some(Terrain::Nile),
        _ if keys.just_pressed(KeyCode::KeyI) => Some(Terrain::Hilltop),
        _ if keys.just_pressed(KeyCode::KeyH) => Some(Terrain::Huts),
        _ if keys.just_pressed(KeyCode::KeyB) => Some(Terrain::Building),
        _ => None,
    };
    if let Some(t) = t {
        editor.pending_apply = Some(PendingApply::Terrain(t));
    }
}

pub fn handle_hex_editor_click(
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
    if !buttons.just_pressed(MouseButton::Left) {
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

    let ctrl = ctrl_held(&keys);
    let shift = shift_held(&keys);

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
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut editor: ResMut<HexEditor>,
) {
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
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    editor: Res<HexEditor>,
    mut game_map: ResMut<GameMap>,
    mut pending: ResMut<PendingEdits>,
    mut dirty: ResMut<crate::AnnotationsDirty>,
    active: Res<crate::ActiveEditMap>,
) {
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

/// Despawn all excluded hex rings (used when leaving Editor mode).
fn hide_excluded_hex_rings(
    mut commands: Commands,
    existing: Query<Entity, With<ExcludedHexRing>>,
) {
    for e in &existing {
        commands.entity(e).despawn();
    }
}

/// Draw excluded hexes with a red outline while in Editor mode, so the holes in
/// the map (board furniture) are visible during terrain editing.
#[derive(Component)]
pub(crate) struct ExcludedHexRing;

pub fn draw_excluded_hex_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    existing: Query<Entity, With<ExcludedHexRing>>,
) {
    for e in &existing {
        commands.entity(e).despawn();
    }
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let size = overlay.params.hex_size;
    for coord in &game_map.excluded {
        let pos = hex_world_pos(*coord, origin, &overlay.params);
        commands.spawn((
            ExcludedHexRing,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.red.clone()),
            Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
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
    mode: Res<State<EditorMode>>,
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
    // Outside hexside mode there is no cursor-driven hover bar, so the quads
    // only move when the mode, calibration, map hexsides, or selection change.
    // Skip the rebuild otherwise (they're world-space, camera moves don't matter).
    // In hexside mode the hover preview follows the cursor, so always run.
    if !mode.is_hexside()
        && !mode.is_changed()
        && !overlay.is_changed()
        && !game_map.is_changed()
        && !editor.is_changed()
    {
        return;
    }

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

// ── Road connections as ground-plane mesh quads ────────────────────────────
//
// Each road edge is a flat brown bar connecting two hex centres, drawn as a
// pooled mesh quad (same technique as hexside bars) so it has visible width.

/// A pooled road bar (a flat quad on the ground plane).
#[derive(Component)]
pub struct RoadQuad;

/// Reusable pool of road-bar entities + the shared unit-square mesh.
#[derive(Resource, Default)]
pub struct RoadQuads {
    mesh: Handle<Mesh>,
    pool: Vec<Entity>,
}

/// One-time setup: the shared unit quad mesh for road bars.
pub fn setup_road_quads(mut quads: ResMut<RoadQuads>, mut meshes: ResMut<Assets<Mesh>>) {
    quads.mesh = meshes.add(Rectangle::new(1.0, 1.0));
}

/// Road bar width as a fraction of hex size — chunky enough to be obvious.
const ROAD_WIDTH_FRAC: f32 = 0.10;

/// How far a road extends from a non-crossroad hex's center toward the edge,
/// as a fraction of the centre-to-edge distance. 0.75 means the road stops
/// 25 % in from the edge, making it visibly enter the tile without reaching
/// the centre (which is what the crossroad flag does).
const ROAD_END_FRAC: f32 = 0.75;

/// Intersection of the ray from `center` toward `target` with the boundary of
/// a regular hexagon of circumradius `size` and the given `orientation`. The
/// returned point lies on the hex edge between two vertices.
fn hex_edge_intersection(center: Vec3, size: f32, orientation: Orientation, target: Vec3) -> Vec3 {
    let dx = target.x - center.x;
    let dz = target.z - center.z;
    let len = (dx * dx + dz * dz).sqrt();
    if len < 0.0001 {
        return center;
    }
    let (ndx, ndz) = (dx / len, dz / len);

    let apothem = size * SQRT_3 / 2.0;

    // Outward edge normals for the hexagon (in the xz plane, y = 0).
    let normals: [(f32, f32); 6] = match orientation {
        Orientation::Pointy => [
            (1.0, 0.0),
            (0.5, SQRT_3 * 0.5),
            (-0.5, SQRT_3 * 0.5),
            (-1.0, 0.0),
            (-0.5, -SQRT_3 * 0.5),
            (0.5, -SQRT_3 * 0.5),
        ],
        Orientation::Flat => [
            (SQRT_3 * 0.5, -0.5),
            (SQRT_3 * 0.5, 0.5),
            (0.0, 1.0),
            (-SQRT_3 * 0.5, 0.5),
            (-SQRT_3 * 0.5, -0.5),
            (0.0, -1.0),
        ],
    };

    let mut min_t = f32::MAX;
    for &(nx, nz) in &normals {
        let dot = ndx * nx + ndz * nz;
        if dot > 0.0 {
            let t = apothem / dot;
            if t < min_t {
                min_t = t;
            }
        }
    }

    Vec3::new(center.x + ndx * min_t, center.y, center.z + ndz * min_t)
}

/// Place a brown road bar for every road edge in the game map. Pool grows on
/// demand; unused bars are parked invisible.
#[allow(clippy::too_many_arguments)]
pub fn update_road_quads(
    mode: Res<State<EditorMode>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut quads: ResMut<RoadQuads>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q: Query<
        (
            &mut Transform,
            &mut Visibility,
            &MeshMaterial3d<StandardMaterial>,
        ),
        With<RoadQuad>,
    >,
) {
    if !mode.is_editor() {
        for &entity in &quads.pool {
            if let Ok((_, mut visibility, _)) = q.get_mut(entity) {
                *visibility = Visibility::Hidden;
            }
        }
        return;
    }

    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let base_w = overlay.params.hex_size * ROAD_WIDTH_FRAC;
    let color = Color::srgb(0.5, 0.3, 0.1);

    let edges: Vec<(Vec3, Vec3)> = game_map
        .roads
        .iter()
        .map(|edge| {
            let a_pos = hex_world_pos(edge.a, origin, &overlay.params);
            let b_pos = hex_world_pos(edge.b, origin, &overlay.params);

            let a_is_crossroad = game_map
                .hexes
                .get(&edge.a)
                .map(|d| d.is_crossroad)
                .unwrap_or(false);
            let b_is_crossroad = game_map
                .hexes
                .get(&edge.b)
                .map(|d| d.is_crossroad)
                .unwrap_or(false);

            let p0 = if a_is_crossroad {
                a_pos
            } else {
                let edge = hex_edge_intersection(
                    a_pos,
                    overlay.params.hex_size,
                    overlay.params.orientation,
                    b_pos,
                );
                a_pos + (edge - a_pos) * ROAD_END_FRAC
            };
            let p1 = if b_is_crossroad {
                b_pos
            } else {
                let edge = hex_edge_intersection(
                    b_pos,
                    overlay.params.hex_size,
                    overlay.params.orientation,
                    a_pos,
                );
                b_pos + (edge - b_pos) * ROAD_END_FRAC
            };

            (Vec3::new(p0.x, 1.3, p0.z), Vec3::new(p1.x, 1.3, p1.z))
        })
        .collect();

    while quads.pool.len() < edges.len() {
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let id = commands
            .spawn((
                RoadQuad,
                Mesh3d(quads.mesh.clone()),
                MeshMaterial3d(material),
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        quads.pool.push(id);
    }

    for (i, &entity) in quads.pool.iter().enumerate() {
        let Ok((mut transform, mut visibility, mat_handle)) = q.get_mut(entity) else {
            continue;
        };
        if let Some(&(p0, p1)) = edges.get(i) {
            if let Some(material) = materials.get_mut(&mat_handle.0) {
                place_hexside_quad(&mut transform, material, p0, p1, base_w, 1.3, color);
            }
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

// ── Nile flow arrows ─────────────────────────────────────────────────────────
//
// Orange arrows on Nile hexes showing the current direction, drawn as triangle
// meshes (not gizmo lines) so they render with proper depth and anti-aliasing.

/// A pooled Nile-current arrow mesh entity.
#[derive(Component)]
pub struct NileArrow;

/// Reusable pool of Nile-arrow entities + the shared arrow mesh/material.
#[derive(Resource, Default)]
pub struct NileArrows {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    pool: Vec<Entity>,
}

/// Build a flat arrow mesh pointing +Z, centred at origin, length ≈ 1.
fn make_arrow_mesh() -> Mesh {
    let sw = 0.04;
    let hw = 0.14;
    let positions = vec![
        Vec3::new(-sw, 0.0, -0.4),
        Vec3::new(sw, 0.0, -0.4),
        Vec3::new(sw, 0.0, 0.2),
        Vec3::new(-sw, 0.0, 0.2),
        Vec3::new(-hw, 0.0, 0.2),
        Vec3::new(hw, 0.0, 0.2),
        Vec3::new(0.0, 0.0, 0.6),
    ];
    let normals = vec![Vec3::Y; 7];
    let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2, 4, 6, 5]);
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(indices)
}

/// One-time setup: the shared arrow mesh/material.
pub fn setup_nile_arrows(
    mut arrows: ResMut<NileArrows>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    arrows.mesh = meshes.add(make_arrow_mesh());
    arrows.material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.55, 0.0),
        unlit: true,
        ..default()
    });
}

/// Arrow length as a fraction of hex size.
const NILE_ARROW_LEN_FRAC: f32 = 0.7;

/// Place one orange flow-direction arrow per Nile hex that has a current
/// annotation; shown only in the terrain editor. Pool grows on demand; unused
/// arrows are parked invisible.
pub fn update_nile_arrows(
    mode: Res<State<EditorMode>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut arrows: ResMut<NileArrows>,
    mut commands: Commands,
    mut q: Query<(&mut Transform, &mut Visibility), With<NileArrow>>,
) {
    let active = mode.is_editor();

    let mut placements: Vec<(Vec3, Vec3)> = Vec::new();
    if active {
        let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
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
            placements.push((Vec3::new(center.x, 1.5, center.z), dir));
        }
    }

    while arrows.pool.len() < placements.len() {
        let id = commands
            .spawn((
                NileArrow,
                Mesh3d(arrows.mesh.clone()),
                MeshMaterial3d(arrows.material.clone()),
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        arrows.pool.push(id);
    }

    let scale = overlay.params.hex_size * NILE_ARROW_LEN_FRAC;
    for (i, &entity) in arrows.pool.iter().enumerate() {
        let Ok((mut transform, mut visibility)) = q.get_mut(entity) else {
            continue;
        };
        if let Some(&(center, dir)) = placements.get(i) {
            *transform = Transform::from_translation(center)
                .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                .with_scale(Vec3::splat(scale));
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn hexside_color(kind: HexsideKind) -> Color {
    match kind {
        HexsideKind::Wall => Color::srgb(0.75, 0.75, 0.75),
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

/// Despawn all editor highlight rings (used when leaving Editor mode).
fn hide_editor_highlight_rings(
    mut commands: Commands,
    existing: Query<Entity, With<EditorHighlightRing>>,
) {
    for e in &existing {
        commands.entity(e).despawn();
    }
}

/// Draw selected hexes with green outlines in Editor mode. The anchor hex
/// (whose state the panel shows) gets a brighter shade.
#[derive(Component)]
pub(crate) struct EditorHighlightRing;

pub fn draw_editor_highlight_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    editor: Res<HexEditor>,
    existing: Query<Entity, With<EditorHighlightRing>>,
) {
    for e in &existing {
        commands.entity(e).despawn();
    }
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let size = overlay.params.hex_size;
    for &coord in &editor.selection {
        let pos = hex_world_pos(coord, origin, &overlay.params);
        let is_anchor = editor.anchor == Some(coord);
        let s = if is_anchor { size } else { size * 0.92 };
        commands.spawn((
            EditorHighlightRing,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(if is_anchor {
                assets.light_green.clone()
            } else {
                assets.green.clone()
            }),
            Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(s)),
            Visibility::Visible,
        ));
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

/// Paint each hex's terrain/name label (and, when enabled, its terrain-colour
/// fill) into the egui background layer, clipped to the canvas left of the
/// sidebar. One pass over the hexes: project the centre once, cull off-screen
/// hexes, then draw the optional colour fill and the label.
fn draw_hex_labels(
    ctx: &egui::Context,
    camera: &Camera,
    cam_transform: &GlobalTransform,
    vp_size: Vec2,
    game_map: &GameMap,
    layout: &HexLayout,
    overlay: &HexOverlay,
    show_terrain_overlay: bool,
    sidebar: Option<egui::Rect>,
) {
    // Clip to the canvas area, excluding the sidebar from the previous frame so
    // background-order painters don't bleed over the panel.
    let canvas_rect = {
        let screen = ctx.viewport_rect();
        match sidebar {
            Some(sidebar) => {
                egui::Rect::from_min_max(screen.min, egui::pos2(sidebar.left(), screen.max.y))
            }
            None => screen,
        }
    };
    // Paint into the shared background layer so shapes append in call-order with
    // panels that share LayerId::background() (CentralPanel, SidePanel). The
    // SidePanel adds its shapes later, so they paint on top — which is what we want.
    let mut painter = ctx.layer_painter(egui::LayerId::background());
    painter.set_clip_rect(canvas_rect);
    // Tile terrain/name labels at 0.75× the former 10pt.
    let font_size = 7.5;
    let char_w = font_size * 0.6;
    let line_h = font_size * 1.4;
    let padding = 3.0;
    let origin = adjusted_origin(layout, overlay.params.offset_x, overlay.params.offset_y);
    let size = overlay.params.hex_size;

    for (coord, data) in &game_map.hexes {
        let center = hex_world_pos(*coord, origin, &overlay.params);
        // Cull off-screen hexes once, on the centre projection, before doing any
        // per-hex shape work (overlay fill or label).
        let Ok(screen) =
            camera.world_to_viewport(cam_transform, Vec3::new(center.x, 0.1, center.z))
        else {
            continue;
        };
        if screen.x < 0.0 || screen.x > vp_size.x || screen.y < 0.0 || screen.y > vp_size.y {
            continue;
        }
        // Terrain colour fill first, so the label paints on top of it.
        if show_terrain_overlay {
            let corners = crate::render::hex_corners(Vec3::new(center.x, 1.5, center.z), size);
            let mut verts = Vec::with_capacity(6);
            for world in corners {
                if let Ok(s) = camera.world_to_viewport(cam_transform, world) {
                    verts.push(egui::pos2(s.x, s.y));
                }
            }
            if verts.len() == 6 {
                let [r, g, b, a] = data.terrain.overlay_color();
                let color = egui::Color32::from_rgba_unmultiplied(
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                    (a * 255.0) as u8,
                );
                painter.add(egui::Shape::convex_polygon(
                    verts,
                    color,
                    egui::Stroke::NONE,
                ));
            }
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
        painter.rect_filled(rect, 3.0, egui::Color32::from_black_alpha(160));
        painter.text(
            egui::pos2(screen.x, screen.y),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::monospace(font_size),
            egui::Color32::WHITE,
        );
    }
}

/// The terrain-editor egui pass: paint the hex labels/overlay and the side
/// panel. The panel reads the anchor's terrain/flow/road straight from the map
/// (via [`HexEditor::anchor_view`]) and only *queues* edits into
/// `pending_apply` — `apply_terrain_edits` consumes them next.
pub fn editor_ui(
    mut contexts: EguiContexts,
    mode: Res<State<EditorMode>>,
    mut editor: ResMut<HexEditor>,
    game_map: Res<GameMap>,
    mut pending: ResMut<PendingEdits>,
    mut clip: ResMut<SidebarClip>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
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

    draw_hex_labels(
        ctx,
        camera,
        cam_transform,
        vp_size,
        &game_map,
        &layout,
        &overlay,
        editor.show_terrain_overlay,
        clip.right_sidebar,
    );

    // Anchor's terrain/flow/road/not-playable, resolved from the map each frame.
    let view = editor.anchor_view(&game_map);

    // ---- sidebar panel (Order::Middle, on top of background) ----
    let rect = editor_side_panel(ctx, "editor_panel", 200.0, 150.0..=500.0, |ui| {
        if let (Some(coord), Some(view)) = (editor.anchor, &view) {
            let n = editor.selection.len();
            if n > 1 {
                ui.label(format!(
                    "{n} hexes selected (anchor q {}  r {})",
                    coord.q, coord.r
                ));
            } else {
                ui.label(format!("hex  q {}  r {}", coord.q, coord.r));
            }

            // Road toggle: if exactly 2 hexes are selected and they are
            // adjacent, show a button to connect/disconnect them.
            if n == 2 {
                let mut iter = editor.selection.iter();
                let a = iter.next().unwrap();
                let b = iter.next().unwrap();
                if a.neighbors().contains(b) {
                    let edge = HexsideRef::new(*a, *b);
                    let has_road = game_map.roads.contains(&edge);
                    let label = if has_road {
                        "remove road"
                    } else {
                        "connect with road"
                    };
                    ui.add_space(4.0);
                    if ui.button(label).clicked() {
                        editor.pending_apply = Some(PendingApply::RoadToggle(edge));
                    }
                }
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("name");
                // Editing the name commits to the anchor on focus loss / Enter.
                let resp = ui
                    .add(egui::TextEdit::singleline(&mut editor.name).desired_width(f32::INFINITY));
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
                let selected_text = if view.not_playable {
                    "Not playable".to_string()
                } else {
                    format!("{}", view.terrain)
                };
                egui::ComboBox::from_id_salt("terrain")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for t in Terrain::iter() {
                            if ui
                                .selectable_label(
                                    !view.not_playable && view.terrain == t,
                                    format!("{}", t),
                                )
                                .clicked()
                            {
                                editor.pending_apply = Some(PendingApply::Terrain(t));
                            }
                        }
                        ui.separator();
                        if ui
                            .selectable_label(view.not_playable, "Not playable")
                            .clicked()
                        {
                            editor.pending_apply = Some(PendingApply::NotPlayable);
                        }
                    });
            });

            // Nile current annotation: a single arrow per hex, pointing
            // downstream, rotated by the +/- buttons (rulebook §5.11,
            // §5.24). Rotating applies to every selected Nile hex.
            if !view.not_playable && view.terrain.is_nile() {
                ui.add_space(6.0);
                ui.label(egui::RichText::new("Nile current").color(egui::Color32::from_gray(200)));
                ui.add_space(2.0);
                // Direction labels in HexCoord::neighbors order.
                const DIR_LABELS: [&str; 6] = ["E", "SE", "SW", "W", "NW", "NE"];
                let dir = view.nile_flow.unwrap_or_default().dir;
                ui.horizontal(|ui| {
                    if ui.button("⟲ -").clicked() {
                        editor.pending_apply = Some(PendingApply::RotateFlow(-1));
                    }
                    ui.label(
                        egui::RichText::new(format!("↦ {} ({})", DIR_LABELS[dir as usize], dir))
                            .color(egui::Color32::from_rgb(255, 160, 60)),
                    );
                    if ui.button("+ ⟳").clicked() {
                        editor.pending_apply = Some(PendingApply::RotateFlow(1));
                    }
                });
            }

            // Crossroad flag: when checked, roads on this hex converge at the
            // centre; when unchecked they stop at the hex edge.
            if !view.not_playable {
                ui.add_space(4.0);
                let mut cr = view.is_crossroad;
                if ui.checkbox(&mut cr, "crossroad").changed() {
                    editor.pending_apply = Some(PendingApply::Crossroad(cr));
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
    clip.right_sidebar = Some(rect);
}

/// Apply the queued terrain-editor action (terrain / Not-playable / flow / name
/// / road) to the whole selection, broadcasting the edit events. Runs in the
/// `Update` schedule after the egui pass has queued the action. Edits are
/// action-triggered, not a continuous diff, so a multi-hex selection never
/// re-writes every frame or fights per-hex differences.
pub fn apply_terrain_edits(
    mut editor: ResMut<HexEditor>,
    mut game_map: ResMut<GameMap>,
    mut pending: ResMut<PendingEdits>,
    mut dirty: ResMut<crate::AnnotationsDirty>,
    active: Res<crate::ActiveEditMap>,
) {
    let Some(action) = editor.pending_apply.take() else {
        return;
    };
    // RoadToggle operates on a specific edge, not per-hex.
    if let PendingApply::RoadToggle(edge) = &action {
        let present = !game_map.roads.contains(edge);
        apply_road_edit(
            *edge,
            present,
            active.0,
            &mut game_map,
            &mut pending,
            &mut dirty,
        );
        return;
    }

    // Name applies to the anchor only; every other action applies to the whole
    // selection.
    let targets: Vec<HexCoord> = match &action {
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
            PendingApply::Terrain(_) if is_excluded => {
                // Restore an excluded hex first; it re-enters the map as Desert
                // and the terrain can be set on a subsequent action.
                pending
                    .outgoing_broadcast
                    .push(NetMsg::Game(GameEvent::ExcludeHex {
                        map: active.0,
                        q: coord.q,
                        r: coord.r,
                        excluded: false,
                    }));
                dirty.mark();
            }
            // The three terrain-side edits all funnel through `apply_map_edit`,
            // which builds the `MapEdit`, diffs, mutates, and marks dirty.
            PendingApply::Terrain(t) => {
                apply_map_edit(
                    coord,
                    active.0,
                    &mut game_map,
                    &mut pending,
                    &mut dirty,
                    |d| {
                        // Preserve the hex's own name/crossroad; set/clear flow per Nile-ness.
                        let flow = t.is_nile().then(|| d.nile_flow.unwrap_or_default());
                        (*t, d.name.clone(), flow, d.is_crossroad)
                    },
                );
            }
            PendingApply::RotateFlow(delta) => {
                apply_map_edit(
                    coord,
                    active.0,
                    &mut game_map,
                    &mut pending,
                    &mut dirty,
                    |d| {
                        let flow = d
                            .terrain
                            .is_nile()
                            .then(|| d.nile_flow.unwrap_or_default().rotated(*delta))
                            // Non-Nile hexes: leave flow as-is (diff makes it a no-op).
                            .or(d.nile_flow);
                        (d.terrain, d.name.clone(), flow, d.is_crossroad)
                    },
                );
            }
            PendingApply::Name => {
                let new_name = (!editor.name.is_empty()).then(|| editor.name.clone());
                apply_map_edit(
                    coord,
                    active.0,
                    &mut game_map,
                    &mut pending,
                    &mut dirty,
                    |d| (d.terrain, new_name.clone(), d.nile_flow, d.is_crossroad),
                );
            }
            PendingApply::Crossroad(on) => {
                apply_map_edit(
                    coord,
                    active.0,
                    &mut game_map,
                    &mut pending,
                    &mut dirty,
                    |d| (d.terrain, d.name.clone(), d.nile_flow, *on),
                );
            }
            PendingApply::RoadToggle(_) => {
                // handled before the selection loop — unreachable
            }
        }
    }
}

/// Side panel for the Hexside editor mode: shows the selected segment's current
/// feature and a button per type (plus "none") to assign it. Applying a type
/// updates the live map and broadcasts a [`GameEvent::HexsideEdit`].
#[allow(clippy::too_many_arguments)]
pub fn hexside_editor_ui(
    mut contexts: EguiContexts,
    mode: Res<State<EditorMode>>,
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

    let rect = editor_side_panel(ctx, "hexside_editor_panel", 180.0, 140.0..=320.0, |ui| {
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
    clip.right_sidebar = Some(rect);

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

use bevy::app::Plugin;

pub(crate) fn init_gizmo_config(mut store: ResMut<GizmoConfigStore>) {
    let (config, _) = store.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -0.5;
    config.line.width = 2.0;
}

pub(crate) fn load_annotations(
    mut commands: Commands,
    mut game_map: ResMut<GameMap>,
    mut overlay: ResMut<HexOverlay>,
    mut loaded: ResMut<LoadedAnnotations>,
) {
    let ron_str = include_str!("../assets/annotations.ron");
    let kind = omdurman_types::MapKind::FallOfKhartoum;
    let annotations = load_annotations_from_str(ron_str, kind, &mut game_map);
    overlay.params = game_map.overlay.clone();
    commands.insert_resource(SpriteAnnotationsResource(
        annotations.sprites.clone(),
    ));
    loaded.0 = annotations;
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_map_selection(
    mut pending: ResMut<PendingMapLoad>,
    loaded: Res<LoadedAnnotations>,
    mut active: ResMut<ActiveEditMap>,
    mut game_map: ResMut<GameMap>,
    mut overlay: ResMut<HexOverlay>,
    mut dims: ResMut<MapDims>,
    mut layout: ResMut<HexLayout>,
    annotations: Option<ResMut<SpriteAnnotationsResource>>,
    mut commands: Commands,
    plane: Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>), With<MapPlane>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cache: ResMut<MapTextureCache>,
    asset_server: Res<AssetServer>,
) {
    let Some(kind) = pending.0.take() else {
        return;
    };
    let map = loaded.0.map(kind);

    load_map_data(map, &mut game_map);
    overlay.params = game_map.overlay.clone();
    *dims = MapDims {
        img_w: map.img_w,
        img_h: map.img_h,
    };
    *layout = HexLayout::calibrated(
        map.overlay.orientation,
        Vec2::new(map.calib.p1_px.0, map.calib.p1_px.1),
        HexCoord::new(map.calib.p1_hex.0, map.calib.p1_hex.1),
        Vec2::new(map.calib.p2_px.0, map.calib.p2_px.1),
        HexCoord::new(map.calib.p2_hex.0, map.calib.p2_hex.1),
        map.img_w,
        map.img_h,
    );
    if annotations.is_none() {
        commands.insert_resource(SpriteAnnotationsResource(loaded.0.sprites.clone()));
    }
    apply_map_data_to_plane(
        &plane,
        &mut meshes,
        &mut materials,
        &mut cache,
        &asset_server,
        &map.image,
        map.img_w,
        map.img_h,
    );
    active.0 = kind;
    info!(%kind, img_w = map.img_w, img_h = map.img_h, "loaded board");
}

pub(crate) fn sync_edit_board_to_mode(
    mode: Res<State<EditorMode>>,
    active: Res<ActiveEditMap>,
    mut pending: ResMut<PendingMapLoad>,
) {
    if let Some(board) = mode.edit_board()
        && board != active.0
        && pending.0.is_none()
    {
        pending.0 = Some(board);
    }
}

pub(crate) fn sync_map_state(
    mode: Res<State<EditorMode>>,
    mut store: ResMut<MapStateStore>,
    mut game_state: ResMut<GameStateResource>,
    mut picker: ResMut<UnitPicker>,
    placed_units: Query<Entity, With<crate::picker::PlacedUnit>>,
    mut commands: Commands,
) {
    let target = match **mode {
        EditorMode::FallOfKhartoumMap => Some(omdurman_types::MapKind::FallOfKhartoum),
        EditorMode::CampaignMap => Some(omdurman_types::MapKind::Campaign),
        _ => None,
    };
    let Some(target_map) = target else {
        return;
    };
    // Skip entirely if the picker hasn't been populated by
    // spawn_picker_assets yet — the Startup system hasn't run.
    if picker.all.is_empty() {
        // Clear any stale stashes created from an empty picker at startup.
        store.fall_of_khartoum_picker = None;
        store.campaign_picker = None;
        return;
    }
    for entity in &placed_units {
        commands.entity(entity).despawn();
    }
    store.stash_current_as(MapStateStore::other(target_map), &game_state, &picker);
    store.restore(target_map, &mut game_state, &mut picker);
    picker.reset_available();
}

pub(crate) fn sync_mode_visibilities(
    mode: Res<State<EditorMode>>,
    mut vis_set: ParamSet<(
        Query<&mut Visibility, With<UnitsPlane>>,
        Query<&mut Visibility, With<MapPlane>>,
        Query<&mut Visibility, With<SpriteBrowserRoot>>,
        Query<&mut Visibility, With<StatusPane>>,
        Query<&mut Visibility, With<PlacedUnit>>,
    )>,
) {
    if let Ok(mut vis) = vis_set.p0().single_mut() {
        *vis = if mode.is_unit_sheet() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = vis_set.p1().single_mut() {
        *vis = if mode.shows_map_plane() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = vis_set.p2().single_mut() {
        *vis = if mode.is_units() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = vis_set.p3().single_mut() {
        *vis = if mode.is_map_mode() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut vis in vis_set.p4().iter_mut() {
        *vis = if mode.is_map_mode() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Registers all editor-domain resources, startup systems, and per-frame
/// systems (terrain editing, hexside editing, map annotations, mode
/// visibilities). Systems that depend on [`EditorMode`] states are assigned
/// to the corresponding [`crate::EditorSet`] / [`crate::HexsideSet`] sets.
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        use crate::EditorSet;
        use crate::HexsideSet;

        app
            // ── Resources ──────────────────────────────────────────────
            .insert_resource(HexEditor::default())
            .insert_resource(HexsideQuads::default())
            .insert_resource(RoadQuads::default())
            .insert_resource(NileArrows::default())
            .insert_resource(crate::SidebarClip::default())
            .insert_resource(crate::AnnotationsDirty::default())
            // ── Startup ────────────────────────────────────────────────
            .add_systems(
                Startup,
                (
                    setup_hexside_quads,
                    setup_road_quads,
                    setup_nile_arrows,
                    load_annotations,
                    init_gizmo_config,
                ),
            )
            // ── Update: terrain editor (EditorSet) ─────────────────────
            .add_systems(
                Update,
                (
                    editor_terrain_keys.in_set(EditorSet),
                    apply_terrain_edits
                        .in_set(EditorSet)
                        .after(editor_terrain_keys),
                    handle_hex_editor_click.in_set(EditorSet),
                    handle_hexside_select.in_set(HexsideSet),
                    handle_hexside_keys.in_set(HexsideSet),
                    draw_editor_highlight_mesh.in_set(EditorSet),
                    update_road_quads.after(apply_map_selection),
                    update_hexside_quads,
                    draw_excluded_hex_mesh.in_set(EditorSet),
                    update_nile_arrows,
                    apply_map_selection,
                    flush_annotations_to_disk,
                ),
            )
            .add_systems(
                OnEnter(EditorMode::FallOfKhartoumMap),
                (
                    sync_edit_board_to_mode,
                    sync_map_state,
                    sync_mode_visibilities,
                    hide_excluded_hex_rings,
                    hide_editor_highlight_rings,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(EditorMode::CampaignMap),
                (
                    sync_edit_board_to_mode,
                    sync_map_state,
                    sync_mode_visibilities,
                    hide_excluded_hex_rings,
                    hide_editor_highlight_rings,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(EditorMode::Overlay),
                (
                    sync_edit_board_to_mode,
                    sync_mode_visibilities,
                    hide_excluded_hex_rings,
                    hide_editor_highlight_rings,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(EditorMode::Editor),
                (
                    sync_edit_board_to_mode,
                    sync_mode_visibilities,
                    hide_excluded_hex_rings,
                    hide_editor_highlight_rings,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(EditorMode::Hexside),
                (
                    sync_edit_board_to_mode,
                    sync_mode_visibilities,
                    hide_excluded_hex_rings,
                    hide_editor_highlight_rings,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(EditorMode::CampaignOverlay),
                (
                    sync_edit_board_to_mode,
                    sync_mode_visibilities,
                    hide_excluded_hex_rings,
                    hide_editor_highlight_rings,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(EditorMode::CampaignEditor),
                (
                    sync_edit_board_to_mode,
                    sync_mode_visibilities,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(EditorMode::CampaignHexside),
                (
                    sync_edit_board_to_mode,
                    sync_mode_visibilities,
                    hide_excluded_hex_rings,
                    hide_editor_highlight_rings,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(EditorMode::UnitSheet),
                (sync_mode_visibilities, hide_excluded_hex_rings, hide_editor_highlight_rings).chain(),
            )
            .add_systems(
                OnEnter(EditorMode::Units),
                (sync_mode_visibilities, hide_excluded_hex_rings, hide_editor_highlight_rings).chain(),
            )
            .add_systems(
                OnEnter(EditorMode::Dice),
                (sync_mode_visibilities, hide_excluded_hex_rings, hide_editor_highlight_rings).chain(),
            )
            .add_systems(
                OnEnter(EditorMode::EventViewer),
                (sync_mode_visibilities, hide_excluded_hex_rings, hide_editor_highlight_rings).chain(),
            )
            // ── Egui UI panels ─────────────────────────────────────────
            .add_systems(EguiPrimaryContextPass, (editor_ui, hexside_editor_ui));
    }
}

/// Persist `assets/annotations.ron` once the dirty flag has been idle for
/// `ANNOTATIONS_FLUSH_SECS`. Coalesces many per-keystroke / per-drag changes
/// into one disk write at the end of an edit burst. On WASM this is a no-op
/// because the underlying `save_annotations_to_file` already skips writes.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn flush_annotations_to_disk(
    time: Res<Time>,
    mut dirty: ResMut<AnnotationsDirty>,
    game_map: Res<GameMap>,
    annotations: Option<Res<SpriteAnnotationsResource>>,
    loaded: Res<LoadedAnnotations>,
    active: Res<ActiveEditMap>,
) {
    if !dirty.dirty {
        return;
    }
    dirty.idle += time.delta_secs();
    if dirty.idle < ANNOTATIONS_FLUSH_SECS {
        return;
    }
    if let Some(ann) = annotations {
        omdurman_hexmap::save_annotations_to_file(
            &game_map,
            &ann.0,
            &loaded.0,
            active.0,
            ANNOTATIONS_SAVE_PATH,
        );
    }
    dirty.dirty = false;
    dirty.idle = 0.0;
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn flush_annotations_to_disk(_dirty: ResMut<AnnotationsDirty>) {}
