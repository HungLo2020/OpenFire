mod movement_controller;
mod ship;
mod camera_rig;

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use camera_rig::{spawn_follow_camera, CameraRigPlugin};
use movement_controller::MovementControllerPlugin;
use ship::{spawn_player_ship, ShipType};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugins(MovementControllerPlugin)
        .add_plugins(CameraRigPlugin)
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

    let ship_entity = spawn_player_ship(&mut commands, &mut meshes, &mut materials, ShipType::Starter);

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

    spawn_follow_camera(&mut commands, ship_entity);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_child((
            Text::new("+"),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
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
