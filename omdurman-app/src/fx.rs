//! Transient board effects: brief, self-removing visuals that show the
//! player *where* something just happened.
//!
//! Every state change reaches the board in a single frame: a counter vanishes,
//! an opponent's stack turns up somewhere off-screen, a shot is resolved in a
//! card at the window's edge. The facts are all in the event feed, but not at
//! the place on the map they concern. These effects add that place:
//!
//! * an attention ripple where the opponent (or the AI) moved, entered or
//!   attacked, plus an edge-of-screen pointer while that hex is out of view
//!   ([`offscreen_pointers_ui`]); clicking a pointer pans there, and
//!   [`MotionSettings::follow_opponent`] pans there by itself;
//! * a shot trace from the firers to the target, a burst where the shot
//!   landed (and a hop to where a howitzer shell scattered), a clash marker
//!   between melee opponents;
//! * a fading ghost where an eliminated counter stood;
//! * a red flash on a hex an order could not use;
//! * a marching trail along a committed move until its echo arrives
//!   ([`pending_order_trail`]), so a sent order never looks ignored;
//! * rings on the hexes of the combat card under the pointer
//!   ([`CardFocus`]).
//!
//! All of it is presentation only: the engine state is final before an effect
//! starts, and nothing here gates input, submissions or the apply path. The
//! effects are driven from *live* changes only -- [`LiveApplied`] (filled on
//! the sequenced echo) and the engine's observations (discarded by a
//! rebuild) -- so a late-join install or a timeline jump shows none; a jump
//! additionally raises [`SnapSprites`], which drops whatever was still on
//! screen. Every effect is finite and asks for frames only while it lives
//! (see [`crate::activity`]). [`MotionSettings`] tones it all down (Reduced:
//! fades instead of motion) or off.

use std::collections::{HashMap, HashSet};

use bevy::asset::RenderAssetUsages;
use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::hex_world_pos;
use omdurman_net::GameEvent;
use omdurman_rules::UnitId;
use omdurman_rules::effects::{GameState, Observation};
use omdurman_types::{HexCoord, Player};

use crate::camera::{RtsCamera, RtsCameraState};

// -- Settings -----------------------------------------------------------------

/// How much the board animates.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MotionLevel {
    /// Every effect, with motion: glides, ripples, spins, camera follow.
    #[default]
    Full,
    /// Effects without motion: counters jump, markers fade in place.
    Reduced,
    /// No transient effects at all: the board changes in one frame.
    Off,
}

impl MotionLevel {
    pub const ALL: [MotionLevel; 3] = [MotionLevel::Full, MotionLevel::Reduced, MotionLevel::Off];

    pub fn label(self) -> &'static str {
        match self {
            MotionLevel::Full => "Full",
            MotionLevel::Reduced => "Reduced",
            MotionLevel::Off => "Off",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            MotionLevel::Full => "Counters glide, markers ripple and spin",
            MotionLevel::Reduced => "Counters jump; markers fade in place, nothing moves",
            MotionLevel::Off => "No effects: the board changes at once",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|level| level.label().eq_ignore_ascii_case(name.trim()))
    }
}

/// The local player's animation preferences (not networked: purely how this
/// window draws the shared state).
#[derive(Resource, Clone, Copy, Debug)]
pub struct MotionSettings {
    pub level: MotionLevel,
    /// Pan the camera to an opponent's action that happens out of view.
    pub follow_opponent: bool,
}

impl Default for MotionSettings {
    fn default() -> Self {
        Self {
            level: initial_level(),
            follow_opponent: false,
        }
    }
}

impl MotionSettings {
    /// Whether transient effects are drawn at all.
    pub fn effects(&self) -> bool {
        self.level != MotionLevel::Off
    }

    /// Whether things may *move* (glide, grow, spin, pan), rather than only
    /// appear and fade.
    pub fn motion(&self) -> bool {
        self.level == MotionLevel::Full
    }
}

/// `OMDURMAN_MOTION=full|reduced|off` (headless runs, measuring), else the
/// browser's reduced-motion preference on the web, else full.
fn initial_level() -> MotionLevel {
    if let Some(level) = std::env::var("OMDURMAN_MOTION")
        .ok()
        .and_then(|v| MotionLevel::from_name(&v))
    {
        return level;
    }
    if prefers_reduced_motion() {
        MotionLevel::Reduced
    } else {
        MotionLevel::Full
    }
}

#[cfg(target_arch = "wasm32")]
fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok().flatten())
        .is_some_and(|query| query.matches())
}

#[cfg(not(target_arch = "wasm32"))]
fn prefers_reduced_motion() -> bool {
    false
}

/// The toolbar's Motion menu: the level, and the follow camera.
pub fn motion_menu(ui: &mut egui::Ui, settings: &mut MotionSettings) {
    ui.set_min_width(220.0);
    for level in MotionLevel::ALL {
        ui.radio_value(&mut settings.level, level, level.label())
            .on_hover_text(level.hint());
    }
    ui.separator();
    ui.add_enabled(
        settings.motion(),
        egui::Checkbox::new(&mut settings.follow_opponent, "Follow the opponent"),
    )
    .on_hover_text("Pan to what the opponent does out of view")
    .on_disabled_hover_text("Needs full motion");
}

// -- Live changes ---------------------------------------------------------------

/// The engine state *jumped* to a new position instead of playing out to it
/// (a history install, a timeline jump): the next sprite reconcile snaps every
/// counter instead of gliding, spinning or ghosting it, and the effects still
/// on screen are dropped. Inserted by the rebuilding system, removed by
/// [`crate::picker::reconcile_unit_sprites`].
#[derive(Resource, Default)]
pub struct SnapSprites;

