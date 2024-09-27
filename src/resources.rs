use bevy::prelude::*;

#[derive(Resource)]
pub struct PowerShotCooldownTimer(pub Timer);
