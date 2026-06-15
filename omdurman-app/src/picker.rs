//! Unit picker: the left sidebar of available counters, placement preview,
//! click handling for place / select / move, and movement animation.
//!
//! Placement and movement are *requested* here by broadcasting
//! [`GameEvent::PlaceUnit`] / [`GameEvent::MoveUnit`]; the authoritative state
//! change (allocating a rules-engine `UnitId`, validating the move against the
//! unit's movement allowance, updating position) happens in
//! `apply_pending_placement`, which consumes those events. Keeping the request
//! and the application separate is what lets the same code path serve live
//! play and history replay.

use bevy::app::Plugin;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::{GameMap, HexLayout};
use omdurman_types::{HexCoord, Terrain};

use std::collections::{HashSet, VecDeque};

use crate::EditorMode;
use crate::browser::{SpriteAnnotationsResource, section_order};
use crate::camera::RtsCamera;
use crate::events;
use crate::render::{HexOverlay, HexRingAssets};
use crate::util::raycast_ground;
use omdurman_hexmap::{adjusted_origin, hex_world_pos, hit_to_hex};
use omdurman_net::GameEvent;
use omdurman_rules::UnitId;

/// The selected unit's rules `UnitId` and hex, if it is engine-tracked.
pub fn selected_unit_id(
    state: &PickerState,
    placed_units: &Query<(Entity, &PlacedUnit)>,
) -> Option<(UnitId, HexCoord)> {
    let PickerState::Selected { source, .. } = *state else {
        return None;
    };
    let (_, placed) = placed_units.get(source).ok()?;
    Some((placed.unit_id?, placed.coord))
}

use omdurman_types::SectionName;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/sprites.rs"));
}

/// Sprite quad size as a fraction of the hex size, so a placed counter sits
/// just inside its hex.
const SPRITE_HEX_FRACTION: f32 = 1.05;
/// Height above the ground plane at which placed-unit quads are drawn.
const UNIT_HEIGHT: f32 = 1.0;
/// Seconds a unit takes to slide from one hex to an adjacent one.
const MOVE_ANIM_SECS: f32 = 0.3;

fn terrain_passable(terrain: Terrain, is_boat: bool) -> bool {
    if is_boat {
        terrain.is_nile()
    } else {
        terrain.passable_by_land()
    }
}

/// Whether a unit may occupy `coord`. Off-map coordinates (those not present
/// in `game_map.hexes`, which is clipped to the active overlay) are never
/// valid -- earlier code allowed land units to be placed off-map because the
/// map wasn't guaranteed loaded; the late-joiner snapshot flow now guarantees
/// `LoadAnnotations` arrives before any placement is possible.
fn coord_passable(game_map: &GameMap, coord: HexCoord, is_boat: bool) -> bool {
    game_map
        .hexes
        .get(&coord)
        .is_some_and(|h| terrain_passable(h.terrain, is_boat))
}

/// Movement points required to enter `coord` for a land unit -- terrain cost
/// from the Terrain Effects Chart.  Returns 0 if the hex is off-map or
/// impassable (callers should check passability separately).
fn floor_movement_cost(game_map: &GameMap, coord: HexCoord) -> i16 {
    let Some(tile) = game_map.hexes.get(&coord) else {
        return 0;
    };
    omdurman_rules::terrain_chart::movement_cost(tile.terrain)
        .map_or(0, |c| c.value() as i16)
}

// -- Resources ------------------------------------------------------------------

#[derive(Resource, Default, Clone)]
pub struct UnitPicker {
    pub available: Vec<PickerUnit>,
    pub all: Vec<(SectionName, u32, u32, Handle<Image>, bool)>,
}

impl UnitPicker {
    pub fn reset_available(&mut self) {
        self.available = self
            .all
            .iter()
            .map(|(sn, col, row, handle, is_boat)| PickerUnit {
                section_name: *sn,
                col: *col,
                row: *row,
                handle: handle.clone(),
                is_boat: *is_boat,
                visible: true,
                egui_texture: None,
                annotations_loaded: false,
            })
            .collect();
    }

