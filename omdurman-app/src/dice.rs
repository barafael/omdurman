use crate::{EditorMode, GameRng, PendingEdits, TurnState};
use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::ecs::message::MessageWriter;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy_egui::{EguiContexts, egui};
use omdurman_net::{GameEvent, NetMsg, NetState};

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
    mode: Res<State<EditorMode>>,
    mut sim: ResMut<DiceSimulator>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !mode.is_dice() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

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
                let tex = images.add(make_d10_texture());
                commands.spawn((
                    RigidBody::Dynamic,
                    Collider::convex_hull(collider_points).unwrap(),
                    Mass(sim.mass),
                    GravityScale(sim.gravity_scale),
                    Mesh3d(meshes.add(d10_mesh_uv(radius, height))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color_texture: Some(tex),
                        unlit: true,
                        alpha_mode: AlphaMode::Mask(0.5),
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

// -- Moved from main.rs --------------------------------------------------

#[derive(Component)]
pub struct Dice {
    timer: Timer,
}

#[derive(Message, Debug)]
pub struct DiceRollResult {
    pub by_me: bool,
    pub data: u32,
}

pub(crate) fn handle_local_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut turn: ResMut<TurnState>,
    net: Res<NetState>,
    rng_opt: Option<ResMut<GameRng>>,
    mut pending: ResMut<PendingEdits>,
    mut ev_action: MessageWriter<DiceRollResult>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_keyboard_input() {
        return;
    }
    if turn.current_turn != turn.my_index {
        return;
    }
    if turn.action_in_flight {
        return;
    }
    if net.peers.is_empty() {
        return;
    }
    let Some(mut rng) = rng_opt else { return };
    let mut local_rng = rand::rng();

    if keys.just_pressed(KeyCode::Space) && turn.pending_roll.is_none() {
        let roll = rng.random_u32() % 10 + 1;
        info!(roll, "rolled");

        turn.pending_roll = Some(roll);

        let radius = 60.0;
        let height = 120.0;
        let throw_dir = Vec3::new(
            rand::RngExt::random_range(&mut local_rng, -1.0..1.0),
            0.0,
            rand::RngExt::random_range(&mut local_rng, -1.0..1.0),
        )
        .normalize_or_zero();
        let initial_spin = throw_dir.cross(Vec3::Y) * 3.0
            + Vec3::new(
                rand::RngExt::random_range(&mut local_rng, -0.75..0.75),
                rand::RngExt::random_range(&mut local_rng, -0.75..0.75),
                rand::RngExt::random_range(&mut local_rng, -0.75..0.75),
            );

        let collider_points = d10_collider_points(radius, height);
        let tex = images.add(make_d10_texture());
        commands.spawn((
            RigidBody::Dynamic,
            Collider::convex_hull(collider_points).unwrap(),
            Mass(1.0),
            GravityScale(30.0),
            Mesh3d(meshes.add(d10_mesh_uv(radius, height))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(tex),
                unlit: true,
                alpha_mode: AlphaMode::Mask(0.5),
                ..default()
            })),
            Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)).with_rotation(
                Quat::from_euler(
                    EulerRot::XYZ,
                    rand::RngExt::random_range(&mut local_rng, 0.0..core::f32::consts::TAU),
                    rand::RngExt::random_range(&mut local_rng, 0.0..core::f32::consts::TAU),
                    rand::RngExt::random_range(&mut local_rng, 0.0..core::f32::consts::TAU),
                ),
            ),
            LinearVelocity(throw_dir * 150.0 + Vec3::Y * 100.0),
            AngularVelocity(initial_spin),
            Restitution::new(0.3),
            Friction::new(0.8),
            Dice {
                timer: Timer::from_seconds(6.0, TimerMode::Once),
            },
        ));
    }

    if keys.just_pressed(KeyCode::Enter)
        && let Some(roll) = turn.pending_roll.take()
    {
        info!(roll, "sending action");

        pending
            .outgoing_broadcast
            .push(NetMsg::Game(GameEvent::Action(roll)));

        ev_action.write(DiceRollResult {
            by_me: true,
            data: roll,
        });
        turn.action_in_flight = true;
    }
}

