use bevy::app::App;
use bevy::prelude::Component;

pub fn plugin(_app: &mut App) {}

/// Hunters have vision, movement, and look for prey. When they see one, they chase it.
#[derive(Component)]
pub struct Hunter;

/// Preys are targets for hunters to see and chase down. There's probably only one - the player.
#[derive(Component)]
pub struct _Prey;