    /// Create a fresh picker with all units available, sharing the same
    /// `all` list. egui textures are cleared (they will be re-loaded).
    pub fn fresh_copy(&self) -> Self {
        let available = self
            .all
            .iter()
            .map(|(sn, col, row, handle, is_boat)| PickerUnit {
                section_name: *sn,
                col: *col,
                row: *row,
                handle: handle.clone(),
                is_boat: *is_boat,
                visible: true,
                egui_texture: None,
                annotations_loaded: false,
            })
            .collect();
        UnitPicker {
            available,
            all: self.all.clone(),
        }
    }
}

#[derive(Clone)]
pub struct PickerUnit {
    pub section_name: SectionName,
    pub col: u32,
    pub row: u32,
    pub handle: Handle<Image>,
    pub is_boat: bool,
    pub visible: bool,
    pub egui_texture: Option<egui::TextureHandle>,
    pub annotations_loaded: bool,
}

#[derive(Resource, Default, Clone, Copy)]
pub enum PickerState {
    #[default]
    Idle,
    Placing {
        unit_idx: usize,
        preview_hex: Option<HexCoord>,
        preview_valid: bool,
        drag_drop: bool,
    },
    /// A friendly unit has been selected.  Actions:
    /// * Left-click on adjacent empty passable hex -> move
    /// * Right-click -> deselect
    Selected {
        source: Entity,
        start_coord: HexCoord,
        remaining_mp: i16,
    },
}

// -- Components -----------------------------------------------------------------

#[derive(Component)]
pub struct PlacedUnit {
    pub coord: HexCoord,
    pub section_name: SectionName,
    pub col: u32,
    pub row: u32,
    pub is_boat: bool,
    /// The rules-engine unit ID, assigned when the unit is first placed
    /// and the corresponding [`omdurman_rules::UnitPlacement`] is created.
    pub unit_id: Option<UnitId>,
    /// Last-rendered disruption state. A disrupted counter is shown
    /// *inverted* (flipped 180 deg in-plane) and dimmed, mirroring the physical
    /// game where a disrupted counter is turned over (rulebook Combat Results
    /// Table note; §5.41). Kept here so the sync system only re-skins the
    /// counter when its state actually changes.
    pub disrupted: bool,
}

/// Marker present on the currently-selected unit entity. Allows ECS queries
/// like `Query<&PlacedUnit, With<Selected>>` without touching `PickerState`.
#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct MovementAnimation {
    pub from: Vec3,
    pub to: Vec3,
    pub progress: f32,
    pub target_coord: HexCoord,
}

// -- Shared spawn helper --------------------------------------------------------

/// Spawn the mesh + material for a placed counter and return its entity.
///
/// Used both by interactive placement here and by `apply_pending_placement`
/// in `main.rs` when applying inbound/replayed `PlaceUnit` events, so the two
/// paths can't drift in how a counter is built.
#[allow(clippy::too_many_arguments)]
pub fn spawn_placed_unit(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    texture: Handle<Image>,
    overlay: &HexOverlay,
    world_pos: Vec3,
    placed: PlacedUnit,
) -> Entity {
    let sprite_size = overlay.params.hex_size * SPRITE_HEX_FRACTION;
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        unlit: true,
        alpha_mode: AlphaMode::Mask(0.1),
        ..default()
    });
    commands
        .spawn((
            placed,
            Mesh3d(meshes.add(Rectangle::new(sprite_size, sprite_size))),
            MeshMaterial3d(material),
            Transform::from_xyz(world_pos.x, UNIT_HEIGHT, world_pos.z)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
            Visibility::Visible,
        ))
        .id()
}

// -- Startup: load sprite handles for the picker -------------------------------

pub fn spawn_picker_assets(mut picker: ResMut<UnitPicker>, asset_server: Res<AssetServer>) {
    let order = section_order();

    let mut section_sprites: Vec<Vec<PickerUnit>> = order.iter().map(|_| Vec::new()).collect();

    for &(filename, col, row) in generated::SPRITE_PATHS {
        let section_idx = order.iter().position(|s| {
            let s = s.to_string();
            filename.starts_with(&s) && filename.as_bytes().get(s.len()) == Some(&b'_')
        });
        if let Some(idx) = section_idx {
            let path = format!("sprites/{}.png", filename);
            let handle = asset_server.load(&path);
            section_sprites[idx].push(PickerUnit {
                section_name: order[idx],
                col,
                row,
                handle,
                is_boat: false,
                visible: true,
                egui_texture: None,
                annotations_loaded: false,
            });
        }
    }

    for sprites in section_sprites {
        for sprite in sprites {
            picker.all.push((
                sprite.section_name,
                sprite.col,
                sprite.row,
                sprite.handle.clone(),
                sprite.is_boat,
            ));
            picker.available.push(sprite);
        }
    }
}