/// What one live (sequenced-echo) event did to the counters: who moved where,
/// who entered. Filled by the receive path, never by a rebuild, so replays and
/// installs raise no effects.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LiveChange {
    /// Units now on another hex: `(unit, new hex)`.
    pub moved: Vec<(UnitId, HexCoord)>,
    /// Units new on the board: `(unit, hex)`.
    pub placed: Vec<(UnitId, HexCoord)>,
    /// The change happened during set-up (deployment is no news).
    pub setup: bool,
}

impl LiveChange {
    /// Positions of every unit, to diff against after an apply.
    pub fn snapshot(gs: &GameState) -> HashMap<UnitId, HexCoord> {
        gs.units.iter().map(|u| (u.id, u.position)).collect()
    }

    /// The change from `before` (a [`snapshot`](Self::snapshot)) to `after`.
    pub fn between(before: &HashMap<UnitId, HexCoord>, after: &GameState, setup: bool) -> Self {
        let mut change = LiveChange {
            setup,
            ..Default::default()
        };
        for unit in &after.units {
            match before.get(&unit.id) {
                Some(&was) if was != unit.position => change.moved.push((unit.id, unit.position)),
                Some(_) => {}
                None => change.placed.push((unit.id, unit.position)),
            }
        }
        change
    }

    pub fn is_empty(&self) -> bool {
        self.moved.is_empty() && self.placed.is_empty()
    }
}

/// Live changes awaiting [`announce_live_changes`].
#[derive(Resource, Default)]
pub struct LiveApplied(pub Vec<LiveChange>);

// -- Requests -------------------------------------------------------------------

/// Ask for a transient effect. Written by whoever knows something happened;
/// [`spawn_fx`] owns the assets and the board geometry.
#[derive(Message, Clone, Debug)]
pub enum FxRequest {
    /// Draw the eye to a hex: two expanding, fading rings.
    Attention { hex: HexCoord },
    /// A shot from one hex at another (`scatter`: a howitzer shell's hop
    /// from the aimed hex to where it landed, drawn smaller).
    Shot {
        from: HexCoord,
        to: HexCoord,
        scatter: bool,
    },
    /// Where a shot landed.
    Impact { hex: HexCoord },
    /// Melee: a clash marker between the two hexes.
    Clash {
        attacker: HexCoord,
        defender: HexCoord,
    },
    /// A hex an order could not use.
    Refused { hex: HexCoord },
    /// A counter that left the board fades and sinks where it stood.
    Ghost {
        transform: Transform,
        mesh: Handle<Mesh>,
        texture: Option<Handle<Image>>,
        tint: Color,
    },
}

// -- Assets ---------------------------------------------------------------------

/// Meshes shared by the effects (materials are per effect: each fades on its
/// own).
#[derive(Resource)]
pub struct FxAssets {
    /// A broad hex ring (outer radius 1, in the XZ plane).
    ring: Handle<Mesh>,
    /// A filled hex (radius 1, in the XZ plane).
    fill: Handle<Mesh>,
    /// The dots of a pending order's trail.
    trail: Handle<StandardMaterial>,
    /// Rings on the hexes of the hovered combat card.
    focus: Handle<StandardMaterial>,
}

mod tint {
    use bevy::prelude::Color;
    /// Attention ripple: warm white, readable on the sepia map and at night.
    pub const ATTENTION: Color = Color::srgba(1.0, 0.92, 0.55, 0.95);
    /// Shot trace.
    pub const SHOT: Color = Color::srgba(0.95, 0.2, 0.08, 0.85);
    /// Shell burst / impact ring.
    pub const IMPACT: Color = Color::srgba(1.0, 0.45, 0.05, 0.95);
    /// Melee clash.
    pub const CLASH: Color = Color::srgba(0.95, 0.1, 0.08, 0.85);
    /// Refused order flash.
    pub const REFUSED: Color = Color::srgba(0.95, 0.1, 0.05, 0.5);
    /// Pending order trail.
    pub const TRAIL: Color = Color::srgba(0.98, 0.95, 0.8, 0.85);
    /// Hovered combat card's hexes.
    pub const FOCUS: Color = Color::srgba(1.0, 0.92, 0.55, 0.9);
}

fn effect_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    }
}

/// The corners of a flat hex of radius 1 in the XZ plane, in the hex-ring
/// mesh's orientation.
fn hex_corner(i: usize, radius: f32) -> Vec3 {
    let a = std::f32::consts::FRAC_PI_6 + i as f32 * std::f32::consts::PI / 3.0;
    Vec3::new(radius * a.cos(), 0.0, radius * a.sin())
}

fn flat_mesh(positions: Vec<Vec3>, indices: Vec<u32>) -> Mesh {
    let n = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::Y; n])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![Vec2::ZERO; n])
}

/// A hex ring of outer radius 1 and the given inner radius.
fn broad_ring_mesh(inner: f32) -> Mesh {
    let mut positions = Vec::with_capacity(12);
    let mut indices = Vec::with_capacity(36);
    for i in 0..6 {
        positions.push(hex_corner(i, 1.0));
        positions.push(hex_corner(i, inner));
    }
    for i in 0..6u32 {
        let (o0, i0) = (2 * i, 2 * i + 1);
        let (o1, i1) = ((2 * i + 2) % 12, (2 * i + 3) % 12);
        indices.extend_from_slice(&[o0, o1, i0, o1, i1, i0]);
    }
    flat_mesh(positions, indices)
}

/// A filled hex of radius 1.
fn hex_fill_mesh() -> Mesh {
    let mut positions = vec![Vec3::ZERO];
    positions.extend((0..6).map(|i| hex_corner(i, 1.0)));
    let indices = (0..6u32)
        .flat_map(|i| [0, 1 + i, 1 + (i + 1) % 6])
        .collect();
    flat_mesh(positions, indices)
}

fn spawn_fx_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(FxAssets {
        ring: meshes.add(broad_ring_mesh(0.86)),
        fill: meshes.add(hex_fill_mesh()),
        trail: materials.add(effect_material(tint::TRAIL)),
        focus: materials.add(effect_material(tint::FOCUS)),
    });
}

