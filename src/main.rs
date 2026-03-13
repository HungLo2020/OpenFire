mod camera_rig;
mod collision;
mod game_state;
mod movement_controller;
mod ship;
mod ship_config_store;

use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::transform::TransformSystem;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowFocused};
use camera_rig::{spawn_follow_camera, CameraRigPlugin};
use collision::{raycast_collision_boxes, CollisionBox};
use game_state::GameScreen;
use movement_controller::MovementControllerPlugin;
use ship::{spawn_player_ship, PlayerShip, ShipDerivedStats, ShipStatsPlugin};
use ship_config_store::{ShipConfigStore, ShipConfigStorePlugin};

#[derive(Resource, Default)]
struct WorldSpawned(bool);

#[derive(Component)]
struct MainMenuUi;

#[derive(Component)]
struct PauseMenuUi;

#[derive(Component)]
struct ProjectedCrosshair;

#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct ExitGameButton;

#[derive(Component)]
struct ResumeButton;

#[derive(Component)]
struct BackToMenuButton;

#[derive(Component)]
struct LaserProjectile {
    velocity: Vec3,
    lifetime: Timer,
}

#[derive(Resource)]
struct LaserAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WorldSpawned::default())
        .add_plugins(DefaultPlugins)
        .init_state::<GameScreen>()
        .add_plugins(ShipConfigStorePlugin)
        .add_plugins(ShipStatsPlugin)
        .add_plugins(MovementControllerPlugin)
        .add_plugins(CameraRigPlugin)
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameScreen::MainMenu), show_main_menu)
        .add_systems(OnExit(GameScreen::MainMenu), hide_main_menu)
        .add_systems(OnEnter(GameScreen::InGame), enter_in_game)
        .add_systems(OnEnter(GameScreen::Paused), show_pause_menu)
        .add_systems(OnExit(GameScreen::Paused), hide_pause_menu)
        .add_systems(Update, main_menu_input.run_if(in_state(GameScreen::MainMenu)))
        .add_systems(Update, main_menu_button_system.run_if(in_state(GameScreen::MainMenu)))
        .add_systems(Update, toggle_pause_input.run_if(in_state(GameScreen::InGame)))
        .add_systems(Update, fire_laser_system.run_if(in_state(GameScreen::InGame)))
        .add_systems(Update, update_lasers_system.run_if(in_state(GameScreen::InGame)))
        .add_systems(Update, unpause_input.run_if(in_state(GameScreen::Paused)))
        .add_systems(Update, pause_menu_button_system.run_if(in_state(GameScreen::Paused)))
        .add_systems(Update, recapture_mouse_on_focus.run_if(in_state(GameScreen::InGame)))
        .add_systems(
            PostUpdate,
            update_projected_crosshair
                .after(TransformSystem::TransformPropagate)
                .run_if(in_state(GameScreen::InGame)),
        )
        .run();
}

fn setup(mut commands: Commands, mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }

    commands.spawn(Camera2d);
}

fn show_main_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            MainMenuUi,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("OPENFIRE"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(260.0),
                        height: Val::Px(52.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.16, 0.16)),
                    PlayButton,
                ))
                .with_child((
                    Text::new("Play"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(260.0),
                        height: Val::Px(52.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.16, 0.16)),
                    ExitGameButton,
                ))
                .with_child((
                    Text::new("Exit Game"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
        });
}

fn hide_main_menu(mut commands: Commands, menu_query: Query<Entity, With<MainMenuUi>>) {
    for entity in &menu_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn main_menu_input(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameScreen>>) {
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameScreen::InGame);
    }
}

fn main_menu_button_system(
    mut interaction_query: Query<
        (&Interaction, Option<&PlayButton>, Option<&ExitGameButton>),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameScreen>>,
    mut exit_events: EventWriter<AppExit>,
) {
    for (interaction, play, exit) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if play.is_some() {
            next_state.set(GameScreen::InGame);
        }
        if exit.is_some() {
            exit_events.send(AppExit::Success);
        }
    }
}

fn enter_in_game(
    mut commands: Commands,
    mut world_spawned: ResMut<WorldSpawned>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    laser_assets: Option<Res<LaserAssets>>,
    ship_config_store: Res<ShipConfigStore>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut crosshair_query: Query<&mut Visibility, With<ProjectedCrosshair>>,
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    for mut visibility in &mut crosshair_query {
        *visibility = Visibility::Visible;
    }

    if laser_assets.is_none() {
        let mesh = meshes.add(Mesh::from(Cylinder::new(0.03, 1.2)));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0),
            emissive: LinearRgba::rgb(1.5, 0.0, 0.0),
            unlit: true,
            ..default()
        });
        commands.insert_resource(LaserAssets { mesh, material });
    }

    if world_spawned.0 {
        return;
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));

    spawn_starfield(&mut commands, &mut meshes, &mut materials);

    let ship_entity = spawn_player_ship(&mut commands, &mut meshes, &mut materials, &ship_config_store);

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
            CollisionBox {
                half_extents: Vec3::splat(0.4),
            },
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
        Visibility::Visible,
    ));

    world_spawned.0 = true;
}