// -- Left sidebar: list available units -----------------------------------------

fn load_egui_texture(
    ctx: &egui::Context,
    image: &Image,
    label: &str,
) -> Option<egui::TextureHandle> {
    let w = image.width() as usize;
    let h = image.height() as usize;
    if w == 0 || h == 0 {
        return None;
    }
    let data = image.data.as_ref()?;
    if data.len() < w * h * 4 {
        return None;
    }
    let pixels: Vec<egui::Color32> = data
        .chunks(4)
        .take(w * h)
        .map(|c| egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
        .collect();
    if pixels.len() != w * h {
        return None;
    }
    let color_image = egui::ColorImage {
        size: [w, h],
        pixels,
        source_size: egui::vec2(w as f32, h as f32),
    };
    Some(ctx.load_texture(label, color_image, egui::TextureOptions::LINEAR))
}

pub fn unit_picker_ui(
    mut contexts: EguiContexts,
    mode: Res<State<EditorMode>>,
    mut picker: ResMut<UnitPicker>,
    mut state: ResMut<PickerState>,
    images: Res<Assets<Image>>,
    annotations: Option<Res<SpriteAnnotationsResource>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_map_mode() {
        return;
    }

    // -- cache egui textures & look up is_boat from annotations --
    for unit in &mut picker.available {
        if unit.egui_texture.is_none()
            && let Some(image) = images.get(&unit.handle)
        {
            let label = format!("picker_{}_{}_{}", unit.section_name, unit.col, unit.row);
            unit.egui_texture = load_egui_texture(ctx, image, &label);
        }
        if !unit.annotations_loaded {
            if (!unit.is_boat || unit.visible)
                && let Some(ref ann) = annotations
            {
                let entry = ann
                    .0
                    .units
                    .get(&unit.section_name)
                    .and_then(|m| m.get(&(unit.col, unit.row)));
                if let Some(a) = entry {
                    if a.is_boat {
                        unit.is_boat = true;
                    }
                    if !a.is_unit {
                        unit.visible = false;
                    }
                }
            }
            unit.annotations_loaded = true;
        }
    }

    // -- sidebar --
    egui::SidePanel::left("unit_picker_panel")
        .resizable(true)
        .default_width(200.0)
        .width_range(140.0..=320.0)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(45))
                .inner_margin(egui::Margin::symmetric(8, 8)),
        )
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(14.0));
            ui.label(
                egui::RichText::new("Unit Picker")
                    .size(16.0)
                    .color(egui::Color32::from_gray(220)),
            );
            ui.separator();
            ui.add_space(4.0);

            if picker.available.is_empty() {
                ui.colored_label(egui::Color32::from_gray(140), "all units placed");
            }

            let mut clicked_idx: Option<usize> = None;
            let mut drag_idx: Option<usize> = None;
            let sprite_size = 44.0;
            let margin = 2.0;
            let cell_size = sprite_size + margin * 2.0;

            // clear selection if the picked unit is now invisible
            if let PickerState::Placing { unit_idx, .. } = *state
                && picker.available.get(unit_idx).is_some_and(|u| !u.visible) {
                    *state = PickerState::Idle;
                }

            ui.style_mut().spacing.scroll.floating = false;
            egui::ScrollArea::vertical()
                .id_salt("unit_picker_scroll")
                .show(ui, |ui| {
                    let mut current_section = None::<SectionName>;

                    for idx in 0..picker.available.len() {
                        if !picker.available[idx].visible {
                            continue;
                        }
                        let section_name = picker.available[idx].section_name;

                        if Some(section_name) != current_section {
                            current_section = Some(section_name);
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(section_name.display_name())
                                .size(13.0)
                                .color(egui::Color32::from_gray(180)),
                            );
                            ui.add_space(2.0);

                            ui.horizontal_wrapped(|ui| {
                                for j in idx..picker.available.len() {
                                    let next_section = &picker.available[j].section_name;
                                    if Some(*next_section) != current_section {
                                        break;
                                    }
                                    if !picker.available[j].visible {
                                        continue;
                                    }

                                    let is_selected = matches!(*state, PickerState::Placing { unit_idx, .. } if unit_idx == j);
                                    let unit = &picker.available[j];

                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::Vec2::new(cell_size, cell_size),
                                        egui::Sense::click_and_drag(),
                                    );

                                    let bg = if is_selected {
                                        egui::Color32::from_rgb(60, 100, 60)
                                    } else if response.hovered() {
                                        egui::Color32::from_rgb(60, 60, 80)
                                    } else {
                                        egui::Color32::from_gray(35)
                                    };
                                    let painter = ui.painter();
                                    painter.rect_filled(rect, 3.0, bg);

                                    if let Some(tex_id) = unit.egui_texture.as_ref().map(|t| t.id()) {
                                        let img_rect = egui::Rect::from_center_size(
                                            rect.center(),
                                            egui::Vec2::new(sprite_size, sprite_size),
                                        );
                                        painter.image(
                                            tex_id,
                                            img_rect,
                                            egui::Rect::from_min_max(
                                                egui::pos2(0.0, 0.0),
                                                egui::pos2(1.0, 1.0),
                                            ),
                                            egui::Color32::WHITE,
                                        );
                                    } else {
                                        painter.text(
                                            rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            format!("{}x{}", unit.col, unit.row),
                                            egui::FontId::proportional(10.0),
                                            egui::Color32::from_gray(120),
                                        );
                                    }

                                    if response.clicked() {
                                        clicked_idx = Some(j);
                                    }
                                    if response.drag_started() {
                                        drag_idx = Some(j);
                                    }
                                }
                            });
                        }
                    }
                });

            if let Some(idx) = clicked_idx {
                match *state {
                    PickerState::Placing { unit_idx, .. } if unit_idx == idx => {
                        *state = PickerState::Idle;
                    }
                    _ => {
                        *state = PickerState::Placing {
                            unit_idx: idx,
                            preview_hex: None,
                            preview_valid: false,
                            drag_drop: false,
                        };
                    }
                }
            }
            if let Some(idx) = drag_idx {
                *state = PickerState::Placing {
                    unit_idx: idx,
                    preview_hex: None,
                    preview_valid: false,
                    drag_drop: true,
                };
            }
        });

    // -- ghost sprite at cursor when placing --
    if let PickerState::Placing { unit_idx, .. } = *state
        && let Some(unit) = picker.available.get(unit_idx)
        && let Some(tex_id) = unit.egui_texture.as_ref().map(|t| t.id())
        && let Some(pos) = ctx.pointer_latest_pos()
    {
        let ghost_size = 48.0;
        let ghost_rect = egui::Rect::from_center_size(pos, egui::Vec2::new(ghost_size, ghost_size));
        ctx.debug_painter().image(
            tex_id,
            ghost_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::from_white_alpha(180),
        );
    }
}

