//! Handles spawning of entities. Here, we are using
//! [observers](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Observer.html)
//! for this, but you could also use `Events<E>` or `Commands`.

use bevy::prelude::*;

mod enemy;
mod exit;
pub mod health;
mod ldtk;
pub mod level;
pub mod player;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        health::plugin,
        level::plugin,
        player::plugin,
        enemy::plugin,
        exit::plugin,
    ));
}