fn toggle_pause_input(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameScreen>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameScreen::Paused);
    }
}

fn unpause_input(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameScreen>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameScreen::InGame);
    }
}

fn show_pause_menu(
    mut commands: Commands,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut crosshair_query: Query<&mut Visibility, With<ProjectedCrosshair>>,
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }

    for mut visibility in &mut crosshair_query {
        *visibility = Visibility::Hidden;
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(14.0),
                ..default()
            },
            PauseMenuUi,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Paused"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(260.0),
                        height: Val::Px(52.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.16, 0.16)),
                    ResumeButton,
                ))
                .with_child((
                    Text::new("Resume"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(260.0),
                        height: Val::Px(52.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.16, 0.16)),
                    BackToMenuButton,
                ))
                .with_child((
                    Text::new("Main Menu"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
        });
}

fn hide_pause_menu(mut commands: Commands, pause_query: Query<Entity, With<PauseMenuUi>>) {
    for entity in &pause_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn pause_menu_button_system(
    mut interaction_query: Query<
        (&Interaction, Option<&ResumeButton>, Option<&BackToMenuButton>),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameScreen>>,
) {
    for (interaction, resume, back_to_menu) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if resume.is_some() {
            next_state.set(GameScreen::InGame);
        }
        if back_to_menu.is_some() {
            next_state.set(GameScreen::MainMenu);
        }
    }
}

fn update_projected_crosshair(
    window_query: Query<&Window, With<PrimaryWindow>>,
    ship_query: Query<(Entity, &GlobalTransform, &ShipDerivedStats), With<PlayerShip>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    colliders_query: Query<(Entity, &CollisionBox, &GlobalTransform)>,
    mut crosshair_query: Query<&mut Node, With<ProjectedCrosshair>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    let Ok((ship_entity, ship_transform, ship_stats)) = ship_query.get_single() else {
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
    let max_distance = 10_000.0;
    let projected_target = raycast_collision_boxes(
        center_of_mass_world,
        ship_forward,
        max_distance,
        Some(ship_entity),
        colliders_query.iter(),
    )
    .map(|hit| hit.world_position)
    .unwrap_or(center_of_mass_world + ship_forward * max_distance);

    let Ok(screen_position) = camera.world_to_viewport(camera_transform, projected_target) else {
        return;
    };

    let width = window.width();
    let height = window.height();
    let padding = 16.0;
    let clamped_x = screen_position.x.clamp(padding, width - padding);
    let clamped_y = screen_position.y.clamp(padding, height - padding);

    crosshair_node.left = Val::Px((clamped_x - 8.0).round());
    crosshair_node.top = Val::Px((clamped_y - 14.0).round());
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

fn fire_laser_system(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    ship_query: Query<(&Transform, &ShipDerivedStats), With<PlayerShip>>,
    laser_assets: Option<Res<LaserAssets>>,
) {
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(laser_assets) = laser_assets else {
        return;
    };

    let Ok((ship_transform, ship_stats)) = ship_query.get_single() else {
        return;
    };

    let forward = ship_transform.rotation.mul_vec3(-Vec3::Z).normalize_or_zero();
    if forward == Vec3::ZERO {
        return;
    }

    let center_of_mass_world = ship_transform.translation
        + ship_transform.rotation.mul_vec3(ship_stats.center_of_mass_local);
    let spawn_position = center_of_mass_world + forward * 1.2;
    let rotation = Quat::from_rotation_arc(Vec3::Y, forward);

    commands.spawn((
        Mesh3d(laser_assets.mesh.clone()),
        MeshMaterial3d(laser_assets.material.clone()),
        Transform {
            translation: spawn_position,
            rotation,
            ..default()
        },
        LaserProjectile {
            velocity: forward * 120.0,
            lifetime: Timer::from_seconds(1.2, TimerMode::Once),
        },
    ));
}

fn update_lasers_system(
    mut commands: Commands,
    time: Res<Time>,
    mut laser_query: Query<(Entity, &mut Transform, &mut LaserProjectile)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut laser) in &mut laser_query {
        transform.translation += laser.velocity * dt;
        laser.lifetime.tick(time.delta());
        if laser.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
