use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;

#[derive(Component, Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum ShipType {
    Starter,
    Interceptor,
    Hauler,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ShipStats {
    pub center_of_mass_local: Vec3,
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
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

impl ShipType {
    pub fn stats(self) -> ShipStats {
        match self {
            ShipType::Starter => ShipStats {
                center_of_mass_local: Vec3::new(0.0, 0.0, 0.0),
                max_speed: 10.0,
                acceleration: 24.0,
                deceleration: 16.0,
                roll_speed: 1.8,
                mouse_sensitivity: 0.006,
                initial_pitch: -0.22,
            },
            ShipType::Interceptor => ShipStats {
                center_of_mass_local: Vec3::new(0.0, 0.0, -0.15),
                max_speed: 16.0,
                acceleration: 38.0,
                deceleration: 22.0,
                roll_speed: 2.8,
                mouse_sensitivity: 0.007,
                initial_pitch: -0.22,
            },
            ShipType::Hauler => ShipStats {
                center_of_mass_local: Vec3::new(0.0, -0.05, 0.25),
                max_speed: 7.0,
                acceleration: 12.0,
                deceleration: 9.0,
                roll_speed: 1.0,
                mouse_sensitivity: 0.004,
                initial_pitch: -0.22,
            },
        }
    }
}

pub fn spawn_player_ship(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    ship_type: ShipType,
) -> Entity {
    let ship_mesh = meshes.add(create_multicolor_prism(Vec3::new(1.8, 0.5, 3.2)));
    let ship_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        ..default()
    });

    let stats = ship_type.stats();

    commands
        .spawn((
            Mesh3d(ship_mesh),
            MeshMaterial3d(ship_material),
            Transform::from_xyz(0.0, -0.4, 0.0),
            PlayerShip,
            ship_type,
            stats,
            ShipMovementState {
                velocity: Vec3::ZERO,
                pitch_angle: stats.initial_pitch,
            },
        ))
        .id()
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
