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
use omdurman_types::{HexCoord, HexsideRef, Terrain};

use std::collections::{HashSet, VecDeque};

use crate::AppState;
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
    let has_road = coord
        .neighbors()
        .iter()
        .any(|n| game_map.roads.contains(&HexsideRef::new(coord, *n)));
    omdurman_rules::terrain_chart::movement_cost_with_road(tile.terrain, has_road)
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

/// The route each unit has taken this turn, keyed by rules-engine `UnitId`, in
/// order (index 0 is the hex the unit started the turn on, the last entry is
/// where it now stands). Populated at the authoritative move-apply point
/// ([`crate::apply_pending_placement`]) so it captures local, remote, and
/// replayed moves for *both* factions alike -- not just the locally-selected
/// unit. Rendered as directional arrows by [`movement_path_arrows`] and cleared
/// wholesale when the active player changes (end of that player's turn), so the
/// paths persist for the whole turn as a review of what moved where.
#[derive(Resource, Default)]
pub struct UnitPaths(pub std::collections::HashMap<UnitId, Vec<HexCoord>>);

impl UnitPaths {
    /// Record a committed step: start a fresh path at `from` for a unit with no
    /// path yet, then append `to`. `from`/`to` are the unit's pre/post-move
    /// hexes for this step, so a multi-step move accumulates the full route.
    pub fn record_step(&mut self, unit: UnitId, from: HexCoord, to: HexCoord) {
        let path = self.0.entry(unit).or_insert_with(|| vec![from]);
        // Guard against a desync where the stored tail doesn't match this step's
        // origin (e.g. an unobserved teleport): restart the path from `from`.
        if path.last() != Some(&from) {
            path.clear();
            path.push(from);
        }
        path.push(to);
    }
}

/// Marker for one arrow segment of a unit's movement path. The whole arrow set
/// is rebuilt (see [`movement_path_arrows`]) whenever the paths or the hovered
/// hex change, and each segment is spawned with the dim or bright material
/// already chosen for its path's hover state -- so no per-segment path data
/// needs to live on the entity.
#[derive(Component)]
pub(crate) struct MovementPathArrow;

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
            let path = format!("sprites/{}.webp", filename);
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

/// Render the visible picker units belonging to `faction`, grouped by section
/// with a label per section and a wrapped grid of sprite cells. Records a click
/// or drag-start into `clicked_idx` / `drag_idx` (an index into
/// `picker.available`). Shared by both faction categories in the picker.
#[allow(clippy::too_many_arguments)]
fn render_faction_units(
    ui: &mut egui::Ui,
    picker: &UnitPicker,
    state: &PickerState,
    faction: omdurman_rules::Player,
    cell_size: f32,
    sprite_size: f32,
    clicked_idx: &mut Option<usize>,
    drag_idx: &mut Option<usize>,
) {
    let mut current_section = None::<SectionName>;
    for idx in 0..picker.available.len() {
        if !picker.available[idx].visible {
            continue;
        }
        let section_name = picker.available[idx].section_name;
        if crate::unit_profiles::section_owner(section_name) != Some(faction) {
            continue;
        }
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
                    if Some(picker.available[j].section_name) != current_section {
                        break;
                    }
                    if !picker.available[j].visible {
                        continue;
                    }
                    let is_selected =
                        matches!(*state, PickerState::Placing { unit_idx, .. } if unit_idx == j);
                    let unit = &picker.available[j];

                    let (rect, response) = ui.allocate_exact_size(
                        egui::Vec2::new(cell_size, cell_size),
                        egui::Sense::click_and_drag(),
                    );

                    let bg = if is_selected {
                        egui::Color32::from_rgb(120, 80, 30)
                    } else if response.hovered() {
                        egui::Color32::from_rgb(80, 65, 45)
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
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
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
                        *clicked_idx = Some(j);
                    }
                    if response.drag_started() {
                        *drag_idx = Some(j);
                    }
                }
            });
        }
    }
}

