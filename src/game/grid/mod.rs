pub mod grid_layout;

use std::ops::{Add, Sub};

use crate::game::grid::grid_layout::GridLayout;
use crate::game::spawn::level::LevelWalls;
use crate::game::spawn::player::Player;
use crate::input::DevAction;
use crate::screen::Screen;
use bevy::app::App;
use bevy::math::Vec2;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

pub fn plugin(app: &mut App) {
    app.init_resource::<GridLayout>()
        .add_systems(Update, update_grid_when_level_changes);

    // draw a grid overlay for debugging, change DebugOverlays #[default] state to stop doing this
    app.add_systems(Update, toggle_debug_overlays);
    app.init_state::<DebugOverlaysState>().add_systems(
        Update,
        (update_grid_debug_overlay, update_player_grid_debug_overlay)
            .run_if(in_state(DebugOverlaysState::Enabled)),
    );

    app.add_plugins(movement::plugin);
    app.add_plugins(collision::plugin);

    app.register_type::<(GridPosition, GridLayout)>();
}

/// Grid-based collision
pub mod collision {
    use bevy::prelude::*;

    use crate::game::grid::movement::GridMovement;
    use crate::game::grid::GridPosition;
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
        mut query_actors: Query<
            (&GridPosition, &GridCollider, &mut GridMovement),
            Without<Immovable>,
        >,
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
}

/// Grid-based movement
pub mod movement {
    use crate::game::grid::{GridLayout, GridPosition};
    use crate::input::PlayerAction;
    use crate::AppSet;
    use std::time::Duration;
    use bevy::prelude::*;
    use leafwing_input_manager::prelude::ActionState;

    pub fn plugin(app: &mut App) {
        app.add_systems(Update, respond_to_input.in_set(AppSet::UpdateVirtualGrid));
        app.add_systems(Update, apply_movement.in_set(AppSet::Update));
        app.add_systems(Update, apply_roll.in_set(AppSet::Update));
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
        timer: Timer,
        total_time: f32,
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

    impl Default for Roll{
        fn default() -> Self {
            Self {
                timer: Timer::from_seconds(0.0, TimerMode::Once),
                total_time: 2.0,
            }
        }
    
    }

    pub fn respond_to_input(mut query: Query<(&ActionState<PlayerAction>, &mut GridMovement)>) {
        for (action_state, mut controller) in query.iter_mut() {
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

            controller.acceleration_player_force =
                intent * controller.acceleration_player_multiplier;

            if action_state.pressed(&PlayerAction::Roll) {
                controller.is_rolling = true; // this does get set to true, but not in apply_roll
            }
        }
    }

    pub fn apply_movement(
        mut query: Query<(&mut GridPosition, &mut GridMovement)>,
        time: Res<Time>,
    ) {
        let dt = time.delta_seconds();
        for (mut position, mut movement) in query.iter_mut() {
            let force = movement.current_force() * dt; // scale it by time

            // apply forces and friction
            let mut velocity = movement.velocity + force;
            velocity *= movement.friction;
            if velocity.length() < 0.01 {
                velocity = Vec2::ZERO;
            }
            movement.velocity = velocity;

            // move the player
            position.offset += movement.velocity * dt;
            position.fix_offset_overflow();
        }
    }

    pub fn apply_roll(
        mut query: Query<(&mut GridMovement, &mut Roll)>,
        time: Res<Time>,
    ) {
        let dt = time.delta_seconds();
        for (mut movement, mut roll) in query.iter_mut() {
            if movement.is_rolling { // this is never true???
                print!("how about in apply_roll? {:?}", movement.is_rolling);
                roll.timer.unpause();
                if roll.timer.elapsed_secs() >= roll.total_time {
                    movement.is_rolling = false;
                } else {
                    movement.acceleration_player_force = movement.current_force() * 2.0 * dt;
                    roll.timer.tick(Duration::from_secs_f32(dt));
                    print!("rolling");
                }
            }
            roll.timer.pause();
            print!("stop rolling")
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
}

#[derive(Component)]
pub struct GridSprite;

#[derive(Component, Reflect, Debug, Copy, Clone, PartialEq)]
#[reflect(Component)]
pub struct GridPosition {
    pub coordinates: Vec2, // the full-square coordinates on the whole grid
    pub offset: Vec2,      // the offset within a single grid cell
}

impl GridPosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            coordinates: Vec2::new(x, y),
            offset: Vec2::ZERO,
        }
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn _actual_coordinates(&self) -> Vec2 {
        Vec2::new(
            self.coordinates.x + self.offset.x,
            self.coordinates.y + self.offset.y,
        )
    }

    /// If the offset is more than a whole cell, then update the coordinates (and bring the offset back within 0..1)
    pub fn fix_offset_overflow(&mut self) {
        if self.offset.x >= 0.5 {
            self.coordinates.x += 1.;
            self.offset.x -= 1.;
        }
        if self.offset.y >= 0.5 {
            self.coordinates.y += 1.;
            self.offset.y -= 1.;
        }
        if self.offset.x < -0.5 {
            self.coordinates.x -= 1.;
            self.offset.x += 1.;
        }
        if self.offset.y < -0.5 {
            self.coordinates.y -= 1.;
            self.offset.y += 1.;
        }
    }
}

impl Sub for GridPosition {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut res = Self {
            coordinates: self.coordinates.sub(rhs.coordinates),
            offset: self.offset.sub(rhs.offset),
        };
        res.fix_offset_overflow();
        res
    }
}

