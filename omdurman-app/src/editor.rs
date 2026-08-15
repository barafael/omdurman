mod hexside;
mod nile;
mod rings;
mod road;

use bevy::{
    gizmos::config::{DefaultGizmoConfigGroup, GizmoConfigStore},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::{
    GameMap, HexLayout, MapDims, SQRT_3, load_map_data,
};
use omdurman_hexmap::{hex_world_pos, hit_to_hex};
use omdurman_types::{
    GroundKind, HexCoord, HexsideKind, HexsideRef, NamedArea, Road, SetupLetter,
    Terrain,
};
use strum::IntoEnumIterator;

use omdurman_net::{GameEvent, NetMsg};

use crate::{
    AppMode, EditorTab, GameStateResource, PendingEdits, SidebarClip,
    browser::SpriteAnnotationsResource,
    browser::SpriteBrowserRoot,
    camera::RtsCamera,
    picker::PlacedUnit,
    render::{HexOverlay, MapPlane, MapTextureCache, apply_map_data_to_plane},
    ui_plugin::StatusPane,
    units::UnitsPlane,
    util::{ctrl_held, raycast_ground, shift_held},
};


/// The active editor tool, resolved from the top-level [`AppMode`] and the
/// [`EditorTab`]. A convenience `SystemParam` so the editor systems can keep
/// asking `mode.is_editor()` / `is_hexside()` / `is_timing()` after the split
/// of the old `EditorMode` enum into two axes. Each predicate is `true` only
/// when [`AppMode::Editor`] is active *and* the matching tab is selected.
#[derive(bevy::ecs::system::SystemParam)]
pub struct EditorToolState<'w> {
    mode: Res<'w, State<AppMode>>,
    tab: Res<'w, State<EditorTab>>,
}

impl EditorToolState<'_> {
    fn is(&self, tab: EditorTab) -> bool {
        **self.mode == AppMode::Editor && **self.tab == tab
    }
    pub fn is_editor(&self) -> bool {
        self.is(EditorTab::Terrain)
    }
    pub fn is_overlay(&self) -> bool {
        self.is(EditorTab::Overlay)
    }
    pub fn is_hexside(&self) -> bool {
        self.is(EditorTab::Hexside)
    }
    pub fn is_timing(&self) -> bool {
        self.is(EditorTab::Timing)
    }
    pub fn is_unit_sheet(&self) -> bool {
        self.is(EditorTab::UnitSheet)
    }
    pub fn is_event_viewer(&self) -> bool {
        self.is(EditorTab::EventViewer)
    }
    /// True if either the top-level mode or the tab changed this frame.
    pub fn is_changed(&self) -> bool {
        self.mode.is_changed() || self.tab.is_changed()
    }
}

/// Bundle of the read-only hex-layout + overlay + game-map resources that
/// several editor systems consume, so their signatures stay under Bevy's
/// system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct EditorBoardView<'w> {
    pub layout: Res<'w, HexLayout>,
    pub overlay: Res<'w, HexOverlay>,
    pub game_map: Res<'w, GameMap>,
}

/// Bundle of the egui contexts + window + camera queries used by editor click
/// handlers and gizmo builders, so their signatures stay under Bevy's
/// system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct WindowCameraCtx<'w, 's> {
    pub contexts: EguiContexts<'w, 's>,
    pub windows: Query<'w, 's, &'static Window>,
    pub cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<RtsCamera>>,
}

/// Bundle of the read-only hex-layout + overlay + game-map resources together
/// with the window + camera queries used by the hexside-quad rebuild, so
/// [`hexside::update_hexside_quads`] stays under Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct HexSpatial<'w, 's> {
    pub layout: Res<'w, HexLayout>,
    pub overlay: Res<'w, HexOverlay>,
    pub game_map: Res<'w, GameMap>,
    pub windows: Query<'w, 's, &'static Window>,
    pub cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<RtsCamera>>,
}

/// Bundle of `PendingEdits` -- the outgoing-edit queue -- so several editor
/// systems stay under Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct AnnotationsWriteState<'w> {
    pub pending: ResMut<'w, PendingEdits>,
}

/// Camera/viewport projection inputs to [`draw_hex_labels`]: the camera, its
/// global transform, and the logical viewport size. Plain struct (the consumer
/// is not a system).
struct CameraView<'a> {
    camera: &'a Camera,
    cam_transform: &'a GlobalTransform,
    vp_size: Vec2,
}

/// The five visibility-toggling queries bundled into a tuple so the
/// `ParamSet<...>` type in [`sync_mode_visibilities`] doesn't trip clippy's
/// `very_complex_type` lint.
type VisibilityQueries<'w, 's> = (
    Query<'w, 's, &'static mut Visibility, With<UnitsPlane>>,
    Query<'w, 's, &'static mut Visibility, With<MapPlane>>,
    Query<'w, 's, &'static mut Visibility, With<SpriteBrowserRoot>>,
    Query<'w, 's, &'static mut Visibility, With<StatusPane>>,
    Query<'w, 's, &'static mut Visibility, With<PlacedUnit>>,
);

// -- Editor resources (moved from main.rs) ----------------------------------

/// The full two-board annotations file, kept in memory so map switches, edits,
/// and disk saves can address either board without re-reading from disk
/// (§dual-map). Seeded from compiled codegen data at startup; the active
/// board's section is rewritten from the live [`GameMap`] on save.
#[derive(Resource)]
pub struct LoadedAnnotations {
    pub fall_of_khartoum: omdurman_types::MapData,
    pub campaign: omdurman_types::MapData,
}

impl LoadedAnnotations {
    pub fn map(&self, kind: omdurman_types::MapKind) -> &omdurman_types::MapData {
        match kind {
            omdurman_types::MapKind::FallOfKhartoum => &self.fall_of_khartoum,
            omdurman_types::MapKind::Campaign => &self.campaign,
        }
    }

    pub fn map_mut(&mut self, kind: omdurman_types::MapKind) -> &mut omdurman_types::MapData {
        match kind {
            omdurman_types::MapKind::FallOfKhartoum => &mut self.fall_of_khartoum,
            omdurman_types::MapKind::Campaign => &mut self.campaign,
        }
    }
}

impl Default for LoadedAnnotations {
    fn default() -> Self {
        Self {
            fall_of_khartoum: omdurman_types::MapData::empty_fall_of_khartoum(),
            campaign: omdurman_types::MapData::empty_campaign(),
        }
    }
}

/// Which board the editor/overlay tools currently act on (§dual-map). Local to
/// each peer -- calibration is a dev tool, not replicated state. Switching it
/// reloads the corresponding board into the live `GameMap`/overlay/layout.
#[derive(Resource, Default)]
pub struct ActiveEditMap(pub omdurman_types::MapKind);

/// When `true`, clicks on the campaign map (in the Timing editor tab) toggle
/// the [`HexData::is_scattergram`] flag of the clicked hex. Lets the designer
/// mark which 7 hexes belong to the printed Howitzer Fire Scattergram diagram
/// (rulebook §6.64). Default off; toggled from the timing editor panel.
#[derive(Resource, Default)]
pub struct ScattergramPaint(pub bool);