// -- Transients -----------------------------------------------------------------

/// How a transient effect develops over its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curve {
    /// Expands from under the hex outline to well past it while fading.
    Pulse,
    /// Pops in, holds, shrinks away (shots, clashes).
    Pop,
    /// Shows at once, then fades (refusals).
    Flash,
    /// Fades while sinking and shrinking (an eliminated counter).
    Sink,
}

/// A frame of a transient: scale and opacity factors, and how far (0..=1 of
/// the sink depth) it has dropped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub scale: f32,
    pub alpha: f32,
    pub drop: f32,
}

/// The look of a `curve` at life fraction `p` (`0..=1`). Without `motion` the
/// same effect only fades: size and height stay put.
pub fn sample(curve: Curve, p: f32, motion: bool) -> Look {
    let p = p.clamp(0.0, 1.0);
    let fade_from = |start: f32| {
        if p <= start {
            1.0
        } else {
            1.0 - (p - start) / (1.0 - start)
        }
    };
    let still = |alpha: f32| Look {
        scale: 1.0,
        alpha,
        drop: 0.0,
    };
    match curve {
        Curve::Pulse if motion => {
            let out = 1.0 - (1.0 - p) * (1.0 - p);
            Look {
                scale: 0.55 + 0.95 * out,
                alpha: (1.0 - p) * (1.0 - p),
                drop: 0.0,
            }
        }
        Curve::Pulse => still(fade_from(0.5)),
        Curve::Pop if motion => {
            // Fast pop-in (~7% of the life), hold, shrink over the last 35%.
            let grow = (p / 0.07).min(1.0);
            Look {
                scale: grow * fade_from(0.65),
                alpha: 1.0,
                drop: 0.0,
            }
        }
        Curve::Pop => still(fade_from(0.65)),
        Curve::Flash => still(fade_from(0.25)),
        Curve::Sink if motion => Look {
            scale: 1.0 - 0.25 * p,
            alpha: 1.0 - p * p,
            drop: p,
        },
        Curve::Sink => still(1.0 - p),
    }
}

/// A self-removing effect entity. Owns its material (fades on its own).
#[derive(Component)]
pub struct Transient {
    /// Seconds lived; negative while delayed (hidden).
    age: f32,
    ttl: f32,
    curve: Curve,
    base_scale: Vec3,
    base_y: f32,
    base_alpha: f32,
    /// The AI waits for it to finish before its next action (a shot, a
    /// clash, a ghost), so its turn reads one action at a time.
    pub blocking: bool,
}

/// How deep (world units) a ghost sinks: from counter height to just above
/// the board.
const SINK_DEPTH: f32 = 0.9;
/// Height of the ripples and markers: above stacked counters.
const FX_HEIGHT: f32 = 2.2;

struct TransientSpec {
    mesh: Handle<Mesh>,
    material: StandardMaterial,
    transform: Transform,
    curve: Curve,
    ttl: f32,
    delay: f32,
    blocking: bool,
}

fn spawn_transient(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    spec: TransientSpec,
) {
    let base_alpha = spec.material.base_color.alpha();
    let look = sample(spec.curve, 0.0, true);
    let mut material = spec.material;
    if spec.delay > 0.0 {
        material.base_color.set_alpha(0.0);
    }
    commands.spawn((
        Transient {
            age: -spec.delay,
            ttl: spec.ttl,
            curve: spec.curve,
            base_scale: spec.transform.scale,
            base_y: spec.transform.translation.y,
            base_alpha,
            blocking: spec.blocking,
        },
        Mesh3d(spec.mesh),
        MeshMaterial3d(materials.add(material)),
        Transform {
            scale: spec.transform.scale * look.scale.max(0.001),
            ..spec.transform
        },
        Visibility::Visible,
    ));
}

