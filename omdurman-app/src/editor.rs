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
    /// When true, left-click paints the current `hexside_kind` on the nearest
    /// hex edge (right-click erases) instead of selecting/painting hexes.
    pub hexside_paint: bool,
    /// The hexside feature painted while `hexside_paint` is on.
    pub hexside_kind: HexsideKind,
}

/// B/C/D/F/P/S/V/W set terrain on the selected hex.
pub fn editor_terrain_keys(
    mode: Res<EditorMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut editor: ResMut<HexEditor>,
) {
    if *mode != EditorMode::Editor {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_keyboard_input()
    {
        return;
    }
    if editor.selected.is_none() {
        return;
    }
    let t = match () {
        _ if keys.just_pressed(KeyCode::KeyB) => Some(Terrain::BlueNile),
        _ if keys.just_pressed(KeyCode::KeyD) => Some(Terrain::Desert),
        _ if keys.just_pressed(KeyCode::KeyF) => Some(Terrain::Fortress),
        _ if keys.just_pressed(KeyCode::KeyP) => Some(Terrain::Palm),
        _ if keys.just_pressed(KeyCode::KeyS) => Some(Terrain::Shrubs),
        _ if keys.just_pressed(KeyCode::KeyW) => Some(Terrain::WhiteNile),
        _ if keys.just_pressed(KeyCode::KeyK) => Some(Terrain::Khartoum),
        _ if keys.just_pressed(KeyCode::KeyT) => Some(Terrain::Tuti),
        _ if keys.just_pressed(KeyCode::KeyH) => Some(Terrain::Hogali),
        _ if keys.just_pressed(KeyCode::KeyU) => Some(Terrain::Buri),
        _ if keys.just_pressed(KeyCode::KeyM) => Some(Terrain::FortMakran),
        _ if keys.just_pressed(KeyCode::Digit1) => Some(Terrain::FortBuri),
        _ if keys.just_pressed(KeyCode::KeyN) => Some(Terrain::NorthFort),
        _ => None,
    };
    if let Some(t) = t {
        editor.terrain = t;
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
    if *mode != EditorMode::Editor || !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    // Hexside paint mode handles clicks in its own system.
    if editor.hexside_paint {
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

    if let Some(data) = game_map.hexes.get(&coord) {
        editor.selected = Some(coord);
        editor.name = data.name.clone().unwrap_or_default();
        editor.terrain = data.terrain;
        editor.nile_flow = data.nile_flow;
    } else if editor.selected == Some(coord) {
        editor.selected = None;
    }
}

/// The edge of `coord` nearest the world point `hit` — i.e. the neighbour
/// whose shared border the click is closest to. Returns the `[coord, neighbour]`
/// pair as a canonical [`HexsideRef`], or `None` if the neighbour is off-map.
fn nearest_edge(
    coord: HexCoord,
    hit: Vec3,
    origin: bevy::math::Vec2,
    overlay: &omdurman_types::OverlayParams,
    game_map: &GameMap,
) -> Option<HexsideRef> {
    let center = hex_world_pos(coord, origin, overlay);
    // The clicked point's offset from the hex centre points toward an edge;
    // the nearest neighbour is the one whose centre direction best matches it.
    let off = Vec3::new(hit.x - center.x, 0.0, hit.z - center.z);
    if off.length() < 1e-3 {
        return None;
    }
    let neighbour = coord
        .neighbors()
        .into_iter()
        .filter(|n| game_map.hexes.contains_key(n))
        .max_by(|a, b| {
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

/// Paint (left-click) or erase (right-click) the hexside nearest the cursor
/// while hexside-paint mode is active. Broadcasts a [`GameEvent::HexsideEdit`].
#[allow(clippy::too_many_arguments)]
pub fn handle_hexside_paint(
    mode: Res<EditorMode>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    mut game_map: ResMut<GameMap>,
    editor: Res<HexEditor>,
    mut pending: ResMut<PendingEdits>,
    mut dirty: ResMut<crate::AnnotationsDirty>,
) {
    if *mode != EditorMode::Editor || !editor.hexside_paint {
        return;
    }
    let paint = buttons.just_pressed(MouseButton::Left);
    let erase = buttons.just_pressed(MouseButton::Right);
    if !paint && !erase {
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
    let Some(edge) = nearest_edge(coord, hit, origin, &overlay.params, &game_map) else {
        return;
    };

    let kind = if paint {
        Some(editor.hexside_kind)
    } else {
        None
    };
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
        .push(NetMsg::Game(GameEvent::HexsideEdit { edge, kind }));
    dirty.mark();
}

/// Draw all hexsides as a coloured segment along the shared edge while in
/// Editor mode, so the painted walls/khors/etc. are visible.
pub fn draw_hexsides(
    mode: Res<EditorMode>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut gizmos: Gizmos,
) {
    if *mode != EditorMode::Editor {
        return;
    }
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    for (edge, kind) in &game_map.hexsides {
        let a = hex_world_pos(edge.a, origin, &overlay.params);
        let b = hex_world_pos(edge.b, origin, &overlay.params);
        // The shared border is the perpendicular bisector segment at the
        // midpoint of the two hex centres; draw a short bar there.
        let mid = (a + b) * 0.5;
        let along = (b - a).normalize_or_zero();
        // Perpendicular in the ground plane.
        let perp = Vec3::new(-along.z, 0.0, along.x);
        let half = overlay.params.hex_size * 0.5;
        let p0 = Vec3::new(mid.x, 1.0, mid.z) - perp * half;
        let p1 = Vec3::new(mid.x, 1.0, mid.z) + perp * half;
        gizmos.line(p0, p1, hexside_color(*kind));
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

pub fn draw_editor_highlight(
    mode: Res<EditorMode>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    editor: Res<HexEditor>,
    mut gizmos: Gizmos,
) {
    if *mode != EditorMode::Editor {
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
    if *mode != EditorMode::Editor {
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
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if *mode != EditorMode::Editor {
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
        let font_size = 10.0;
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
                    egui::ComboBox::from_id_salt("terrain")
                        .selected_text(format!("{}", editor.terrain))
                        .show_ui(ui, |ui| {
                            for t in Terrain::iter() {
                                ui.selectable_value(&mut editor.terrain, t, format!("{}", t));
                            }
                        });
                });

                // Nile current annotation: a single arrow per hex, pointing
                // downstream, rotated by the +/- buttons (rulebook §5.11,
                // §5.24). Every Nile hex always carries a current, so the only
                // choice is its direction.
                if editor.terrain.is_nile() {
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

            // ── Hexside painting (§5.23, §6.3, §7.2) ──────────────────────
            ui.add_space(8.0);
            ui.separator();
            ui.checkbox(&mut editor.hexside_paint, "paint hexsides");
            if editor.hexside_paint {
                ui.horizontal(|ui| {
                    ui.label("side");
                    egui::ComboBox::from_id_salt("hexside_kind")
                        .selected_text(editor.hexside_kind.to_string())
                        .show_ui(ui, |ui| {
                            for k in HexsideKind::iter() {
                                ui.selectable_value(&mut editor.hexside_kind, k, k.to_string());
                            }
                        });
                });
                ui.label(
                    egui::RichText::new("L-click edge: paint · R-click: erase")
                        .size(11.0)
                        .color(egui::Color32::from_gray(160)),
                );
            }
        });
    clip.right_sidebar = Some(response.response.rect);
    if let Some(coord) = editor.selected
        && game_map.hexes.contains_key(&coord)
    {
        let terrain = editor.terrain;
        let editor_name = editor.name.clone();
        // Flow is only carried by Nile hexes; on any other terrain the
        // annotation is dropped.
        let new_flow = if terrain.is_nile() {
            editor.nile_flow
        } else {
            None
        };
        if let Some(d) = game_map.hexes.get_mut(&coord) {
            let new_name = (!editor_name.is_empty()).then(|| editor_name.clone());
            let changed = d.terrain != terrain || d.name != new_name || d.nile_flow != new_flow;
            if changed {
                pending
                    .outgoing_broadcast
                    .push(NetMsg::Game(GameEvent::MapEdit {
                        q: coord.q,
                        r: coord.r,
                        terrain: terrain.to_u8(),
                        name: editor_name,
                        nile_flow: new_flow,
                    }));
                d.terrain = terrain;
                d.name = new_name;
                d.nile_flow = new_flow;
                // Map edits mutate in-memory state and are recorded in the
                // event log. Mark annotations.ron dirty; the flush system
                // debounces writes until edits go idle.
                dirty.mark();
            }
        }
    }
}