/// A deferred request to (re)load a board into the live `GameMap`/`HexOverlay`/
/// `MapDims`/`HexLayout` and re-texture the map plane. Set by the `StartGame`
/// handler and the editor's map toggle; consumed by `apply_map_selection`,
/// which has the asset/material access those handlers lack (§dual-map).
#[derive(Resource, Default)]
pub struct PendingMapLoad(pub Option<omdurman_types::MapKind>);

/// Which board the editor tools currently act on, chosen by a scenario picker
/// (Fall of Khartoum / Historical / Campaign) in the editor's tab bar. Historical
/// and Campaign share the Campaign board (§9.1/§9.2), so the picker selects a
/// scenario and the board follows via [`crate::map_kind_for_scenario`]. Local
/// editor state, not replicated.
#[derive(Resource)]
pub struct EditorBoard(pub omdurman_types::Scenario);

impl Default for EditorBoard {
    fn default() -> Self {
        Self(omdurman_types::Scenario::FallOfKhartoum)
    }
}

impl EditorBoard {
    pub fn map_kind(&self) -> omdurman_types::MapKind {
        crate::map_kind_for_scenario(self.0)
    }
}

/// A queued edit to apply to every selected hex on the next frame. Multi-select
/// edits are *action-triggered* (set a terrain, press Delete, rotate the
/// current) rather than the old continuous diff against a single hex, so
/// applying never fights per-hex differences across the set.
#[derive(Clone, Debug)]
pub enum PendingApply {
    /// Set all selected hexes to this terrain (making them playable).
    Terrain(Terrain),
    /// Exclude all selected hexes from the map.
    Playable,
    /// Rotate the Nile current on all selected Nile hexes by +/-1 sixth.
    RotateFlow(i8),
    /// Set the anchor hex's name to the panel's text.
    Name,
    /// Toggle a road connection between two adjacent hexes.
    RoadToggle(HexsideRef),
    /// Set or clear the historical-scenario setup letter on the anchor hex
    /// (rulebook §9.212).
    SetupLetter(Option<SetupLetter>),
    /// Set or clear the named-area membership of the anchor hex (rulebook
    /// §9.113).
    NamedArea(Option<NamedArea>),
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
    /// Whether the hex is playable (`true`) or excluded board furniture
    /// (logo, turn track, ...) excluded from the map via [`GameEvent::ExcludeHex`].
    pub playable: bool,
    pub setup_letter: Option<SetupLetter>,
    pub named_area: Option<NamedArea>,
}

impl HexEditor {
    /// Resolve the anchor's display state from `game_map`. `None` when there is
    /// no anchor or it is off-grid. An excluded (in-grid) anchor reads back as
    /// non-playable.
    pub fn anchor_view(&self, game_map: &GameMap) -> Option<AnchorView> {
        let coord = self.anchor?;
        if let Some(d) = game_map.hexes.get(&coord) {
            Some(AnchorView {
                terrain: d.terrain,
                playable: true,
                setup_letter: d.setup_letter,
                named_area: d.named_area,
            })
        } else if game_map.excluded.contains(&coord) {
            Some(AnchorView {
                terrain: Terrain::default(),
                playable: false,
                setup_letter: None,
                named_area: None,
            })
        } else {
            None
        }
    }
}

/// Apply a [`GameEvent::MapEdit`] to the playable hex at `coord`: `edit` takes
/// the hex's current data and returns the desired
/// `(terrain, name)`; if anything changed, broadcast
/// the edit and mutate the live hex. No-op for
/// excluded / off-map hexes. The terrain-side edits (set terrain, rotate flow,
/// rename, toggle crossroad) all funnel through here so the `MapEdit`
/// construction lives in one place.
fn apply_map_edit(
    coord: HexCoord,
    map: omdurman_types::MapKind,
    game_map: &mut GameMap,
    pending: &mut PendingEdits,
    edit: impl FnOnce(&omdurman_types::HexData) -> (Terrain, Option<String>),
) {
    let Some(d) = game_map.hexes.get(&coord) else {
        return;
    };
    let (terrain, name) = edit(d);
    if d.terrain == terrain && d.name == name {
        return;
    }
    pending
        .outgoing_broadcast
        .push(NetMsg::Game(GameEvent::MapEdit {
            map,
            coord,
            terrain,
            name: name.clone().unwrap_or_default(),
        }));
    if let Some(d) = game_map.hexes.get_mut(&coord) {
        d.terrain = terrain;
        d.name = name;
    }
}

