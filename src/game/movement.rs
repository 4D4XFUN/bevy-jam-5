use std::time::Duration;

use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::game::grid::GridPosition;
use crate::game::spawn::level::LevelWalls;
use crate::game::spawn::player::Player;
/// Grid-based movement
use crate::input::PlayerAction;
use crate::AppSet;

use super::line_of_sight::front_facing_edges::RebuildCache;
use super::spawn::health::OnDeath;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update_roll_timer.in_set(AppSet::TickTimers));
    app.add_systems(
        Update,
        (respond_to_input, apply_movement)
            .chain()
            .in_set(AppSet::UpdateVirtualGrid),
    );

    app.register_type::<GridMovement>();
    app.observe(reset_roll_timer_on_death);
}

#[derive(Component, Reflect, Debug, Copy, Clone, PartialEq)]
#[reflect(Component)]
pub struct GridMovement {
    pub velocity: Vec2,
    pub friction: f32,
    pub acceleration_player_force: Vec2,
    pub acceleration_external_force: Vec2,
    pub acceleration_player_multiplier: f32,
    pub is_rolling: bool,
}

#[derive(Component, Reflect, Debug, PartialEq)]
#[reflect(Component)]
pub struct Roll {
    pub timer: Timer,
    pub velocity_multiplier: f32,
    cooldown: Timer,
}

impl Roll {
    pub const ROLL_TIME: Duration = Duration::from_millis(300);
}

impl Default for Roll {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.25, TimerMode::Once),
            velocity_multiplier: 3.0,
            cooldown: Timer::from_seconds(5.0, TimerMode::Once)
                .tick(Duration::from_secs_f32(5.0))
                .clone(),
        }
    }
}

impl GridMovement {
    /// Sets every variable relevant to movement back to default
    pub fn reset(&mut self) {
        self.velocity = Vec2::ZERO;
        self.acceleration_player_force = Vec2::ZERO;
        self.acceleration_external_force = Vec2::ZERO;
        self.is_rolling = false;
    }
}

impl Default for GridMovement {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            friction: 0.85,
            acceleration_player_force: Vec2::ZERO,
            acceleration_external_force: Vec2::ZERO,
            acceleration_player_multiplier: 0.7,
            is_rolling: false,
        }
    }
}

impl GridMovement {
    pub fn current_force(&self) -> Vec2 {
        self.acceleration_player_force + self.acceleration_external_force
    }
}

pub fn respond_to_input(mut query: Query<(&ActionState<PlayerAction>, &mut GridMovement)>) {
    for (action_state, mut movement) in query.iter_mut() {
        let mut intent = Vec2::ZERO;

        if action_state.pressed(&PlayerAction::MoveUp) {
            intent.y += 1.0;
        }
        if action_state.pressed(&PlayerAction::MoveDown) {
            intent.y -= 1.0;
        }
        if action_state.pressed(&PlayerAction::MoveLeft) {
            intent.x -= 1.0;
        }
        if action_state.pressed(&PlayerAction::MoveRight) {
            intent.x += 1.0;
        }
        // Normalize so that diagonal movement has the same speed as horizontal and vertical movement.
        let intent = intent.normalize_or_zero();

        movement.acceleration_player_force = intent * movement.acceleration_player_multiplier;
    }
}

