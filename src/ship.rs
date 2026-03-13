use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use crate::collision::CollisionBox;
use crate::ship_config_store::ShipConfigStore;

pub struct ShipStatsPlugin;

impl Plugin for ShipStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (ship_modifier_debug_input_system, recompute_derived_stats_system));
    }
}

#[derive(Component, Clone, Debug)]
#[allow(dead_code)]
pub struct ShipIdentity {
    pub display_name: String,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ShipBaseStats {
    pub center_of_mass_local: Vec3,
    pub max_speed: f32,
    pub acceleration_forward: f32,
    pub acceleration_backward: f32,
    pub acceleration_right: f32,
    pub acceleration_left: f32,
    pub acceleration_up: f32,
    pub acceleration_down: f32,
    pub roll_speed: f32,
    pub mouse_sensitivity: f32,
    pub initial_pitch: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ShipStatModifiers {
    pub center_of_mass_offset: Vec3,
    pub max_speed_multiplier: f32,
    pub acceleration_forward_multiplier: f32,
    pub acceleration_backward_multiplier: f32,
    pub acceleration_right_multiplier: f32,
    pub acceleration_left_multiplier: f32,
    pub acceleration_up_multiplier: f32,
    pub acceleration_down_multiplier: f32,
    pub roll_speed_multiplier: f32,
    pub mouse_sensitivity_multiplier: f32,
}

impl Default for ShipStatModifiers {
    fn default() -> Self {
        Self {
            center_of_mass_offset: Vec3::ZERO,
            max_speed_multiplier: 1.0,
            acceleration_forward_multiplier: 1.0,
            acceleration_backward_multiplier: 1.0,
            acceleration_right_multiplier: 1.0,
            acceleration_left_multiplier: 1.0,
            acceleration_up_multiplier: 1.0,
            acceleration_down_multiplier: 1.0,
            roll_speed_multiplier: 1.0,
            mouse_sensitivity_multiplier: 1.0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ShipDerivedStats {
    pub center_of_mass_local: Vec3,
    pub max_speed: f32,
    pub acceleration_forward: f32,
    pub acceleration_backward: f32,
    pub acceleration_right: f32,
    pub acceleration_left: f32,
    pub acceleration_up: f32,
    pub acceleration_down: f32,
    pub roll_speed: f32,
    pub mouse_sensitivity: f32,
    pub initial_pitch: f32,
}

#[derive(Component, Debug)]
pub struct ShipMovementState {
    pub velocity: Vec3,
    pub pitch_angle: f32,
}

#[derive(Component)]
pub struct PlayerShip;

pub fn spawn_player_ship(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config_store: &ShipConfigStore,
) -> Entity {
    let ship_mesh = meshes.add(create_multicolor_prism(Vec3::new(1.8, 0.5, 3.2)));
    let ship_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        ..default()
    });

    let config = config_store.get_default();
    let base_stats = ShipBaseStats {
        center_of_mass_local: Vec3::from_array(config.center_of_mass_local),
        max_speed: config.max_speed,
        acceleration_forward: config.acceleration_forward,
        acceleration_backward: config.acceleration_backward,
        acceleration_right: config.acceleration_right,
        acceleration_left: config.acceleration_left,
        acceleration_up: config.acceleration_up,
        acceleration_down: config.acceleration_down,
        roll_speed: config.roll_speed,
        mouse_sensitivity: config.mouse_sensitivity,
        initial_pitch: config.initial_pitch,
    };
    let modifiers = ShipStatModifiers::default();
    let derived = compose_derived_stats(base_stats, modifiers);

    commands
        .spawn((
            Mesh3d(ship_mesh),
            MeshMaterial3d(ship_material),
            Transform::from_xyz(0.0, -0.4, 0.0),
            CollisionBox {
                half_extents: Vec3::new(0.9, 0.25, 1.6),
            },
            PlayerShip,
            ShipIdentity {
                display_name: config.display_name,
            },
            base_stats,
            modifiers,
            derived,
            ShipMovementState {
                velocity: Vec3::ZERO,
                pitch_angle: derived.initial_pitch,
            },
        ))
        .id()
}

fn compose_derived_stats(base: ShipBaseStats, modifiers: ShipStatModifiers) -> ShipDerivedStats {
    ShipDerivedStats {
        center_of_mass_local: base.center_of_mass_local + modifiers.center_of_mass_offset,
        max_speed: (base.max_speed * modifiers.max_speed_multiplier).max(0.0),
        acceleration_forward: (base.acceleration_forward * modifiers.acceleration_forward_multiplier).max(0.0),
        acceleration_backward: (base.acceleration_backward * modifiers.acceleration_backward_multiplier).max(0.0),
        acceleration_right: (base.acceleration_right * modifiers.acceleration_right_multiplier).max(0.0),
        acceleration_left: (base.acceleration_left * modifiers.acceleration_left_multiplier).max(0.0),
        acceleration_up: (base.acceleration_up * modifiers.acceleration_up_multiplier).max(0.0),
        acceleration_down: (base.acceleration_down * modifiers.acceleration_down_multiplier).max(0.0),
        roll_speed: base.roll_speed * modifiers.roll_speed_multiplier,
        mouse_sensitivity: base.mouse_sensitivity * modifiers.mouse_sensitivity_multiplier,
        initial_pitch: base.initial_pitch,
    }
}

fn recompute_derived_stats_system(
    mut ship_query: Query<
        (&ShipBaseStats, &ShipStatModifiers, &mut ShipDerivedStats),
        Or<(Changed<ShipBaseStats>, Changed<ShipStatModifiers>)>,
    >,
) {
    for (base, modifiers, mut derived) in &mut ship_query {
        *derived = compose_derived_stats(*base, *modifiers);
    }
}

fn ship_modifier_debug_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ship_query: Query<&mut ShipStatModifiers, With<PlayerShip>>,
) {
    let Ok(mut modifiers) = ship_query.get_single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::BracketRight) {
        modifiers.acceleration_forward_multiplier += 0.1;
        modifiers.acceleration_backward_multiplier += 0.1;
        modifiers.acceleration_right_multiplier += 0.1;
        modifiers.acceleration_left_multiplier += 0.1;
        modifiers.acceleration_up_multiplier += 0.1;
        modifiers.acceleration_down_multiplier += 0.1;
    }
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        modifiers.acceleration_forward_multiplier = (modifiers.acceleration_forward_multiplier - 0.1).max(0.1);
        modifiers.acceleration_backward_multiplier = (modifiers.acceleration_backward_multiplier - 0.1).max(0.1);
        modifiers.acceleration_right_multiplier = (modifiers.acceleration_right_multiplier - 0.1).max(0.1);
        modifiers.acceleration_left_multiplier = (modifiers.acceleration_left_multiplier - 0.1).max(0.1);
        modifiers.acceleration_up_multiplier = (modifiers.acceleration_up_multiplier - 0.1).max(0.1);
        modifiers.acceleration_down_multiplier = (modifiers.acceleration_down_multiplier - 0.1).max(0.1);
    }
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