/// Toggle a road connection between two adjacent hexes: set or clear the road
/// edge, broadcast a [`GameEvent::RoadEdit`].
fn apply_road_edit(
    edge: HexsideRef,
    present: bool,
    map: omdurman_types::MapKind,
    game_map: &mut GameMap,
    pending: &mut PendingEdits,
) {
    if present {
        let a_nile = game_map
            .hexes
            .get(&edge.a)
            .is_some_and(|h| h.terrain.is_nile());
        let b_nile = game_map
            .hexes
            .get(&edge.b)
            .is_some_and(|h| h.terrain.is_nile());
        if a_nile || b_nile {
            return;
        }
        game_map.roads.insert(edge);
    } else {
        game_map.roads.remove(&edge);
    }
    pending
        .outgoing_broadcast
        .push(NetMsg::Game(GameEvent::RoadEdit { map, edge, present }));
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
    let mut __ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("editor_side_panel"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    let response = egui::Panel::right(id.to_string())
        .resizable(true)
        .show_separator_line(false)
        .default_size(default_width)
        .size_range(width_range)
        .frame(
            egui::Frame::default()
                .fill(crate::ui::panel_bg())
                .inner_margin(egui::Margin::symmetric(12, 12)),
        )
        .show(&mut __ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
            content(ui);
        });
    crate::ui_plugin::register_panel_rect(ctx, response.response.rect);
    response.response.rect
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
        && ctx.egui_wants_keyboard_input()
    {
        return;
    }
    let Some(anchor) = editor.anchor else {
        return;
    };
    let ctrl = ctrl_held(&keys);

    // Ctrl+arrows extend the selection to a neighbour of the anchor (and move
    // the anchor there). Left/Right step -/+q, Up/Down step -/+r. Off-grid ignored.
    if ctrl {
        let target = if keys.just_pressed(KeyCode::ArrowLeft) {
            Some(HexCoord::new(anchor.q - 1, anchor.r))
        } else if keys.just_pressed(KeyCode::ArrowRight) {
            Some(HexCoord::new(anchor.q + 1, anchor.r))
        } else if keys.just_pressed(KeyCode::ArrowUp) {
            Some(HexCoord::new(anchor.q, anchor.r - 1))
        } else if keys.just_pressed(KeyCode::ArrowDown) {
            Some(HexCoord::new(anchor.q, anchor.r + 1))
        } else {
            None
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

    // Delete/Backspace excludes every selected hex from the map.
    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        editor.pending_apply = Some(PendingApply::Playable);
        return;
    }

    let t = if keys.just_pressed(KeyCode::KeyC) {
        Some(Terrain::ground(GroundKind::Clear))
    } else if keys.just_pressed(KeyCode::KeyR) {
        Some(Terrain::ground(GroundKind::Rough))
    } else if keys.just_pressed(KeyCode::KeyT) {
        Some(Terrain::ground(GroundKind::Trees))
    } else if keys.just_pressed(KeyCode::KeyS) {
        Some(Terrain::ground(GroundKind::Swamp))
    } else if keys.just_pressed(KeyCode::KeyN) {
        Some(Terrain::Nile {
            direction: omdurman_types::HexDirection::default(),
        })
    } else if keys.just_pressed(KeyCode::KeyI) {
        Some(Terrain::ground(GroundKind::Hilltop))
    } else if keys.just_pressed(KeyCode::KeyH) {
        Some(Terrain::ground(GroundKind::Huts))
    } else if keys.just_pressed(KeyCode::KeyB) {
        Some(Terrain::ground(GroundKind::Building))
    } else {
        None
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
    view: EditorBoardView,
    mut editor: ResMut<HexEditor>,
) {
    let EditorBoardView {
        layout,
        overlay,
        game_map,
    } = view;
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut()
        && crate::ui_plugin::egui_wants_pointer_input(ctx)
    {
        return;
    }
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = layout.adjusted_origin(&overlay.params);
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

/// When `ScattergramPaint` is enabled (timing editor tab), left-clicking a
/// campaign-map hex toggles its `is_scattergram` flag. Lets the designer
/// annotate the seven printed Howitzer Fire Scattergram reference hexes
/// (rulebook §6.64) directly on the map.
pub fn handle_scattergram_click(
    buttons: Res<ButtonInput<MouseButton>>,
    win_cam: WindowCameraCtx,
    view: EditorBoardView,
    paint: Res<ScattergramPaint>,
    active: Res<ActiveEditMap>,
    writes: AnnotationsWriteState,
) {
    let WindowCameraCtx {
        mut contexts,
        windows,
        cameras,
    } = win_cam;
    let EditorBoardView {
        layout,
        overlay,
        game_map,
    } = view;
    let AnnotationsWriteState { mut pending } = writes;
    if !paint.0 {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut()
        && crate::ui_plugin::egui_wants_pointer_input(ctx)
    {
        return;
    }
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = layout.adjusted_origin(&overlay.params);
    let coord = hit_to_hex(hit, origin, &overlay.params);
    let Some(d) = game_map.hexes.get(&coord) else {
        return;
    };
    let next = !d.is_scattergram;
    pending
        .outgoing_broadcast
        .push(NetMsg::Game(GameEvent::ScattergramEdit {
            map: active.0,
            coord,
            is_scattergram: next,
        }));
}

/// The edge of `coord` nearest the world point `hit` -- i.e. the neighbour
/// whose shared border the click is closest to. Returns the `[coord, neighbour]`
/// pair as a canonical [`HexsideRef`].
///
/// All six edges are candidates, including those toward off-map or excluded
/// neighbours: a wall/khor can sit on the board's outer border, so the editor
/// must be able to select any of a hex's sides.
pub(super) fn nearest_edge(
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
/// hex centre toward `neighbour` -- higher means the click is more toward that
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
/// selection. No edit is broadcast here -- that happens when a type is chosen in
/// [`hexside_editor_ui`].
pub fn handle_hexside_select(
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    view: EditorBoardView,
    mut editor: ResMut<HexEditor>,
) {
    let EditorBoardView {
        layout,
        overlay,
        game_map,
    } = view;
    let select = buttons.just_pressed(MouseButton::Left);
    let clear = buttons.just_pressed(MouseButton::Right);
    if !select && !clear {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut()
        && crate::ui_plugin::egui_wants_pointer_input(ctx)
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
    let origin = layout.adjusted_origin(&overlay.params);
    let coord = hit_to_hex(hit, origin, &overlay.params);
    if !game_map.hexes.contains_key(&coord) {
        return;
    }
    if let Some(edge) = nearest_edge(coord, hit, origin, &overlay.params) {
        editor.selected_hexside = Some(edge);
    }
}

/// The hotkey letter shown for a hexside kind in the panel. Kept in sync with
/// [`hexside_hotkey`] (the single source of truth for the key->action mapping).
fn hexside_hotkey_label(kind: HexsideKind) -> &'static str {
    match kind {
        HexsideKind::Wall => "W",
        HexsideKind::Gate => "G",
        HexsideKind::Breach => "B",
        HexsideKind::Khor => "K",
        HexsideKind::Crest => "C",
        HexsideKind::ZaribaThornHedge => "T",
        HexsideKind::ZaribaTrench => "R",
        HexsideKind::ZaribaTrenchEndA => "E",
        HexsideKind::ZaribaTrenchEndB => "F",
        HexsideKind::KhorShambat => "S",
    }
}

/// The hexside action a hotkey maps to in the Hexside editor:
/// `Some(Some(kind))` sets that feature, `Some(None)` clears it, `None` is no
/// binding. Mnemonic where possible; these only fire in hexside mode, so they
/// don't clash with the terrain-editor terrain keys.
fn hexside_hotkey(keys: &ButtonInput<KeyCode>) -> Option<Option<HexsideKind>> {
    let k = |code| keys.just_pressed(code);
    if k(KeyCode::KeyW) {
        Some(Some(HexsideKind::Wall))
    } else if k(KeyCode::KeyG) {
        Some(Some(HexsideKind::Gate))
    } else if k(KeyCode::KeyB) {
        Some(Some(HexsideKind::Breach))
    } else if k(KeyCode::KeyK) {
        Some(Some(HexsideKind::Khor))
    } else if k(KeyCode::KeyC) {
        Some(Some(HexsideKind::Crest))
    } else if k(KeyCode::KeyT) {
        Some(Some(HexsideKind::ZaribaThornHedge))
    } else if k(KeyCode::KeyR) {
        Some(Some(HexsideKind::ZaribaTrench))
    } else if k(KeyCode::KeyE) {
        Some(Some(HexsideKind::ZaribaTrenchEndA))
    } else if k(KeyCode::KeyF) {
        Some(Some(HexsideKind::ZaribaTrenchEndB))
    } else if k(KeyCode::KeyS) {
        Some(Some(HexsideKind::KhorShambat))
    } else if k(KeyCode::KeyN) || k(KeyCode::Delete) || k(KeyCode::Backspace) {
        // Clear the feature.
        Some(None)
    } else {
        None
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
}

/// In the Hexside editor mode, number/letter keys assign a feature type to the
/// currently selected segment (see [`hexside_hotkey`]). No-op when no segment is
/// selected or a text field has keyboard focus.
pub fn handle_hexside_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    editor: Res<HexEditor>,
    mut game_map: ResMut<GameMap>,
    mut pending: ResMut<PendingEdits>,
    active: Res<ActiveEditMap>,
) {
    let Some(edge) = editor.selected_hexside else {
        return;
    };
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.egui_wants_keyboard_input()
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
        );
    }
}

/// Paint each hex's terrain/name label (and, when enabled, its terrain-colour
/// fill) into the egui background layer, clipped to the canvas left of the
/// sidebar. One pass over the hexes: project the centre once, cull off-screen
/// hexes, then draw the optional colour fill and the label.
fn draw_hex_labels(
    ctx: &egui::Context,
    view: CameraView,
    game_map: &GameMap,
    layout: &HexLayout,
    overlay: &HexOverlay,
    show_terrain_overlay: bool,
    sidebar: Option<egui::Rect>,
) {
    let CameraView {
        camera,
        cam_transform,
        vp_size,
    } = view;
    // Clip to the canvas area, excluding the sidebar from the previous frame so
    // background-order painters don't bleed over the panel. Also drop everything
    // above `available_rect().top()`: that excludes the docked top bar, which
    // shares `LayerId::background()` -- without this the hex labels paint over
    // the tab bar (they are added to the layer after it, so they'd win).
    let canvas_rect = {
        let screen = ctx.viewport_rect();
        let top = ctx.content_rect().top();
        let right = match sidebar {
            Some(sidebar) => sidebar.left(),
            None => screen.max.x,
        };
        egui::Rect::from_min_max(
            egui::pos2(screen.min.x, top),
            egui::pos2(right, screen.max.y),
        )
    };
    // Paint into the shared background layer so shapes append in call-order with
    // panels that share LayerId::background() (CentralPanel, SidePanel). The
    // SidePanel adds its shapes later, so they paint on top -- which is what we want.
    let mut painter = ctx.layer_painter(egui::LayerId::background());
    painter.set_clip_rect(canvas_rect);
    // Tile terrain/name labels at 0.75x the former 10pt.
    let font_size = 7.5;
    let char_w = font_size * 0.6;
    let line_h = font_size * 1.4;
    let padding = 3.0;
    let origin = layout.adjusted_origin(&overlay.params);
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
                let [r, g, b, a] = crate::render::terrain_overlay_color(data.terrain);
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
/// `pending_apply` -- `apply_terrain_edits` consumes them next.
pub fn editor_ui(
    mut contexts: EguiContexts,
    mode: EditorToolState,
    mut editor: ResMut<HexEditor>,
    view: EditorBoardView,
    mut pending: ResMut<PendingEdits>,
    mut clip: ResMut<SidebarClip>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
) {
    let EditorBoardView {
        layout,
        overlay,
        game_map,
    } = view;
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
        CameraView {
            camera,
            cam_transform,
            vp_size,
        },
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
                // `n == 2` guarantees exactly two members; destructure instead
                // of `iter.next().unwrap()` so a future invariant slip is a
                // caught panic at the boundary rather than a silent mis-pairing.
                let [a, b] = editor
                    .selection
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
                    .as_slice()
                    .try_into()
                    .expect("selection has exactly 2 members");
                if a.neighbors().contains(&b) {
                    let edge = HexsideRef::new(a, b);
                    let a_nile = game_map
                        .hexes
                        .get(&a)
                        .is_some_and(|h| h.terrain.is_nile());
                    let b_nile = game_map
                        .hexes
                        .get(&b)
                        .is_some_and(|h| h.terrain.is_nile());
                    if !(a_nile || b_nile) {
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
                let selected_text = if !view.playable {
                    "Exclude".to_string()
                } else {
                    format!("{}", view.terrain)
                };
                egui::ComboBox::from_id_salt("terrain")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        // Show ground types (Nile is special: no road, separate entry).
                        for kind in GroundKind::iter() {
                            let t = Terrain::ground(kind);
                            if ui
                                .selectable_label(
                                    view.playable && view.terrain.ground_kind() == Some(kind),
                                    format!("{}", t),
                                )
                                .clicked()
                            {
                                editor.pending_apply = Some(PendingApply::Terrain(t));
                            }
                        }
                        // Nile entry
                        let nile = Terrain::Nile { direction: omdurman_types::HexDirection::default() };
                        if ui
                            .selectable_label(
                                view.playable && view.terrain.is_nile(),
                                format!("{}", nile),
                            )
                            .clicked()
                        {
                            editor.pending_apply = Some(PendingApply::Terrain(nile));
                        }
                        ui.separator();
                        if ui
                            .selectable_label(!view.playable, "Exclude")
                            .clicked()
                        {
                            editor.pending_apply = Some(PendingApply::Playable);
                        }
                    });
            });

            // Nile current annotation: a single arrow per hex, pointing
            // downstream, rotated by the +/- buttons (rulebook §5.11,
            // §5.24). Rotating applies to every selected Nile hex.
            if view.playable && view.terrain.is_nile() {
                ui.add_space(6.0);
                ui.label(egui::RichText::new("Nile current").color(egui::Color32::from_gray(200)));
                ui.add_space(2.0);
                // Direction labels in HexCoord::neighbors order.
                const DIR_LABELS: [&str; 6] = ["E", "SE", "SW", "W", "NW", "NE"];
                let dir = view.terrain.nile_direction().unwrap_or_default();
                ui.horizontal(|ui| {
                    if ui.button("[cw] -").clicked() {
                        editor.pending_apply = Some(PendingApply::RotateFlow(-1));
                    }
                    ui.label(
                        egui::RichText::new(format!("|-> {} ({})", DIR_LABELS[dir as usize], dir))
                            .color(egui::Color32::from_rgb(255, 160, 60)),
                    );
                    if ui.button("+ [ccw]").clicked() {
                        editor.pending_apply = Some(PendingApply::RotateFlow(1));
                    }
                });
            }

            // Crossroad flag: when checked, roads on this hex converge at the
            // centre; when unchecked they stop at the hex edge.
            if view.playable {
                ui.add_space(4.0);
                let mut cr = view.terrain.is_crossroad();
                if ui.checkbox(&mut cr, "crossroad").changed() {
                    let new_road = if cr { Road::Crossroad } else { Road::Road };
                    editor.pending_apply = Some(PendingApply::Terrain(view.terrain.with_road(new_road)));
                }
            }

            // Setup letter (rulebook §9.212): the historical-scenario anchor
            // hex where a Dervish leader is placed. Anchor-only, like Name.
            if view.playable {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label("letter");
                    let current = view.setup_letter;
                    if ui
                        .add(egui::Button::selectable(current.is_none(), "none"))
                        .clicked()
                    {
                        editor.pending_apply = Some(PendingApply::SetupLetter(None));
                    }
                    for letter in [
                        SetupLetter::A,
                        SetupLetter::D,
                        SetupLetter::Y,
                        SetupLetter::K,
                        SetupLetter::S,
                        SetupLetter::O,
                    ] {
                        if ui
                            .add(egui::Button::selectable(
                                current == Some(letter),
                                letter.to_string(),
                            ))
                            .clicked()
                        {
                            editor.pending_apply = Some(PendingApply::SetupLetter(Some(letter)));
                        }
                    }
                });
            }

            // Named area (rulebook §9.113): marks the anchor as part of a
            // multi-hex rules-significant region (e.g. the Anglo-Egyptian
            // entrance area on the west bank of the campaign map).
            if view.playable {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label("area");
                    let selected_text = view
                        .named_area
                        .map(|a| a.to_string())
                        .unwrap_or_else(|| "none".to_string());
                    egui::ComboBox::from_id_salt("named_area")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(view.named_area.is_none(), "none").clicked() {
                                editor.pending_apply = Some(PendingApply::NamedArea(None));
                            }
                            if ui
                                .selectable_label(
                                    view.named_area == Some(NamedArea::AngloEgyptianEntrance),
                                    NamedArea::AngloEgyptianEntrance.to_string(),
                                )
                                .clicked()
                            {
                                editor.pending_apply =
                                    Some(PendingApply::NamedArea(Some(NamedArea::AngloEgyptianEntrance)));
                            }
                        });
                });
            }
        } else {
            ui.label("click a hex to select * Ctrl+click adds * Ctrl+Shift+click removes");
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
        // logos, turn track, ...). Hexside/wall editing lives in its own mode.
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
    active: Res<ActiveEditMap>,
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
        );
        return;
    }

    // Name and other anchor-only fields (setup letter, named area) apply to
    // the anchor only; every other action applies to the whole selection.
    let targets: Vec<HexCoord> = match &action {
        PendingApply::Name | PendingApply::SetupLetter(_) | PendingApply::NamedArea(_) => {
            editor.anchor.into_iter().collect()
        }
        _ => editor.selection.iter().copied().collect(),
    };

    for coord in targets {
        let is_excluded = game_map.excluded.contains(&coord);
        match &action {
            PendingApply::Playable => {
                // Exclude playable hexes; already-excluded ones are a no-op.
                if !is_excluded && game_map.hexes.contains_key(&coord) {
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Game(GameEvent::ExcludeHex {
                            map: active.0,
                            coord,
                            excluded: true,
                        }));
                }
            }
            PendingApply::Terrain(_) if is_excluded => {
                // Restore an excluded hex first; it re-enters the map as Desert
                // and the terrain can be set on a subsequent action.
                pending
                    .outgoing_broadcast
                    .push(NetMsg::Game(GameEvent::ExcludeHex {
                        map: active.0,
                        coord,
                        excluded: false,
                    }));
            }
            // The three terrain-side edits all funnel through `apply_map_edit`,
            // which builds the `MapEdit`, diffs, mutates, and marks dirty.
            PendingApply::Terrain(t) => {
                apply_map_edit(
                    coord,
                    active.0,
                    &mut game_map,
                    &mut pending,
                    |d| {
                        // When switching to a non-Nile type, strip the Nile direction.
                        // When switching to Nile, keep default direction if the old hex
                        // had one, otherwise default.
                        let new_terrain = *t;
                        (new_terrain, d.name.clone())
                    },
                );
            }
            PendingApply::RotateFlow(delta) => {
                apply_map_edit(
                    coord,
                    active.0,
                    &mut game_map,
                    &mut pending,
                    |d| {
                        let new_terrain = d.terrain.with_rotated_flow(*delta);
                        (new_terrain, d.name.clone())
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
                    |d| (d.terrain, new_name.clone()),
                );
            }
            PendingApply::SetupLetter(letter) => {
                if game_map.hexes.contains_key(&coord) {
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Game(GameEvent::SetupLetterEdit {
                            map: active.0,
                            coord,
                            letter: *letter,
                        }));
                }
            }
            PendingApply::NamedArea(area) => {
                if game_map.hexes.contains_key(&coord) {
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Game(GameEvent::NamedAreaEdit {
                            map: active.0,
                            coord,
                            area: *area,
                        }));
                }
            }
            PendingApply::RoadToggle(_) => {
                // handled before the selection loop -- unreachable
            }
        }
    }
}