pub fn apply_movement(
    mut query: Query<(&mut GridPosition, &mut GridMovement, Option<&Roll>)>,
    time: Res<Time>,
    walls: Res<LevelWalls>,
    mut commands: Commands,
) {
    let dt = time.delta_seconds();
    for (mut position, mut movement, maybe_roll) in query.iter_mut() {
        let prev_x = position.coordinates.x;
        let prev_y = position.coordinates.y;
        let force = movement.current_force() * dt; // scale it by time

        // apply forces and friction
        let mut velocity = movement.velocity + force;
        velocity *= movement.friction;
        if velocity.length() < 0.0001 {
            velocity = Vec2::ZERO;
        }
        movement.velocity = velocity;

        // move the player
        let roll_multi = match (maybe_roll, movement.is_rolling) {
            (Some(roll), true) => roll.velocity_multiplier,
            _ => 1.0,
        };

        // brute force check if next step would put us inside a wall square, and cancel if it would
        // one downside of this is that walls feel "sticky" instead of being able to slide along them, but it fixes the rolling through wall glitch at high speeds/low framerates
        let mut next_pos = *position;
        let adjusted_velocity = movement.velocity * roll_multi;
        next_pos.offset += adjusted_velocity;
        next_pos.fix_offset_overflow();
        if walls.collides_gridpos(&next_pos) {
            if next_pos.coordinates.x == position.coordinates.x {
                next_pos.coordinates.y = position.coordinates.y;
                next_pos.offset.y = position.offset.y;
                next_pos.fix_offset_overflow();

                if walls.collides_gridpos(&next_pos) {
                    next_pos = *position;
                }
            } else if next_pos.coordinates.y == position.coordinates.y {
                next_pos.coordinates.x = position.coordinates.x;
                next_pos.offset.x = position.offset.x;
                next_pos.fix_offset_overflow();

                if walls.collides_gridpos(&next_pos) {
                    next_pos = *position;
                }
            } else {
                //diagonal move between grid positions
                let mut temp_pos = next_pos;
                temp_pos.coordinates.y = position.coordinates.y;
                temp_pos.offset.y = position.offset.y;
                temp_pos.fix_offset_overflow();

                if walls.collides_gridpos(&temp_pos) {
                    let mut temp_pos2 = next_pos;
                    temp_pos2.coordinates.x = position.coordinates.x;
                    temp_pos2.offset.x = position.offset.x;
                    temp_pos2.fix_offset_overflow();

                    if walls.collides_gridpos(&temp_pos2) {
                        next_pos = *position;
                    } else {
                        next_pos = temp_pos2;
                    }
                } else {
                    next_pos = temp_pos;
                }
            }
        } else {
            movement.acceleration_external_force = Vec2::ZERO;
        }

        // apply the movement to our actual position
        position.coordinates = next_pos.coordinates;
        position.offset = next_pos.offset;

        if prev_x != position.coordinates.x || prev_y != position.coordinates.y {
            commands.trigger(RebuildCache);
        }
    }
}

fn update_roll_timer(
    time: Res<Time>,
    mut query: Query<(&mut Roll, &mut GridMovement, &ActionState<PlayerAction>)>,
    mut player_sprite: Query<&mut Sprite, With<Player>>,
) {
    let dt = time.delta_seconds();
    if let Ok(mut sprite) = player_sprite.get_single_mut() {
        for (mut roll, mut movement, action_state) in query.iter_mut() {
            roll.timer.tick(Duration::from_secs_f32(dt));

            if roll.timer.finished() {
                movement.is_rolling = false;
                roll.cooldown.tick(Duration::from_secs_f32(dt));
            }

            if !roll.cooldown.finished() {
                if sprite.color.luminance() > 0.1 {
                    sprite.color = sprite.color.darker(0.1);
                }
            } else {
                sprite.color = sprite.color.lighter(0.1);
            }

            if roll.cooldown.finished() && action_state.pressed(&PlayerAction::Roll) {
                movement.is_rolling = true;
                roll.cooldown.reset();
                roll.timer.reset();
            }
        }
    }
}

fn reset_roll_timer_on_death(
    _trigger: Trigger<OnDeath>,
    mut query: Query<(&mut Roll, &mut GridMovement)>,
) {
    for (mut roll, mut movement) in &mut query {
        let timer_duration = roll.timer.duration();
        roll.timer.tick(timer_duration);

        let cooldown_duration = roll.cooldown.duration();
        roll.cooldown.tick(cooldown_duration);

        movement.is_rolling = false;
    }
}
