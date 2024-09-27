use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use bundles::WarningSignBundle;
use components::{
    Arrow, Enemy, GameOverText, Player, PowerShotCooldownUI, Score, ThirdPersonCamera,
    Ui2DComponent, Ui2DText, WarningSign, WarningSignAnimation,
};
use rand::Rng;
use resources::PowerShotCooldownTimer;
use states::GameState;

mod bundles;
mod components;
mod resources;
mod states;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
struct CooldownUiMaterial {
    #[uniform(0)]
    color: Vec4,
}

impl UiMaterial for CooldownUiMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/circle_shader.wgsl".into()
    }
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
struct CrosshairUiMaterial {
    #[uniform(0)]
    color: Vec4,
}

impl UiMaterial for CrosshairUiMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/crosshair_shader.wgsl".into()
    }
}

// Refactor: How to detect maximum screen resolution specific to the device?
const MAX_WIDTH: f32 = 1536.0;
const MAX_HEIGHT: f32 = 792.8;
const MIN_WIDTH: f32 = 1080.0;
const MIN_HEIGHT: f32 = MIN_WIDTH / (MAX_WIDTH / MAX_HEIGHT);

fn main() {
    // Set timer to be finished
    let mut power_shot_cooldown_timer =
        PowerShotCooldownTimer(Timer::from_seconds(3.0, TimerMode::Once));
    power_shot_cooldown_timer
        .0
        .tick(power_shot_cooldown_timer.0.duration());

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Archery".into(),
                resolution: (MAX_WIDTH, MAX_HEIGHT).into(),
                resize_constraints: WindowResizeConstraints {
                    min_width: MIN_WIDTH,
                    min_height: MIN_HEIGHT,
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins((
            UiMaterialPlugin::<CooldownUiMaterial>::default(),
            UiMaterialPlugin::<CrosshairUiMaterial>::default(),
        ))
        .insert_resource(power_shot_cooldown_timer)
        .init_state::<GameState>()
        .add_systems(Startup, (setup, setup_player_score, game_over))
        .add_systems(
            Update,
            (
                move_player,
                (player_firing_arrows, player_arrow_charging, player_shoot).chain(),
                arrow_movement,
                update_power_cooldown_ui,
                random_spawn_enemies,
                hit_collision,
                update_player_score,
                enemies_walking,
                check_enemy_proximity,
                update_warning_positions,
                remove_far_warning_signs,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, play_again.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, update_ui_2d)
        .add_systems(
            Update,
            (setup, setup_player_score, game_over)
                .chain()
                .run_if(in_state(GameState::Restarting)),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut cooldown_ui_materials: ResMut<Assets<CooldownUiMaterial>>,
    mut crosshair_ui_materials: ResMut<Assets<CrosshairUiMaterial>>,
) {
    // Set cooldowns
    let mut shoot_cooldown = Timer::from_seconds(0.75, TimerMode::Once);
    shoot_cooldown.tick(shoot_cooldown.duration());

    // Player
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.5, 0.5, 0.5)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.0, 0.25, 0.0),
            ..default()
        },
        Player {
            is_charging: false,
            charge_timer: Timer::from_seconds(1.0, TimerMode::Once),
            max_charge_duration: 1.0,
            should_start_charge: false,
            shoot_cooldown,
            score: 0,
        },
    ));

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    let image_handle = asset_server.load("images/power_shot.png");
    commands
        .spawn((
            ImageBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    height: Val::Px(100.0),
                    width: Val::Px(100.0),
                    left: Val::Px(500.0),
                    bottom: Val::Px(25.0),
                    ..Default::default()
                },
                image: UiImage {
                    texture: image_handle,
                    ..Default::default()
                },
                ..Default::default()
            },
            BorderRadius {
                top_left: Val::Px(50.0),
                top_right: Val::Px(50.0),
                bottom_left: Val::Px(50.0),
                bottom_right: Val::Px(50.0),
            },
            Ui2DComponent::new(Vec2::new(500.0, 25.0), Vec2::new(100.0, 100.0)),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    MaterialNodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            width: Val::Px(105.),
                            height: Val::Px(105.),
                            ..default()
                        },
                        material: cooldown_ui_materials.add(CooldownUiMaterial {
                            color: [0.15, 0.15, 0.15, 1.].into(),
                        }),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    Ui2DComponent::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
                    PowerShotCooldownUI {},
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                position_type: PositionType::Relative,
                                ..default()
                            },
                            text: Text {
                                sections: vec![TextSection {
                                    value: "".to_string(),
                                    ..Default::default()
                                }],
                                ..Default::default()
                            },
                            visibility: Visibility::Hidden,
                            ..Default::default()
                        },
                        Ui2DComponent::new(Vec2::new(42.5, -33.0), Vec2::new(100.0, 100.0)),
                        Ui2DText { font_size: 30.0 },
                        PowerShotCooldownUI {},
                    ));
                });
        });

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.5, 1.0, 3.0),
            ..default()
        },
        ThirdPersonCamera {
            offset: Vec3::new(0.5, 0.5, 3.0), // Offset matches the camera position
        },
    ));

    // Plane
    let plane = Plane3d::new(Vec3::new(0.0, 1.0, 0.0), Vec2::new(2.0, 2.0));

    // Top left plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(plane),
        material: materials.add(Color::srgb_u8(255, 0, 0)),
        transform: Transform::from_xyz(3.0, 0.0, 1.5),
        ..default()
    });

    // Top right plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(plane),
        material: materials.add(Color::srgb_u8(0, 255, 0)),
        transform: Transform::from_xyz(-1.0, 0.0, 1.5),
        ..default()
    });

    // Bottom left plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(plane),
        material: materials.add(Color::srgb_u8(0, 0, 255)),
        transform: Transform::from_xyz(3.0, 0.0, -2.5),
        ..default()
    });

    // Bottom right plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(plane),
        material: materials.add(Color::srgb_u8(255, 255, 0)),
        transform: Transform::from_xyz(-1.0, 0.0, -2.5),
        ..default()
    });

    // Crosshair
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..Default::default()
            },
            Ui2DComponent::new(Vec2::new(725.0, 325.0), Vec2::new(100.0, 100.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                MaterialNodeBundle {
                    style: Style {
                        position_type: PositionType::Relative,
                        width: Val::Px(50.),
                        height: Val::Px(50.),
                        ..default()
                    },
                    material: crosshair_ui_materials.add(CrosshairUiMaterial {
                        color: [0.5, 0.5, 0.5, 0.5].into(),
                    }),
                    ..default()
                },
                Ui2DComponent::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
            ));
        });
}