/// Side panel for the Hexside editor mode: shows the selected segment's current
/// feature and a button per type (plus "none") to assign it. Applying a type
/// updates the live map and broadcasts a [`GameEvent::HexsideEdit`].
pub fn hexside_editor_ui(
    mut contexts: EguiContexts,
    mode: EditorToolState,
    editor: Res<HexEditor>,
    mut game_map: ResMut<GameMap>,
    writes: AnnotationsWriteState,
    mut clip: ResMut<SidebarClip>,
    active: Res<ActiveEditMap>,
) {
    let AnnotationsWriteState { mut pending } = writes;
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
            egui::RichText::new("L-click a segment to select * R-click to deselect")
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
            "({}, {}) -- ({}, {})",
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
    let kind = omdurman_types::MapKind::FallOfKhartoum;
    loaded.campaign = omdurman_rules::board_data::campaign_map_data();
    loaded.fall_of_khartoum = omdurman_rules::board_data::fall_of_khartoum_map_data();
    load_map_data(loaded.map(kind), &mut game_map);
    overlay.params = game_map.overlay.clone();
    commands.insert_resource(SpriteAnnotationsResource::default());
}

/// Bundle of resources mutated when (re)loading a board into the live
/// `GameMap` / overlay / layout / texture (§dual-map). Keeps
/// [`apply_map_selection`] under the system-parameter limit without hiding
/// framework types (`Commands`, `Query`, asset stores) that other systems also
/// depend on.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct MapLoadContext<'w> {
    pub pending: ResMut<'w, PendingMapLoad>,
    pub loaded: Res<'w, LoadedAnnotations>,
    pub active: ResMut<'w, ActiveEditMap>,
    pub game_state: ResMut<'w, GameStateResource>,
    pub game_map: ResMut<'w, GameMap>,
    pub overlay: ResMut<'w, HexOverlay>,
    pub dims: ResMut<'w, MapDims>,
    pub layout: ResMut<'w, HexLayout>,
    pub annotations: Option<ResMut<'w, SpriteAnnotationsResource>>,
    pub cache: ResMut<'w, MapTextureCache>,
}

