//! Spawn the main level by triggering other observers.

use super::player::SpawnPlayer;
use crate::game::spawn::ldtk::LdtkEntityBundle;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.register_ldtk_entity::<LdtkEntityBundle>("Goal");
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(_trigger: Trigger<SpawnLevel>, mut commands: Commands) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    commands.trigger(SpawnPlayer);
}
