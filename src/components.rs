use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub is_charging: bool,
    pub charge_timer: Timer,
    pub max_charge_duration: f32,
    pub should_start_charge: bool,
    pub shoot_cooldown: Timer,
    pub score: u32,
}

#[derive(Component)]
pub struct ThirdPersonCamera {
    pub offset: Vec3,
}

#[derive(Component)]
pub struct Arrow {
    pub speed: f32,
    pub range: f32,
    pub direction: Vec3,
    pub distance_travelled: f32,
    pub strength: f32,
}

#[derive(Component)]
pub struct Enemy {}

#[derive(Component)]
pub struct PowerShotCooldownUI {}

#[derive(Component)]
pub struct Ui2DComponent {
    pub base_position: Vec2, // The original position (e.g., 400, 300)
    pub base_size: Vec2,     // The original size (e.g., 200, 50)
}

impl Ui2DComponent {
    pub fn new(base_position: Vec2, base_size: Vec2) -> Self {
        Self {
            base_position,
            base_size,
        }
    }

    // Adjust the position and size using the scaling factor
    pub fn adjust_position_and_size(&self, scaling_factor: f32) -> (Vec2, Vec2) {
        let adjusted_position = self.base_position * scaling_factor;
        let adjusted_size = self.base_size * scaling_factor;
        (adjusted_position, adjusted_size)
    }
}

#[derive(Component)]
pub struct Ui2DText {
    pub font_size: f32,
}

#[derive(Component)]
pub struct Score {}

#[derive(Component)]
pub struct WarningSign {
    pub enemy_entity: Entity, // Link to the enemy associated with this warning
}

#[derive(Component)]
pub struct WarningSignAnimation {}

#[derive(Component)]
pub struct GameOverText {}