fn move_player(
    mut player_query: Query<(&mut Transform, &Player), Without<Camera>>,
    mut camera_query: Query<(&mut Transform, &ThirdPersonCamera), Without<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // Player movement variables
    let player_speed = 2.5;
    let rotation_speed = 1.0; // Rotation speed in radians
    let (mut player_transform, player) = player_query.single_mut();

    // The point we will rotate and look at
    let mut look_at_point = Vec3::new(0.5, 0.5, 0.0);

    // Handle player rotation using ArrowLeft (left) and ArrowRight (right)
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        player_transform.rotate(Quat::from_rotation_y(rotation_speed * time.delta_seconds()));
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        player_transform.rotate(Quat::from_rotation_y(
            -rotation_speed * time.delta_seconds(),
        ));
    }

    // Rotate the look_at_point the same way as the player
    let rotation_quat = player_transform.rotation;
    look_at_point = rotation_quat.mul_vec3(look_at_point);

    // Calculate forward vector based on player rotation (looking direction)
    let forward = player_transform.forward();

    // Handle player forward/backward movement using ArrowUp and ArrowDown
    if keyboard_input.pressed(KeyCode::ArrowUp) && !player.is_charging {
        player_transform.translation += forward * player_speed * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) && !player.is_charging {
        player_transform.translation -= forward * player_speed * time.delta_seconds();
    }

    // Camera logic
    let (mut camera_transform, camera) = camera_query.single_mut();

    // Maintain camera's offset relative to the player
    let rotated_offset = rotation_quat.mul_vec3(camera.offset);

    // Set the camera's position based on player's position and rotated offset
    camera_transform.translation = player_transform.translation + rotated_offset;

    // Ensure the camera is looking at the rotated look_at_point
    camera_transform.look_at(player_transform.translation + look_at_point, Vec3::Y);
}