pub(crate) fn despawn_dice(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Dice)>,
) {
    for (entity, mut dice) in query.iter_mut() {
        dice.timer.tick(time.delta());
        if dice.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn d10_collider_points(radius: f32, height: f32) -> Vec<Vec3> {
    let n = 5;
    let mut points = vec![
        Vec3::new(0.0, height / 2.0, 0.0),
        Vec3::new(0.0, -height / 2.0, 0.0),
    ];
    for k in 0..n {
        let a = core::f32::consts::TAU * k as f32 / n as f32;
        points.push(Vec3::new(radius * a.cos(), 0.0, radius * a.sin()));
    }
    points
}

pub fn d10_mesh_uv(radius: f32, height: f32) -> Mesh {
    let n = 5usize;
    let top = [0.0, height / 2.0, 0.0];
    let bot = [0.0, -height / 2.0, 0.0];

    let mut ring = Vec::new();
    for k in 0..n {
        let a = core::f32::consts::TAU * k as f32 / n as f32;
        ring.push([radius * a.cos(), 0.0, radius * a.sin()]);
    }

    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let w = n as f32;

    for k in 0..n {
        let face = k as f32;
        let u0 = face / w;
        let u1 = (face + 1.0) / w;
        let uc = (face + 0.5) / w;

        positions.push(top);
        positions.push(ring[(k + 1) % n]);
        positions.push(ring[k]);

        uvs.push([uc, 0.0]);
        uvs.push([u0, 1.0]);
        uvs.push([u1, 1.0]);
    }

    for j in 0..n {
        let tile = 9 - (j + 3) % n;
        let u0 = tile as f32 / w;
        let u1 = (tile as f32 + 1.0) / w;
        let uc = (tile as f32 + 0.5) / w;

        positions.push(bot);
        positions.push(ring[j]);
        positions.push(ring[(j + 1) % n]);

        uvs.push([uc, 1.0]);
        uvs.push([u0, 0.0]);
        uvs.push([u1, 0.0]);
    }

    let indices: Vec<u32> = (0..positions.len() as u32).collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.compute_normals();
    mesh
}

// -- helpers for make_d10_texture ----------------------------------------

/// Draw a 1-px-wide anti-aliased line using a simple Bresenham-style walk.
pub(crate) fn draw_line(
    data: &mut [u8],
    stride: u32,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    color: [u8; 4],
) {
    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = -(y1 as i32 - y0 as i32).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0 as i32;
    let mut y = y0 as i32;
    loop {
        let idx = ((y as u32 * stride + x as u32) * 4) as usize;
        if idx + 3 < data.len() {
            data[idx..idx + 4].copy_from_slice(&color);
        }
        if x == x1 as i32 && y == y1 as i32 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// Generate a 10-tile texture atlas with digits 1...10 for the d10 faces.
pub fn make_d10_texture() -> Image {
    let tile_w = 64u32;
    let tile_h = 64u32;
    let w = tile_w * 10;
    let h = tile_h;

    let mut data = vec![0u8; (w * h * 4) as usize];
    for px in data.chunks_exact_mut(4) {
        px.copy_from_slice(&[245, 235, 220, 255]);
    }

    let gray = [180u8, 180, 180, 255];

    for tile in 0..10u32 {
        let ox = tile * tile_w;
        if tile < 5 {
            draw_line(&mut data, w, ox + 32, 0, ox, 63, gray);
            draw_line(&mut data, w, ox + 32, 0, ox + 63, 63, gray);
            draw_line(&mut data, w, ox, 63, ox + 63, 63, gray);
        } else {
            draw_line(&mut data, w, ox, 0, ox + 63, 0, gray);
            draw_line(&mut data, w, ox, 0, ox + 32, 63, gray);
            draw_line(&mut data, w, ox + 63, 0, ox + 32, 63, gray);
        }
    }

    let font: [[u8; 7]; 10] = [
        [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        [
            0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110,
        ],
        [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
    ];
    let scale = 5u32;
    let fw = 5u32;
    let fh = 7u32;
    let rw = fw * scale;
    let rh = fh * scale;

    for tile in 0..10u32 {
        let num = tile + 1;
        let s = num.to_string();
        let chars: Vec<_> = s.bytes().map(|b| (b - b'0') as usize).collect();
        let total_w = chars.len() as u32 * (rw + scale);
        let ox = tile * tile_w + (tile_w - total_w) / 2;
        let oy = (tile_h - rh) / 2;

        for (ci, &digit) in chars.iter().enumerate() {
            let bx = ox + ci as u32 * (rw + scale);
            for row in 0..fh {
                let bits = font[digit][row as usize];
                for col in 0..fw {
                    if bits & (1 << (4 - col)) != 0 {
                        for dy in 0..scale {
                            for dx in 0..scale {
                                let px = bx + col * scale + dx;
                                let py = oy + row * scale + dy;
                                let idx = ((py * w + px) * 4) as usize;
                                if idx + 3 < data.len() {
                                    data[idx] = 0;
                                    data[idx + 1] = 0;
                                    data[idx + 2] = 0;
                                    data[idx + 3] = 255;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Image {
        data: data.into(),
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..Default::default()
    }
}

pub struct DicePlugin;

impl Plugin for DicePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DiceRollResult>()
            .add_systems(Update, (despawn_dice, handle_local_input));
    }
}
