use bevy::app::App;
use bevy::prelude::*;

use crate::game::animation::PlayerAnimation;
use crate::game::audio::sfx::Sfx;
use crate::game::grid::GridPosition;
use crate::game::movement::GridMovement;

/// Handles all health code.
///
/// Triggers OnDeath on death
pub(super) fn plugin(app: &mut App) {
    app.register_type::<CanReceiveDamage>();
    app.register_type::<CanApplyDamage>();
    app.register_type::<Health>();
    app.add_systems(Update, apply_damage_on_collision);
    app.observe(on_receive_damage);
}

#[derive(Component, Reflect, Default)]
pub struct CanReceiveDamage;

#[derive(Component, Reflect, Default)]
pub struct CanApplyDamage;

#[derive(Component, Reflect, Default)]
pub struct Health(pub f32);

#[derive(Component, Reflect, Default)]
pub struct SpawnPointGridPosition(pub Vec2);

#[derive(Event)]
pub struct ReceiveDamage;

#[derive(Event)]
pub struct OnDeath;

const ENTITY_COLLISION_RADIUS: f32 = 15.0;

fn apply_damage_on_collision(
    attacker_transforms: Query<(&Name, &Transform), With<CanApplyDamage>>,
    receiver_transforms: Query<(&Name, Entity, &Transform), With<CanReceiveDamage>>,
    mut commands: Commands,
) {
    for (attacker_name, attacker_transform) in &attacker_transforms {
        for (receiver_name, receiver, receiver_transform) in &receiver_transforms {
            if attacker_transform
                .translation
                .distance(receiver_transform.translation)
                <= ENTITY_COLLISION_RADIUS
            {
                info!("{} killed by {}", &receiver_name, &attacker_name);
                commands.trigger_targets(ReceiveDamage, receiver);
            }
        }
    }
}

fn on_receive_damage(
    trigger: Trigger<ReceiveDamage>,
    mut receiver_transforms: Query<
        (
            Entity,
            &mut GridPosition,
            &SpawnPointGridPosition,
            &mut PlayerAnimation,
            &mut GridMovement,
        ),
        With<CanReceiveDamage>,
    >,
    mut commands: Commands,
) {
    let id = trigger.entity();

    for (
        receiver,
        mut receiver_grid_position,
        spawn_point,
        mut player_animation,
        mut grid_movement,
    ) in &mut receiver_transforms
    {
        if id == receiver {
            receiver_grid_position.coordinates = spawn_point.0;
            receiver_grid_position.offset = Vec2::ZERO; // reset offset within the tile
            grid_movement.reset();
            player_animation.reset();
            commands.trigger(OnDeath);
            commands.trigger(Sfx::Death);
        }
    }
}
