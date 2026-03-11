mod movement_controller;
mod ship;
mod camera_rig;

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowFocused};
use camera_rig::{spawn_follow_camera, CameraRigPlugin};
use movement_controller::MovementControllerPlugin;
use ship::{spawn_player_ship, PlayerShip, ShipStats, ShipType};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugins(MovementControllerPlugin)
        .add_plugins(CameraRigPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (update_projected_crosshair, recapture_mouse_on_focus))
        .run();
}

    #[derive(Component)]
    struct ProjectedCrosshair;

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

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
        Text::new("+"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        ProjectedCrosshair,
    ));
}

fn update_projected_crosshair(
    window_query: Query<&Window, With<PrimaryWindow>>,
    ship_query: Query<(&GlobalTransform, &ShipStats), With<PlayerShip>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut crosshair_query: Query<&mut Node, With<ProjectedCrosshair>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    let Ok((ship_transform, ship_stats)) = ship_query.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };
    let Ok(mut crosshair_node) = crosshair_query.get_single_mut() else {
        return;
    };

    let (_, ship_rotation, ship_translation) = ship_transform.to_scale_rotation_translation();
    let center_of_mass_world = ship_translation + ship_rotation.mul_vec3(ship_stats.center_of_mass_local);
    let ship_forward = ship_rotation * -Vec3::Z;
    let projected_target = center_of_mass_world + ship_forward * 10_000.0;

    let Ok(screen_position) = camera.world_to_viewport(camera_transform, projected_target) else {
        return;
    };

    let width = window.width();
    let height = window.height();
    let padding = 16.0;
    let clamped_x = screen_position.x.clamp(padding, width - padding);
    let clamped_y = screen_position.y.clamp(padding, height - padding);

    crosshair_node.left = Val::Px(clamped_x - 8.0);
    crosshair_node.top = Val::Px(clamped_y - 14.0);
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

fn recapture_mouse_on_focus(
    mut focus_events: EventReader<WindowFocused>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut regained_focus = false;
    for event in focus_events.read() {
        if event.focused {
            regained_focus = true;
        }
    }

    if regained_focus {
        if let Ok(mut window) = primary_window.get_single_mut() {
            window.cursor_options.visible = false;
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
        }
    }
}
