/// Grid-based collision
use bevy::prelude::*;

use crate::game::grid::GridPosition;
use crate::game::movement::GridMovement;
use crate::game::spawn::level::LevelWalls;
use crate::AppSet;

pub fn plugin(app: &mut App) {
    app.register_type::<GridCollider>();
    // app.add_systems(Update, apply_collision_forces.in_set(AppSet::UpdateVirtualGrid));
    app.add_systems(Update, simple_wall_collisions.in_set(AppSet::Update));
}

#[derive(Component, Reflect, Debug, Copy, Clone, PartialEq)]
#[reflect(Component)]
pub struct GridCollider {
    radius: f32,
}

#[derive(Component, Reflect, Debug, Copy, Clone, PartialEq)]
#[reflect(Component)]
pub struct Immovable;

impl Default for GridCollider {
    fn default() -> Self {
        Self { radius: 1. }
    }
}

/// https://deepnight.net/tutorial/a-simple-platformer-engine-part-1-basics/
pub fn simple_wall_collisions(
    walls: Res<LevelWalls>,
    mut query_player: Query<(&mut GridPosition, &mut GridMovement)>,
) {
    const COLLIDE_SUBGRID_DIST_POS: f32 = 0.2; // between 0..1, a fractional piece of the grid that we're allowed to move towards a wall while next to it
    const COLLIDE_SUBGRID_DIST_NEG: f32 = -0.2; // between 0..1, a fractional piece of the grid that we're allowed to move towards a wall while next to it

    let walls = &walls;
    for (mut player, _movement) in query_player.iter_mut() {
        let (cx, cy) = (player.coordinates.x as i32, player.coordinates.y as i32);
        let (xr, yr) = (player.offset.x, player.offset.y);

        if walls.collides(cx - 1, cy) && xr < COLLIDE_SUBGRID_DIST_NEG {
            // println!("{cx} {xr} collided in x");
            player.offset.x = COLLIDE_SUBGRID_DIST_NEG;
        }
        if walls.collides(cx, cy - 1) && yr < COLLIDE_SUBGRID_DIST_NEG {
            // println!("{cy} {yr} collided in y");
            player.offset.y = COLLIDE_SUBGRID_DIST_NEG;
        }
        if walls.collides(cx + 1, cy) && xr > COLLIDE_SUBGRID_DIST_POS {
            // println!("{cx} {xr} collided in x");
            player.offset.x = COLLIDE_SUBGRID_DIST_POS;
        }
        if walls.collides(cx, cy + 1) && yr > COLLIDE_SUBGRID_DIST_POS {
            // println!("{cy} {yr} collided in y");
            player.offset.y = COLLIDE_SUBGRID_DIST_POS;
        }
    }
}

/// https://deepnight.net/tutorial/a-simple-platformer-engine-part-2-collisions/
/// this didn't end up working as well as the simpler one above. Keeping for ref, we can always delete if we hate it
pub fn _apply_collision_forces(
    walls: Res<LevelWalls>,
    mut query_actors: Query<(&GridPosition, &GridCollider, &mut GridMovement), Without<Immovable>>,
) {
    const WALL_COLLIDER_RADIUS: f32 = 1.0; // assume walls all have a 1-grid-unit-radius circle around them
    const REPEL_FORCE: f32 = 1.; // grid units per sec squared

    for (actor_pos, actor_collider, mut actor_movement) in query_actors.iter_mut() {
        // get any wall within 2 units of us as a fast distance check
        let nearby_walls: Vec<Vec2> = walls
            .wall_locations
            .iter()
            .map(|&w| GridPosition::new(w.x as f32, w.y as f32)._actual_coordinates())
            .filter(|&w| {
                (w.x - actor_pos.coordinates.x).abs() <= 2.
                    && (w.y - actor_pos.coordinates.y).abs() <= 2.
            })
            .collect();

        for wall in nearby_walls.into_iter() {
            let actor_pos = actor_pos._actual_coordinates();
            let dx = actor_pos.x - wall.x;
            let dy = actor_pos.y - wall.y;
            let dist = ((dx * dx) + (dy * dy)).sqrt();

            let max_collide_dist = actor_collider.radius + WALL_COLLIDER_RADIUS;
            if dist >= max_collide_dist {
                continue;
            }

            println!(
                "Player at {},{} Colliding with wall at {},{} - dist {}, max_dist: {}",
                actor_pos.x, actor_pos.y, wall.x, wall.y, dist, max_collide_dist
            );

            let repel_power = (max_collide_dist - dist) / max_collide_dist;

            let ang = dy.atan2(dx); // wtf?
            let ddx = ang.cos() * repel_power * REPEL_FORCE;
            let ddy = ang.sin() * repel_power * REPEL_FORCE;
            actor_movement.velocity.x += ddx;
            actor_movement.velocity.y += ddy;

            println!("Applied collision force of {}, {} to player", ddx, ddy);
        }
    }
}