pub fn unit_picker_ui(
    mut contexts: EguiContexts,
    mode: Res<State<crate::AppMode>>,
    mut picker: ResMut<UnitPicker>,
    mut state: ResMut<PickerState>,
    images: Res<Assets<Image>>,
    annotations: Option<Res<SpriteAnnotationsResource>>,
    factions: Res<crate::PlayerFactions>,
    net: Res<omdurman_net::NetState>,
    mut was_game_started: Local<bool>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_play() {
        return;
    }
    // Spectators have no units to place -- hide the picker entirely so they
    // can't enter a placement (the click handler also rejects it defensively).
    if factions.local_is_spectator(&net) {
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
                && picker.available.get(unit_idx).is_some_and(|u| !u.visible)
            {
                *state = PickerState::Idle;
            }

            // Once a game starts, default-open the local player's faction and
            // collapse the other. This is a local view choice -- afterwards the
            // user may fold/unfold either heading freely, and nothing is sent
            // over the network.
            let local_faction = factions.local(&net);
            let game_started = !factions.by_peer.is_empty();

            ui.style_mut().spacing.scroll.floating = false;
            egui::ScrollArea::vertical()
                .id_salt("unit_picker_scroll")
                .show(ui, |ui| {
                    use omdurman_rules::Player;
                    // On the transition into a started game, force each category
                    // open/closed once: the local faction open, the foreign one
                    // collapsed. `default_open` alone wouldn't do this, because
                    // egui persists the header's open state from before the game
                    // (when both were open), so we set it explicitly on the edge.
                    let just_started = game_started && !*was_game_started;
                    *was_game_started = game_started;

                    for (faction, heading) in [
                        (Player::Dervish, "Dervish"),
                        (Player::AngloEgyptian, "Anglo-Egyptian"),
                    ] {
                        // Skip a category with no visible units.
                        let any_visible = picker.available.iter().any(|u| {
                            u.visible
                                && crate::unit_profiles::section_owner(u.section_name)
                                    == Some(faction)
                        });
                        if !any_visible {
                            continue;
                        }

                        let header_id = ui.make_persistent_id(("picker_faction", heading));
                        let mut header =
                            egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(),
                                header_id,
                                true,
                            );
                        // Force open/closed at the game-start edge.
                        if just_started {
                            header.set_open(local_faction == Some(faction));
                        }
                        header
                            .show_header(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(heading)
                                        .size(14.0)
                                        .color(egui::Color32::from_gray(210)),
                                );
                            })
                            .body(|ui| {
                                render_faction_units(
                                    ui,
                                    &picker,
                                    &state,
                                    faction,
                                    cell_size,
                                    sprite_size,
                                    &mut clicked_idx,
                                    &mut drag_idx,
                                );
                            });
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
    move_gate: crate::MoveGate,
) {
    let game_state = move_gate.game_state.as_deref();
    let pressed = buttons.just_pressed(MouseButton::Left);
    let released = buttons.just_released(MouseButton::Left);
    if !pressed && !released {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_pointer_input() {
        return;
    }

    // §turn-order: a unit may only be moved on its owner's turn. When a game is
    // live, gate interactive movement on the local player being the rules
    // engine's active player (`handle_idle_click`/move path below). Placement
    // during set-up is not gated. With no game state (editor) there is no gate.
    let may_move = game_state.is_none_or(|gs| move_gate.gate.may_act(gs.0.active_player));

    // In bound multiplayer a player may only pick up their own faction's units;
    // an unbound sandbox / single-seat session (no faction bindings) may move
    // either side. `may_move` already gates *that it's the right turn*.
    let restrict_to = if move_gate.gate.factions.by_peer.is_empty() {
        None
    } else {
        move_gate.gate.factions.local(&move_gate.gate.net)
    };

    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    match *state {
        // Selecting a unit to move is only meaningful on your own turn.
        PickerState::Idle if may_move => {
            handle_idle_click(
                pressed,
                coord,
                &placed_units,
                &mut state,
                &mut commands,
                game_state,
                restrict_to,
            );
        }
        PickerState::Idle => {}
        // A spectator (bound game, no faction) may never place units. The picker
        // panel is hidden for spectators (`unit_picker_ui` early-returns), which
        // is the normal way `Placing` is entered -- this arm is the state-machine
        // backstop for any other path into `Placing` (stale state carried across
        // a role change, a future input source), resetting it rather than
        // committing a placement.
        PickerState::Placing { .. }
            if move_gate
                .gate
                .factions
                .local_is_spectator(&move_gate.gate.net) =>
        {
            *state = PickerState::Idle;
        }
        // During deployment, a unit may only be placed inside its owner's
        // deployment zone (§9.2/§9.3). We gate the *click* on the same engine
        // predicate the deployment overlay is drawn from, so the UI can't commit an
        // out-of-zone `PlaceUnit`. (Placement otherwise isn't phase-gated.)
        PickerState::Placing { unit_idx, .. }
            if game_state.is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::Setup))
                && !deploy_hex_allowed(game_state, &picker, unit_idx, coord) =>
        {
            // Off-zone: ignore the click, keep the unit in hand.
        }
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
                // The path is recorded authoritatively when the move is applied
                // (`apply_pending_placement`), covering local, remote, and
                // replayed moves alike -- nothing to track here.
                action_writer.write(events::LocalAction { event });
            }
            if matches!(*state, PickerState::Idle) {
                commands.entity(source).remove::<Selected>();
            }
        }
    }
}

