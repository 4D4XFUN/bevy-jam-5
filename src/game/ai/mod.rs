use bevy::app::App;
use bevy::prelude::Component;

pub fn plugin(app: &mut App) {
    // plugins
    app.add_plugins(patrol::plugin);

    // systems

    // reflection
}

/// Hunters have vision, movement, and look for prey. When they see one, they chase it.
#[derive(Component)]
pub struct Hunter;

/// Preys are targets for hunters to see and chase down. There's probably only one - the player.
#[derive(Component)]
pub struct _Prey;

mod patrol {
    use std::time::Duration;

    use bevy::app::App;
    use bevy::prelude::{Component, Reflect};

    use crate::game::grid::GridPosition;
    use crate::game::line_of_sight::vision::Facing;

    pub fn plugin(app: &mut App) {
        // systems

        // reflection
        app.register_type::<PatrolWaypoint>();
        app.register_type::<PatrolRoute>();
    }

    #[derive(Component, Reflect, Debug, Clone)]
    #[reflect(Component)]
    pub struct PatrolWaypoint {
        pub position: GridPosition,
        pub facing: Facing,
        pub wait_time: Duration,
    }

    #[derive(Component, Reflect, Debug, Clone)]
    #[reflect(Component)]
    pub struct PatrolRoute {
        pub waypoints: Vec<PatrolWaypoint>,
        pub current: usize,
        pub mode: PatrolMode,
    }

    #[derive(Debug, Clone, PartialEq, Reflect)]
    pub enum PatrolMode {
        /// Stop at the last waypoint
        Once,

        /// Cycle through the waypoints in order, returning to the start once finished
        Cycle,

        /// Go down-and-back
        PingPong,
    }
}