/// Turn [`FxRequest`]s into transient entities at this board's geometry.
#[allow(clippy::too_many_arguments)]
pub fn spawn_fx(
    mut requests: MessageReader<FxRequest>,
    settings: Res<MotionSettings>,
    assets: Option<Res<FxAssets>>,
    arrows: Option<Res<crate::render::MovementArrowAssets>>,
    markers: Option<Res<crate::timeline::SpectatorMarkerAssets>>,
    board: crate::BoardGeometry,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if !settings.effects() {
        requests.clear();
        return;
    }
    let (Some(assets), Some(arrows), Some(markers)) = (assets, arrows, markers) else {
        requests.clear();
        return;
    };
    let origin = board.layout.adjusted_origin(&board.overlay.params);
    let size = board.overlay.params.hex_size;
    let world = |hex: HexCoord| hex_world_pos(hex, origin, &board.overlay.params);
    let at = |hex: HexCoord, height: f32, scale: f32| {
        let p = world(hex);
        Transform::from_xyz(p.x, height, p.z).with_scale(Vec3::splat(size * scale))
    };

    for request in requests.read() {
        let mut spawn = |spec| spawn_transient(&mut commands, &mut materials, spec);
        match request {
            FxRequest::Attention { hex } => {
                // Two rings, a beat apart: a ripple, not a blink.
                for delay in [0.0, 0.3] {
                    spawn(TransientSpec {
                        mesh: assets.ring.clone(),
                        material: effect_material(tint::ATTENTION),
                        transform: at(*hex, FX_HEIGHT, 1.0),
                        curve: Curve::Pulse,
                        ttl: 1.2,
                        delay,
                        blocking: false,
                    });
                }
            }
            FxRequest::Impact { hex } => {
                spawn(TransientSpec {
                    mesh: assets.ring.clone(),
                    material: effect_material(tint::IMPACT),
                    transform: at(*hex, FX_HEIGHT, 0.9),
                    curve: Curve::Pulse,
                    ttl: 0.9,
                    delay: 0.15,
                    blocking: true,
                });
                spawn(TransientSpec {
                    mesh: assets.fill.clone(),
                    material: effect_material(tint::IMPACT.with_alpha(0.3)),
                    transform: at(*hex, FX_HEIGHT - 0.1, 0.95),
                    curve: Curve::Flash,
                    ttl: 0.7,
                    delay: 0.15,
                    blocking: false,
                });
            }
            FxRequest::Shot { from, to, scatter } => {
                let (a, b) = (world(*from), world(*to));
                let delta = Vec3::new(b.x - a.x, 0.0, b.z - a.z);
                let len = delta.length();
                if len < f32::EPSILON {
                    continue;
                }
                let dir = delta / len;
                // As the live fire arrow: inset from both ends, the arrow
                // mesh points +Z, rotated onto the heading.
                let inset = size * 0.18;
                let draw_len = (len - inset).max(len * 0.4);
                let tail = a + dir * ((len - draw_len) * 0.5);
                let width = if *scatter { 0.3 } else { 0.5 };
                spawn(TransientSpec {
                    mesh: arrows.mesh.clone(),
                    material: effect_material(tint::SHOT),
                    transform: Transform::from_xyz(tail.x, FX_HEIGHT + 0.1, tail.z)
                        .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                        .with_scale(Vec3::new(size * width, 1.0, draw_len)),
                    curve: Curve::Pop,
                    ttl: 1.0,
                    delay: if *scatter { 0.35 } else { 0.0 },
                    blocking: true,
                });
            }
            FxRequest::Clash { attacker, defender } => {
                let (a, d) = (world(*attacker), world(*defender));
                let delta = Vec3::new(d.x - a.x, 0.0, d.z - a.z);
                let len = delta.length();
                if len < f32::EPSILON {
                    continue;
                }
                let dir = delta / len;
                let mid = (a + d) / 2.0 - dir * (size * 0.1);
                spawn(TransientSpec {
                    mesh: markers.melee_triangle.clone(),
                    material: effect_material(tint::CLASH),
                    transform: Transform::from_xyz(mid.x, FX_HEIGHT + 0.1, mid.z)
                        .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                        .with_scale(Vec3::splat(size * 0.45)),
                    curve: Curve::Pop,
                    ttl: 1.1,
                    delay: 0.0,
                    blocking: true,
                });
            }
            FxRequest::Refused { hex } => {
                spawn(TransientSpec {
                    mesh: assets.fill.clone(),
                    material: effect_material(tint::REFUSED),
                    transform: at(*hex, FX_HEIGHT, 0.95),
                    curve: Curve::Flash,
                    ttl: 0.45,
                    delay: 0.0,
                    blocking: false,
                });
            }
            FxRequest::Ghost {
                transform,
                mesh,
                texture,
                tint,
            } => {
                spawn(TransientSpec {
                    mesh: mesh.clone(),
                    material: StandardMaterial {
                        base_color: *tint,
                        base_color_texture: texture.clone(),
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    },
                    transform: *transform,
                    curve: Curve::Sink,
                    ttl: 0.7,
                    delay: 0.0,
                    blocking: true,
                });
            }
        }
    }
}

