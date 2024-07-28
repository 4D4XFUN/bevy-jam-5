use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{game::line_of_sight::BlocksVision, screen::Screen};

use super::level::BlocksMovement;

pub fn plugin(app: &mut App) {
    // spawning
    app.register_ldtk_entity::<LdtkDoorBundle>("Door");
    app.add_systems(Update, check_door.run_if(in_state(Screen::Playing)));
}

#[derive(Component, Default, Copy, Clone)]
pub struct LdtkDoor;

#[derive(Default, Bundle, LdtkEntity)]
struct LdtkDoorBundle {
    tag: LdtkDoor,
    #[sprite_sheet_bundle]
    sprite_bundle: LdtkSpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
    movement: BlocksMovement,
    vision: BlocksVision,
}

fn check_door() {

}
