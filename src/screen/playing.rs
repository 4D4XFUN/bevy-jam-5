//! The screen state for the main game loop.

use bevy::prelude::*;

use crate::game::threat::ThreatTimer;
use crate::game::{audio::soundtrack::Soundtrack, spawn::level::SpawnLevel};
use crate::ui::prelude::*;

use super::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), enter_playing);
    app.add_systems(OnExit(Screen::Playing), exit_playing);

    app.add_systems(Update, update_timer.run_if(in_state(Screen::Playing)));
}

#[derive(Component)]
struct PlayTime;

fn enter_playing(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Playing))
        .with_children(|children| {
            children
                .spawn((
                    Name::new("Header Text"),
                    TextBundle::from_section(
                        "TempText".to_string(),
                        TextStyle {
                            font_size: 30.0,
                            ..default()
                        },
                    )
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(15.0),
                        ..default()
                    })
                    .with_text_justify(JustifyText::Center),
                ))
                .insert(PlayTime);
        });

    commands.trigger(SpawnLevel);
    commands.trigger(Soundtrack::Gameplay);
}

fn exit_playing(mut commands: Commands) {
    // We could use [`StateScoped`] on the sound playing entites instead.
    commands.trigger(Soundtrack::Disable);
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
            let millis = ((time.as_millis() as f32 % 1000.0) / 100.0).floor();
            text.sections[0].value = format!(
                "THREAT LEVEL {}\n(next level in {}.{}s)",
                threat_timer.current_level + 1,
                seconds,
                millis,
            );
        } else {
            text.sections[0].value = "RUN FOR YOUR LIFE!".to_string();
        }
    }
}