/// Idle: a left-press on a placed unit selects it. `restrict_to`, when `Some`,
/// is the only faction whose units may be picked up -- set in bound multiplayer
/// so a player can't grab an enemy counter on their own turn. `None` (unbound
/// sandbox / single-seat) allows selecting either side, so solo play/testing
/// can drive both factions.
fn handle_idle_click(
    pressed: bool,
    coord: HexCoord,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    state: &mut PickerState,
    commands: &mut Commands,
    game_state: Option<&crate::GameStateResource>,
    restrict_to: Option<omdurman_rules::Player>,
) {
    if !pressed {
        return;
    }
    if let Some((entity, placed)) = placed_units.iter().find(|(_, u)| u.coord == coord) {
        let remaining_mp = if let Some(uid) = placed.unit_id
            && let Some(gs) = game_state
            && let Some(unit) = gs.0.find_unit(uid)
        {
            if let Some(faction) = restrict_to
                && unit.profile.identity.owner() != faction
            {
                return; // not your unit -- ignore the click
            }
            // Remaining allowance = full allowance minus what the unit has
            // already spent this turn (§5.11/§5.12), so re-selecting a unit that
            // has partly moved shows only its leftover movement -- not a fresh
            // full budget. The engine caps cumulatively regardless, but the
            // overlay should reflect the truth.
            match unit.profile.movement {
                omdurman_rules::UnitMovement::Land(a) => {
                    (a.value() as i16 - gs.0.mp_spent(uid)).max(0)
                }
                _ => 99,
            }
        } else {
            99
        };
        commands.entity(entity).insert(Selected);
        *state = PickerState::Selected {
            source: entity,
            start_coord: coord,
            remaining_mp,
        };
    }
}

