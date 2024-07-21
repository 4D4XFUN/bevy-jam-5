//! Game mechanics and content.

use bevy::prelude::*;

pub mod ai;
mod animation;
pub mod assets;
pub mod audio;
mod movement;
pub mod spawn;
mod grid;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        audio::plugin,
        movement::plugin,
        spawn::plugin,
        grid::plugin,
    ));
}