fn player_firing_arrows(
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut players: Query<&mut Player>,
    time: Res<Time>,
    mut power_shot_cooldown_timer: ResMut<PowerShotCooldownTimer>,
) {
    let mut player = players.single_mut();

    // If cooldowns are not finished, tick the timers
    if !player.shoot_cooldown.finished() {
        player.shoot_cooldown.tick(time.delta());
    }
    if !power_shot_cooldown_timer.0.finished() {
        power_shot_cooldown_timer.0.tick(time.delta());
    }

    // If the space key is held down and the player is charging, set the flag
    if keyboard_input.clear_just_pressed(KeyCode::Space) {
        // Set the charge flag even if cooldown is active
        if !player.shoot_cooldown.finished() || !power_shot_cooldown_timer.0.finished() {
            player.should_start_charge = true; // Keep track of the pending charge
        } else {
            // Start charging if cooldowns are done
            player.is_charging = true;
            player.charge_timer.reset();
        }
    }

    // If the space key is released, stop charging and reset the flag
    if keyboard_input.clear_just_released(KeyCode::Space) {
        player.is_charging = false;
        player.should_start_charge = false; // Reset pending charge flag
    }

    // Automatically start charging if the cooldowns finish and the spacebar is still held
    if player.shoot_cooldown.finished() && power_shot_cooldown_timer.0.finished() {
        if player.should_start_charge {
            // Start charging automatically after cooldown finishes
            player.is_charging = true;
            player.charge_timer.reset();
            player.should_start_charge = false; // Clear the flag
        }
    }
}

fn player_arrow_charging(
    mut players: Query<(&mut Player, &Handle<StandardMaterial>)>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (mut player, material_handle) = players.single_mut();
    let base_color = Color::srgb_u8(124, 144, 255);
    let material = materials.get_mut(material_handle).unwrap();

    if player.is_charging {
        if player.charge_timer.elapsed_secs() < player.max_charge_duration {
            player.charge_timer.tick(time.delta());
            let charge_ratio = player
                .charge_timer
                .elapsed_secs()
                .min(player.max_charge_duration)
                / player.max_charge_duration;
            let glow_intensity = 0.299 * 124.0 / 255.0
                + 0.587 * 144.0 / 255.0
                + 0.114 * 255.0 / 255.0
                + charge_ratio;
            let glowing_color = base_color.with_luminance(glow_intensity);
            material.base_color = glowing_color;
        } else {
            material.base_color = base_color.with_luminance(2.0);
        }
    } else {
        material.base_color = Color::srgb_u8(124, 144, 255);
    }
}

