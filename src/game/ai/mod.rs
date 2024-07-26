use bevy::app::App;
use bevy::prelude::{Component, Query, With, Without};
use crate::game::grid::GridPosition;
use crate::game::line_of_sight::front_facing_edges::FacingWallsCache;
use crate::game::line_of_sight::vision::{Facing, VisionAbility};

pub fn plugin(_app: &mut App) {
}

/// Hunters have vision, movement, and look for prey. When they see one, they chase it.
#[derive(Component)]
pub struct Hunter;

/// Preys are targets for hunters to see and chase down. There's probably only one - the player.
#[derive(Component)]
pub struct Prey;

pub fn find_visible_targets(
    hunters: Query<
        (&GridPosition, &Facing, &VisionAbility, &FacingWallsCache),
        (With<Hunter>, Without<Prey>)>,
    targets: Query<
        (&GridPosition),
        (With<Prey>, Without<Hunter>)>,
) {

}
