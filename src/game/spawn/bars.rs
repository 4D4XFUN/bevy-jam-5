use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::game::line_of_sight::BlocksVision;
use crate::game::spawn::ldtk::LdtkEntityBundle;

use super::level::BlocksMovement;

pub fn plugin(app: &mut App) {
    // spawning
    app.register_ldtk_entity::<LdtkBarsBundle>("Barrel");
    app.register_ldtk_entity::<LdtkBarsBundle>("Chair");
    app.register_ldtk_entity::<LdtkBarsBundle>("Table");
    app.register_ldtk_entity::<LdtkBarsBundle>("Crate1");
    app.register_ldtk_entity::<LdtkBarsBundle>("Crate2");
    app.register_ldtk_entity::<LdtkBarsBundle>("Crate3");
    app.register_ldtk_entity::<LdtkBarsBundle>("Crate4");
    app.register_ldtk_entity::<LdtkBarsBundle>("Podium");
    app.register_ldtk_entity::<LdtkBarsBundle>("Pouch");
    app.register_ldtk_entity::<LdtkEntityBundle>("Bones");
    app.register_ldtk_entity::<LdtkEntityBundle>("Bones2");
    app.register_ldtk_entity::<LdtkEntityBundle>("Light");
    app.register_ldtk_entity::<LdtkBarsBundle>("Podium2");
    app.register_ldtk_entity::<LdtkBarsBundle>("Chest");
    app.register_ldtk_entity::<LdtkBarsBundle>("Chest2");
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
    blocks_vision: BlocksVision,
}