// -- Placement preview: green/red hex highlight ---------------------------------

#[derive(Component)]
pub(crate) struct PreviewHexRing;

pub fn placement_preview_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    picker: Res<UnitPicker>,
    mut state: ResMut<PickerState>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    placed_units: Query<&PlacedUnit>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    existing: Query<Entity, With<PreviewHexRing>>,
) {
    for e in &existing {
        commands.entity(e).despawn();
    }

    let PickerState::Placing {
        unit_idx,
        ref mut preview_hex,
        ref mut preview_valid,
        ..
    } = *state
    else {
        return;
    };

    let Some(unit) = picker.available.get(unit_idx) else {
        *preview_hex = None;
        return;
    };

    let Some(hit) = raycast_ground(&windows, &cameras) else {
        *preview_hex = None;
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    if !game_map.hexes.contains_key(&coord) {
        *preview_hex = None;
        return;
    }

    let occupied = placed_units.iter().any(|u| u.coord == coord);
    let valid = !occupied && coord_passable(&game_map, coord, unit.is_boat);
    *preview_hex = Some(coord);
    *preview_valid = valid;

    let pos = hex_world_pos(coord, origin, &overlay.params);
    let material = if valid {
        assets.green.clone()
    } else {
        assets.red.clone()
    };

    commands.spawn((
        PreviewHexRing,
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(overlay.params.hex_size)),
        Visibility::Visible,
    ));
}

