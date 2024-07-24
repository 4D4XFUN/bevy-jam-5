use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.observe(use_stamina)
        .register_type::<Stamina>()
        .add_systems(Update, recharge_stamina);
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

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct GridCollider;

fn use_stamina(
    _trigger: Trigger<UseStamina>,
    mut query: Query<&mut Stamina>,
    mut event: EventWriter<UseStamina>,
) {
    for mut stamina in query.iter_mut() {
        if stamina.current > stamina.max/3.0 {
            stamina.current -= 1.0;
            event.send(UseStamina);
        }
    }
    
}

fn recharge_stamina(
    mut query: Query<(&mut Stamina, &mut RechargeTimer)>,
) {
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