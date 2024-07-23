//! Game mechanics and content.

use bevy::prelude::*;

pub mod ai;
mod animation;
pub mod assets;
pub mod audio;
mod camera;
pub mod grid;
mod movement;
pub mod spawn;
pub mod line_of_sight;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        audio::plugin,
        movement::plugin,
        spawn::plugin,
        grid::plugin,
        camera::plugin,
        line_of_sight::plugin,
    ));
}
