/// Grid-based movement

use crate::game::grid::{GridLayout, GridPosition};
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

    app.add_systems(
        Update,
        set_real_position_based_on_grid.in_set(AppSet::UpdateWorld),
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
}

impl Roll {
    pub const ROLL_TIME: Duration = Duration::from_millis(300);
}

impl Default for Roll {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            velocity_multiplier: 3.0,
        }
    }
}

impl Default for GridMovement {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            friction: 0.85,
            acceleration_player_force: Vec2::ZERO,
            acceleration_external_force: Vec2::ZERO,
            acceleration_player_multiplier: 66.,
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

        if !movement.is_rolling && action_state.pressed(&PlayerAction::Roll) {
            movement.is_rolling = true;
        }
    }
}

pub fn apply_movement(
    mut query: Query<(&mut GridPosition, &mut GridMovement, &mut Roll)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut position, mut movement, roll) in query.iter_mut() {
        let force = movement.current_force() * dt; // scale it by time

        // apply forces and friction
        let mut velocity = movement.velocity + force;
        velocity *= movement.friction;
        if velocity.length() < 0.01 {
            velocity = Vec2::ZERO;
        }
        movement.velocity = velocity;

        // move the player
        let multiplier = if movement.is_rolling {
            roll.velocity_multiplier
        } else {
            1.0
        };
        position.offset += movement.velocity * dt * multiplier;
        position.fix_offset_overflow();
    }
}

fn update_roll_timer(time: Res<Time>, mut query: Query<(&mut Roll, &mut GridMovement)>) {
    let dt = time.delta_seconds();
    for (mut roll, mut movement) in query.iter_mut() {
        roll.timer.tick(Duration::from_secs_f32(dt));
        if roll.timer.finished() {
            movement.is_rolling = false;
            roll.timer.reset();
        }
    }
}

/// Any entity that has a GridPosition and a Transform gets put in the world wherever its grid position says.
/// This does mean that Transform mutations get overwritten by grid position calculated ones.
pub fn set_real_position_based_on_grid(
    mut query: Query<(&mut Transform, &GridPosition)>,
    grid: Res<GridLayout>,
) {
    for (mut t, gp) in query.iter_mut() {
        let pos = grid.grid_to_world(gp);
        t.translation.x = pos.x;
        t.translation.y = pos.y;
    }
}
