use bevy::prelude::*;

use crate::ship::{PlayerShip, ShipMovementState, ShipStats};

const ALT_DOUBLE_TAP_WINDOW_SECS: f32 = 0.3;

pub struct CameraRigPlugin;

impl Plugin for CameraRigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (camera_orbit_input_system, camera_follow_system));
    }
}

#[derive(Component)]
pub struct FollowCamera {
    pub target: Entity,
    pub offset: Vec3,
    pub look_at_offset: Vec3,
    pub movement_lag_smoothing: f32,
    pub velocity_lag: f32,
    pub orbit_sensitivity: f32,
    pub orbit_pitch_min: f32,
    pub orbit_pitch_max: f32,
}

#[derive(Component, Default)]
pub struct FollowCameraState {
    pub movement_lag_offset: Vec3,
    pub orbit_yaw: f32,
    pub orbit_pitch: f32,
    pub last_alt_press_time: f32,
}

pub fn spawn_follow_camera(commands: &mut Commands, target: Entity) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.6, 8.2).looking_at(Vec3::new(0.0, 1.2, -3.5), Vec3::Y),
        FollowCamera {
            target,
            offset: Vec3::new(0.0, 2.6, 8.2),
            look_at_offset: Vec3::new(0.0, 1.2, -3.5),
            movement_lag_smoothing: 14.0,
            velocity_lag: 0.06,
            orbit_sensitivity: 0.004,
            orbit_pitch_min: -1.2,
            orbit_pitch_max: 1.2,
        },
        FollowCameraState::default(),
    ));
}

fn camera_orbit_input_system(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion_events: EventReader<bevy::input::mouse::MouseMotion>,
    mut camera_query: Query<(&FollowCamera, &mut FollowCameraState)>,
) {
    let now = time.elapsed_secs();

    for (_follow, mut state) in &mut camera_query {
        if keyboard.just_pressed(KeyCode::AltLeft) {
            if now - state.last_alt_press_time <= ALT_DOUBLE_TAP_WINDOW_SECS {
                state.orbit_yaw = 0.0;
                state.orbit_pitch = 0.0;
            }
            state.last_alt_press_time = now;
        }
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }

    if !keyboard.pressed(KeyCode::AltLeft) || delta == Vec2::ZERO {
        return;
    }

    for (follow, mut state) in &mut camera_query {
        state.orbit_yaw -= delta.x * follow.orbit_sensitivity;
        state.orbit_pitch = (state.orbit_pitch - delta.y * follow.orbit_sensitivity)
            .clamp(follow.orbit_pitch_min, follow.orbit_pitch_max);
    }
}

fn camera_follow_system(
    time: Res<Time>,
    mut camera_query: Query<
        (&FollowCamera, &mut FollowCameraState, &mut Transform),
        (With<Camera3d>, Without<PlayerShip>),
    >,
    ship_query: Query<(&Transform, &ShipMovementState, &ShipStats), (With<PlayerShip>, Without<FollowCamera>)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (follow, mut camera_state, mut camera_transform) in &mut camera_query {
        let Ok((ship_transform, movement_state, _stats)) = ship_query.get(follow.target) else {
            continue;
        };

        let orbit_rotation = Quat::from_rotation_y(camera_state.orbit_yaw)
            * Quat::from_rotation_x(camera_state.orbit_pitch);
        let offset_with_orbit = orbit_rotation.mul_vec3(follow.offset);

        let base_position = ship_transform.translation + ship_transform.rotation.mul_vec3(offset_with_orbit);
        let target_movement_lag = -movement_state.velocity * follow.velocity_lag;
        let lag_alpha = 1.0 - (-follow.movement_lag_smoothing * dt).exp();
        camera_state.movement_lag_offset = camera_state
            .movement_lag_offset
            .lerp(target_movement_lag, lag_alpha);

        let desired_position = base_position + camera_state.movement_lag_offset;

        let desired_look_at = ship_transform.translation + ship_transform.rotation.mul_vec3(follow.look_at_offset);
        let target_up = ship_transform.rotation.mul_vec3(Vec3::Y);

        camera_transform.translation = desired_position;

        let desired_rotation = Transform::from_translation(desired_position)
            .looking_at(desired_look_at, target_up)
            .rotation;
        camera_transform.rotation = desired_rotation;
    }
}
