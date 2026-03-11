use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::app::AppExit;

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

#[derive(Component)]
pub struct ShipController {
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub velocity: Vec3,
    pub roll_speed: f32,
    pub mouse_sensitivity: f32,
    pub pitch_angle: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
}

fn ship_movement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ship_query: Query<(&mut ShipController, &mut Transform)>,
) {
    let Ok((mut controller, mut transform)) = ship_query.get_single_mut() else {
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
        let acceleration = controller.acceleration;
        controller.velocity += direction * acceleration * dt;
    } else {
        let speed = controller.velocity.length();
        if speed > 0.0 {
            let decel_step = controller.deceleration * dt;
            if decel_step >= speed {
                controller.velocity = Vec3::ZERO;
            } else {
                let velocity_direction = controller.velocity.normalize();
                controller.velocity -= velocity_direction * decel_step;
            }
        }
    }

    let speed = controller.velocity.length();
    if speed > controller.max_speed {
        controller.velocity = controller.velocity.normalize() * controller.max_speed;
    }

    transform.translation += controller.velocity * dt;
}

fn ship_rotation_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ship_query: Query<(&ShipController, &mut Transform)>,
) {
    let Ok((controller, mut transform)) = ship_query.get_single_mut() else {
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
        transform.rotate_local_z(roll_input * controller.roll_speed * time.delta_secs());
    }
}

fn mouse_look_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut ship_query: Query<(&mut ShipController, &mut Transform)>,
) {
    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    let Ok((mut controller, mut ship_transform)) = ship_query.get_single_mut() else {
        return;
    };

    ship_transform.rotate_y(-delta.x * controller.mouse_sensitivity);

    let next_pitch = (controller.pitch_angle - delta.y * controller.mouse_sensitivity)
        .clamp(controller.pitch_min, controller.pitch_max);
    let delta_pitch = next_pitch - controller.pitch_angle;
    controller.pitch_angle = next_pitch;
    ship_transform.rotate_local_x(delta_pitch);
}

fn exit_on_esc_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit_events: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit_events.send(AppExit::Success);
    }
}
