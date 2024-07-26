use std::time::Duration;

use bevy::prelude::*;

use crate::screen::Screen;

use super::end_game::EndGameCondition;

/// Handles threat levels.
///
/// Will signal `ThreatLevelIncreased(u8)` with the new threat level.
/// Will additionally signal `EndGameCondition::TimeOut` when time ran out
/// and set AppState to `Screen::GameOver`
pub fn plugin(app: &mut App) {
    let settings = ThreatTimerSettings {
        levels: 3,
        seconds_between_levels: 60.0,
    };
    app.insert_resource(ThreatTimer {
        timer: Timer::new(
            Duration::from_secs_f32(settings.seconds_between_levels),
            TimerMode::Repeating,
        ),
        current_level: 0,
    });
    app.insert_resource(PlayTimer(Timer::from_seconds(
        settings.levels as f32 * settings.seconds_between_levels,
        TimerMode::Once,
    )));
    app.insert_resource(settings);
    app.add_systems(Update, tick.run_if(in_state(Screen::Playing)));
}

#[derive(Resource)]
pub struct PlayTimer(pub Timer);

/// Is triggered when the threat level increases.
/// Property is the new threat level.
#[allow(dead_code)]
#[derive(Event)]
pub struct ThreatLevelIncreased(u8);

#[derive(Resource)]
pub struct ThreatTimer {
    pub timer: Timer,
    current_level: u8,
}

#[derive(Resource)]
struct ThreatTimerSettings {
    levels: u8,
    seconds_between_levels: f32,
}

fn tick(
    time: Res<Time>,
    threat_settings: Res<ThreatTimerSettings>,
    mut threat_timer: ResMut<ThreatTimer>,
    mut play_timer: ResMut<PlayTimer>,
    mut commands: Commands,
) {
    threat_timer.timer.tick(time.delta());
    play_timer.0.tick(time.delta());
    if threat_timer.timer.finished() {
        threat_timer.current_level += 1;
        commands.trigger(ThreatLevelIncreased(threat_timer.current_level));
        if threat_timer.current_level >= threat_settings.levels {
            commands.trigger(EndGameCondition::TimeOut);
            threat_timer.current_level = 0;
            threat_timer.timer.reset();
            play_timer.0.reset();
        }
    }
}
