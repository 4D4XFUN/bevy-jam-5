use crate::game::grid::grid_layout::GridLayout;
use crate::game::grid::GridPosition;
use crate::game::line_of_sight::vision::VisionAbility;
use crate::game::spawn::level::LevelVisionBlockers;
use crate::geometry_2d::line_segment::LineSegment;

/// Finds front facing edges of walls (from player's perspective)
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    // Systems
    app.observe(update_front_facing_edges);
}

#[derive(Event)]
pub struct RebuildCache;

#[derive(Component, Debug, Clone, Default)]
pub struct FacingWallsCache {
    // previous grid position, used to prevent recomputing LOS if tile hasn't changed
    pub last_grid_position: Vec2,
    pub facing_wall_edges: Vec<LineSegment>,
}

impl FacingWallsCache {
    pub fn new() -> Self {
        Self {
            last_grid_position: Vec2::splat(-10.),
            facing_wall_edges: vec![],
        }
    }
}

/// Whenever the player moves a whole tile, we have to recompute which parts of walls are facing them
pub fn update_front_facing_edges(
    _trigger: Trigger<RebuildCache>,
    mut query: Query<(&GridPosition, &VisionAbility, &mut FacingWallsCache)>,
    walls: Res<LevelVisionBlockers>,
    grid: Res<GridLayout>,
) {
    for (&player_position, vision_ability, mut facing_walls_cache) in query.iter_mut() {
        // skip if we haven't actually changed tiles
        if player_position.coordinates == facing_walls_cache.last_grid_position {
            continue;
        }
        // info!("Recomputing facing walls for {:?} (was {:?}", player_position.coordinates, facing_walls_cache.last_grid_position);
        facing_walls_cache.last_grid_position = player_position.coordinates;

        // compute nearest edges for every wall
        let mut edges: Vec<LineSegment> = vec![];
        let pc = player_position.coordinates;

        // todo we can use a grid bounding box and iterate x,y over the vision radius instead to do less work when there's lots of entities with LOS
        for wall in walls.vision_blocker_locations.iter() {
            let wall_pos =
                GridPosition::new(wall.x as f32, wall.y as f32).with_offset(Vec2::new(-0.5, -0.5));

            // skip walls that are further than our LOS distance
            if (player_position - wall_pos).coordinates.length()
                > vision_ability.range_in_grid_units
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

        // sort edges based on distance from player
        let player_world_pos = grid.grid_to_world(&player_position);
        edges.sort_by(|a, b| {
            let dist_to_a = (player_world_pos - a.center).length();
            let dist_to_b = (player_world_pos - b.center).length();
            dist_to_a.total_cmp(&dist_to_b)
        });

        facing_walls_cache.facing_wall_edges = edges;
    }
}