pub(crate) fn apply_map_selection(
    mut ctx: MapLoadContext,
    mut commands: Commands,
    plane: Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>), With<MapPlane>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let Some(kind) = ctx.pending.0.take() else {
        return;
    };
    debug!(?kind, "applying PendingMapLoad");
    let map = ctx.loaded.map(kind);

    // Attach the engine's view of this board so map-dependent rules (ZOC across
    // hexsides §5.44, gunboat upstream/downstream §5.24, terrain movement cost
    // §5.11, Friendlies bank §9.14) can be enforced deterministically. Carried
    // inside the serialized GameState, so replay/late-join reproduce it.
    ctx.game_state.0.board = omdurman_rules::board::BoardInfo::from_map_data(map);

    load_map_data(map, &mut ctx.game_map);
    ctx.overlay.params = ctx.game_map.overlay.clone();
    *ctx.dims = MapDims {
        img_w: map.img_w,
        img_h: map.img_h,
    };
    *ctx.layout = HexLayout::calibrated(
        map.overlay.orientation,
        omdurman_hexmap::CalibrationAnchor {
            px: Vec2::new(map.calib.p1_px.0, map.calib.p1_px.1),
            hex: HexCoord::new(map.calib.p1_hex.0, map.calib.p1_hex.1),
        },
        omdurman_hexmap::CalibrationAnchor {
            px: Vec2::new(map.calib.p2_px.0, map.calib.p2_px.1),
            hex: HexCoord::new(map.calib.p2_hex.0, map.calib.p2_hex.1),
        },
        Vec2::new(map.img_w, map.img_h),
    );
    if ctx.annotations.is_none() {
        commands.insert_resource(SpriteAnnotationsResource::default());
    }
    apply_map_data_to_plane(
        &plane,
        &mut crate::render::PlaneTextureStores {
            meshes: &mut meshes,
            materials: &mut materials,
            cache: &mut ctx.cache,
            asset_server: &asset_server,
        },
        &map.image,
        map.img_w,
        map.img_h,
    );
    ctx.active.0 = kind;
    info!(%kind, img_w = map.img_w, img_h = map.img_h, "loaded board");
}

