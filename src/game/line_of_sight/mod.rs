use bevy::prelude::*;

use crate::game::grid::grid_layout::LineSegment;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(front_facing_edges::plugin);

    #[cfg(feature = "dev")]
    app.add_plugins(debug_overlay::plugin);

    // Reflection registrations
    app.register_type::<LineOfSightSource>();
}

#[derive(Bundle)]
pub struct LineOfSightBundle {
    pub line_of_sight_source: LineOfSightSource,
    pub facing_walls_cache: FacingWallsCache,
}

impl Default for LineOfSightBundle {
    fn default() -> Self {
        Self {
            line_of_sight_source: LineOfSightSource {
                max_distance_in_grid_units: 10.,
            },
            facing_walls_cache: FacingWallsCache::new(),
        }
    }
}

#[derive(Component, Debug, Reflect, Copy, Clone)]
#[reflect(Component)]
pub struct LineOfSightSource {
    pub max_distance_in_grid_units: f32,
}

#[derive(Component, Debug, Clone)]
pub struct FacingWallsCache {
    // previous grid position, used to prevent recomputing LOS if tile hasn't changed
    pub last_grid_position: Vec2,
    pub facing_wall_edges: Vec<LineSegment>,
}

impl FacingWallsCache {
    pub fn new() -> Self {
        Self {
            last_grid_position: Vec2::splat(-69420.),
            facing_wall_edges: vec![],
        }
    }
}

/// Finds front facing edges of walls (from player's perspective)
pub mod front_facing_edges {
    use crate::game::grid::grid_layout::{GridLayout, LineSegment};
    use crate::game::grid::GridPosition;
    use crate::game::line_of_sight::{FacingWallsCache, LineOfSightSource};
    use crate::game::spawn::level::LevelWalls;
    use crate::AppSet;
    use bevy::prelude::*;

    pub fn plugin(app: &mut App) {
        // Systems
        app.add_systems(
            Update,
            update_front_facing_edges_when_grid_pos_changes.in_set(AppSet::Update),
        );
    }

    #[derive(Event, Debug, Copy, Clone)]
    pub struct FacingWallsChanged {
        pub previous_source: Vec2, // grid coordinates
        pub new_source: Vec2,      // grid coordinates
    }

    /// Whenever the player moves a whole tile, we have to recompute which parts of walls are facing them
    pub fn update_front_facing_edges_when_grid_pos_changes(
        mut commands: Commands,
        mut query: Query<
            (
                Entity,
                &GridPosition,
                &mut LineOfSightSource,
                &mut FacingWallsCache,
            ),
            Changed<GridPosition>,
        >,
        walls: Res<LevelWalls>,
        grid: Res<GridLayout>,
    ) {
        for (e, &player_position, player_los, mut facing_walls_cache) in query.iter_mut() {
            // skip if we haven't actually changed tiles
            if player_position.coordinates == facing_walls_cache.last_grid_position {
                continue;
            }
            commands.trigger_targets(
                FacingWallsChanged {
                    previous_source: facing_walls_cache.last_grid_position,
                    new_source: player_position.coordinates,
                },
                e,
            ); // notify observers
            facing_walls_cache.last_grid_position = player_position.coordinates;

            // compute nearest edges for every wall
            let mut edges: Vec<LineSegment> = vec![];
            let pc = player_position.coordinates;
            for wall in walls.wall_locations.iter() {
                // for now, stick all edges in for drawing
                let wall_pos = GridPosition::new(wall.x as f32, wall.y as f32)
                    .with_offset(Vec2::new(-0.5, -0.5));

                // skip walls that are further than our LOS distance
                if (player_position - wall_pos).coordinates.length()
                    > player_los.max_distance_in_grid_units
                {
                    continue;
                }

                // add all near-facing edges of walls to a list
                let sides = grid.sides(&wall_pos);
                if pc.x > wall_pos.coordinates.x && !walls.collides(wall.x + 1, wall.y) {
                    edges.push(sides.east)
                } else if pc.x < wall_pos.coordinates.x && !walls.collides(wall.x - 1, wall.y) {
                    edges.push(sides.west)
                }
                if pc.y > wall_pos.coordinates.y && !walls.collides(wall.x, wall.y + 1) {
                    edges.push(sides.north)
                } else if pc.y < wall_pos.coordinates.y && !walls.collides(wall.x, wall.y - 1) {
                    edges.push(sides.south)
                }
            }

            facing_walls_cache.facing_wall_edges = edges;
        }
    }
}

pub mod debug_overlay {
    use bevy::prelude::*;

    use crate::game::grid::DebugOverlaysState;
    use crate::game::line_of_sight::FacingWallsCache;
    use crate::AppSet;

    pub fn plugin(app: &mut App) {
        app.add_systems(
            Update,
            redraw_front_facing_edges
                .in_set(AppSet::UpdateWorld)
                .run_if(in_state(DebugOverlaysState::Enabled)),
        );
    }

    /// Every update, if the grid coords changed, redraw the overlay of edges facing the player
    pub fn redraw_front_facing_edges(
        mut gizmos: Gizmos,
        front_facing_edges_query: Query<&FacingWallsCache>,
    ) {
        for wall_cache in front_facing_edges_query.iter() {
            for edge in wall_cache.facing_wall_edges.iter() {
                gizmos.line_2d(edge.a, edge.b, Color::srgb(0., 1., 0.)); // bright green
            }
        }
    }
}
