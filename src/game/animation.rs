//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use std::time::Duration;

use bevy::prelude::*;
use bevy::utils::HashMap;

use super::audio::sfx::Sfx;
use crate::game::movement::GridMovement;
use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSet::TickTimers),
            (
                update_animation_movement,
                update_animation_atlas,
                trigger_step_sfx,
                trigger_roll_sfx,
            )
                .chain()
                .in_set(AppSet::Update),
        ),
    );
    app.register_type::<PlayerAnimation>();
}

/// Update the sprite direction and animation state (idling/walking).
fn update_animation_movement(
    mut player_query: Query<(&GridMovement, &mut Sprite, &mut PlayerAnimation)>,
) {
    for (controller, mut sprite, mut animation) in &mut player_query {
        let ddx = controller.acceleration_player_force.x;
        if ddx != 0.0 {
            sprite.flip_x = ddx < 0.0;
        }

        let ddy = controller.acceleration_player_force.y;

        let animation_state = if controller.acceleration_player_force == Vec2::ZERO {
            PlayerAnimationState::Idling
        } else if ddy < 0. {
            if controller.is_rolling {
                PlayerAnimationState::FrontRolling
            } else {
                PlayerAnimationState::FrontWalking
            }
        } else if controller.is_rolling {
            PlayerAnimationState::Rolling
        } else {
            PlayerAnimationState::Walking
        };

        animation.update_state(animation_state);
    }
}

/// Update the animation timer.
fn update_animation_timer(time: Res<Time>, mut query: Query<&mut PlayerAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas(mut query: Query<(&PlayerAnimation, &mut TextureAtlas)>) {
    for (animation, mut atlas) in &mut query {
        if animation.changed() {
            atlas.index = animation.get_atlas_index();
        }
    }
}

/// If the player is moving, play a step sound effect synchronized with the animation.
fn trigger_step_sfx(mut commands: Commands, mut step_query: Query<&PlayerAnimation>) {
    for animation in &mut step_query {
        if animation.state == PlayerAnimationState::Walking
            && animation.changed()
            && (animation.frame == 2 || animation.frame == 5)
        {
            commands.trigger(Sfx::Step);
        }
    }
}

// If the player is rolling, play a roll sound effect.
fn trigger_roll_sfx(mut commands: Commands, mut roll_query: Query<&PlayerAnimation>) {
    for animation in &mut roll_query {
        if animation.state == PlayerAnimationState::Rolling
            && animation.changed()
            && animation.frame == 1
        {
            commands.trigger(Sfx::Roll);
        }
    }
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAnimation {
    timer: Timer,
    frame: usize,
    state: PlayerAnimationState,
    /// frames contains a hashmap of animation state, start frame, frame length and duration
    frames: HashMap<PlayerAnimationState, (usize, usize, Duration)>,
}

#[derive(Clone, Copy, Reflect, PartialEq, Debug, Eq, Hash)]
pub enum PlayerAnimationState {
    Idling,
    Walking,
    Rolling,
    FrontIdling,
    FrontWalking,
    FrontRolling,
}

impl PlayerAnimation {
    pub fn new(frames: HashMap<PlayerAnimationState, (usize, usize, Duration)>) -> Self {
        Self {
            timer: Timer::new(
                frames[&PlayerAnimationState::Idling].2,
                TimerMode::Repeating,
            ),
            frame: 0,
            state: PlayerAnimationState::Idling,
            frames,
        }
    }

    /// Update animation timers.
    fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.finished() {
            return;
        }
        self.frame = (self.frame + 1) % self.frames[&self.state].1;
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, new_state: PlayerAnimationState) {
        if self.state != new_state {
            self.state = new_state;
            self.frame = 0;
            self.timer = Timer::new(self.frames[&new_state].2, TimerMode::Repeating);
        }
    }

    /// Whether animation changed this tick.
    fn changed(&self) -> bool {
        self.timer.finished()
    }

    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        self.frames[&self.state].0 + self.frame
    }

    pub fn get_current_state(&self) -> PlayerAnimationState {
        self.state
    }

    /// resets the animation component to default
    pub fn reset(&mut self) {
        self.update_state(PlayerAnimationState::Idling);
        self.frame = 0;
        self.timer.tick(self.timer.duration());
    }
}