// -- Click handling: placement + movement ---------------------------------------

pub fn handle_picker_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut picker: ResMut<UnitPicker>,
    mut state: ResMut<PickerState>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut action_writer: MessageWriter<events::LocalAction>,
    game_state: Option<Res<crate::GameStateResource>>,
) {
    let pressed = buttons.just_pressed(MouseButton::Left);
    let released = buttons.just_released(MouseButton::Left);
    if !pressed && !released {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_pointer_input() {
        return;
    }

    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    match *state {
        PickerState::Idle => handle_idle_click(
            pressed,
            coord,
            &placed_units,
            &mut state,
            &mut commands,
            game_state.as_deref(),
        ),
        PickerState::Placing {
            unit_idx,
            drag_drop,
            ..
        } => {
            let mut placing = PlacingClick {
                picker: &mut picker,
                state: &mut state,
                overlay: &overlay,
                game_map: &game_map,
                commands: &mut commands,
                meshes: &mut meshes,
                materials: &mut materials,
                origin,
            };
            if let Some(event) = placing.handle(&placed_units, released, unit_idx, drag_drop, coord)
            {
                action_writer.write(events::LocalAction { event });
            }
        }
        PickerState::Selected {
            source,
            start_coord,
            remaining_mp,
        } => {
            let mut sel = SelectedClick {
                state: &mut state,
                overlay: &overlay,
                game_map: &game_map,
                commands: &mut commands,
                origin,
                remaining_mp,
            };
            if let Some(event) = sel.handle(&placed_units, released, source, start_coord, coord) {
                info!("writing LocalAction for MoveUnit");
                action_writer.write(events::LocalAction { event });
            }
            if matches!(*state, PickerState::Idle) {
                commands
                    .entity(source)
                    .remove::<Selected>();
            }
        }
    }
}

/// Idle: a left-press on a placed unit selects it.
fn handle_idle_click(
    pressed: bool,
    coord: HexCoord,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    state: &mut PickerState,
    commands: &mut Commands,
    game_state: Option<&crate::GameStateResource>,
) {
    if !pressed {
        return;
    }
    if let Some((entity, placed)) = placed_units.iter().find(|(_, u)| u.coord == coord) {
        let remaining_mp = if let Some(uid) = placed.unit_id
            && let Some(gs) = game_state
            && let Some(unit) = gs.0.find_unit(uid)
        {
            match unit.profile.movement {
                omdurman_rules::UnitMovement::Land(a) => a.value() as i16,
                _ => 99,
            }
        } else {
            99
        };
        commands
            .entity(entity)
            .insert(Selected);
        *state = PickerState::Selected {
            source: entity,
            start_coord: coord,
            remaining_mp,
        };
    }
}

/// Borrowed context for resolving a click while placing a counter.
///
/// The map query is *not* stored here -- it is passed to [`handle`](Self::handle)
/// as a parameter. `Query` is invariant over its data, so coupling its
/// world/state lifetimes to the struct's other borrows (notably `Commands`)
/// would make the struct unconstructible from a normal Bevy system.
struct PlacingClick<'a, 'w, 's> {
    picker: &'a mut UnitPicker,
    state: &'a mut PickerState,
    overlay: &'a HexOverlay,
    game_map: &'a GameMap,
    commands: &'a mut Commands<'w, 's>,
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<StandardMaterial>,
    origin: Vec2,
}

