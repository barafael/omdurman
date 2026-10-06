//! Counters in motion: a counter glides hex by hex along the route it took,
//! and turns over in place when it is disrupted or rallies. Both are started
//! by [`crate::picker::reconcile_unit_sprites`] (which decides *whether* a
//! change plays out or lands at once) and played here; both are purely
//! visual -- the counter's `PlacedUnit` already holds the engine state.

use bevy::prelude::*;

/// Seconds a counter takes to slide from one hex to the next.
const MOVE_ANIM_SECS: f32 = 0.3;
/// Seconds a counter takes to turn over.
const TURN_SECS: f32 = 0.25;

/// Smoothstep easing: 0 at t=0, 1 at t=1, zero slope at both ends.
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// A counter gliding across the board, one hex step at a time: `from` → `to`
/// is the current step, `rest` the remaining waypoints (world positions). The
/// glide owns the counter's translation until it finishes.
#[derive(Component)]
pub struct MovementAnimation {
    pub from: Vec3,
    pub to: Vec3,
    pub progress: f32,
    pub rest: std::collections::VecDeque<Vec3>,
}

pub fn animate_unit_movement(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut MovementAnimation)>,
    mut commands: Commands,
    mut activity: ResMut<crate::activity::Activity>,
) {
    for (entity, mut transform, mut anim) in query.iter_mut() {
        activity.keep_running();
        anim.progress += time.delta_secs() / MOVE_ANIM_SECS;
        if anim.progress >= 1.0 {
            transform.translation = anim.to;
            if let Some(next) = anim.rest.pop_front() {
                // Next hex of the route.
                anim.from = anim.to;
                anim.to = next;
                anim.progress = 0.0;
            } else {
                commands.entity(entity).remove::<MovementAnimation>();
            }
        } else {
            transform.translation = anim.from.lerp(anim.to, smoothstep(anim.progress));
        }
    }
}

/// A counter turning over in place (disrupted, or rallied back): it spins
/// from one face-up to the other while its tint follows.
#[derive(Component)]
pub struct CounterTurn {
    from: Quat,
    to: Quat,
    from_tint: Color,
    to_tint: Color,
    progress: f32,
}

impl CounterTurn {
    pub fn new(from: Quat, to: Quat, from_tint: Color, to_tint: Color) -> Self {
        Self {
            from,
            to,
            from_tint,
            to_tint,
            progress: 0.0,
        }
    }
}

pub fn animate_counter_turns(
    time: Res<Time>,
    mut turns: Query<(
        Entity,
        &mut Transform,
        &mut CounterTurn,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut activity: ResMut<crate::activity::Activity>,
) {
    use bevy::color::Mix;
    for (entity, mut transform, mut turn, material) in &mut turns {
        activity.keep_running();
        turn.progress = (turn.progress + time.delta_secs() / TURN_SECS).min(1.0);
        let e = smoothstep(turn.progress);
        transform.rotation = turn.from.slerp(turn.to, e);
        if let Some(mut mat) = materials.get_mut(&material.0) {
            mat.base_color = turn.from_tint.mix(&turn.to_tint, e);
        }
        if turn.progress >= 1.0 {
            transform.rotation = turn.to;
            commands.entity(entity).remove::<CounterTurn>();
        }
    }
}
