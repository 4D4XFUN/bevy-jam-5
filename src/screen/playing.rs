//! The screen state for the main game loop.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use super::Screen;
use crate::game::{audio::soundtrack::Soundtrack, spawn::level::SpawnLevel};
use crate::{game::threat::PlayTimer, ui::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), enter_playing);
    app.add_systems(OnExit(Screen::Playing), exit_playing);

    app.add_systems(Update, update_timer.run_if(in_state(Screen::Playing)));

    app.add_systems(
        Update,
        return_to_title_screen
            .run_if(in_state(Screen::Playing).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

#[derive(Component)]
struct PlayTime;

fn enter_playing(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Playing))
        .with_children(|children| {
            children
                .spawn(TextBundle::from("X:XX").with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(15.0),
                    ..default()
                }))
                .insert(PlayTime);
        });

    commands.trigger(SpawnLevel);
    commands.trigger(Soundtrack::Gameplay);
}

fn exit_playing(mut commands: Commands) {
    // We could use [`StateScoped`] on the sound playing entites instead.
    commands.trigger(Soundtrack::Disable);
}

pub(crate) fn return_to_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn update_timer(play_timer: Res<PlayTimer>, mut query: Query<&mut Text, With<PlayTime>>) {
    for mut text in &mut query {
        let time = play_timer.0.remaining();
        let minutes = time.as_secs() / 60;
        let seconds = time.as_secs() % 60;
        let millis = time.as_millis() % 1000;
        text.sections[0].value = format!("{}:{}.{:0<3}", minutes, seconds, millis);
    }
}
