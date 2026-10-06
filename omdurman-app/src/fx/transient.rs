//! Self-removing marks: a ring, a trace, a flash, a fading counter. Each owns
//! its material (it fades on its own) and despawns at the end of its life.

use bevy::asset::RenderAssetUsages;
use bevy::ecs::message::MessageReader;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use omdurman_hexmap::hex_world_pos;
use omdurman_types::HexCoord;

use super::{FxRequest, MotionSettings, SnapSprites};

/// Shared meshes and the materials of the steady (non-fading) marks.
#[derive(Resource)]
pub struct FxAssets {
    /// A thin hex ring (outer radius 1, in the XZ plane).
    pub(super) ring: Handle<Mesh>,
    /// A filled hex (radius 1, in the XZ plane).
    pub(super) fill: Handle<Mesh>,
    /// A unit triangle in the XZ plane pointing +Z, centred on its centroid.
    pub(super) triangle: Handle<Mesh>,
    /// The dots of a pending order.
    pub(super) trail: Handle<StandardMaterial>,
    /// Rings on the hexes of the hovered combat card.
    pub(super) focus: Handle<StandardMaterial>,
}

/// The marks' colours: muted, and translucent, so the map and the counters
/// stay the loudest things on the board.
mod tint {
    use bevy::prelude::Color;
    /// Attention ring: warm white, readable on the sepia map and at night.
    pub const ATTENTION: Color = Color::srgba(1.0, 0.94, 0.75, 0.6);
    /// Shot trace.
    pub const SHOT: Color = Color::srgba(0.8, 0.16, 0.1, 0.5);
    /// Melee mark.
    pub const CLASH: Color = Color::srgba(0.8, 0.12, 0.08, 0.5);
    /// Refused order.
    pub const REFUSED: Color = Color::srgba(0.85, 0.12, 0.08, 0.3);
    /// Pending order dots.
    pub const TRAIL: Color = Color::srgba(0.98, 0.95, 0.85, 0.45);
    /// Hovered combat card's hexes.
    pub const FOCUS: Color = Color::srgba(1.0, 0.94, 0.75, 0.7);
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

/// A corner of a flat hex of the given radius in the XZ plane, in the
/// hex-ring mesh's orientation.
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
fn ring_mesh(inner: f32) -> Mesh {
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

/// An equilateral triangle with side 1, pointing +Z.
fn triangle_mesh() -> Mesh {
    let h = 3f32.sqrt() / 2.0;
    flat_mesh(
        vec![
            Vec3::new(0.0, 0.0, h * 2.0 / 3.0),
            Vec3::new(0.5, 0.0, -h / 3.0),
            Vec3::new(-0.5, 0.0, -h / 3.0),
        ],
        vec![0, 2, 1],
    )
}

pub(super) fn spawn_fx_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(FxAssets {
        ring: meshes.add(ring_mesh(0.93)),
        fill: meshes.add(hex_fill_mesh()),
        triangle: meshes.add(triangle_mesh()),
        trail: materials.add(effect_material(tint::TRAIL)),
        focus: materials.add(effect_material(tint::FOCUS)),
    });
}

// -- Curves ---------------------------------------------------------------------

/// How a mark develops over its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curve {
    /// Widens a little past the hex outline while fading (attention).
    Pulse,
    /// Draws itself in, then fades (shots, melee).
    Trace,
    /// Shows at once, then fades (a refusal).
    Flash,
    /// Fades out in place (an eliminated counter).
    Fade,
}

/// A frame of a mark: scale and opacity factors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub scale: f32,
    pub alpha: f32,
}

/// The look of a `curve` at life fraction `p` (`0..=1`). Without `motion`
/// nothing changes size: the same mark only fades.
pub fn sample(curve: Curve, p: f32, motion: bool) -> Look {
    let p = p.clamp(0.0, 1.0);
    // Full until `start`, then down to nothing, easing out.
    let fade_from = |start: f32| {
        if p <= start {
            1.0
        } else {
            let q = (p - start) / (1.0 - start);
            1.0 - q * q
        }
    };
    let scale = match curve {
        Curve::Pulse if motion => 1.0 + 0.2 * (1.0 - (1.0 - p) * (1.0 - p)),
        // Drawn in over the first fifth of its life.
        Curve::Trace if motion => {
            let q = (p / 0.2).min(1.0);
            1.0 - (1.0 - q) * (1.0 - q)
        }
        _ => 1.0,
    };
    let alpha = match curve {
        Curve::Pulse => 1.0 - p,
        Curve::Trace => fade_from(0.45),
        Curve::Flash => fade_from(0.2),
        Curve::Fade => 1.0 - p,
    };
    Look { scale, alpha }
}

// -- Marks ----------------------------------------------------------------------

