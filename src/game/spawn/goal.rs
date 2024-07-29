use std::f32::consts;

use bevy::{prelude::*, render::primitives::Aabb};
use bevy_ecs_ldtk::prelude::*;

use crate::game::{
    end_game::EndGameCondition,
    grid::GridPosition,
    line_of_sight::{vision::VisionAbility, CanRevealFog, PlayerLineOfSightBundle},
    utilities::intersect,
};

use super::{enemy::SpawnCoords, player::Player};

pub fn plugin(app: &mut App) {
    app.register_ldtk_entity::<LdtkGoalBundle>("Goal");
    // systems
    app.add_systems(Update, fix_loaded_ldtk_entities);
    app.add_systems(Update, check_exit);
}

#[derive(Component, Default, Copy, Clone)]
struct LdtkGoal;

#[derive(Default, Bundle, LdtkEntity)]
struct LdtkGoalBundle {
    tag: LdtkGoal,
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Component)]
struct Goal;

#[derive(Bundle)]
struct GoalBundle {
    spawn_coords: SpawnCoords,
    grid_position: GridPosition,
    player_line_of_sight_bundle: PlayerLineOfSightBundle,
}

impl GoalBundle {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            spawn_coords: SpawnCoords(GridPosition::new(x, y)),
            grid_position: GridPosition::new(x, y),
            player_line_of_sight_bundle: PlayerLineOfSightBundle {
                facing: Default::default(),
                can_reveal_fog: CanRevealFog,
                vision_ability: VisionAbility {
                    field_of_view_radians: 2.0 * consts::PI,
                    range_in_grid_units: 1.0,
                },
                facing_walls_cache: Default::default(),
                visible_squares: Default::default(),
            },
        }
    }
}

fn fix_loaded_ldtk_entities(
    query: Query<(Entity, &GridCoords), With<LdtkGoal>>,
    mut commands: Commands,
) {
    for (ldtk_entity, grid_coords) in query.iter() {
        commands
            .entity(ldtk_entity)
            .remove::<LdtkGoal>() // we have to remove it because it's used as the query for this function
            .insert((
                Name::new("Goal"),
                GoalBundle::new(grid_coords.x as f32, grid_coords.y as f32),
                Goal,
            ));
    }
}

fn check_exit(
    player_query: Query<(&Transform, &Aabb), With<Player>>,
    exit_query: Query<(&Transform, &Aabb), With<Goal>>,
    mut commands: Commands,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };
    let Ok(exit) = exit_query.get_single() else {
        return;
    };
    if intersect(player, exit) {
        commands.trigger(EndGameCondition::Win);
    }
}