/// Reconcile the live board with the active view every frame (§dual-map).
/// In the editor the board follows [`EditorBoard`] (a board-specific tab);
/// board-agnostic editor tabs (sprites/etc.) keep whatever is loaded. In a
/// play view (Game) the board follows the scenario's map. Sets
/// [`PendingMapLoad`] when the desired board differs from what's loaded.
pub(crate) fn sync_edit_board_to_mode(
    mode: Res<State<crate::AppMode>>,
    tab: Res<State<crate::EditorTab>>,
    editor_board: Res<EditorBoard>,
    game_state: Res<GameStateResource>,
    active: Res<ActiveEditMap>,
    mut pending: ResMut<PendingMapLoad>,
) {
    let desired = match **mode {
        crate::AppMode::Editor if tab.is_board_specific() => Some(editor_board.map_kind()),
        crate::AppMode::Editor => None,
        crate::AppMode::Game => {
            Some(crate::map_kind_for_scenario(game_state.0.scenario))
        }
        crate::AppMode::Menu | crate::AppMode::Lobby => None,
    };
    if let Some(board) = desired
        && board != active.0
        && pending.0.is_none()
    {
        pending.0 = Some(board);
    }
}

pub(crate) fn sync_mode_visibilities(
    mode: Res<State<crate::AppMode>>,
    tab: Res<State<crate::EditorTab>>,
    mut vis_set: ParamSet<VisibilityQueries<'_, '_>>,
) {
    let is_menu = **mode == crate::AppMode::Menu;
    let is_editor = **mode == crate::AppMode::Editor;
    let is_play = mode.is_play();
    // In the menu, show the map plane (for the semi-transparent overlay) but
    // hide everything else (units, status, editor tools).
    let unit_sheet = is_editor && **tab == crate::EditorTab::UnitSheet;
    let sprites = is_editor && **tab == crate::EditorTab::Sprites;
    let shows_map_plane = is_play || (is_editor && tab.shows_map_plane()) || is_menu;

    if let Ok(mut vis) = vis_set.p0().single_mut() {
        *vis = vis_if(unit_sheet);
    }
    if let Ok(mut vis) = vis_set.p1().single_mut() {
        *vis = vis_if(shows_map_plane);
    }
    if let Ok(mut vis) = vis_set.p2().single_mut() {
        *vis = vis_if(sprites);
    }
    if let Ok(mut vis) = vis_set.p3().single_mut() {
        *vis = vis_if(is_play && !is_menu);
    }
    for mut vis in vis_set.p4().iter_mut() {
        *vis = vis_if(is_play && !is_menu);
    }
}

fn vis_if(show: bool) -> Visibility {
    if show {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

/// Top tab bar for switching the active [`EditorTab`] while in Editor mode.
/// Renders a thin `egui::Panel::top` over the map view using the same
/// background-layer trick as `editor_side_panel` so it overlays the 3D scene
/// without claiming CentralPanel space. Gated to `AppMode::Editor` at the
/// system registration site.
pub(crate) fn editor_tab_bar_ui(
    mut contexts: EguiContexts,
    tab: Res<State<crate::EditorTab>>,
    mut next_tab: ResMut<NextState<crate::EditorTab>>,
    mut editor_board: ResMut<EditorBoard>,
    active_map: Res<ActiveEditMap>,
    mut _pending: ResMut<PendingMapLoad>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let mut __ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("editor_tab_bar_root"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    let __panel = egui::Panel::top("editor_tab_bar")
        .exact_size(34.0)
        .frame(
            egui::Frame::default()
                .fill(crate::ui::panel_bg())
                .inner_margin(egui::Margin::symmetric(8, 4)),
        )
        .show(&mut __ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(14.0));
            ui.horizontal(|ui| {
                for &candidate in crate::EditorTab::ALL.iter() {
                    let selected = **tab == candidate;
                    if ui.selectable_label(selected, candidate.label()).clicked() {
                        next_tab.set(candidate);
                    }
                    ui.separator();
                }

                // Board picker on the right-hand side of the tab bar.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let active = active_map.0;
                    ui.label(
                        egui::RichText::new("Board:")
                            .size(11.0)
                            .color(egui::Color32::from_gray(140)),
                    );
                    for candidate in [
                        omdurman_types::Scenario::Campaign,
                        omdurman_types::Scenario::Historical,
                        omdurman_types::Scenario::FallOfKhartoum,
                    ] {
                        let label = match candidate {
                            omdurman_types::Scenario::Campaign => "Campaign",
                            omdurman_types::Scenario::Historical => "Historical",
                            omdurman_types::Scenario::FallOfKhartoum => "FoK",
                        };
                        let selected = editor_board.0 == candidate;
                        let _loaded = crate::map_kind_for_scenario(candidate) == active;
                        if ui.add(egui::Button::selectable(selected, label)).clicked()
                            && editor_board.0 != candidate
                        {
                            editor_board.0 = candidate;
                            if tab.is_board_specific() {
                                _pending.0 = Some(editor_board.map_kind());
                            }
                        }
                    }
                    if active != editor_board.map_kind() {
                        ui.label(
                            egui::RichText::new("(not loaded)")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(200, 160, 60)),
                        );
                    }
                });
            });
        });
    crate::ui_plugin::register_panel_rect(ctx, __panel.response.rect);
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
            // -- Resources ----------------------------------------------
            .insert_resource(HexEditor::default())
            .insert_resource(hexside::HexsideQuads::default())
            .insert_resource(road::RoadQuads::default())
            .insert_resource(nile::NileArrows::default())
            .insert_resource(crate::SidebarClip::default())
            .insert_resource(ScattergramPaint::default())
            // -- Startup ------------------------------------------------
            .add_systems(
                Startup,
                (
                    hexside::setup_hexside_quads,
                    road::setup_road_quads,
                    nile::setup_nile_arrows,
                    load_annotations,
                    init_gizmo_config,
                ),
            )
            // -- Update: terrain editor (EditorSet) ---------------------
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
                    handle_scattergram_click,
                    rings::draw_editor_highlight_mesh.in_set(EditorSet),
                    road::update_road_quads.after(apply_map_selection),
                    hexside::update_hexside_quads,
                    rings::draw_excluded_hex_mesh.in_set(EditorSet),
                    nile::update_nile_arrows,
                    apply_map_selection,
                ),
            )
            // Board / map-state / visibility reconcilers. Formerly fired
            // per-EditorMode-variant on OnEnter; now that mode is split into
            // AppMode + EditorTab + EditorBoard, they run every frame and
            // self-guard (each is a no-op unless its input changed), so any
            // mode/tab/board switch is picked up without a transition matrix.
            .add_systems(
                Update,
                (
                    sync_edit_board_to_mode.before(apply_map_selection),
                    sync_mode_visibilities,
                )
                    .chain(),
            )
            // Highlight / excluded-hex rings self-clean via `DespawnOnExit`
            // components on each spawned ring entity, so no OnExit handlers.
            // Leaving the Timing tab turns off scattergram paint mode so clicks
            // in other tabs don't silently toggle scattergram flags.
            .add_systems(
                OnExit(crate::EditorTab::Timing),
                |mut paint: ResMut<ScattergramPaint>| {
                    paint.0 = false;
                },
            )
            // -- Egui UI panels -----------------------------------------
            .add_systems(
                EguiPrimaryContextPass,
                (
                    editor_tab_bar_ui
                        .in_set(crate::ui_plugin::PanelUiSet)
                        .run_if(in_state(crate::AppMode::Editor)),
                    editor_ui.in_set(crate::ui_plugin::PanelUiSet),
                    hexside_editor_ui.in_set(crate::ui_plugin::PanelUiSet),
                    campaign_timing_ui.in_set(crate::ui_plugin::PanelUiSet),
                    turn_track_labels,
                ),
            )
            // -- Turn track overlay ------------------------------------
            .add_systems(Update, draw_turn_track_overlay);
    }
}

