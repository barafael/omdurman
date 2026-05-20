use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Component)]
pub struct RtsCamera;

#[derive(Component)]
pub struct RtsCameraState {
    pub focus: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub smooth_focus: Vec3,
    pub smooth_distance: f32,
    pub smooth_yaw: f32,
    pub smooth_pitch: f32,
}

impl Default for RtsCameraState {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            distance: 1500.0,
            yaw: 0.0,
            pitch: PI / 2.0 - 0.02,
            smooth_focus: Vec3::ZERO,
            smooth_distance: 1500.0,
            smooth_yaw: 0.0,
            smooth_pitch: PI / 2.0 - 0.02,
        }
    }
}

#[derive(Resource, Default)]
pub struct CameraDragState {
    pub active: bool,
    pub last_cursor: Vec2,
}

#[derive(Resource)]
pub struct CameraSettings {
    pub pan_speed: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub smoothing: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            pan_speed: 600.0,
            min_distance: 100.0,
            max_distance: 3000.0,
            min_pitch: PI / 6.0,
            max_pitch: PI / 2.0 - 0.02,
            smoothing: 6.0,
        }
    }
}
