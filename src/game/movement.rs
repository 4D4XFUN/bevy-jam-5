use crate::game::grid::GridPosition;
/// Grid-based movement
use crate::input::PlayerAction;
use crate::AppSet;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use std::time::Duration;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update_roll_timer.in_set(AppSet::TickTimers));
    app.add_systems(
        Update,
        (respond_to_input, apply_movement)
            .chain()
            .in_set(AppSet::UpdateVirtualGrid),
    );

    app.register_type::<GridMovement>();
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
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            velocity_multiplier: 3.0,
            cooldown: Timer::from_seconds(2.0, TimerMode::Once),
        }
    }
}

impl GridMovement {
    pub fn _immobile() -> Self {
        Self {
            velocity: Vec2::ZERO,
            friction: 0.0,
            acceleration_player_force: Vec2::ZERO,
            acceleration_external_force: Vec2::ZERO,
            acceleration_player_multiplier: 0.,
            is_rolling: false,
        }
    }

    /// Sets every variable relevant to movement back to default
    pub fn reset(&mut self) {
        self.velocity = Vec2::ZERO;
        self.acceleration_player_force = Vec2::ZERO;
        self.acceleration_external_force = Vec2::ZERO;
        self.is_rolling = false;
    }
}

impl Default for GridMovement {
    // todo create "presets" like slow, medium, fast for use by enemies or players
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            friction: 0.85,
            acceleration_player_force: Vec2::ZERO,
            acceleration_external_force: Vec2::ZERO,
            acceleration_player_multiplier: 50.,
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
    mut query: Query<(&mut GridPosition, &mut GridMovement, Option<&mut Roll>)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut position, mut movement, maybe_roll) in query.iter_mut() {
        let force = movement.current_force() * dt; // scale it by time

        // apply forces and friction
        let mut velocity = movement.velocity + force;
        velocity *= movement.friction;
        if velocity.length() < 0.01 {
            velocity = Vec2::ZERO;
        }
        movement.velocity = velocity;

        // move the player
        let roll_multi = match (maybe_roll, movement.is_rolling) {
            (Some(roll), true) => roll.velocity_multiplier,
            _ => 1.0,
        };

        position.offset += movement.velocity * dt * roll_multi;
        position.fix_offset_overflow();
    }
}

fn update_roll_timer(time: Res<Time>, mut query: Query<(&mut Roll, &mut GridMovement, &ActionState<PlayerAction>)>) {
    let dt = time.delta_seconds();
    for (mut roll, mut movement, action_state) in query.iter_mut() {
        roll.timer.tick(Duration::from_secs_f32(dt));
        
        if roll.timer.finished() {
            movement.is_rolling = false;
            roll.cooldown.tick(Duration::from_secs_f32(dt));
        }

        if roll.cooldown.finished() {
            if action_state.pressed(&PlayerAction::Roll) {
                movement.is_rolling = true;
                roll.cooldown.reset();
                roll.timer.reset();
            }
        }

    }
}


