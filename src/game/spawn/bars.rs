use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use super::level::BlocksMovement;

pub fn plugin(app: &mut App) {
    // spawning
    app.register_ldtk_entity::<LdtkBarsBundle>("CanPlayerSeeThrough");
}

#[derive(Component, Default, Copy, Clone)]
pub struct LdtkBars;

#[derive(Default, Bundle, LdtkEntity)]
struct LdtkBarsBundle {
    tag: LdtkBars,
    #[sprite_sheet_bundle]
    sprite_bundle: LdtkSpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
    wall: BlocksMovement,
}