/// Play every transient forward; despawn it at the end of its life.
pub fn animate_transients(
    time: Res<Time>,
    settings: Res<MotionSettings>,
    mut query: Query<(
        Entity,
        &mut Transient,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut activity: ResMut<crate::activity::Activity>,
) {
    let dt = time.delta_secs();
    let motion = settings.motion();
    for (entity, mut fx, mut transform, material) in &mut query {
        activity.keep_running();
        fx.age += dt;
        if fx.age < 0.0 {
            continue; // still delayed (spawned transparent)
        }
        let p = fx.age / fx.ttl;
        if p >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let look = sample(fx.curve, p, motion);
        transform.scale = fx.base_scale * look.scale.max(0.001);
        transform.translation.y = fx.base_y - look.drop * SINK_DEPTH;
        if let Some(mut mat) = materials.get_mut(&material.0) {
            mat.base_color.set_alpha(fx.base_alpha * look.alpha);
        }
    }
}

/// Whether the board is still playing something the AI should let finish
/// before its next action: a counter gliding, a shot, a clash, a ghost.
#[derive(bevy::ecs::system::SystemParam)]
pub struct BoardBusy<'w, 's> {
    glides: Query<'w, 's, (), With<crate::picker::MovementAnimation>>,
    transients: Query<'w, 's, &'static Transient>,
}

impl BoardBusy<'_, '_> {
    pub fn busy(&self) -> bool {
        !self.glides.is_empty() || self.transients.iter().any(|t| t.blocking)
    }
}

/// Drop every transient and pointer: the state jumped (rebuild) or the board
/// is no longer shown.
fn clear_effects(
    commands: &mut Commands,
    transients: &Query<Entity, With<Transient>>,
    sightings: &mut Sightings,
) {
    for entity in transients {
        commands.entity(entity).despawn();
    }
    sightings.0.clear();
}

/// A rebuild jumped the state: drop the effects of the old position.
fn clear_effects_on_snap(
    snap: Option<Res<SnapSprites>>,
    mut commands: Commands,
    transients: Query<Entity, With<Transient>>,
    mut sightings: ResMut<Sightings>,
) {
    if snap.is_some() {
        clear_effects(&mut commands, &transients, &mut sightings);
    }
}

fn clear_effects_on_exit(
    mut commands: Commands,
    transients: Query<Entity, With<Transient>>,
    rings: Query<Entity, With<CardFocusRing>>,
    mut sightings: ResMut<Sightings>,
    mut focus: ResMut<CardFocus>,
) {
    clear_effects(&mut commands, &transients, &mut sightings);
    for entity in &rings {
        commands.entity(entity).despawn();
    }
    focus.0.clear();
}

// -- Live changes -> effects ----------------------------------------------------

/// Whether `owner`'s actions are news to the local player: the other side's
/// (an opponent, the AI), or anyone's for a spectator. An unbound session
/// (no seats) plays both sides itself, so nothing is news.
fn is_foreign(owner: Option<Player>, peers: &crate::peers::Peers) -> bool {
    let Some(owner) = owner else { return false };
    match peers.local() {
        Some(local) => owner != local,
        None => peers.is_spectator(),
    }
}

fn owner_of(unit: UnitId) -> Option<Player> {
    omdurman_rules::unit_profiles::section_owner(unit.section_pos().0)
}

/// Opponents' moves and entries: an attention ripple at each hex they ended
/// on, plus a sighting for the off-screen pointer.
pub fn announce_live_changes(
    mut live: ResMut<LiveApplied>,
    peers: crate::peers::Peers,
    mut fx: MessageWriter<FxRequest>,
    mut sightings: ResMut<Sightings>,
) {
    if live.0.is_empty() {
        return;
    }
    let mut hexes: Vec<HexCoord> = Vec::new();
    for change in live.0.drain(..) {
        if change.setup || change.is_empty() {
            continue;
        }
        for (unit, hex) in change.moved.iter().chain(&change.placed) {
            if is_foreign(owner_of(*unit), &peers) && !hexes.contains(hex) {
                hexes.push(*hex);
            }
        }
    }
    for hex in hexes {
        fx.write(FxRequest::Attention { hex });
        sightings.see(hex);
    }
}

/// Combat on the board: a trace from each firing hex to the target, a burst
/// where the shot landed (with a hop to it when a howitzer shell scattered),
/// a clash marker for melee. An opponent's attack is also a sighting.
pub fn announce_combat(
    mut observations: MessageReader<crate::events::ObservationEvent>,
    game_state: Res<crate::GameStateResource>,
    peers: crate::peers::Peers,
    mut fx: MessageWriter<FxRequest>,
    mut sightings: ResMut<Sightings>,
) {
    for event in observations.read() {
        match &event.observation {
            Observation::FireResolved { attack, impact, .. } => {
                let firers = attack.all_firing_units();
                let mut from_hexes: Vec<HexCoord> = Vec::new();
                for id in &firers {
                    if let Some(unit) = game_state.0.find_unit(*id)
                        && !from_hexes.contains(&unit.position)
                    {
                        from_hexes.push(unit.position);
                    }
                }
                let aimed = attack.target_hex;
                let landed = impact.map_or(aimed, |(_, hex)| hex);
                for from in from_hexes {
                    fx.write(FxRequest::Shot {
                        from,
                        to: aimed,
                        scatter: false,
                    });
                }
                if landed != aimed {
                    fx.write(FxRequest::Shot {
                        from: aimed,
                        to: landed,
                        scatter: true,
                    });
                }
                fx.write(FxRequest::Impact { hex: landed });
                if is_foreign(firers.first().copied().and_then(owner_of), &peers) {
                    sightings.see(landed);
                }
            }
            Observation::MeleeResolved { attack, .. } => {
                fx.write(FxRequest::Clash {
                    attacker: attack.attacker_hex,
                    defender: attack.defender_hex,
                });
                let attacker_owner = game_state
                    .0
                    .units
                    .iter()
                    .find(|u| u.position == attack.attacker_hex)
                    .and_then(|u| owner_of(u.id))
                    .or(Some(game_state.0.active_player));
                if is_foreign(attacker_owner, &peers) {
                    sightings.see(attack.defender_hex);
                }
            }
            _ => {}
        }
    }
}

// -- Off-screen pointers ----------------------------------------------------------

/// How long a sighting keeps its edge pointer (seconds).
const SIGHTING_TTL: f32 = 6.0;
/// Fade-out at the end of a sighting's life (seconds).
const SIGHTING_FADE: f32 = 1.0;

/// Recent hexes where something happened that the local player did not do:
/// an edge-of-screen pointer leads to each while it is out of view.
#[derive(Resource, Default)]
pub struct Sightings(Vec<Sighting>);

struct Sighting {
    hex: HexCoord,
    age: f32,
    /// The follow-camera already considered it.
    followed: bool,
}

impl Sightings {
    pub fn see(&mut self, hex: HexCoord) {
        self.0.retain(|s| s.hex != hex);
        self.0.push(Sighting {
            hex,
            age: 0.0,
            followed: false,
        });
    }
}

/// The hexes of the combat card under the pointer (written by the event
/// feed each frame): ringed on the board, and pointed at when off-screen.
#[derive(Resource, Default, PartialEq, Eq)]
pub struct CardFocus(pub Vec<HexCoord>);

#[derive(Component)]
struct CardFocusRing;

/// Ring the hovered combat card's hexes (rebuilt only when they change).
fn card_focus_rings(
    focus: Res<CardFocus>,
    assets: Option<Res<FxAssets>>,
    board: crate::BoardGeometry,
    existing: Query<Entity, With<CardFocusRing>>,
    mut commands: Commands,
) {
    if !focus.is_changed() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let Some(assets) = assets else { return };
    let origin = board.layout.adjusted_origin(&board.overlay.params);
    let size = board.overlay.params.hex_size;
    for &hex in &focus.0 {
        let p = hex_world_pos(hex, origin, &board.overlay.params);
        commands.spawn((
            CardFocusRing,
            Mesh3d(assets.ring.clone()),
            MeshMaterial3d(assets.focus.clone()),
            Transform::from_xyz(p.x, FX_HEIGHT, p.z).with_scale(Vec3::splat(size * 1.05)),
            Visibility::Visible,
        ));
    }
}

/// Where a pointer at `target` (screen space) sits on the edge of `board`,
/// inset by `margin`, and the direction it points. `None` when the target is
/// on the board already.
pub fn edge_anchor(board: egui::Rect, target: egui::Pos2, margin: f32) -> Option<(egui::Pos2, egui::Vec2)> {
    if board.contains(target) {
        return None;
    }
    let inner = board.shrink(margin);
    if inner.width() <= 0.0 || inner.height() <= 0.0 {
        return None;
    }
    let center = inner.center();
    let dir = target - center;
    if dir.length_sq() < f32::EPSILON {
        return None;
    }
    // Scale the ray from the centre so it just touches the inner rect.
    let tx = if dir.x.abs() > f32::EPSILON {
        (inner.width() * 0.5) / dir.x.abs()
    } else {
        f32::INFINITY
    };
    let ty = if dir.y.abs() > f32::EPSILON {
        (inner.height() * 0.5) / dir.y.abs()
    } else {
        f32::INFINITY
    };
    let t = tx.min(ty);
    Some((center + dir * t, dir.normalized()))
}

/// Pointers at the window edge for sightings (and the hovered card's hexes)
/// that lie outside the visible board; a click pans the camera there. With
/// [`MotionSettings::follow_opponent`], a fresh off-screen sighting pans by
/// itself.
#[allow(clippy::too_many_arguments)]
fn offscreen_pointers_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    settings: Res<MotionSettings>,
    mut sightings: ResMut<Sightings>,
    focus: Res<CardFocus>,
    board_geometry: crate::BoardGeometry,
    layout: Res<crate::ScreenLayout>,
    mut cameras: Query<(&Camera, &GlobalTransform, &mut RtsCameraState), With<RtsCamera>>,
) {
    let dt = time.delta_secs();
    for sighting in &mut sightings.0 {
        sighting.age += dt;
    }
    sightings.0.retain(|s| s.age < SIGHTING_TTL);
    if sightings.0.is_empty() && focus.0.is_empty() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((camera, cam_transform, mut cam_state)) = cameras.single_mut() else {
        return;
    };
    let screen = ctx.content_rect();
    let board = egui::Rect::from_min_max(
        egui::pos2(layout.left_inset, layout.top_bar_height),
        egui::pos2(screen.right() - layout.right_inset, screen.bottom()),
    );
    let origin = board_geometry
        .layout
        .adjusted_origin(&board_geometry.overlay.params);
    let world = |hex: HexCoord| hex_world_pos(hex, origin, &board_geometry.overlay.params);
    let on_screen = |p: Vec3| {
        camera
            .world_to_viewport(cam_transform, p)
            .ok()
            .map(|v| egui::pos2(v.x, v.y))
    };

    // (target hex, opacity) for every pointer candidate.
    let mut targets: Vec<(HexCoord, f32)> = Vec::new();
    let mut pan_to: Option<Vec3> = None;
    for sighting in &mut sightings.0 {
        let alpha = ((SIGHTING_TTL - sighting.age) / SIGHTING_FADE).clamp(0.0, 1.0);
        let p = world(sighting.hex);
        let visible = on_screen(p).is_some_and(|s| board.contains(s));
        if !sighting.followed {
            sighting.followed = true;
            if settings.follow_opponent && settings.motion() && !visible {
                pan_to = Some(p);
            }
        }
        targets.push((sighting.hex, alpha));
    }
    targets.extend(focus.0.iter().map(|&hex| (hex, 1.0)));

    egui::Area::new(egui::Id::new("offscreen_pointers"))
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            let painter = ui.painter().with_clip_rect(board);
            let mut drawn: Vec<egui::Pos2> = Vec::new();
            for &(hex, alpha) in &targets {
                let p = world(hex);
                let Some(target) = on_screen(p) else { continue };
                let Some((anchor, dir)) = edge_anchor(board, target, 28.0) else {
                    continue;
                };
                // Several sightings past the same edge spot: one pointer.
                if drawn.iter().any(|d| d.distance(anchor) < 20.0) {
                    continue;
                }
                drawn.push(anchor);
                let fill = crate::ui::palette::INK.gamma_multiply(0.85 * alpha);
                let edge = egui::Color32::from_rgb(250, 235, 140).gamma_multiply(alpha);
                let tip = anchor + dir * 16.0;
                let side = egui::vec2(-dir.y, dir.x) * 9.0;
                painter.circle_filled(anchor, 13.0, fill);
                painter.circle_stroke(anchor, 13.0, egui::Stroke::new(2.0, edge));
                painter.add(egui::Shape::convex_polygon(
                    vec![tip, anchor + dir * 8.0 + side, anchor + dir * 8.0 - side],
                    edge,
                    egui::Stroke::NONE,
                ));
                painter.text(
                    anchor,
                    egui::Align2::CENTER_CENTER,
                    "!",
                    egui::FontId::proportional(14.0),
                    edge,
                );
                let response = ui
                    .interact(
                        egui::Rect::from_center_size(anchor, egui::vec2(30.0, 30.0)),
                        egui::Id::new(("offscreen_pointer", hex.q, hex.r)),
                        egui::Sense::click(),
                    )
                    .on_hover_text(format!("Something happened at {hex} — click to look"));
                if response.clicked() {
                    pan_to = Some(p);
                }
            }
        });

    if let Some(p) = pan_to {
        cam_state.focus.x = p.x;
        cam_state.focus.z = p.z;
    }
}