fn player_shoot(
    mut commands: Commands,
    mut player: Query<(&Transform, &mut Player), Without<Camera>>,
    camera: Query<(&Transform, &Camera, &ThirdPersonCamera), Without<Player>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut power_shot_cooldown_timer: ResMut<PowerShotCooldownTimer>,
) {
    let (player_transform, mut player) = player.single_mut();
    let (camera_transform, _, _) = camera.single();

    // Get the player's position and rotation
    let player_translation = player_transform.translation;
    let player_rotation = player_transform.rotation;

    // Offset to spawn the projectile in front of the player
    let spawn_offset = player_rotation * -Vec3::Z; // Forward direction (assuming Z is forward)
    let spawn_position = player_translation + spawn_offset * 1.0; // Adjust the multiplier for distance from the player

    if !player.is_charging && player.charge_timer.elapsed_secs() > 0.0 {
        // Calculate strength based on charge time
        let charge_time = player
            .charge_timer
            .elapsed_secs()
            .min(player.max_charge_duration);
        let strength = charge_time / player.max_charge_duration;

        // Define color and shape based on charge level
        let (projectile_color, projectile_mesh): (Color, Handle<Mesh>) = if strength < 0.33 {
            (
                Color::srgb_u8(0, 0, 255),
                meshes.add(Mesh::from(Sphere { radius: 0.2 })),
            )
        } else if strength < 0.66 {
            (
                Color::srgb_u8(0, 255, 0),
                meshes.add(Mesh::from(Cuboid {
                    half_size: Vec3::splat(0.1),
                })),
            )
        } else {
            (
                Color::srgb_u8(255, 0, 0),
                meshes.add(Mesh::from(Torus {
                    minor_radius: 0.1,
                    major_radius: 0.2,
                })),
            )
        };

        let camera_forward = camera_transform.forward();
        let camera_right = camera_transform.right();

        // Adjust by adding a small amount of the right direction
        let adjusted_forward =
            Dir3::new_unchecked((camera_forward.as_vec3() + camera_right * 0.03).normalize());

        // Spawn the projectile at the calculated position
        commands
            .spawn(PbrBundle {
                mesh: projectile_mesh,
                material: materials.add(projectile_color),
                transform: Transform {
                    translation: spawn_position,
                    rotation: Quat::from_rotation_arc(Vec3::Z, *adjusted_forward), // Set rotation to face the camera's forward direction
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Arrow {
                speed: 10.0 * (1.0 / strength), // Adjust speed based on strength
                range: 20.0,
                direction: *adjusted_forward, // Use the camera's forward direction instead of spawn_offset
                distance_travelled: 0.0,
                strength,
            });

        player.charge_timer.reset();
        player.shoot_cooldown.reset();

        if strength >= 0.66 {
            power_shot_cooldown_timer.0.reset();
        }
    }
}

fn arrow_movement(
    mut commands: Commands,
    mut arrows: Query<(Entity, &mut Transform, &mut Arrow)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut arrow) in arrows.iter_mut() {
        transform.translation += arrow.direction * arrow.speed * time.delta_seconds();
        arrow.distance_travelled += arrow.speed * time.delta_seconds();

        if arrow.distance_travelled >= arrow.range {
            commands.entity(entity).despawn();
        }
    }
}

fn update_power_cooldown_ui(
    mut cooldown_ui: Query<(&mut Visibility, &PowerShotCooldownUI)>,
    mut cooldown_text: Query<(&mut Text, &PowerShotCooldownUI)>,
    power_shot_cooldown_timer: ResMut<PowerShotCooldownTimer>,
) {
    let cooldown_timer = &power_shot_cooldown_timer.0;

    if cooldown_ui.iter().count() == 0 {
        return;
    }

    for (mut visibility, _) in cooldown_ui.iter_mut() {
        if cooldown_timer.remaining_secs() > 0.0 {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Visible;
        }
    }

    if power_shot_cooldown_timer.0.remaining_secs() > 0.0 {
        let (mut text, _) = cooldown_text.single_mut();
        text.sections[0].value = format!("{:.0}", power_shot_cooldown_timer.0.remaining_secs());
    } else {
        for (mut visibility, _) in cooldown_ui.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}

fn update_ui_2d(
    mut ui_2d_icons: Query<(&mut Style, &Ui2DComponent), Without<Ui2DText>>,
    mut ui_2d_text: Query<(&mut Style, &mut Text, &Ui2DComponent, &Ui2DText)>,
    window: Query<&Window>,
) {
    let mut current_window_width = MIN_WIDTH;
    let mut current_window_height = MIN_HEIGHT;

    for window in window.iter() {
        current_window_width = window.width();
        current_window_height = window.height();
    }

    // Loop through all UI components
    for (mut style, ui_component) in ui_2d_icons.iter_mut() {
        // Adjust position and size using the scaling factors
        style.left = Val::Px((current_window_width / MAX_WIDTH) * ui_component.base_position.x);
        style.bottom = Val::Px((current_window_height / MAX_HEIGHT) * ui_component.base_position.y);
        style.width = Val::Px((current_window_height / MAX_HEIGHT) * ui_component.base_size.x);
        style.height = Val::Px((current_window_height / MAX_HEIGHT) * ui_component.base_size.y);
    }

    // Loop through all UI text components
    for (mut style, mut text, ui_component, ui_text) in ui_2d_text.iter_mut() {
        // Adjust position and size using the scaling factors
        style.left = Val::Px((current_window_width / MAX_WIDTH) * ui_component.base_position.x);
        style.bottom = Val::Px((current_window_height / MAX_HEIGHT) * ui_component.base_position.y);
        style.width = Val::Px((current_window_height / MAX_HEIGHT) * ui_component.base_size.x);
        style.height = Val::Px((current_window_height / MAX_HEIGHT) * ui_component.base_size.y);

        // Adjust font size using the scaling factor
        text.sections[0].style.font_size = ui_text.font_size * (current_window_height / MAX_HEIGHT);
    }
}

fn random_spawn_enemies(
    mut commands: Commands,
    player: Query<(&Transform, &Player)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    let (player_transform, _) = player.single();
    let player_translation = player_transform.translation;
    let mut rng = rand::thread_rng();
    let spawn_interval = 2.0;
    let spawn_chance = 0.5;

    if time.elapsed_seconds() % spawn_interval < time.delta_seconds() {
        if rng.gen::<f32>() < spawn_chance {
            let enemy_color = Color::srgb(rng.gen(), rng.gen(), rng.gen());
            let enemy_mesh = meshes.add(Mesh::from(Cuboid::new(0.5, 0.5, 0.5)));
            let min_distance = 1.0;
            let max_distance = 20.0;
            let angle = rng.gen_range(0.0..std::f32::consts::TAU); // Random angle in radians
            let distance = rng.gen_range(min_distance..max_distance); // Random distance between 5 and max

            let enemy_position = Vec3::new(
                player_translation.x + distance * angle.cos(),
                0.25,
                player_translation.z + distance * angle.sin(),
            );

            commands.spawn((
                PbrBundle {
                    mesh: enemy_mesh,
                    material: materials.add(enemy_color),
                    transform: Transform::from_translation(enemy_position),
                    ..Default::default()
                },
                Enemy {},
            ));
        }
    }
}

fn hit_collision(
    mut commands: Commands,
    mut arrows: Query<(Entity, &Transform, &Arrow)>,
    mut enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut player: Query<(&mut Player, &Transform)>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut game_over_text: Query<(&mut Visibility, &GameOverText)>,
) {
    let (mut player, player_transform) = player.single_mut();
    for (arrow_entity, arrow_transform, _) in arrows.iter_mut() {
        for (enemy_entity, enemy_transform) in enemies.iter_mut() {
            // Distance for collision could be adjusted
            if arrow_transform
                .translation
                .distance(enemy_transform.translation)
                < 1.0
            {
                player.score += 1;
                commands.entity(arrow_entity).despawn();
                commands.entity(enemy_entity).despawn();
            }
        }
    }

    for (_, enemy_transform) in enemies.iter_mut() {
        // Distance for collision could be adjusted
        if player_transform
            .translation
            .distance(enemy_transform.translation)
            < 1.0
        {
            next_game_state.set(GameState::GameOver);
            let (mut visibility, _) = game_over_text.single_mut();
            *visibility = Visibility::Visible;
            break;
        }
    }
}

fn setup_player_score(mut commands: Commands) {
    commands.spawn((
        TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Score: 0".into(),
                    style: TextStyle {
                        font: Default::default(),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                }],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            ..Default::default()
        },
        Ui2DComponent::new(Vec2::new(20.0, 680.0), Vec2::new(200.0, 100.0)),
        Ui2DText { font_size: 40.0 },
        Score {},
    ));
}

