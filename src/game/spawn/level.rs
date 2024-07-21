//! Spawn the main level by triggering other observers.

use bevy::prelude::*;
use bevy_ecs_ldtk::LdtkIntCell;
use bevy_ecs_ldtk::prelude::{LdtkEntityAppExt, LdtkIntCellAppExt};
use crate::game::spawn::ldtk::LdtkEntityBundle;
use super::player::SpawnPlayer;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.register_ldtk_entity::<LdtkEntityBundle>("Goal");
    app.register_ldtk_int_cell::<WallBundle>(1);
}

const GRID_SIZE: i32 = 16;

#[derive(Default, Component)]
struct Wall;

#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall,
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(_trigger: Trigger<SpawnLevel>, mut commands: Commands) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    commands.trigger(SpawnPlayer);
}