pub(crate) fn campaign_timing_ui(
    mut contexts: EguiContexts,
    mode: EditorToolState,
    mut loaded: ResMut<LoadedAnnotations>,
    active: Res<ActiveEditMap>,
    game_map: Res<GameMap>,
    mut paint: ResMut<ScattergramPaint>,
) {
    if !mode.is_timing() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let map = loaded.map_mut(active.0);

    let mut track = map
        .campaign_turn_track
        .unwrap_or(omdurman_types::CampaignTurnTrack {
            x: 0.0,
            y: 0.0,
            w: 200.0,
            h: 60.0,
        });

    let mut __ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("campaign_timing_panel"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    let __panel = egui::Panel::left("campaign_timing_panel")
        .default_size(280.0)
        .show(&mut __ui, |ui| {
            ui.heading("Campaign Turn Track");
            ui.separator();
            ui.add_space(4.0);

            // --- Bounding box editor ---
            ui.label("Pixel bounding box on the campaign-map image:");
            ui.add_space(2.0);

            let mut changed = false;
            ui.horizontal(|ui| {
                ui.label("x:");
                changed |= ui
                    .add(egui::DragValue::new(&mut track.x).speed(1.0).prefix("x: "))
                    .changed();
                ui.label("y:");
                changed |= ui
                    .add(egui::DragValue::new(&mut track.y).speed(1.0).prefix("y: "))
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("w:");
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut track.w)
                            .speed(1.0)
                            .range(1.0..=f32::MAX)
                            .prefix("w: "),
                    )
                    .changed();
                ui.label("h:");
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut track.h)
                            .speed(1.0)
                            .range(1.0..=f32::MAX)
                            .prefix("h: "),
                    )
                    .changed();
            });

            if changed {
                map.campaign_turn_track = Some(track);
            }

            // --- Howitzer Scattergram reference hexes (§6.64) ---
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Howitzer Scattergram (\u{00a7}6.64)")
                    .size(13.0)
                    .color(egui::Color32::from_gray(220)),
            );
            ui.label(
                egui::RichText::new(
                    "Seven printable reference hexes (center + ring of six) on the mapsheet.",
                )
                .size(10.0)
                .color(egui::Color32::from_gray(160)),
            );
            ui.add_space(4.0);

            // Count of currently-marked scattergram hexes on the active board.
            let marked: usize = game_map.hexes.values().filter(|d| d.is_scattergram).count();
            let target = 7;
            let count_color = if marked == target {
                egui::Color32::from_rgb(80, 200, 80)
            } else {
                egui::Color32::from_rgb(220, 160, 60)
            };
            ui.horizontal(|ui| {
                ui.label("marked:");
                ui.label(
                    egui::RichText::new(format!("{}/{}", marked, target))
                        .color(count_color)
                        .strong(),
                );
            });

            // Paint-mode toggle: when on, map clicks toggle the flag.
            let label = if paint.0 {
                "paint ON -- click hexes to toggle"
            } else {
                "paint: off"
            };
            if ui.button(label).clicked() {
                paint.0 = !paint.0;
            }

            // Small 7-hex ring diagram (flower) as a visual reminder of the
            // physical-map scattergram layout: a center hex surrounded by six
            // neighbours. Neighbour offsets use flat-side angles (k*60°, no
            // FRAC_PI_6 offset) at distance R*sqrt(3), matching the pointy-top
            // tessellation implied by `hex_corners_egui` (corners at 30°+k*60°).
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(140.0, 130.0), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            let hex_radius = 18.0;
            let center = rect.center();
            let mut positions = vec![center];
            for k in 0..6 {
                // Flat-side direction = k * 60°. Distance between touching
                // pointy-top hex centres = corner_radius * sqrt(3).
                let ang = (k as f32) * std::f32::consts::PI / 3.0;
                let off = egui::vec2(
                    ang.cos() * hex_radius * SQRT_3,
                    ang.sin() * hex_radius * SQRT_3,
                );
                positions.push(egui::pos2(center.x + off.x, center.y + off.y));
            }
            for (idx, &pos) in positions.iter().enumerate() {
                let pts = hex_corners_egui(pos, hex_radius);
                let color = if idx == 0 {
                    egui::Color32::from_rgb(120, 80, 30)
                } else {
                    egui::Color32::from_rgb(160, 120, 60)
                };
                painter.add(egui::Shape::convex_polygon(
                    pts,
                    color,
                    egui::Stroke::new(1.0, egui::Color32::from_gray(40)),
                ));
            }
        });
    crate::ui_plugin::register_panel_rect(ctx, __panel.response.rect);
}

