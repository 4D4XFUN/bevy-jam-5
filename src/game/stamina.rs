use std::time::Duration;

use bevy::prelude::*;
use crate::AppSet;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, recharge_bar.in_set(AppSet::TickTimers));
    app.add_systems(Update, use_stamina.in_set(AppSet::Update)); //added a UpdateStamina set in AppSet
    app.register_type::<Stamina>();
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Stamina { 
    pub total_bars: u8,
    pub current_bars: u8,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RechargeTimer {
    pub timer: Timer,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            total_bars: 3,
            current_bars: 3,
        }
    }
}

impl Default for RechargeTimer {
    fn default() -> Self { 
        Self {
            timer: Timer::from_seconds(10.0, TimerMode::Once),
        }
    }
}

pub fn use_stamina(mut stamina: Query<&mut Stamina>) {
    for mut stamina in &mut stamina.iter_mut() {
        stamina.current_bars -= 1;
    }
}

pub fn recharge_bar(mut stamina: Query<(&mut Stamina, &mut RechargeTimer)>, time: Res<Time>) {
    let dt = Duration::from_secs_f32(time.delta_seconds());
    for (mut stamina, mut recharge) in &mut stamina.iter_mut() { 

        if stamina.current_bars != stamina.total_bars {
            recharge.timer.unpause();
            recharge.timer.tick(dt);

            if recharge.timer.finished() {
                stamina.current_bars += 1;
            }

        } else {
            recharge.timer.pause();
            recharge.timer.reset();
        }
    }
}