fn update_player_score(player: Query<&Player>, mut text: Query<(&mut Text, &Score)>) {
    let player = player.single();
    let (mut text, _) = text.single_mut();
    text.sections[0].value = format!("Score: {}", player.score);
}

fn enemies_walking(
    mut enemies: Query<(&mut Transform, &Enemy), Without<Player>>,
    player_query: Query<(&Transform, &Player), Without<Enemy>>,
) {
    // Get the player's position (assuming there is only one player)
    let (player_transform, _) = player_query.single();

    for (mut transform, _) in enemies.iter_mut() {
        // Calculate the direction from the enemy to the player
        let direction_to_player = player_transform.translation - transform.translation;

        // Normalize the direction to get a unit vector
        let direction = direction_to_player.normalize();

        // Move the enemy towards the player
        transform.translation += direction * 0.01; // Adjust the speed as necessary

        // Calculate the new rotation for the enemy to face the player
        // Assumes 2D movement on the XZ plane; you can adjust if using 3D
        let target_rotation = Quat::from_rotation_arc(Vec3::Z, direction);

        // Set the enemy's rotation
        transform.rotation = target_rotation;
    }
}

fn check_enemy_proximity(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    warning_query: Query<(Entity, &WarningSign)>,
) {
    let player_transform = player_query.single();

    for (enemy_entity, enemy_transform) in enemy_query.iter() {
        let distance = player_transform
            .translation
            .distance(enemy_transform.translation);

        // Check if the enemy is within the danger zone
        if distance < 10.0 {
            // Check if a warning sign for this enemy already exists
            let existing_warning = warning_query
                .iter()
                .any(|(_, warning)| warning.enemy_entity == enemy_entity);

            if !existing_warning {
                // Spawn a warning sign and link it to the enemy
                spawn_warning_exclamation(
                    &mut commands,
                    player_transform,
                    enemy_transform,
                    enemy_entity,
                );
            }
        }
    }
}