/// Whether the picker unit at `unit_idx` may be deployed on `coord` during
/// setup: `coord` must lie in that unit's owner's deployment zone (§9.2/§9.3).
/// Returns `true` when there is no game state or the owner can't be resolved, so
/// non-setup / sandbox placement is never blocked by this gate.
fn deploy_hex_allowed(
    game_state: Option<&crate::GameStateResource>,
    picker: &UnitPicker,
    unit_idx: usize,
    coord: HexCoord,
) -> bool {
    let Some(gs) = game_state else { return true };
    let Some(unit) = picker.available.get(unit_idx) else {
        return true;
    };
    match crate::unit_profiles::section_owner(unit.section_name) {
        Some(owner) => gs.0.in_deployment_zone(owner, coord),
        None => true,
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

        // §9.346: a Dervish unit may move *onto* the Palace even though GORDON
        // occupies it -- "passing through or occupying the palace hex" is how he
        // is eliminated. So the normal "can't enter an occupied hex" gate is
        // waived for the Palace (the faction gate upstream already ensures only
        // the side whose turn it is can reach here; the engine resolves GORDON's
        // death for a Dervish occupant).
        let dest_is_palace = self.game_map.hexes.get(&coord).is_some_and(|h| {
            h.name
                .as_deref()
                .and_then(omdurman_types::Location::from_tile_name)
                == Some(omdurman_types::Location::Palace)
        });
        let target_occupied = !dest_is_palace && placed_units.iter().any(|(_, u)| u.coord == coord);
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
            // Interactive movement commits one adjacent hex per click, so the
            // route the engine costs/classifies is the single step to `to`.
            // (The reachable-range overlay only previews multi-turn reach; the
            // player still steps hex-by-hex, each step validated on its own.)
            path: vec![to],
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
    // Rebuild only when the selection/remaining-MP key actually differs from
    // the one we last built for. We key on the *value* rather than on
    // `Res::is_changed()`: the click handler takes `ResMut<PickerState>` every
    // frame but only writes it on click frames, yet a stray mutable deref
    // elsewhere could still flip the change flag and force a needless rebuild.
    let PickerState::Selected {
        source,
        remaining_mp,
        ..
    } = *state
    else {
        // No selection: clear any leftover rings and reset the cache.
        for e in &existing_green {
            commands.entity(e).despawn();
        }
        for e in &existing_gray {
            commands.entity(e).despawn();
        }
        *last_key = None;
        return;
    };

    // Nothing changed since we last built the overlay: leave the existing
    // rings in place. (Despawning unconditionally above and then bailing here
    // would erase the overlay one frame after spawning it.)
    if *last_key == Some((source, remaining_mp)) {
        return;
    }

    // Selection or remaining MP changed: rebuild from scratch. Resolve the unit
    // *before* despawning the old rings or updating the cache: if the entity
    // isn't queryable this frame, bail without touching either -- so the cache
    // never advances to a key whose rings we didn't actually spawn (which is
    // what stranded the overlay after a single frame).
    let Ok((_, placed)) = placed_units.get(source) else {
        return;
    };

    for e in &existing_green {
        commands.entity(e).despawn();
    }
    for e in &existing_gray {
        commands.entity(e).despawn();
    }

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

// -- Deployment-zone overlay (Setup phase): brown hex outlines ------------------

#[derive(Component)]
pub(crate) struct DeploymentZoneRing;

/// During [`omdurman_rules::Phase::Setup`], outline the hexes where the local
/// player may deploy (§9.2/§9.3), so setup is legible. Highlights the local
/// faction's zone (or, in an unbound sandbox, the active player's). Cleared
/// automatically once play leaves Setup. Rebuilt only when the phase/faction key
/// changes, to avoid per-frame entity churn (cf. `movement_overlay_mesh`).
pub fn deployment_zone_overlay_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    game_state: Option<Res<crate::GameStateResource>>,
    factions: Res<crate::PlayerFactions>,
    net: Res<omdurman_net::NetState>,
    existing: Query<Entity, With<DeploymentZoneRing>>,
    mut last_key: Local<Option<omdurman_rules::Player>>,
) {
    let in_setup = game_state
        .as_deref()
        .is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::Setup));
    let Some(gs) = game_state.as_deref().filter(|_| in_setup) else {
        // Not in setup: clear any leftover rings and reset the cache.
        if last_key.is_some() {
            for e in &existing {
                commands.entity(e).despawn();
            }
            *last_key = None;
        }
        return;
    };

    // Whose zone to show: the local faction, or the active player in an unbound
    // sandbox (no faction binding).
    let who = factions.local(&net).unwrap_or(gs.0.active_player);
    if *last_key == Some(who) {
        return; // unchanged -- leave the rings in place
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    *last_key = Some(who);

    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let size = overlay.params.hex_size;
    for coord in game_map.hexes.keys() {
        if gs.0.in_deployment_zone(who, *coord) {
            let pos = hex_world_pos(*coord, origin, &overlay.params);
            commands.spawn((
                DeploymentZoneRing,
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(assets.brown.clone()),
                Transform::from_xyz(pos.x, 1.4, pos.z).with_scale(Vec3::splat(size)),
                Visibility::Visible,
            ));
        }
    }
}