impl Add for GridPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut res = Self {
            coordinates: self.coordinates.add(rhs.coordinates),
            offset: self.offset.add(rhs.offset),
        };
        res.fix_offset_overflow();
        res
    }
}

impl Default for GridLayout {
    fn default() -> Self {
        GridLayout {
            square_size: 32.,
            width: 20,
            height: 10,
            origin: Vec2::ZERO,
            padding: 0.,
        }
    }
}

fn update_grid_when_level_changes(mut grid: ResMut<GridLayout>, level_walls: Res<LevelWalls>) {
    if !level_walls.is_changed() {
        return;
    }

    println!(
        "grid changed, level_walls: ({:?}, {:?})",
        level_walls.level_width, level_walls.level_height
    );
    let square_size = 16.; // we should reconcile this with the LDTK tile size
    grid.padding = square_size / 2.0;
    grid.width = level_walls.level_width as usize;
    grid.height = level_walls.level_height as usize;
    grid.square_size = square_size;

    grid.origin = Vec2::new(0., 0.);

    println!("Grid initialized: {:?}", grid);
}

#[derive(Component, Reflect, Debug, Copy, Clone, PartialEq)]
#[reflect(Component)]
struct GridOverlay;

#[derive(Component, Reflect, Debug, Copy, Clone, PartialEq)]
#[reflect(Component)]
struct PlayerGridSquareOverlay;

fn update_player_grid_debug_overlay(
    mut commands: Commands,
    grid: Res<GridLayout>,
    query: Query<
        &GridPosition,
        (
            With<Player>,
            Changed<GridPosition>,
            Without<PlayerGridSquareOverlay>,
        ),
    >,
    mut overlay_sprite: Query<&mut GridPosition, (With<PlayerGridSquareOverlay>, Without<Player>)>,
) {
    for player_pos in query.iter() {
        if overlay_sprite.is_empty() {
            commands.spawn((
                Name::new("DebugPlayerGridSquareMarker"),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(0.9, 0.0, 0.0, 0.2),
                        custom_size: Some(Vec2::splat(grid.square_size)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0., 0., 50.)),
                    ..default()
                },
                *player_pos,             // grid position
                PlayerGridSquareOverlay, // marker
            ));
        } else {
            for mut gp in overlay_sprite.iter_mut().take(1) {
                gp.coordinates = player_pos.coordinates;
            }
        }
    }
}

fn update_grid_debug_overlay(
    mut commands: Commands,
    grid: Res<GridLayout>,
    existing_overlays: Query<Entity, With<GridOverlay>>,
) {
    if !grid.is_changed() {
        return;
    }

    // despawn old overlays
    for e in existing_overlays.into_iter() {
        commands.entity(e).despawn_recursive()
    }

    // spawn a new overlay
    let name = format!("GridOverlay_{}x{}", grid.width, grid.height);
    let grid_entity = commands
        .spawn((
            GridOverlay,
            Name::new(name),
            GridSprite,
            SpatialBundle::default(),
            StateScoped(Screen::Playing),
        ))
        .id();

    // Spawn child sprites for each grid cell
    for y in 0..grid.height {
        for x in 0..grid.width {
            let position = grid.grid_to_world(&GridPosition::new(x as f32, y as f32));

            let alpha = 0.1;
            let color = if (x + y) % 2 == 0 {
                Color::srgba(0.9, 0.9, 0.9, alpha)
            } else {
                Color::srgba(0.8, 0.8, 0.8, alpha)
            };

            // Spawn the child sprite and parent it to the GridSprite
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::splat(grid.square_size)),
                        ..default()
                    },
                    transform: Transform::from_translation(position.extend(10.0)),
                    ..default()
                })
                .set_parent(grid_entity);
        }
    }
}

pub fn toggle_debug_overlays(
    current_state: Res<State<DebugOverlaysState>>,
    query: Query<&ActionState<DevAction>>,
    mut set_next_state: ResMut<NextState<DebugOverlaysState>>,
) {
    for act in query.iter() {
        if act.just_pressed(&DevAction::ToggleDebugOverlays) {
            set_next_state.set(match current_state.get() {
                DebugOverlaysState::Disabled => DebugOverlaysState::Enabled,
                DebugOverlaysState::Enabled => DebugOverlaysState::Disabled,
            });
        }
    }
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum DebugOverlaysState {
    #[default] // change this to disable all the debug grid drawing
    Disabled,
    Enabled,
}

#[cfg(test)]
mod tests {
    use crate::assert_vec2_close;

    use super::*;

    #[test]
    fn gridposition_subtraction() {
        let a = GridPosition {
            coordinates: Vec2::new(1., 1.),
            offset: Vec2::new(0.3, 0.3),
        };
        let b = GridPosition {
            coordinates: Vec2::new(2., 2.),
            offset: Vec2::new(0.7, 0.7),
        };

        let aminb = a - b;
        assert_vec2_close!(aminb.coordinates, Vec2::new(-2., -2.));
        assert_vec2_close!(aminb.offset, Vec2::new(0.6, 0.6));

        let bmina = b - a;
        assert_vec2_close!(bmina.coordinates, Vec2::new(1., 1.));
        assert_vec2_close!(bmina.offset, Vec2::new(0.4, 0.4));
    }
}
