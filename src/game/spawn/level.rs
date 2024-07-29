//! Spawn the main level by triggering other observers.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy_ecs_ldtk::{GridCoords, LdtkIntCell, LevelEvent};
use bevy_ecs_ldtk::assets::LdtkProject;
use bevy_ecs_ldtk::prelude::{LdtkEntityAppExt, LdtkIntCellAppExt, LevelMetadataAccessor};

use crate::game::grid::GridPosition;
use crate::game::line_of_sight::BlocksVision;
use crate::game::line_of_sight::front_facing_edges::RebuildCache;
use crate::game::spawn::enemy::SpawnEnemyTrigger;
use crate::game::spawn::ldtk::LdtkEntityBundle;

use super::exit::SpawnExitTrigger;
use super::player::SpawnPlayerTrigger;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.register_ldtk_entity::<LdtkEntityBundle>("Goal");
    app.register_ldtk_int_cell::<WallBundle>(1);
    app.register_ldtk_int_cell::<GroundBundle>(2);
    app.init_resource::<LevelWalls>();
    app.init_resource::<LevelVisionBlockers>();
    app.add_systems(Update, cache_wall_locations);
    app.add_systems(Update, cache_vision_blocker_locations);
    app.observe(rebuild_movement_cache_on_remove);
    app.observe(rebuild_movement_cache_on_add);
    // reflection
    app.register_type::<LevelWalls>();
    app.register_type::<LevelVisionBlockers>();
}

pub const GRID_SIZE: i32 = 16;

#[derive(Default, Component, Copy, Clone)]
pub struct BlocksMovement;

#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: BlocksMovement,
    vision: BlocksVision,
}

#[derive(Default, Bundle, LdtkIntCell)]
struct GroundBundle {}

#[derive(Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct LevelVisionBlockers {
    pub vision_blocker_locations: HashSet<GridCoords>,
    pub level_width: i32,
    pub level_height: i32,
}

impl LevelVisionBlockers {
    pub fn collides(&self, x: i32, y: i32) -> bool {
        x < 0
            || y < 0
            || x >= self.level_width
            || y >= self.level_height
            || self
                .vision_blocker_locations
                .contains(&GridCoords::new(x, y))
    }
}

#[derive(Default, Resource, Reflect)]
#[reflect(Resource)]
pub(crate) struct LevelWalls {
    pub wall_locations: HashSet<GridCoords>,
    pub level_width: i32, // grid units
    pub level_height: i32,
}

impl LevelWalls {
    pub fn collides(&self, x: i32, y: i32) -> bool {
        x < 0
            || y < 0
            || x >= self.level_width
            || y >= self.level_height
            || self.wall_locations.contains(&GridCoords::new(x, y))
    }

    pub fn collides_gridpos(&self, gridpos: &GridPosition) -> bool {
        self.collides(gridpos.coordinates.x as i32, gridpos.coordinates.y as i32)
    }
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(_trigger: Trigger<SpawnLevel>, mut commands: Commands) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    commands.trigger(SpawnPlayerTrigger);
    commands.trigger(SpawnEnemyTrigger);
    commands.trigger(SpawnExitTrigger);
}

fn cache_wall_locations(
    mut level_walls: ResMut<LevelWalls>,
    mut level_events: EventReader<LevelEvent>,
    walls_query: Query<&GridCoords, With<BlocksMovement>>,
    ldtk_project_entities: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    for level_event in level_events.read() {
        let LevelEvent::Spawned(level_iid) = level_event else {
            return;
        };
        let ldtk_project = ldtk_project_assets
            .get(ldtk_project_entities.single())
            .expect("LdtkProject should be loaded when level is spawned");
        let level = ldtk_project
            .get_raw_level_by_iid(level_iid.get())
            .expect("spawned level should exist in project");
        let wall_locations = walls_query.iter().copied().collect();
        let new_level_walls = LevelWalls {
            wall_locations,
            level_width: level.px_wid / GRID_SIZE,
            level_height: level.px_hei / GRID_SIZE,
        };
        *level_walls = new_level_walls;
    }
}

fn cache_vision_blocker_locations(
    mut level_vision_blocker: ResMut<LevelVisionBlockers>,
    mut level_events: EventReader<LevelEvent>,
    vision_blocker_query: Query<&GridCoords, With<BlocksVision>>,
    ldtk_project_entities: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    mut commands: Commands,
) {
    for level_event in level_events.read() {
        let LevelEvent::Spawned(level_iid) = level_event else {
            return;
        };
        let ldtk_project = ldtk_project_assets
            .get(ldtk_project_entities.single())
            .expect("LdtkProject should be loaded when level is spawned");
        let level = ldtk_project
            .get_raw_level_by_iid(level_iid.get())
            .expect("spawned level should exist in project");
        let blocker_locations = vision_blocker_query.iter().copied().collect();
        let new_vision_blocker = LevelVisionBlockers {
            vision_blocker_locations: blocker_locations,
            level_width: level.px_wid / GRID_SIZE,
            level_height: level.px_hei / GRID_SIZE,
        };
        *level_vision_blocker = new_vision_blocker;
        commands.trigger(RebuildCache);
    }
}

fn rebuild_movement_cache_on_remove(
    trigger: Trigger<OnRemove, BlocksVision>,
    mut movement_blocker: ResMut<LevelWalls>,
    query: Query<(Entity, &GridCoords)>,
) {
    let entity = trigger.entity();
    if let Ok((_, coordinates)) = query.get(entity) {
        movement_blocker.wall_locations.remove(coordinates);
    }
}

fn rebuild_movement_cache_on_add(
    trigger: Trigger<OnAdd, BlocksVision>,
    mut movement_blocker: ResMut<LevelWalls>,
    query: Query<(Entity, &GridCoords)>,
) {
    let entity = trigger.entity();
    if let Ok((_, coordinates)) = query.get(entity) {
        movement_blocker.wall_locations.insert(*coordinates);
    }
}