impl PlacingClick<'_, '_, '_> {
    fn handle(
        &mut self,
        placed_units: &Query<(Entity, &PlacedUnit)>,
        released: bool,
        unit_idx: usize,
        drag_drop: bool,
        coord: HexCoord,
    ) -> Option<GameEvent> {
        if released && !drag_drop {
            *self.state = PickerState::Placing {
                unit_idx,
                preview_hex: None,
                preview_valid: false,
                drag_drop: false,
            };
            return None;
        }

        let Some(unit) = self.picker.available.get(unit_idx) else {
            *self.state = PickerState::Idle;
            return None;
        };

        let can_place = !placed_units.iter().any(|(_, u)| u.coord == coord)
            && coord_passable(self.game_map, coord, unit.is_boat);

        if can_place {
            let pos = hex_world_pos(coord, self.origin, &self.overlay.params);
            let unit = self.picker.available.remove(unit_idx);

            spawn_placed_unit(
                self.commands,
                self.meshes,
                self.materials,
                unit.handle.clone(),
                self.overlay,
                pos,
                PlacedUnit {
                    coord,
                    section_name: unit.section_name,
                    col: unit.col,
                    row: unit.row,
                    is_boat: unit.is_boat,
                    unit_id: None,
                    disrupted: false,
                },
            );

            info!(
                section_name = %unit.section_name,
                col = unit.col,
                row = unit.row,
                coord.q = coord.q,
                coord.r = coord.r,
                "placing unit"
            );
            *self.state = PickerState::Idle;
            return Some(GameEvent::PlaceUnit {
                sprite: omdurman_types::SpriteRef {
                    section_name: unit.section_name,
                    col: unit.col,
                    row: unit.row,
                },
                coord_q: coord.q,
                coord_r: coord.r,
                is_boat: unit.is_boat,
            });
        }
        *self.state = PickerState::Idle;
        None
    }
}

/// Borrowed context for resolving a click while a unit is selected.
///
/// As with [`PlacingClick`], the map query is passed to
/// [`handle`](Self::handle) rather than stored, so the invariant `Query`
/// lifetimes never couple to the struct's `Commands` borrow.
struct SelectedClick<'a, 'w, 's> {
    state: &'a mut PickerState,
    overlay: &'a HexOverlay,
    game_map: &'a GameMap,
    commands: &'a mut Commands<'w, 's>,
    origin: Vec2,
    remaining_mp: i16,
}

impl SelectedClick<'_, '_, '_> {
    fn handle(
        &mut self,
        placed_units: &Query<(Entity, &PlacedUnit)>,
        released: bool,
        source: Entity,
        start_coord: HexCoord,
        coord: HexCoord,
    ) -> Option<GameEvent> {
        if !released {
            return None;
        }
        if coord == start_coord {
            return None;
        }
        let Ok((_, placed)) = placed_units.get(source) else {
            *self.state = PickerState::Idle;
            return None;
        };

        let target_occupied = placed_units.iter().any(|(_, u)| u.coord == coord);
        let adjacent = placed.coord.neighbors().contains(&coord);
        let passable = coord_passable(self.game_map, coord, placed.is_boat);
        let cost = if adjacent {
            floor_movement_cost(self.game_map, coord)
        } else {
            0
        };
        let affordable = cost > 0 && self.remaining_mp >= cost;

        
        if adjacent
            && !target_occupied
            && passable
            && affordable
            && self.rules_allow_move(placed, coord)
        {
            let new_remaining = self.remaining_mp - cost;
            info!(
                "move accepted, cost={}, remaining_mp={}",
                cost, new_remaining
            );
            if new_remaining > 0 {
                *self.state = PickerState::Selected {
                    source,
                    start_coord: coord,
                    remaining_mp: new_remaining,
                };
            } else {
                *self.state = PickerState::Idle;
            }
            Some(self.commit_move(source, placed.coord, coord, placed))
        } else {
            info!(
                source = source.to_bits(),
                adjacent,
                target_occupied,
                passable,
                affordable,
                cost,
                remaining_mp = self.remaining_mp,
                "move rejected",
            );
            *self.state = PickerState::Idle;
            None
        }
    }

    fn rules_allow_move(&self, placed: &PlacedUnit, to: HexCoord) -> bool {
        if self
            .game_map
            .hexside_between(placed.coord, to)
            .is_some_and(|s| s.blocks_movement())
        {
            info!("move blocked by wall hexside");
            return false;
        }
        true
    }

    fn commit_move(
        &mut self,
        source: Entity,
        from: HexCoord,
        to: HexCoord,
        placed: &PlacedUnit,
    ) -> GameEvent {
        let from_pos = hex_world_pos(from, self.origin, &self.overlay.params);
        let to_pos = hex_world_pos(to, self.origin, &self.overlay.params);

        info!(
            section_name = %placed.section_name,
            col = placed.col,
            row = placed.row,
            from.q = from.q,
            from.r = from.r,
            to.q = to.q,
            to.r = to.r,
            "moving unit"
        );
        self.commands.entity(source).insert(MovementAnimation {
            from: Vec3::new(from_pos.x, UNIT_HEIGHT, from_pos.z),
            to: Vec3::new(to_pos.x, UNIT_HEIGHT, to_pos.z),
            progress: 0.0,
            target_coord: to,
        });

        let cost = floor_movement_cost(self.game_map, to);
        GameEvent::MoveUnit {
            sprite: omdurman_types::SpriteRef {
                section_name: placed.section_name,
                col: placed.col,
                row: placed.row,
            },
            to_q: to.q,
            to_r: to.r,
            cost,
        }
    }
}