/// Draw every unit's movement path this turn as directional arrows (start ->
/// step -> ... -> current hex), so the route each unit took is visible until the
/// turn ends. The path whose start, end, or any crossed hex is under the cursor
/// is drawn bright; all others are drawn dim.
///
/// Rebuilt only when the paths or the hovered hex change (not every frame): the
/// arrow entities otherwise churn, and -- as with the reachable-range overlay --
/// unconditional per-frame despawn/respawn risks a one-frame flash.
pub fn movement_path_arrows(
    mut commands: Commands,
    assets: Res<crate::render::MovementArrowAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    paths: Res<UnitPaths>,
    hovered: Res<crate::HoveredHex>,
    existing: Query<Entity, With<MovementPathArrow>>,
) {
    if !paths.is_changed() && !hovered.is_changed() {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }

    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let size = overlay.params.hex_size;

    for path in paths.0.values() {
        // A path needs at least a start and one step to draw an arrow.
        if path.len() < 2 {
            continue;
        }
        // Bright if the cursor is on any hex of this path (start/end included).
        let hovered_here = hovered.0.is_some_and(|h| path.contains(&h));
        let material = if hovered_here {
            assets.bright.clone()
        } else {
            assets.dim.clone()
        };

        for pair in path.windows(2) {
            let from = hex_world_pos(pair[0], origin, &overlay.params);
            let to = hex_world_pos(pair[1], origin, &overlay.params);
            let delta = Vec3::new(to.x - from.x, 0.0, to.z - from.z);
            let len = delta.length();
            if len < f32::EPSILON {
                continue;
            }
            let dir = delta / len;
            // Shorten slightly at both ends so consecutive arrows read as
            // separate hops and the head doesn't bury under the next counter.
            let inset = size * 0.18;
            let draw_len = (len - inset).max(len * 0.4);
            let tail = from + dir * ((len - draw_len) * 0.5);
            // The unit arrow points along +Z; rotate that onto the heading and
            // scale length (Z) to the segment, width (X) to a fraction of a hex.
            commands.spawn((
                MovementPathArrow,
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(tail.x, 1.45, tail.z)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                    .with_scale(Vec3::new(size * 0.5, 1.0, draw_len)),
                Visibility::Visible,
            ));
        }
    }
}

// -- Stacked-unit layout: offset co-located counters, expand on hover ----------

