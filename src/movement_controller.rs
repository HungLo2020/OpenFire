use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::app::AppExit;
use crate::ship::{PlayerShip, ShipDerivedStats, ShipMovementState};

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
    mut ship_query: Query<(&ShipDerivedStats, &mut ShipMovementState, &mut Transform), With<PlayerShip>>,
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

    let mut local_velocity = Vec3::new(
        movement_state.velocity.dot(right),
        movement_state.velocity.dot(up),
        movement_state.velocity.dot(forward),
    );

    if forward_axis > 0.0 {
        local_velocity.z += stats.acceleration_forward * dt;
    } else if forward_axis < 0.0 {
        local_velocity.z -= stats.acceleration_backward * dt;
    }

    if right_axis > 0.0 {
        local_velocity.x += stats.acceleration_right * dt;
    } else if right_axis < 0.0 {
        local_velocity.x -= stats.acceleration_left * dt;
    }

    if up_axis > 0.0 {
        local_velocity.y += stats.acceleration_up * dt;
    } else if up_axis < 0.0 {
        local_velocity.y -= stats.acceleration_down * dt;
    }

    if forward_axis == 0.0 {
        let decel = if local_velocity.z > 0.0 {
            stats.acceleration_backward * 0.5
        } else {
            stats.acceleration_forward * 0.5
        };
        local_velocity.z = approach_zero(local_velocity.z, decel * dt);
    }

    if right_axis == 0.0 {
        let decel = if local_velocity.x > 0.0 {
            stats.acceleration_left * 0.5
        } else {
            stats.acceleration_right * 0.5
        };
        local_velocity.x = approach_zero(local_velocity.x, decel * dt);
    }

    if up_axis == 0.0 {
        let decel = if local_velocity.y > 0.0 {
            stats.acceleration_down * 0.5
        } else {
            stats.acceleration_up * 0.5
        };
        local_velocity.y = approach_zero(local_velocity.y, decel * dt);
    }

    movement_state.velocity = right * local_velocity.x + up * local_velocity.y + forward * local_velocity.z;

    let speed = movement_state.velocity.length();
    if speed > stats.max_speed {
        movement_state.velocity = movement_state.velocity.normalize() * stats.max_speed;
    }

    transform.translation += movement_state.velocity * dt;
}

fn ship_rotation_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ship_query: Query<(&ShipDerivedStats, &mut Transform), With<PlayerShip>>,
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
    mut ship_query: Query<(&ShipDerivedStats, &mut ShipMovementState, &mut Transform), With<PlayerShip>>,
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

    let next_pitch = movement_state.pitch_angle - delta.y * stats.mouse_sensitivity;
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

fn approach_zero(value: f32, max_step: f32) -> f32 {
    if value > 0.0 {
        (value - max_step).max(0.0)
    } else if value < 0.0 {
        (value + max_step).min(0.0)
    } else {
        0.0
    }
}

fn exit_on_esc_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit_events: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit_events.send(AppExit::Success);
    }
}
