use bevy::app::App;
use bevy::prelude::*;

use crate::AppSet;
use crate::screen::Screen;

pub fn plugin(app: &mut App) {
    // plugins
    app.add_plugins(patrol::plugin);

    // systems
    app.add_systems(
        Update,
        (main_ai_behavior_system)
            .chain()
            .run_if(in_state(Screen::Playing))
            .in_set(AppSet::UpdateAi),
    );
}

/// Hunters have vision, movement, and look for prey. When they see one, they chase it.
#[derive(Component)]
pub struct Hunter;

/// Preys are targets for hunters to see and chase down. There's probably only one - the player.
#[derive(Component)]
pub struct _Prey;

pub fn main_ai_behavior_system() {}

mod patrol {
    use std::time::Duration;

    use bevy::app::App;
    use bevy::prelude::*;

    use crate::game::grid::GridPosition;
    use crate::game::line_of_sight::vision::Facing;

    pub fn plugin(app: &mut App) {
        // systems

        // reflection
        app.register_type::<PatrolWaypoint>();
        app.register_type::<PatrolRoute>();
        app.register_type::<PatrolState>();
    }

    #[derive(Component, Reflect, Debug, Clone)]
    #[reflect(Component)]
    pub struct PatrolWaypoint {
        pub position: GridPosition,
        pub facing: Facing,
        pub wait_time: Duration,
    }

    #[derive(Component, Reflect, Debug, Clone, Default)]
    #[reflect(Component)]
    pub struct PatrolRoute {
        pub waypoints: Vec<PatrolWaypoint>,
        pub current: usize,
        pub mode: PatrolMode,
    }

    #[derive(Debug, Clone, PartialEq, Reflect, Default)]
    pub enum PatrolMode {
        /// Stop at the last waypoint
        Once,

        /// Cycle through the waypoints in order, returning to the start once finished
        #[default]
        Cycle,

        /// Go down-and-back
        PingPong,
    }

    #[derive(Component, Reflect, Debug, Clone, Default)]
    #[reflect(Component)]
    struct PatrolState {
        current_waypoint: usize,
        wait_timer: Timer,
        direction: i8, // 1 for forward, -1 for backward along route
    }

    #[derive(Bundle, Default)]
    pub struct PatrolBundle {
        state: PatrolState,
        route: PatrolRoute,
    }
}
