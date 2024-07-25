//! Game mechanics and content.

use bevy::prelude::*;

pub mod ai;
mod animation;
pub mod assets;
pub mod audio;
mod camera;
pub mod end_game;
mod ghost;
pub mod grid;
pub mod line_of_sight;
pub mod spawn;
pub mod stamina;
mod threat;
pub mod movement;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        audio::plugin,
        movement::plugin,
        spawn::plugin,
        grid::plugin,
        camera::plugin,
        stamina::plugin,
        line_of_sight::plugin,
        ghost::plugin,
        threat::plugin,
    ));
}