// -- Movement overlay: light-green hex outlines ---------------------------------

#[derive(Component)]
pub(crate) struct MovementHexRing;

#[derive(Component)]
pub(crate) struct MovementRangeRing;

pub fn movement_overlay_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    existing_green: Query<Entity, With<MovementHexRing>>,
    existing_gray: Query<Entity, With<MovementRangeRing>>,
    mut last_key: Local<Option<(Entity, i16)>>,
) {
    // Despawn all existing overlay rings (both green and gray).
    for e in &existing_green {
        commands.entity(e).despawn();
    }
    for e in &existing_gray {
        commands.entity(e).despawn();
    }

    let PickerState::Selected {
        source,
        remaining_mp,
        ..
    } = *state
    else {
        *last_key = None;
        return;
    };

    if *last_key == Some((source, remaining_mp)) {
        return;
    }

    let Ok((_, placed)) = placed_units.get(source) else {
        return;
    };

    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let size = overlay.params.hex_size;

    // BFS from the unit's coord, accumulating terrain costs.
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut green_spawned = 0u32;
    let mut gray_spawned = 0u32;

    queue.push_back((placed.coord, 0i16));
    visited.insert(placed.coord);

    while let Some((cur, cost_so_far)) = queue.pop_front() {
        for neighbor in cur.neighbors() {
            if visited.contains(&neighbor) {
                continue;
            }
            if placed_units.iter().any(|(_, u)| u.coord == neighbor) {
                continue;
            }
            if !coord_passable(&game_map, neighbor, placed.is_boat) {
                continue;
            }
            let terrain_cost = floor_movement_cost(&game_map, neighbor);
            if terrain_cost <= 0 {
                continue;
            }
            let new_cost = cost_so_far + terrain_cost;
            if new_cost > remaining_mp {
                continue;
            }
            visited.insert(neighbor);
            queue.push_back((neighbor, new_cost));

            let pos = hex_world_pos(neighbor, origin, &overlay.params);
            let is_adjacent = placed.coord.neighbors().contains(&neighbor);
            if is_adjacent {
                commands.spawn((
                    MovementHexRing,
                    Mesh3d(assets.mesh.clone()),
                    MeshMaterial3d(assets.light_green.clone()),
                    Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(size)),
                    Visibility::Visible,
                ));
                green_spawned += 1;
            } else {
                commands.spawn((
                    MovementRangeRing,
                    Mesh3d(assets.mesh.clone()),
                    MeshMaterial3d(assets.gray.clone()),
                    Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(size)),
                    Visibility::Visible,
                ));
                gray_spawned += 1;
            }
        }
    }

    info!(
        green_spawned,
        gray_spawned, remaining_mp, "movement_overlay_mesh: done"
    );
    *last_key = Some((source, remaining_mp));
}

// -- Animation: lerp unit movement ----------------------------------------------

pub fn animate_unit_movement(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut MovementAnimation,
        &mut PlacedUnit,
    )>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut anim, mut placed) in query.iter_mut() {
        anim.progress += time.delta_secs() / MOVE_ANIM_SECS;
        if anim.progress >= 1.0 {
            transform.translation = anim.to;
            placed.coord = anim.target_coord;
            info!(
                entity = entity.to_bits(),
                coord.q = placed.coord.q,
                coord.r = placed.coord.r,
                "movement animation complete"
            );
            commands.entity(entity).remove::<MovementAnimation>();
        } else {
            let t = anim.progress;
            let ease = smoothstep(t);
            transform.translation = anim.from.lerp(anim.to, ease);
        }
    }
}

/// Smoothstep easing: 0 at t=0, 1 at t=1, zero slope at both ends.
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