/// Six pointy-top hex corners around `center` in egui screen space, matching
/// the 3D `hex_corners()` math in `render.rs` (angles at PI/3 stride offset by
/// FRAC_PI_6, i.e. 30°+k·60°). In egui's y-down space this produces a hex with
/// points at top (270°) and bottom (90°).
fn hex_corners_egui(center: egui::Pos2, radius: f32) -> Vec<egui::Pos2> {
    (0..6)
        .map(|k| {
            let ang = std::f32::consts::FRAC_PI_6 + (k as f32) * std::f32::consts::PI / 3.0;
            egui::pos2(
                center.x + ang.cos() * radius,
                center.y + ang.sin() * radius,
            )
        })
        .collect()
}

/// Draw the turn-track bounding-box and grid overlay (9×3) on the campaign map
/// using Bevy Gizmos, matching the pattern used by the unit-sheet grid overlay.
pub(crate) fn draw_turn_track_overlay(
    mode: EditorToolState,
    turn: Res<crate::GameTurn>,
    loaded: Res<LoadedAnnotations>,
    active: Res<ActiveEditMap>,
    mut gizmos: Gizmos,
) {
    if !mode.is_timing() {
        return;
    }
    let map = loaded.map(active.0);
    let Some(track) = map.campaign_turn_track else {
        return;
    };

    let y = 1.0;
    let cell_w = track.w / 9.0;
    let cell_h = track.h / 3.0;
    let outline_color = Color::srgb(1.0, 0.0, 0.0);
    let grid_color = Color::srgb(0.6, 0.0, 0.0);
    let highlight_color = Color::srgb(1.0, 0.2, 0.2);

    // Corners of the whole bounding box in pixel space.
    let (tl_px, tl_py) = (track.x, track.y);
    let (br_px, br_py) = (track.x + track.w, track.y + track.h);

    let tl = omdurman_hexmap::pixel_to_world_dims(tl_px, tl_py, map.img_w, map.img_h);
    let br = omdurman_hexmap::pixel_to_world_dims(br_px, br_py, map.img_w, map.img_h);

    let left = tl.x;
    let right = br.x;
    let top = tl.z;
    let bottom = br.z;

    // Outer border.
    gizmos.line(
        Vec3::new(left, y, top),
        Vec3::new(right, y, top),
        outline_color,
    );
    gizmos.line(
        Vec3::new(right, y, top),
        Vec3::new(right, y, bottom),
        outline_color,
    );
    gizmos.line(
        Vec3::new(right, y, bottom),
        Vec3::new(left, y, bottom),
        outline_color,
    );
    gizmos.line(
        Vec3::new(left, y, bottom),
        Vec3::new(left, y, top),
        outline_color,
    );

    // Vertical grid lines (cols 1..8).
    for c in 1..9 {
        let cx_px = track.x + c as f32 * cell_w;
        let cx = omdurman_hexmap::pixel_to_world_dims(cx_px, tl_py, map.img_w, map.img_h).x;
        gizmos.line(Vec3::new(cx, y, top), Vec3::new(cx, y, bottom), grid_color);
    }
    // Horizontal grid lines (rows 1..2).
    for r in 1..3 {
        let cy_px = track.y + r as f32 * cell_h;
        let cz = omdurman_hexmap::pixel_to_world_dims(tl_px, cy_px, map.img_w, map.img_h).z;
        gizmos.line(Vec3::new(left, y, cz), Vec3::new(right, y, cz), grid_color);
    }

    // Highlight the current-turn cell.
    let idx = (**turn as usize).saturating_sub(1);
    let row = idx / 9;
    let col = idx % 9;
    if row < 3 {
        let n_cols = if row == 2 { 4 } else { 9 };
        if col < n_cols {
            let cell_left_px = match row {
                0 | 2 => track.x + col as f32 * cell_w,
                1 => track.x + (9.0_f32 - col as f32 - 1.0) * cell_w,
                _ => return,
            };
            let cell_right_px = cell_left_px + cell_w;
            let cell_top_px = track.y + row as f32 * cell_h;
            let cell_bottom_px = cell_top_px + cell_h;

            let cl = omdurman_hexmap::pixel_to_world_dims(
                cell_left_px,
                cell_top_px,
                map.img_w,
                map.img_h,
            );
            let cr = omdurman_hexmap::pixel_to_world_dims(
                cell_right_px,
                cell_bottom_px,
                map.img_w,
                map.img_h,
            );

            let hx = cl.x;
            let hz = cl.z;
            let hx2 = cr.x;
            let hz2 = cr.z;

            gizmos.line(Vec3::new(hx, y, hz), Vec3::new(hx2, y, hz), highlight_color);
            gizmos.line(
                Vec3::new(hx2, y, hz),
                Vec3::new(hx2, y, hz2),
                highlight_color,
            );
            gizmos.line(
                Vec3::new(hx2, y, hz2),
                Vec3::new(hx, y, hz2),
                highlight_color,
            );
            gizmos.line(Vec3::new(hx, y, hz2), Vec3::new(hx, y, hz), highlight_color);
        }
    }
}

/// Render turn-track cell labels (Sept 1, Sept 2, …) at each grid cell centre
/// by projecting the 3D world position to screen coordinates with egui painter.
pub(crate) fn turn_track_labels(
    mut contexts: EguiContexts,
    mode: EditorToolState,
    loaded: Res<LoadedAnnotations>,
    active: Res<ActiveEditMap>,
    cameras: Query<(&Camera, &GlobalTransform), With<crate::camera::RtsCamera>>,
) {
    if !mode.is_timing() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let map = loaded.map(active.0);
    let Some(track) = map.campaign_turn_track else {
        return;
    };

    let cell_w = track.w / 9.0;
    let cell_h = track.h / 3.0;

    let screen_centre = |px: f32, py: f32| -> Option<egui::Pos2> {
        let world = omdurman_hexmap::pixel_to_world_dims(px, py, map.img_w, map.img_h);
        let world_pos = Vec3::new(world.x, 0.0, world.z);
        let viewport = camera.world_to_viewport(cam_transform, world_pos).ok()?;
        Some(egui::pos2(viewport.x, viewport.y))
    };

    for row in 0..3 {
        let n_cols = if row == 2 { 4 } else { 9 };
        for col in 0..n_cols {
            let idx = row * 9 + col;
            let turn_num = (idx + 1) as u8;
            let label = omdurman_rules::turn_track::TurnLabel::from_turn(turn_num);

            let cx_px = match row {
                0 | 2 => track.x + (col as f32 + 0.5) * cell_w,
                1 => track.x + (9.0_f32 - col as f32 - 0.5) * cell_w,
                _ => unreachable!(),
            };
            let cy_px = track.y + (row as f32 + 0.5) * cell_h;

            let Some(screen) = screen_centre(cx_px, cy_px) else {
                continue;
            };

            let text = match label {
                Some(l) => l.to_string(),
                None => format!("Turn {turn_num}"),
            };

            ctx.debug_painter().text(
                screen,
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId {
                    size: 10.0,
                    family: egui::FontFamily::Monospace,
                },
                egui::Color32::from_rgba_premultiplied(220, 80, 80, 200),
            );
        }
    }
}

