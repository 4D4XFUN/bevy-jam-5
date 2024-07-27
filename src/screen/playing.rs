//! The screen state for the main game loop.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::game::{audio::soundtrack::Soundtrack, spawn::level::SpawnLevel};
use crate::game::threat::ThreatTimer;
use crate::ui::prelude::*;

use super::Screen;

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

fn update_timer(
    threat_settings: Res<crate::game::threat::ThreatTimerSettings>,
    threat_timer: Res<ThreatTimer>,
    mut query: Query<&mut Text, With<PlayTime>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        if threat_timer.current_level < threat_settings.levels - 1 {
            let time = threat_timer.timer.remaining();
            let seconds = time.as_secs() % 60;
            text.sections[0].value = format!(
                "THREAT LEVEL {} ({}s until next level)",
                threat_timer.current_level + 1,
                seconds,
            );
        } else {
            text.sections[0].value = "RUN FOR YOUR LIFE!".to_string();
        }
    }
}
