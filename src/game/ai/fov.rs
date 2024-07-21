use bevy::prelude::*;

pub fn plugin(app: &mut App) {
}

#[derive(Component)]
pub struct Fov {
    pub(crate) angle: f32,
    pub(crate) radius: f32,
}
