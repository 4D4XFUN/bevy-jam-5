//! The game's main screen states and transitions between them.
mod credits;
mod game_over;
mod loading;
mod playing;
mod title;

use bevy::prelude::*;
use game_over::EndGame;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>().add_sub_state::<EndGame>();
    app.enable_state_scoped_entities::<Screen>();

    app.add_plugins((
        loading::plugin,
        title::plugin,
        credits::plugin,
        playing::plugin,
        game_over::plugin,
    ));
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum Screen {
    #[default]
    Loading,
    Title,
    Credits,
    Playing,
    GameOver,
}