fn spawn_warning_exclamation(
    commands: &mut Commands,
    player_transform: &Transform,
    enemy_transform: &Transform,
    enemy_entity: Entity,
) {
    let direction = enemy_transform.translation - player_transform.translation;
    let direction_normalized = direction.normalize();

    let screen_position = calculate_ui_position(direction_normalized);

    // Spawn the warning sign linked to this enemy
    commands.spawn((
        WarningSignBundle::new(screen_position),
        WarningSign {
            enemy_entity, // Link this warning to the enemy
        },
        WarningSignAnimation {},
        Ui2DComponent::new(screen_position, Vec2::new(50.0, 50.0)),
    ));
}

fn calculate_ui_position(direction: Vec3) -> Vec2 {
    // Convert the 3D direction into a 2D screen-space position
    let x = direction.x * 100.0; // Scale factor to control how far on the screen
    let y = direction.z * 100.0; // You can also use y here for vertical movement

    Vec2::new(x, y)
}

fn update_warning_positions(
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut warning_query: Query<(&mut Style, &WarningSign)>,
) {
    let player_transform = player_query.single();

    for (mut style, warning_sign) in warning_query.iter_mut() {
        if let Ok((_, enemy_transform)) = enemy_query.get(warning_sign.enemy_entity) {
            let direction = enemy_transform.translation - player_transform.translation;
            let direction_normalized = direction.normalize();

            let screen_position = calculate_ui_position(direction_normalized);

            // Update the position of the warning sign
            style.left = Val::Px(screen_position.x);
            style.top = Val::Px(screen_position.y);
        }
    }
}

fn remove_far_warning_signs(
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut warning_query: Query<(Entity, &WarningSign)>,
    mut commands: Commands,
) {
    for (warning_entity, warning_sign) in warning_query.iter_mut() {
        if let Err(_) = enemy_query.get(warning_sign.enemy_entity) {
            commands.entity(warning_entity).despawn();
        }
    }
}

fn game_over(
    mut commands: Commands,
    curr_game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    commands.spawn((
        TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Game Over!\nPress R to restart".to_string(),
                    style: TextStyle {
                        font: Default::default(),
                        font_size: 75.0,
                        color: Color::WHITE,
                    },
                }],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            visibility: Visibility::Hidden,
            ..Default::default()
        },
        Ui2DComponent::new(
            Vec2::new(
                MAX_WIDTH / 4.0 + MAX_WIDTH / 32.0,
                MAX_HEIGHT / 4.0 + MAX_HEIGHT / 16.0,
            ),
            Vec2::new(MAX_WIDTH / 2.0, MAX_HEIGHT / 2.0),
        ),
        Ui2DText { font_size: 75.0 },
        GameOverText {},
    ));

    match curr_game_state.get() {
        GameState::Restarting => next_game_state.set(GameState::Playing),
        _ => {}
    }
}

fn play_again(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    transforms: Query<(Entity, &Transform)>,
    ui_2d_icons: Query<(Entity, &Ui2DComponent), Without<Ui2DText>>,
    ui_2d_text: Query<(Entity, &Ui2DComponent, &Ui2DText)>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for (entity, _) in transforms.iter() {
            commands.entity(entity).despawn();
        }

        for (entity, _) in ui_2d_icons.iter() {
            if let Ok(_) = ui_2d_icons.get(entity) {
                commands.entity(entity).despawn();
            }
        }

        for (entity, _, _) in ui_2d_text.iter() {
            if let Ok(_) = ui_2d_text.get(entity) {
                commands.entity(entity).despawn();
            }
        }

        next_game_state.set(GameState::Restarting);
    }
}
