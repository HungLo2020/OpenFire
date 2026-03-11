use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::app::AppExit;
use crate::ship::{PlayerShip, ShipMovementState, ShipStats};

pub struct MovementControllerPlugin;

impl Plugin for MovementControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                ship_movement_system,
                ship_rotation_system,
                mouse_look_system,
                exit_on_esc_system,
            ),
        );
    }
}

fn ship_movement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ship_query: Query<(&ShipStats, &mut ShipMovementState, &mut Transform), With<PlayerShip>>,
) {
    let Ok((stats, mut movement_state, mut transform)) = ship_query.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    let mut forward_axis = 0.0;
    if keyboard.pressed(KeyCode::KeyW) {
        forward_axis += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        forward_axis -= 1.0;
    }

    let mut right_axis = 0.0;
    if keyboard.pressed(KeyCode::KeyD) {
        right_axis += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        right_axis -= 1.0;
    }

    let mut up_axis = 0.0;
    if keyboard.pressed(KeyCode::Space) {
        up_axis += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyC) {
        up_axis -= 1.0;
    }

    let forward = transform.rotation.mul_vec3(-Vec3::Z);
    let right = transform.rotation.mul_vec3(Vec3::X);
    let up = transform.rotation.mul_vec3(Vec3::Y);

    let mut direction = forward * forward_axis + right * right_axis + up * up_axis;
    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
        movement_state.velocity += direction * stats.acceleration * dt;
    } else {
        let speed = movement_state.velocity.length();
        if speed > 0.0 {
            let decel_step = stats.deceleration * dt;
            if decel_step >= speed {
                movement_state.velocity = Vec3::ZERO;
            } else {
                let velocity_direction = movement_state.velocity.normalize();
                movement_state.velocity -= velocity_direction * decel_step;
            }
        }
    }

    let speed = movement_state.velocity.length();
    if speed > stats.max_speed {
        movement_state.velocity = movement_state.velocity.normalize() * stats.max_speed;
    }

    transform.translation += movement_state.velocity * dt;
}

fn ship_rotation_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ship_query: Query<(&ShipStats, &mut Transform), With<PlayerShip>>,
) {
    let Ok((stats, mut transform)) = ship_query.get_single_mut() else {
        return;
    };

    let mut roll_input = 0.0;
    if keyboard.pressed(KeyCode::KeyQ) {
        roll_input += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        roll_input -= 1.0;
    }

    if roll_input != 0.0 {
        let delta_rotation = Quat::from_rotation_z(roll_input * stats.roll_speed * time.delta_secs());
        rotate_around_center_of_mass(&mut transform, stats.center_of_mass_local, delta_rotation);
    }
}

fn mouse_look_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut ship_query: Query<(&ShipStats, &mut ShipMovementState, &mut Transform), With<PlayerShip>>,
) {
    if keyboard.pressed(KeyCode::AltLeft) {
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    let Ok((stats, mut movement_state, mut ship_transform)) = ship_query.get_single_mut() else {
        return;
    };

    let yaw_delta = Quat::from_rotation_y(-delta.x * stats.mouse_sensitivity);
    rotate_around_center_of_mass(&mut ship_transform, stats.center_of_mass_local, yaw_delta);

    let next_pitch = (movement_state.pitch_angle - delta.y * stats.mouse_sensitivity)
        .clamp(stats.pitch_min, stats.pitch_max);
    let delta_pitch = next_pitch - movement_state.pitch_angle;
    movement_state.pitch_angle = next_pitch;
    let pitch_delta = Quat::from_rotation_x(delta_pitch);
    rotate_around_center_of_mass(&mut ship_transform, stats.center_of_mass_local, pitch_delta);
}

fn rotate_around_center_of_mass(transform: &mut Transform, center_of_mass_local: Vec3, local_delta: Quat) {
    let world_com = transform.translation + transform.rotation.mul_vec3(center_of_mass_local);
    transform.rotation *= local_delta;
    transform.translation = world_com - transform.rotation.mul_vec3(center_of_mass_local);
}

fn exit_on_esc_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit_events: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit_events.send(AppExit::Success);
    }
}
