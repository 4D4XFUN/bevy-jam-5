use bevy::prelude::Bundle;
use bevy_ecs_ldtk::{LdtkEntity, LdtkSpriteSheetBundle};

#[derive(Default, Bundle, LdtkEntity)]
pub struct LdtkEntityBundle{
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
}