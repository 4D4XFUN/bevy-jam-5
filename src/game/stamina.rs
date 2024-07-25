use std::time::Duration;

use bevy::prelude::*;
use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, recharge_bar.in_set(AppSet::TickTimers));
    app.add_systems(Update, use_stamina.in_set(AppSet::UpdateStamina)); //added a UpdateStamina set in AppSet
    app.register_type::<Stamina>();
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Stamina { 
    total_bars: u8,
    current_bars: u8,
    recharge_rate: f32,
    recharge_delay: f32,
    recharge_timer: Timer,
    recharge_delay_timer: Timer,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            total_bars: 3,
            current_bars: 3,
            recharge_rate: 1.0,
            recharge_delay: 1.0,            
            recharge_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            recharge_delay_timer: Timer::from_seconds(1.0, TimerMode::Once), //delay before recharging stamina
        }
    }
}

pub fn use_stamina(mut stamina: Query<&mut Stamina>) {
    for mut stamina in &mut stamina.iter_mut() {
        if stamina.recharge_timer.finished() {
            stamina.current_bars -= 1;
        }
    }
}

pub fn recharge_bar(mut stamina: Query<&mut Stamina>, time: Res<Time>) {
    let dt = Duration::from_secs_f32(time.delta_seconds());
    for mut stamina in &mut stamina.iter_mut() { 

        if stamina.current_bars != stamina.total_bars {
            stamina.recharge_delay_timer.tick(dt);

            if stamina.recharge_delay_timer.finished() {
                stamina.recharge_timer.tick(dt);

                if stamina.recharge_timer.finished() {
                    stamina.current_bars += 1;
                }
            }
        }
    }
}




