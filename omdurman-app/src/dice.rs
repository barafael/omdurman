use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{Dice, EditorMode, d10_collider_points, d10_mesh_colored};

#[derive(Resource)]
pub struct DiceSimulator {
    pub radius: f32,
    pub height: f32,
    pub throw_strength: f32,
    pub upward_velocity: f32,
    pub gravity_scale: f32,
    pub restitution: f32,
    pub friction: f32,
    pub dice_lifetime: f32,
    pub mass: f32,
    pub spin_strength: f32,
    pub random_spread: f32,
    pub throw_spread: f32,
    pub spawn_height: f32,
}

impl Default for DiceSimulator {
    fn default() -> Self {
        Self {
            radius: 60.0,
            height: 120.0,
            throw_strength: 150.0,
            upward_velocity: 100.0,
            gravity_scale: 30.0,
            restitution: 0.3,
            friction: 0.8,
            dice_lifetime: 6.0,
            mass: 1.0,
            spin_strength: 3.0,
            random_spread: 0.75,
            throw_spread: 1.0,
            spawn_height: 100.0,
        }
    }
}

pub fn dice_sim_ui(
    mut contexts: EguiContexts,
    mode: Res<EditorMode>,
    mut sim: ResMut<DiceSimulator>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ctx = guard_mode!(contexts, mode, Dice);

    egui::SidePanel::right("dice_meta_panel")
        .resizable(true)
        .default_width(280.0)
        .width_range(200.0..=500.0)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(45))
                .inner_margin(egui::Margin::symmetric(16, 16)),
        )
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));

            egui::Grid::new("dice_params")
                .striped(true)
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Radius");
                    ui.add(egui::Slider::new(&mut sim.radius, 10.0..=200.0));
                    ui.end_row();

                    ui.label("Height");
                    ui.add(egui::Slider::new(&mut sim.height, 10.0..=300.0));
                    ui.end_row();

                    ui.label("Throw Strength");
                    ui.add(egui::Slider::new(&mut sim.throw_strength, 0.0..=500.0));
                    ui.end_row();

                    ui.label("Upward Velocity");
                    ui.add(egui::Slider::new(&mut sim.upward_velocity, 0.0..=500.0));
                    ui.end_row();

                    ui.label("Gravity Scale");
                    ui.add(egui::Slider::new(&mut sim.gravity_scale, 0.0..=100.0));
                    ui.end_row();

                    ui.label("Restitution (bounce)");
                    ui.add(egui::Slider::new(&mut sim.restitution, 0.0..=1.0));
                    ui.end_row();

                    ui.label("Friction");
                    ui.add(egui::Slider::new(&mut sim.friction, 0.0..=1.0));
                    ui.end_row();

                    ui.label("Dice Lifetime (s)");
                    ui.add(egui::Slider::new(&mut sim.dice_lifetime, 0.5..=30.0));
                    ui.end_row();

                    ui.label("Mass");
                    ui.add(egui::Slider::new(&mut sim.mass, 0.1..=100.0));
                    ui.end_row();

                    ui.label("Spin Strength");
                    ui.add(egui::Slider::new(&mut sim.spin_strength, 0.0..=20.0));
                    ui.end_row();

                    ui.label("Random Spread");
                    ui.add(egui::Slider::new(&mut sim.random_spread, 0.0..=5.0));
                    ui.end_row();

                    ui.label("Throw Spread");
                    ui.add(egui::Slider::new(&mut sim.throw_spread, 0.0..=5.0));
                    ui.end_row();

                    ui.label("Spawn Height");
                    ui.add(egui::Slider::new(&mut sim.spawn_height, 0.0..=500.0));
                    ui.end_row();
                });

            ui.separator();

            if ui.button("Throw!").clicked() {
                let mut local_rng = rand::rng();
                let radius = sim.radius;
                let height = sim.height;

                let throw_dir = Vec3::new(
                    rand::RngExt::random_range(&mut local_rng, -sim.throw_spread..sim.throw_spread),
                    0.0,
                    rand::RngExt::random_range(&mut local_rng, -sim.throw_spread..sim.throw_spread),
                )
                .normalize_or_zero();

                let initial_spin = throw_dir.cross(Vec3::Y) * sim.spin_strength
                    + Vec3::new(
                        rand::RngExt::random_range(
                            &mut local_rng,
                            -sim.random_spread..sim.random_spread,
                        ),
                        rand::RngExt::random_range(
                            &mut local_rng,
                            -sim.random_spread..sim.random_spread,
                        ),
                        rand::RngExt::random_range(
                            &mut local_rng,
                            -sim.random_spread..sim.random_spread,
                        ),
                    );

                let collider_points = d10_collider_points(radius, height);
                commands.spawn((
                    RigidBody::Dynamic,
                    Collider::convex_hull(collider_points).unwrap(),
                    Mass(sim.mass),
                    GravityScale(sim.gravity_scale),
                    Mesh3d(meshes.add(d10_mesh_colored(radius, height))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_translation(Vec3::new(0.0, sim.spawn_height, 0.0))
                        .with_rotation(Quat::from_euler(
                            EulerRot::XYZ,
                            rand::RngExt::random_range(&mut local_rng, 0.0..core::f32::consts::TAU),
                            rand::RngExt::random_range(&mut local_rng, 0.0..core::f32::consts::TAU),
                            rand::RngExt::random_range(&mut local_rng, 0.0..core::f32::consts::TAU),
                        )),
                    LinearVelocity(throw_dir * sim.throw_strength + Vec3::Y * sim.upward_velocity),
                    AngularVelocity(initial_spin),
                    Restitution::new(sim.restitution),
                    Friction::new(sim.friction),
                    Dice {
                        timer: Timer::from_seconds(sim.dice_lifetime, TimerMode::Once),
                    },
                ));
            }
        });
}