// -- Pending orders ---------------------------------------------------------------

/// A dot of a committed move's trail, shown until the move's echo arrives.
#[derive(Component)]
struct TrailDot {
    /// Position along the route (the marching phase).
    step: usize,
    base_scale: Vec3,
}

/// How long a submission must wait for its echo before its trail shows:
/// offline and on a fast link the echo beats it, and nothing flickers.
const TRAIL_DELAY_SECS: f64 = 0.15;

/// A committed move whose echo has not come back yet keeps a marching trail
/// of dots along its route, so the order visibly travels instead of looking
/// ignored; the dots go once the engine accepts it (and the counter glides
/// along the same route) or rejects it.
#[allow(clippy::too_many_arguments)]
fn pending_order_trail(
    time: Res<Time>,
    settings: Res<MotionSettings>,
    pending: Res<crate::PendingEdits>,
    assets: Option<Res<FxAssets>>,
    board: crate::BoardGeometry,
    mut dots: Query<(Entity, &TrailDot, &mut Transform)>,
    mut commands: Commands,
    mut activity: ResMut<crate::activity::Activity>,
    mut first_seen: Local<HashMap<u64, f64>>,
    mut shown: Local<Vec<u64>>,
) {
    let now = time.elapsed_secs_f64();
    let moves: Vec<(u64, &GameEvent)> = pending
        .unconfirmed
        .iter()
        .filter(|(_, ev)| matches!(ev, GameEvent::MoveUnit { .. }))
        .map(|(uid, ev)| (*uid, ev))
        .collect();
    let live: HashSet<u64> = moves.iter().map(|(uid, _)| *uid).collect();
    first_seen.retain(|uid, _| live.contains(uid));
    for (uid, _) in &moves {
        first_seen.entry(*uid).or_insert(now);
    }
    let due: Vec<u64> = moves
        .iter()
        .map(|(uid, _)| *uid)
        .filter(|uid| first_seen.get(uid).is_some_and(|t| now - t >= TRAIL_DELAY_SECS))
        .collect();
    if !moves.is_empty() || !dots.is_empty() {
        // Waiting out the delay, or marching: frames needed.
        activity.keep_running();
    }

    if *shown != due || board.layout.is_changed() || board.overlay.is_changed() {
        for (entity, ..) in &dots {
            commands.entity(entity).despawn();
        }
        *shown = due.clone();
        let Some(assets) = assets.as_ref() else { return };
        let origin = board.layout.adjusted_origin(&board.overlay.params);
        let size = board.overlay.params.hex_size;
        for (uid, event) in &moves {
            let GameEvent::MoveUnit {
                to_q, to_r, path, ..
            } = event
            else {
                continue;
            };
            if !due.contains(uid) {
                continue;
            }
            let route: Vec<HexCoord> = if path.is_empty() {
                vec![HexCoord::new(*to_q, *to_r)]
            } else {
                path.clone()
            };
            for (step, hex) in route.iter().enumerate() {
                let p = hex_world_pos(*hex, origin, &board.overlay.params);
                let base_scale = Vec3::splat(size * 0.16);
                commands.spawn((
                    TrailDot { step, base_scale },
                    Mesh3d(assets.fill.clone()),
                    MeshMaterial3d(assets.trail.clone()),
                    Transform::from_xyz(p.x, FX_HEIGHT, p.z).with_scale(base_scale),
                    Visibility::Visible,
                ));
            }
        }
        return;
    }
    // March: a swell running along the route toward the destination.
    if settings.motion() {
        let t = time.elapsed_secs();
        for (_, dot, mut transform) in &mut dots {
            let wave = 0.5 + 0.5 * (std::f32::consts::TAU * (t * 1.4 - dot.step as f32 * 0.2)).sin();
            transform.scale = dot.base_scale * (0.6 + 0.5 * wave);
        }
    }
}

