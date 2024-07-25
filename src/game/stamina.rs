use bevy::prelude::*;
use crate::AppSet;

pub fn plugin(app: &mut App) {
    app.add_event::<UseStamina>();
    app.add_systems(Update, handle_event::<UseStamina>);
    app.register_type::<Stamina>();
    app.add_systems(Update, use_stamina.in_set(AppSet::Update));
    app.add_systems(Update, recharge_stamina.in_set(AppSet::TickTimers));
    app.add_systems(Update, send_event.in_set(AppSet::RecordInput));
}

#[derive(Event, Default, Debug)]
pub struct UseStamina;

#[derive(Component, Reflect)]
pub struct Stamina {
    pub(crate) current: f32,
    pub(crate) max: f32,
    pub(crate) regen: f32,
}

#[derive(Component, Reflect)]
pub struct RechargeTimer {
    pub(crate) current: f32,
    pub(crate) max: f32,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            regen: 1.0,
        }
    }
}

impl Default for RechargeTimer {
    fn default() -> Self {
        Self { current: 0.0, max: 60.0 }
    }
}

pub fn send_event(mut event: EventWriter<UseStamina>) {
    event.send(UseStamina);
}

pub fn handle_event<T: Event>(mut events: ResMut<Events<T>>) {
    events.update();
}

pub fn use_stamina(mut query: Query<&mut Stamina>) {
    if query.single().current < 1.0 {
        return;
    } else {
        query.single_mut().current -= 1.0;
    }
}

pub fn recharge_stamina(mut query: Query<(&mut Stamina, &mut RechargeTimer)>) {
    for (mut stamina, mut timer) in query.iter_mut() {
        if stamina.current < stamina.max {
            timer.current += 1.0;
            if timer.current >= timer.max {
                stamina.current += stamina.regen;
                timer.current = 0.0;
            }
        }
    }
}
