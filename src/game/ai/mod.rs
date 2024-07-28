use bevy::app::App;
use bevy::prelude::*;

use crate::AppSet;
use crate::game::ai::patrol::Patrolling;
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

// It's wrapping an enum to ensure we only have one of these at a time
#[derive(Component)]
pub struct HasAiBehavior(pub AiBehavior);

pub enum AiBehavior {
    Idle,
    Patrolling,
    Chasing,
    Searching,
    ReturningToPatrol,
}

pub fn main_ai_behavior_system(
    mut query: Query<(Entity, &mut HasAiBehavior), (Without<Patrolling>)>,
    mut commands: Commands,
) {
    for (entity, ai_behavior) in query.iter_mut() {
        commands.entity(entity).insert(Patrolling);
    }
}

pub mod patrol {
    use std::time::Duration;

    use bevy::app::App;
    use bevy::prelude::*;

    use crate::AppSet::UpdateAi;
    use crate::game::grid::GridPosition;
    use crate::game::line_of_sight::vision::Facing;
    use crate::game::movement::GridMovement;
    use crate::game::spawn::enemy::{Enemy, ENEMY_PATROL_SPEED};
    use crate::screen::Screen;

    pub fn plugin(app: &mut App) {
        // systems
        app.add_systems(
            Update,
            follow_patrol_route
                .run_if(in_state(Screen::Playing))
                .in_set(UpdateAi),
        );

        // reflection
        app.register_type::<PatrolWaypoint>();
        app.register_type::<PatrolRoute>();
        app.register_type::<PatrolState>();
    }

    fn follow_patrol_route(
        mut query: Query<
            (
                &mut PatrolState,
                &PatrolRoute,
                &GridPosition,
                &mut Facing,
                &mut GridMovement,
            ),
            (With<Enemy>, With<Patrolling>),
        >,
        time: Res<Time>,
    ) {
        for (mut state, route, entity_position, mut facing, mut movement) in query.iter_mut() {
            // we're at the waypoint
            let direction_to_waypoint =
                entity_position.direction_to(&route.waypoints[state.current_waypoint].position);
            if direction_to_waypoint.length() <= 1.0 {
                state.wait_timer.tick(time.delta());
                facing.direction = route.waypoints[state.current_waypoint].facing.direction;

                // we've waited here long enough, advance the waypoint
                if state.wait_timer.finished() {
                    state.wait_timer.reset();
                    state.next_waypoint(route);
                }
            }
            // we're not at our target yet, so move towards it
            else {
                const ACCEL: f32 = ENEMY_PATROL_SPEED;
                movement.acceleration_player_force = direction_to_waypoint.normalize() * ACCEL;
            }
        }
    }

    #[derive(Component, Reflect, Debug, Clone)]
    pub struct Patrolling;

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
        pub mode: PatrolMode,
    }

    impl PatrolState {
        pub fn next_waypoint(&mut self, patrol_route: &PatrolRoute) -> usize {
            let current = self.current_waypoint;
            let direction = self.direction;
            let next = match patrol_route.mode {
                PatrolMode::Cycle => (current + 1) % patrol_route.waypoints.len(),
                PatrolMode::PingPong => {
                    if current == 0 && direction < 0 {
                        self.direction = -self.direction;
                        current + 1
                    } else if current == patrol_route.waypoints.len() - 1 && direction > 0 {
                        self.direction = -self.direction;
                        current - 1
                    } else {
                        (current as i32 + direction as i32) as usize
                    }
                }
            };

            self.current_waypoint = next;
            next
        }
    }

    #[derive(Debug, Clone, PartialEq, Reflect, Default)]
    pub enum PatrolMode {
        /// Cycle through the waypoints in order, returning to the start once finished
        #[default]
        Cycle,

        /// Go down-and-back
        PingPong,
    }

    #[derive(Component, Reflect, Debug, Clone, Default)]
    #[reflect(Component)]
    pub struct PatrolState {
        pub current_waypoint: usize,
        pub wait_timer: Timer,
        pub direction: i8, // 1 for forward, -1 for backward along route
    }

    #[derive(Bundle, Default)]
    pub struct PatrolBundle {
        pub state: PatrolState,
        pub route: PatrolRoute,
    }
}