// -- Refusals -----------------------------------------------------------------------

/// A plotted leg the picker refused (unaffordable, blocked, over the stacking
/// cap) flashes red where the player clicked: the click did register.
fn flash_refused_legs(
    mut path: ResMut<crate::picker::MovementPath>,
    mut fx: MessageWriter<FxRequest>,
) {
    if path.refused.is_none() {
        return;
    }
    if let Some(hex) = path.refused.take() {
        fx.write(FxRequest::Refused { hex });
    }
}

// -- Phase banner ---------------------------------------------------------------------

/// Seconds a phase banner stays up, fades included.
const BANNER_SECS: f32 = 2.2;
/// Seconds of fade at each end.
const BANNER_FADE: f32 = 0.3;

/// A phase banner on screen.
struct Banner {
    title: String,
    subtitle: String,
    color: egui::Color32,
    age: f32,
}

/// A new phase or turn says so in the middle of the board for a moment (the
/// toolbar keeps saying it after): "Dervish — Melee", and whether the move is
/// the local player's. Never modal, never clickable; off with effects off.
#[allow(clippy::too_many_arguments)]
fn phase_banner_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    settings: Res<MotionSettings>,
    machine: Res<State<crate::ui_phase_state::UiPhaseState>>,
    game_state: Res<crate::GameStateResource>,
    peers: crate::peers::Peers,
    layout: Res<crate::ScreenLayout>,
    mut banner: Local<Option<Banner>>,
) {
    use crate::ui_phase_state::UiPhaseState;
    if machine.is_changed() {
        *banner = match **machine {
            state @ UiPhaseState::Turn { active, .. } if settings.effects() => {
                let actor = state.acting_player().unwrap_or(active);
                let subtitle = if peers.commands_faction(actor) {
                    "Your move".to_string()
                } else {
                    format!("{} to act", crate::ui::faction_name(actor))
                };
                Some(Banner {
                    title: format!(
                        "Turn {} \u{2014} {} {}",
                        game_state.0.current_turn.value(),
                        crate::ui::faction_name(active),
                        state.phase_label()
                    ),
                    subtitle,
                    color: crate::ui::faction_color(active),
                    age: 0.0,
                })
            }
            _ => None,
        };
    }
    let Some(shown) = banner.as_mut() else { return };
    shown.age += time.delta_secs();
    if shown.age >= BANNER_SECS {
        *banner = None;
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let fade_in = (shown.age / BANNER_FADE).min(1.0);
    let alpha = fade_in.min((BANNER_SECS - shown.age) / BANNER_FADE).clamp(0.0, 1.0);
    // Drops into place with full motion; only fades otherwise.
    let slide = if settings.motion() {
        -12.0 * (1.0 - fade_in) * (1.0 - fade_in)
    } else {
        0.0
    };
    let board_centre_x = (layout.left_inset
        + (ctx.content_rect().right() - layout.right_inset))
        * 0.5
        - ctx.content_rect().center().x;
    egui::Area::new(egui::Id::new("phase_banner"))
        .order(egui::Order::Foreground)
        .interactable(false)
        .anchor(
            egui::Align2::CENTER_TOP,
            egui::vec2(board_centre_x, layout.center_stack_y + 24.0 + slide),
        )
        .show(ctx, |ui| {
            ui.set_opacity(alpha);
            crate::ui::frames::paper(egui::Stroke::new(2.0, shown.color))
                .inner_margin(egui::Margin::symmetric(22, 10))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(&shown.title)
                                .size(18.0)
                                .strong()
                                .color(crate::ui::palette::INK),
                        );
                        ui.label(
                            egui::RichText::new(&shown.subtitle)
                                .size(13.0)
                                .color(crate::ui::palette::FAINT_INK),
                        );
                    });
                });
        });
    ctx.request_repaint();
}

