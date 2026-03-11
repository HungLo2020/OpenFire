mod movement_controller;

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use movement_controller::{MovementControllerPlugin, ShipController};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugins(MovementControllerPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));

    spawn_starfield(&mut commands, &mut meshes, &mut materials);

    let ship_mesh = meshes.add(create_multicolor_prism(Vec3::new(1.8, 0.5, 3.2)));
    let ship_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        ..default()
    });

    let ship_entity = commands
        .spawn((
            Mesh3d(ship_mesh),
            MeshMaterial3d(ship_material),
            Transform::from_xyz(0.0, -0.4, 0.0),
            ShipController {
                max_speed: 10.0,
                acceleration: 24.0,
                deceleration: 16.0,
                velocity: Vec3::ZERO,
                roll_speed: 1.8,
                mouse_sensitivity: 0.006,
                pitch_angle: -0.22,
                pitch_min: -1.2,
                pitch_max: 1.2,
            },
        ))
        .id();

    let test_cube_mesh = meshes.add(Mesh::from(Cuboid::new(0.8, 0.8, 0.8)));
    let test_cube_materials = [
        materials.add(Color::srgb(1.0, 0.4, 0.4)),
        materials.add(Color::srgb(0.4, 1.0, 0.4)),
        materials.add(Color::srgb(0.4, 0.6, 1.0)),
        materials.add(Color::srgb(1.0, 0.9, 0.4)),
    ];
    let test_cube_positions = [
        Vec3::new(3.0, -0.4, -2.0),
        Vec3::new(-3.0, -0.4, -2.5),
        Vec3::new(0.0, 1.2, -4.0),
        Vec3::new(2.0, 0.8, 1.5),
    ];

    for (position, material) in test_cube_positions.into_iter().zip(test_cube_materials.into_iter()) {
        commands.spawn((
            Mesh3d(test_cube_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(position),
        ));
    }

    commands.entity(ship_entity).with_children(|parent| {
        parent.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 1.8, 7.0).looking_at(Vec3::new(0.0, 0.1, -3.0), Vec3::Y),
        ));
    });
}

fn spawn_starfield(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let star_mesh = meshes.add(Mesh::from(Cuboid::new(0.1, 0.1, 0.1)));
    let star_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        emissive: LinearRgba::rgb(2.5, 2.5, 2.5),
        unlit: true,
        ..default()
    });

    for i in 0..900 {
        let fx = hash_01(i as f32 * 11.31 + 1.7) * 2.0 - 1.0;
        let fy = hash_01(i as f32 * 7.97 + 2.9) * 2.0 - 1.0;
        let fz = hash_01(i as f32 * 5.63 + 3.1) * 2.0 - 1.0;

        let dir = Vec3::new(fx, fy, fz).normalize_or_zero();
        if dir == Vec3::ZERO {
            continue;
        }

        let radius = 120.0 + hash_01(i as f32 * 17.21 + 4.3) * 450.0;
        commands.spawn((
            Mesh3d(star_mesh.clone()),
            MeshMaterial3d(star_material.clone()),
            Transform::from_translation(dir * radius),
        ));
    }
}

fn hash_01(seed: f32) -> f32 {
    (seed.sin() * 43_758.547).rem_euclid(1.0)
}

fn create_multicolor_prism(size: Vec3) -> Mesh {
    let hx = size.x * 0.5;
    let hy = size.y * 0.5;
    let hz = size.z * 0.5;

    let positions = vec![
        [-hx, -hy, hz],
        [hx, -hy, hz],
        [hx, hy, hz],
        [-hx, hy, hz],
        [hx, -hy, -hz],
        [-hx, -hy, -hz],
        [-hx, hy, -hz],
        [hx, hy, -hz],
        [-hx, -hy, -hz],
        [-hx, -hy, hz],
        [-hx, hy, hz],
        [-hx, hy, -hz],
        [hx, -hy, hz],
        [hx, -hy, -hz],
        [hx, hy, -hz],
        [hx, hy, hz],
        [-hx, hy, hz],
        [hx, hy, hz],
        [hx, hy, -hz],
        [-hx, hy, -hz],
        [-hx, -hy, -hz],
        [hx, -hy, -hz],
        [hx, -hy, hz],
        [-hx, -hy, hz],
    ];

    let normals = vec![
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
    ];

    let face_colors = [
        [1.0, 0.1, 0.1, 1.0],
        [0.1, 1.0, 0.1, 1.0],
        [0.1, 0.3, 1.0, 1.0],
        [1.0, 0.9, 0.1, 1.0],
        [1.0, 0.2, 0.9, 1.0],
        [0.1, 1.0, 1.0, 1.0],
    ];

    let mut colors = Vec::with_capacity(24);
    for color in face_colors {
        colors.extend([color; 4]);
    }

    let uvs = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ];

    let indices = vec![
        0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16,
        17, 18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}