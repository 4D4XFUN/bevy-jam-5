//! Spawn the main level by triggering other observers.

use std::collections::HashSet;

use crate::game::spawn::enemy::SpawnEnemyTrigger;
use crate::game::spawn::ldtk::LdtkEntityBundle;
use bevy::prelude::*;
use bevy_ecs_ldtk::assets::LdtkProject;
use bevy_ecs_ldtk::prelude::{LdtkEntityAppExt, LdtkIntCellAppExt, LevelMetadataAccessor};
use bevy_ecs_ldtk::{GridCoords, LdtkIntCell, LevelEvent};

use super::player::SpawnPlayerTrigger;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.register_ldtk_entity::<LdtkEntityBundle>("Goal");
    app.register_ldtk_int_cell::<WallBundle>(1);
    app.init_resource::<LevelWalls>();
    app.add_systems(Update, cache_wall_locations);

    // reflection
    app.register_type::<LevelWalls>();
}

pub const GRID_SIZE: i32 = 16;

#[derive(Default, Component)]
pub struct Wall;

#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall,
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
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(_trigger: Trigger<SpawnLevel>, mut commands: Commands) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    commands.trigger(SpawnPlayerTrigger);
    commands.trigger(SpawnEnemyTrigger);
}

fn cache_wall_locations(
    mut level_walls: ResMut<LevelWalls>,
    mut level_events: EventReader<LevelEvent>,
    walls_query: Query<&GridCoords, With<Wall>>,
    ldtk_project_entities: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    for level_event in level_events.read() {
        if let LevelEvent::Spawned(level_iid) = level_event {
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
}