// -- Plugin ---------------------------------------------------------------------------

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        let on_board = in_state(crate::AppMode::Game);
        app.init_resource::<MotionSettings>()
            .init_resource::<LiveApplied>()
            .init_resource::<Sightings>()
            .init_resource::<CardFocus>()
            .add_message::<FxRequest>()
            .add_systems(Startup, spawn_fx_assets)
            .add_systems(
                Update,
                (
                    clear_effects_on_snap
                        .after(crate::net_socket::handle_reconnect)
                        .after(crate::timeline::scrub_rebuild)
                        .before(crate::picker::reconcile_unit_sprites),
                    announce_live_changes
                        .after(crate::net_socket::handle_socket)
                        .before(spawn_fx),
                    announce_combat
                        .after(crate::events::drain_observations)
                        .before(spawn_fx),
                    flash_refused_legs
                        .in_set(crate::GameSet)
                        .after(crate::picker::handle_picker_clicks)
                        .before(spawn_fx),
                    pending_order_trail.in_set(crate::GameSet),
                    card_focus_rings.run_if(on_board.clone()),
                    spawn_fx
                        .after(crate::picker::reconcile_unit_sprites)
                        .run_if(on_board.clone()),
                    animate_transients.after(spawn_fx),
                ),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    offscreen_pointers_ui.run_if(crate::map_view_active),
                    phase_banner_ui
                        .after(crate::ui_plugin::mode_toolbar_ui)
                        .run_if(in_state(crate::AppState::InGame).and_then(on_board.clone())),
                ),
            )
            .add_systems(OnExit(crate::AppMode::Game), clear_effects_on_exit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_curve_starts_visible_and_ends_invisible_or_gone() {
        for curve in [Curve::Pulse, Curve::Pop, Curve::Flash, Curve::Sink] {
            for motion in [true, false] {
                let end = sample(curve, 1.0, motion);
                assert!(
                    end.alpha <= 1e-6 || end.scale <= 1e-6,
                    "{curve:?} (motion {motion}) must vanish at the end: {end:?}"
                );
                let mid = sample(curve, 0.3, motion);
                assert!(mid.alpha > 0.0 && mid.scale > 0.0, "{curve:?} shows mid-life");
            }
        }
    }

    #[test]
    fn reduced_motion_never_moves_or_resizes() {
        for curve in [Curve::Pulse, Curve::Pop, Curve::Flash, Curve::Sink] {
            for i in 0..=20 {
                let look = sample(curve, i as f32 / 20.0, false);
                assert_eq!(look.scale, 1.0, "{curve:?} keeps its size");
                assert_eq!(look.drop, 0.0, "{curve:?} keeps its height");
            }
        }
    }

    #[test]
    fn a_ghost_sinks_and_shrinks_with_motion() {
        let start = sample(Curve::Sink, 0.0, true);
        let late = sample(Curve::Sink, 0.9, true);
        assert!(late.drop > start.drop);
        assert!(late.scale < start.scale);
    }

    #[test]
    fn the_live_change_lists_moves_and_entries_only() {
        use omdurman_rules::unit_profiles::profile_for_unit;
        use omdurman_rules::{UnitPlacement, UnitState};
        let mut gs = GameState::new(omdurman_types::Scenario::Campaign);
        let unit = |id, q| UnitPlacement {
            id,
            position: HexCoord::new(q, 0),
            profile: profile_for_unit(id).unwrap(),
            state: UnitState::default(),
        };
        gs.units.push(unit(UnitId::Taiasha_0_0, 1));
        gs.units.push(unit(UnitId::BritishBoats_3_0, 5));
        let before = LiveChange::snapshot(&gs);
        gs.units[0].position = HexCoord::new(2, 0);
        gs.units.push(unit(UnitId::AliWadHelu_0_0, 7));
        let change = LiveChange::between(&before, &gs, false);
        assert_eq!(change.moved, vec![(UnitId::Taiasha_0_0, HexCoord::new(2, 0))]);
        assert_eq!(change.placed, vec![(UnitId::AliWadHelu_0_0, HexCoord::new(7, 0))]);
    }

    #[test]
    fn a_pointer_sits_on_the_edge_toward_its_target() {
        let board = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(200.0, 100.0));
        assert!(edge_anchor(board, egui::pos2(50.0, 50.0), 10.0).is_none(), "visible: no pointer");
        let (at, dir) = edge_anchor(board, egui::pos2(500.0, 50.0), 10.0).unwrap();
        assert!((at.x - 190.0).abs() < 1e-3 && (at.y - 50.0).abs() < 1e-3);
        assert!(dir.x > 0.99);
        let (at, dir) = edge_anchor(board, egui::pos2(100.0, -400.0), 10.0).unwrap();
        assert!((at.y - 10.0).abs() < 1e-3);
        assert!(dir.y < -0.99);
    }

    #[test]
    fn the_motion_level_parses_its_labels() {
        assert_eq!(MotionLevel::from_name("off"), Some(MotionLevel::Off));
        assert_eq!(MotionLevel::from_name(" Reduced "), Some(MotionLevel::Reduced));
        assert_eq!(MotionLevel::from_name("FULL"), Some(MotionLevel::Full));
        assert_eq!(MotionLevel::from_name("wobbly"), None);
    }
}