/// A self-removing mark entity. Owns its material.
#[derive(Component)]
pub struct Transient {
    /// Seconds lived; negative while delayed (hidden).
    age: f32,
    ttl: f32,
    curve: Curve,
    base_scale: Vec3,
    base_alpha: f32,
    /// The AI waits for it to finish before its next action (a shot, a
    /// melee, an elimination), so its turn reads one action at a time.
    pub blocking: bool,
    /// On the review timeline: held at this life fraction while playback is
    /// paused, so a paused event keeps its mark.
    held_at: Option<f32>,
}

/// Height of the marks: above stacked counters.
pub(super) const FX_HEIGHT: f32 = 2.2;

struct TransientSpec {
    mesh: Handle<Mesh>,
    material: StandardMaterial,
    transform: Transform,
    curve: Curve,
    ttl: f32,
    delay: f32,
    blocking: bool,
    held_at: Option<f32>,
}

fn spawn_transient(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    motion: bool,
    spec: TransientSpec,
) {
    let base_alpha = spec.material.base_color.alpha();
    let look = sample(spec.curve, 0.0, motion);
    let mut material = spec.material;
    material.base_color.set_alpha(if spec.delay > 0.0 {
        0.0
    } else {
        base_alpha * look.alpha
    });
    commands.spawn((
        Transient {
            age: -spec.delay,
            ttl: spec.ttl,
            curve: spec.curve,
            base_scale: spec.transform.scale,
            base_alpha,
            blocking: spec.blocking,
            held_at: spec.held_at,
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

/// Turn [`FxRequest`]s into marks at this board's geometry.
#[allow(clippy::too_many_arguments)]
pub fn spawn_fx(
    mut requests: MessageReader<FxRequest>,
    settings: Res<MotionSettings>,
    app_state: Res<State<crate::AppState>>,
    assets: Option<Res<FxAssets>>,
    arrows: Option<Res<crate::render::MovementArrowAssets>>,
    board: crate::BoardGeometry,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if !settings.effects() {
        requests.clear();
        return;
    }
    let (Some(assets), Some(arrows)) = (assets, arrows) else {
        requests.clear();
        return;
    };
    let motion = settings.motion();
    // Reviewing: the combat marks hold while the timeline is paused.
    let reviewing = *app_state.get() == crate::AppState::Spectating;
    let held = reviewing.then_some(0.4);
    let origin = board.layout.adjusted_origin(&board.overlay.params);
    let size = board.overlay.params.hex_size;
    let world = |hex: HexCoord| hex_world_pos(hex, origin, &board.overlay.params);
    let at = |hex: HexCoord, scale: f32| {
        let p = world(hex);
        Transform::from_xyz(p.x, FX_HEIGHT, p.z).with_scale(Vec3::splat(size * scale))
    };

    for request in requests.read() {
        let spec = match request {
            FxRequest::Attention { hex } => TransientSpec {
                mesh: assets.ring.clone(),
                material: effect_material(tint::ATTENTION),
                transform: at(*hex, 1.0),
                curve: Curve::Pulse,
                ttl: 0.9,
                delay: 0.0,
                blocking: false,
                held_at: None,
            },
            FxRequest::Shot { from, to, scatter } => {
                let (a, b) = (world(*from), world(*to));
                let delta = Vec3::new(b.x - a.x, 0.0, b.z - a.z);
                let len = delta.length();
                if len < f32::EPSILON {
                    continue;
                }
                let dir = delta / len;
                // As the live fire arrow: inset from both ends, the arrow
                // mesh points +Z, rotated onto the heading. Drawn in from the
                // firer, so the length (z) is what grows.
                let inset = size * 0.18;
                let draw_len = (len - inset).max(len * 0.4);
                let tail = a + dir * ((len - draw_len) * 0.5);
                let width = if *scatter { 0.18 } else { 0.28 };
                TransientSpec {
                    mesh: arrows.mesh.clone(),
                    material: effect_material(tint::SHOT),
                    transform: Transform::from_xyz(tail.x, FX_HEIGHT, tail.z)
                        .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                        .with_scale(Vec3::new(size * width, 1.0, draw_len)),
                    curve: Curve::Trace,
                    ttl: 0.8,
                    delay: if *scatter { 0.2 } else { 0.0 },
                    blocking: true,
                    held_at: held,
                }
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
                TransientSpec {
                    mesh: assets.triangle.clone(),
                    material: effect_material(tint::CLASH),
                    transform: Transform::from_xyz(mid.x, FX_HEIGHT, mid.z)
                        .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                        .with_scale(Vec3::splat(size * 0.32)),
                    curve: Curve::Trace,
                    ttl: 0.8,
                    delay: 0.0,
                    blocking: true,
                    held_at: held,
                }
            }
            FxRequest::Refused { hex } => TransientSpec {
                mesh: assets.fill.clone(),
                material: effect_material(tint::REFUSED),
                transform: at(*hex, 0.95),
                curve: Curve::Flash,
                ttl: 0.35,
                delay: 0.0,
                blocking: false,
                held_at: None,
            },
            FxRequest::Ghost {
                transform,
                mesh,
                texture,
                tint,
            } => TransientSpec {
                mesh: mesh.clone(),
                material: StandardMaterial {
                    base_color: *tint,
                    base_color_texture: texture.clone(),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                },
                transform: *transform,
                curve: Curve::Fade,
                ttl: 0.5,
                delay: 0.0,
                blocking: true,
                held_at: None,
            },
        };
        spawn_transient(&mut commands, &mut materials, motion, spec);
    }
}

/// Play every mark forward; despawn it at the end of its life. On a paused
/// review timeline the combat marks hold at their [`Transient::held_at`].
pub fn animate_transients(
    time: Res<Time>,
    settings: Res<MotionSettings>,
    timeline: Option<Res<crate::timeline::SpectatorTimeline>>,
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
    let paused = timeline.is_some_and(|t| t.record.is_some() && !t.playing);
    for (entity, mut fx, mut transform, material) in &mut query {
        let hold = fx.held_at.filter(|_| paused).map(|at| at * fx.ttl);
        if hold.is_some_and(|at| fx.age >= at) {
            continue; // parked on its event: nothing moves
        }
        activity.keep_running();
        fx.age += dt;
        if let Some(at) = hold {
            fx.age = fx.age.min(at);
        }
        if fx.age < 0.0 {
            continue; // still delayed (spawned transparent)
        }
        let p = fx.age / fx.ttl;
        if p >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let look = sample(fx.curve, p, motion);
        let scale = fx.base_scale * look.scale.max(0.001);
        if transform.scale != scale {
            transform.scale = scale;
        }
        if let Some(mut mat) = materials.get_mut(&material.0) {
            mat.base_color.set_alpha(fx.base_alpha * look.alpha);
        }
    }
}

/// A counter gliding or turning over.
type CountersInMotion = Or<(With<super::MovementAnimation>, With<super::CounterTurn>)>;
/// The steady (non-fading) marks: card rings, pending-order dots.
type SteadyMarks = Or<(
    With<super::pointers::CardFocusRing>,
    With<super::orders::TrailDot>,
)>;

/// Whether the board is still showing something the AI should let finish
/// before its next action: a counter gliding or turning, a shot, a melee, an
/// elimination.
#[derive(bevy::ecs::system::SystemParam)]
pub struct BoardBusy<'w, 's> {
    moving: Query<'w, 's, (), CountersInMotion>,
    marks: Query<'w, 's, &'static Transient>,
}

impl BoardBusy<'_, '_> {
    pub fn busy(&self) -> bool {
        !self.moving.is_empty() || self.marks.iter().any(|t| t.blocking)
    }
}

/// Drop every mark and pointer: the state jumped (rebuild) or the board is
/// no longer shown.
pub(super) fn clear_marks(
    commands: &mut Commands,
    marks: &Query<Entity, With<Transient>>,
    sightings: &mut super::pointers::Sightings,
) {
    for entity in marks {
        commands.entity(entity).despawn();
    }
    sightings.clear();
}

/// A rebuild jumped the state: drop the marks of the old position.
pub(super) fn clear_effects_on_snap(
    snap: Option<Res<SnapSprites>>,
    mut commands: Commands,
    marks: Query<Entity, With<Transient>>,
    mut sightings: ResMut<super::pointers::Sightings>,
) {
    if snap.is_some() {
        clear_marks(&mut commands, &marks, &mut sightings);
    }
}

pub(super) fn clear_effects_on_exit(
    mut commands: Commands,
    marks: Query<Entity, With<Transient>>,
    steady: Query<Entity, SteadyMarks>,
    mut sightings: ResMut<super::pointers::Sightings>,
    mut focus: ResMut<super::CardFocus>,
) {
    clear_marks(&mut commands, &marks, &mut sightings);
    for entity in &steady {
        commands.entity(entity).despawn();
    }
    focus.0.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURVES: [Curve; 4] = [Curve::Pulse, Curve::Trace, Curve::Flash, Curve::Fade];

    #[test]
    fn every_mark_shows_mid_life_and_is_gone_at_the_end() {
        for curve in CURVES {
            for motion in [true, false] {
                let end = sample(curve, 1.0, motion);
                assert!(
                    end.alpha <= 1e-6,
                    "{curve:?} (motion {motion}) fades out: {end:?}"
                );
                let mid = sample(curve, 0.3, motion);
                assert!(mid.alpha > 0.0 && mid.scale > 0.0, "{curve:?} shows");
            }
        }
    }

    #[test]
    fn without_motion_nothing_changes_size() {
        for curve in CURVES {
            for i in 0..=20 {
                assert_eq!(sample(curve, i as f32 / 20.0, false).scale, 1.0);
            }
        }
    }

    /// Quiet by construction: an attention ring never grows past a fifth
    /// beyond its hex, and nothing overshoots its full size.
    #[test]
    fn marks_stay_small() {
        for curve in CURVES {
            for i in 0..=50 {
                let look = sample(curve, i as f32 / 50.0, true);
                assert!(look.scale <= 1.2 + 1e-6, "{curve:?}: {look:?}");
                assert!(look.alpha <= 1.0, "{curve:?}: {look:?}");
            }
        }
    }
}