/// Lay out the counters that share a hex so they don't overlap (or z/y-fight):
/// each is nudged by a small per-index offset in xz and a tiny per-index step in
/// y (so the quads never sit in the same plane). When the hovered hex holds more
/// than one counter, that hex's units fan out to ~2x the spread so all of them
/// are readable. Transforms are eased toward the target each frame, giving the
/// expand/collapse a smooth animation. Units currently sliding between hexes
/// (`MovementAnimation`) are left to `animate_unit_movement`.
pub fn layout_stacked_units(
    time: Res<Time>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    hovered: Res<crate::HoveredHex>,
    mut units: Query<(Entity, &PlacedUnit, &mut Transform), Without<MovementAnimation>>,
) {
    use std::collections::HashMap;

    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let size = overlay.params.hex_size;

    // Group the (non-animating) counters by hex, in a stable order (entity id),
    // so each unit's slot index is deterministic frame to frame.
    let mut by_hex: HashMap<HexCoord, Vec<Entity>> = HashMap::new();
    for (entity, placed, _) in &units {
        by_hex.entry(placed.coord).or_default().push(entity);
    }
    for ents in by_hex.values_mut() {
        ents.sort_by_key(|e| e.to_bits());
    }

    let lerp = (time.delta_secs() * 12.0).min(1.0);

    for (entity, placed, mut transform) in &mut units {
        let stack = &by_hex[&placed.coord];
        let n = stack.len();
        let idx = stack.iter().position(|e| *e == entity).unwrap_or(0);
        let center = hex_world_pos(placed.coord, origin, &overlay.params);

        // Per-index offset. A single unit sits centred; a stack fans along a
        // short diagonal so each counter peeks out from under the one above.
        // Hovering a multi-unit hex doubles the spread for readability.
        let expanded = n > 1 && hovered.0 == Some(placed.coord);
        let spread = if expanded { 0.34 } else { 0.14 } * size;
        let off = if n > 1 {
            // Centre the fan around the hex: index 0..n-1 -> -(n-1)/2 .. +(n-1)/2.
            let k = idx as f32 - (n as f32 - 1.0) / 2.0;
            Vec3::new(k * spread, 0.0, k * spread * 0.6)
        } else {
            Vec3::ZERO
        };
        // A tiny per-index height step keeps the quads out of the same plane
        // (no y-fighting); the hovered stack lifts a hair more.
        let y_step = if expanded { 0.12 } else { 0.04 };
        let target = Vec3::new(
            center.x + off.x,
            UNIT_HEIGHT + idx as f32 * y_step,
            center.z + off.z,
        );
        transform.translation = transform.translation.lerp(target, lerp);
    }
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

/// Despawn the sprite of any counter the rules engine has eliminated. A placed
/// counter that carries a rules `UnitId` no longer present in `GameState.units`
/// has been removed by combat (fire/melee, §6/§7), desertion (§8.2), or GORDON's
/// fall (§9.346); its sprite must leave the board too. Counters not yet bound to
/// a rules id (mid-placement) are left alone.
pub fn sync_eliminated_visuals(
    game_state: Option<Res<crate::GameStateResource>>,
    mut commands: Commands,
    query: Query<(Entity, &PlacedUnit)>,
) {
    let Some(game_state) = game_state else {
        return;
    };
    for (entity, placed) in query.iter() {
        let Some(uid) = placed.unit_id else {
            continue;
        };
        if game_state.0.find_unit(uid).is_none() {
            commands.entity(entity).despawn();
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

/// Despawn every gameplay overlay marker. Registered on exit from each map mode
/// so leaving the board (to the lobby, an editor, or any tool) leaves no
/// stranded movement/fire/melee/retreat/trail/entry/preview rings.
fn clear_gameplay_overlays(
    mut commands: Commands,
    rings: Query<
        Entity,
        Or<(
            With<MovementHexRing>,
            With<MovementRangeRing>,
            With<MovementPathArrow>,
            With<DeploymentZoneRing>,
            With<PreviewHexRing>,
            With<crate::fire::FireTargetRing>,
            With<crate::melee::MeleeTargetRing>,
            With<crate::retreat::RetreatTargetRing>,
            With<crate::fok_entry::FokEntryRing>,
        )>,
    >,
) {
    for e in &rings {
        commands.entity(e).despawn();
    }
}

/// Clear every unit's movement path when the active player changes -- i.e. at
/// the end of a player's turn -- so the arrows show the moves made *this* turn
/// and reset for the next. The turn lives in the rules engine
/// (`GameState.active_player`); we watch it via a `Local` snapshot rather than a
/// dedicated event, so the reset also fires correctly on replay and snapshot
/// convergence, where no local "end turn" click occurs.
pub fn clear_paths_on_turn_change(
    game_state: Option<Res<crate::GameStateResource>>,
    mut paths: ResMut<UnitPaths>,
    mut last_active: Local<Option<omdurman_rules::Player>>,
) {
    let Some(gs) = game_state else { return };
    let active = gs.0.active_player;
    if *last_active != Some(active) {
        if last_active.is_some() && !paths.0.is_empty() {
            paths.0.clear();
        }
        *last_active = Some(active);
    }
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
            .insert_resource(UnitPaths::default())
            // -- Mode-exit cleanup: leaving a play view (or the game itself)
            //    despawns all gameplay overlay rings, so none linger over the
            //    editor / lobby (the per-frame overlay systems only clean up
            //    while running).
            .add_systems(OnExit(crate::AppMode::Game), clear_gameplay_overlays)
            .add_systems(OnExit(crate::AppMode::Sandbox), clear_gameplay_overlays)
            .add_systems(OnExit(AppState::InGame), clear_gameplay_overlays)
            // -- Startup ------------------------------------------------
            .add_systems(
                Startup,
                (
                    spawn_picker_assets,
                    crate::render::spawn_movement_arrow_assets,
                ),
            )
            // -- Update: gameplay (GameSet) -----------------------------
            .add_systems(
                Update,
                (
                    crate::dice::despawn_dice,
                    crate::apply_pending_placement.after(crate::net_socket::handle_socket),
                    (
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
                        clear_paths_on_turn_change,
                        movement_path_arrows
                            .in_set(crate::GameSet)
                            .after(clear_paths_on_turn_change)
                            .after(crate::apply_pending_placement),
                        deployment_zone_overlay_mesh.in_set(crate::GameSet),
                        crate::fok_entry::fok_entry_overlay_mesh.in_set(crate::GameSet),
                        animate_unit_movement,
                        layout_stacked_units.after(animate_unit_movement),
                        sync_disrupted_visuals,
                        sync_eliminated_visuals,
                        cancel_placement.in_set(crate::GameSet),
                    ),
                ),
            )
            // -- Egui UI panels -----------------------------------------
            // These in-game side panels run only while actually in a game, so
            // they don't linger over the lobby (the EditorMode can still be a
            // map mode in the lobby, which is an AppState, not a mode).
            .add_systems(
                EguiPrimaryContextPass,
                (
                    unit_picker_ui,
                    crate::melee::melee_reaction_ui,
                    crate::overview::unit_overview_ui,
                )
                    .run_if(in_state(crate::AppState::InGame)),
            );
    }
}
