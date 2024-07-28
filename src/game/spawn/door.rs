use bevy::{prelude::*, render::primitives::Aabb};
use bevy_ecs_ldtk::prelude::*;

use crate::{
    game::{line_of_sight::BlocksVision, utilities::intersect},
    screen::Screen,
};

use super::{level::BlocksMovement, player::Player};

pub fn plugin(app: &mut App) {
    // spawning
    app.register_ldtk_entity::<LdtkDoorBundle>("Door");
    app.add_systems(
        Update,
        (open_doors, close_doors).run_if(in_state(Screen::Playing)),
    );
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

fn open_doors(
    player_query: Query<(&Transform, &Aabb), With<Player>>,
    mut door_query: Query<
        (Entity, &mut Visibility, &Transform, &Aabb),
        (With<LdtkDoor>, With<BlocksMovement>, With<BlocksVision>),
    >,
    mut commands: Commands,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };

    for (entity, mut visibility, transform, aabb) in &mut door_query {
        if intersect(player, (transform, aabb)) {
            commands.entity(entity).remove::<BlocksMovement>();
            commands.entity(entity).remove::<BlocksVision>();
            *visibility = Visibility::Hidden;
        }
    }
}

fn close_doors(
    player_query: Query<(&Transform, &Aabb), With<Player>>,
    mut door_query: Query<
        (Entity, &mut Visibility, &Transform, &Aabb),
        (
            With<LdtkDoor>,
            Without<BlocksMovement>,
            Without<BlocksVision>,
        ),
    >,
    mut commands: Commands,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };

    for (entity, mut visibility, transform, aabb) in &mut door_query {
        if !intersect(player, (transform, aabb)) {
            commands.entity(entity).insert(BlocksMovement);
            commands.entity(entity).insert(BlocksVision);
            *visibility = Visibility::Inherited;
        }
    }
}