// -- Disruption visuals: inverted + dimmed counter ------------------------------

/// Lay a counter quad flat on the ground, optionally *inverted* (turned over)
/// to show disruption. Inversion is a 180 deg spin about the vertical axis, the
/// 3D analogue of flipping the physical counter face-down (rulebook Combat
/// Results Table note; §5.41).
fn counter_rotation(disrupted: bool) -> Quat {
    let flat = Quat::from_rotation_x(-std::f32::consts::PI / 2.0);
    if disrupted {
        Quat::from_rotation_y(std::f32::consts::PI) * flat
    } else {
        flat
    }
}

/// Mirror each placed counter's disruption state from the authoritative
/// rules engine into its visuals: a disrupted unit is shown inverted and
/// dimmed, recovering to upright/full-colour when the rules engine clears the
/// flag at end of the owning player's turn (rulebook §5.41, Combat Results
/// Table note). Only re-skins a counter when its state actually changes.
pub fn sync_disrupted_visuals(
    game_state: Option<Res<crate::GameStateResource>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(
        &mut PlacedUnit,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let Some(game_state) = game_state else {
        return;
    };
    for (mut placed, mut transform, material) in query.iter_mut() {
        let Some(uid) = placed.unit_id else {
            continue;
        };
        let disrupted = game_state
            .0
            .find_unit(uid)
            .is_some_and(|u| u.state.disrupted);
        if disrupted == placed.disrupted {
            continue;
        }
        placed.disrupted = disrupted;
        transform.rotation = counter_rotation(disrupted);
        if let Some(mat) = materials.get_mut(&material.0) {
            // Dim disrupted counters; full brightness when recovered.
            mat.base_color = if disrupted {
                Color::srgb(0.55, 0.55, 0.55)
            } else {
                Color::WHITE
            };
        }
    }
}

// -- Cancel placement/movement on right-click ----------------------------------

pub fn cancel_placement(
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut state: ResMut<PickerState>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_pointer_input()
    {
        return;
    }
    *state = PickerState::Idle;
}

/// Registers all game-domain resources and systems: unit picker, combat
/// (fire/melee/retreat), movement animation, dice rolling, and placement
/// application. Systems are gated via [`crate::GameSet`] (active in map modes).
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // -- Resources ----------------------------------------------
            .insert_resource(UnitPicker::default())
            .insert_resource(PickerState::default())
            // -- Startup ------------------------------------------------
            .add_systems(Startup, (spawn_picker_assets,))
            // -- Update: gameplay (GameSet) -----------------------------
            .add_systems(
                Update,
                (
                    crate::dice::despawn_dice,
                    crate::apply_pending_placement.after(crate::net_socket::handle_socket),
                    (
                        crate::dice::handle_local_input.after(crate::net_socket::handle_socket),
                        placement_preview_mesh.in_set(crate::GameSet),
                        crate::fire::handle_fire_combat
                            .in_set(crate::GameSet)
                            .before(handle_picker_clicks),
                        crate::melee::handle_melee_combat
                            .in_set(crate::GameSet)
                            .before(handle_picker_clicks),
                        crate::melee::handle_advance_after_combat
                            .in_set(crate::GameSet)
                            .after(crate::melee::handle_melee_combat)
                            .after(crate::fire::handle_fire_combat)
                            .before(handle_picker_clicks),
                        crate::retreat::handle_retreat
                            .in_set(crate::GameSet)
                            .before(handle_picker_clicks),
                        handle_picker_clicks.in_set(crate::GameSet),
                        movement_overlay_mesh.in_set(crate::GameSet),
                        crate::fire::fire_target_overlay_mesh.in_set(crate::GameSet),
                        crate::melee::melee_target_overlay_mesh.in_set(crate::GameSet),
                        crate::retreat::retreat_overlay_mesh.in_set(crate::GameSet),
                        animate_unit_movement,
                        sync_disrupted_visuals,
                        cancel_placement.in_set(crate::GameSet),
                    ),
                ),
            )
            // -- Egui UI panels -----------------------------------------
            .add_systems(
                EguiPrimaryContextPass,
                (
                    unit_picker_ui,
                    crate::melee::melee_reaction_ui,
                    crate::overview::unit_overview_ui,
                ),
            );
    }
}
