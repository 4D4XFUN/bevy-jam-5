//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use std::time::Duration;

use bevy::prelude::*;

use super::audio::sfx::Sfx;
use crate::game::movement::{GridMovement, Roll};
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

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAnimation {
    timer: Timer,
    frame: usize,
    state: PlayerAnimationState,
}

#[derive(Clone, Copy, Reflect, PartialEq, Debug)]
pub enum PlayerAnimationState {
    Idling,
    Walking,
    Rolling,
    FrontIdling,
    FrontWalking,
    FrontRolling,
}

impl PlayerAnimation {
    /// The number of idle frames.
    const IDLE_FRAMES: usize = 4;
    /// The duration of each idle frame.
    const IDLE_INTERVAL: Duration = Duration::from_millis(500);
    fn idling() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Idling,
        }
    }
    fn idling_front() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::FrontIdling,
        }
    }

    /// The number of walking frames.
    const WALKING_FRAMES: usize = 4;
    /// The duration of each walking frame.
    const WALKING_INTERVAL: Duration = Duration::from_millis(100);

    fn walking() -> Self {
        Self {
            timer: Timer::new(Self::WALKING_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Walking,
        }
    }
    fn walking_front() -> Self {
        Self {
            timer: Timer::new(Self::WALKING_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::FrontWalking,
        }
    }

    const ROLLING_FRAMES: usize = 7;
    fn rolling_interval() -> Duration {
        Roll::ROLL_TIME / Self::ROLLING_FRAMES as u32
    }

    fn rolling() -> Self {
        Self {
            timer: Timer::new(Self::rolling_interval(), TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Rolling,
        }
    }
    fn rolling_front() -> Self {
        Self {
            timer: Timer::new(Self::rolling_interval(), TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::FrontRolling,
        }
    }

    pub fn new() -> Self {
        Self::idling()
    }

    /// Update animation timers.
    fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.finished() {
            return;
        }
        self.frame = (self.frame + 1)
            % match self.state {
                PlayerAnimationState::Idling => Self::IDLE_FRAMES,
                PlayerAnimationState::Walking => Self::WALKING_FRAMES,
                PlayerAnimationState::Rolling => Self::ROLLING_FRAMES,
                PlayerAnimationState::FrontIdling => Self::IDLE_FRAMES,
                PlayerAnimationState::FrontWalking => Self::WALKING_FRAMES,
                PlayerAnimationState::FrontRolling => Self::ROLLING_FRAMES,
            };
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, state: PlayerAnimationState) {
        if self.state != state {
            match state {
                PlayerAnimationState::Idling => *self = Self::idling(),
                PlayerAnimationState::Walking => *self = Self::walking(),
                PlayerAnimationState::Rolling => *self = Self::rolling(),
                PlayerAnimationState::FrontIdling => *self = Self::idling_front(),
                PlayerAnimationState::FrontWalking => *self = Self::walking_front(),
                PlayerAnimationState::FrontRolling => *self = Self::rolling_front(),
            }
        }
    }

    /// Whether animation changed this tick.
    fn changed(&self) -> bool {
        self.timer.finished()
    }

    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        match self.state {
            PlayerAnimationState::FrontIdling => self.frame,
            PlayerAnimationState::FrontWalking => 7 + self.frame,
            PlayerAnimationState::FrontRolling => 7 * 2 + self.frame,
            PlayerAnimationState::Idling => 7 * 3 + self.frame,
            PlayerAnimationState::Walking => 7 * 4 + self.frame,
            PlayerAnimationState::Rolling => 7 * 5 + self.frame,
        }
    }

    pub fn get_current_state(&self) -> PlayerAnimationState {
        self.state
    }
}
