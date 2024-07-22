use crate::game::spawn::level::LevelWalls;
use crate::screen::Screen;
use bevy::app::App;
use bevy::math::Vec2;
use bevy::prelude::*;
use std::ops::{Add, Sub};

pub fn plugin(app: &mut App) {
    app.init_resource::<GridLayout>()
        .add_systems(Update, update_grid_when_level_changes)
        .add_systems(Update, update_transform_for_entities_on_grid);

    // draw a grid overlay for debugging, change DebugOverlays #[default] state to stop doing this
    app.init_state::<DebugOverlays>().add_systems(
        Update,
        update_grid_debug_overlay.run_if(in_state(DebugOverlays::Enabled)),
    );

    app.add_plugins(movement::plugin);
    app.register_type::<(GridPosition, GridLayout)>();
}

/// Grid-based movement using
pub mod movement {
    use bevy::prelude::*;
    use crate::AppSet;
    use crate::game::grid::{GridLayout, GridPosition};

    pub fn plugin(app: &mut App) {
        app.add_systems(Update, respond_to_input.in_set(AppSet::UpdateVirtualGrid));
        app.add_systems(Update, apply_movement.in_set(AppSet::Update));
        app.add_systems(Update, set_real_position_based_on_grid.in_set(AppSet::UpdateWorld));

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
    }

    impl Default for GridMovement {
        fn default() -> Self {
            Self {
                velocity: Vec2::ZERO,
                friction: 0.9,
                acceleration_player_force: Vec2::ZERO,
                acceleration_external_force: Vec2::ZERO,
                acceleration_player_multiplier: 50.,
            }
        }
    }

    pub fn respond_to_input(
        input: Res<ButtonInput<KeyCode>>,
        mut controller_query: Query<&mut GridMovement>,
    ) {
        let mut intent = Vec2::ZERO;
        if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
            intent.y += 1.0;
        }
        if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
            intent.y -= 1.0;
        }
        if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
            intent.x -= 1.0;
        }
        if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
            intent.x += 1.0;
        }
        // Normalize so that diagonal movement has the same speed as
        // horizontal and vertical movement.
        let intent = intent.normalize_or_zero();

        for mut controller in &mut controller_query {
            controller.acceleration_player_force = intent * controller.acceleration_player_multiplier;
        }
    }
    pub fn apply_movement(mut query: Query<(&mut GridPosition, &mut GridMovement)>, time: Res<Time>) {
        let dt = time.delta_seconds();
        for (mut position, mut movement) in query.iter_mut() {
            let force: Vec2 = movement.acceleration_player_force + movement.acceleration_external_force;
            let force = force * dt; // scale it by time

            // apply forces and friction
            let mut velocity = movement.velocity + force;
            velocity *= movement.friction;
            movement.velocity = velocity;

            // move the player
            position.offset += movement.velocity * dt;
            position.fix_offset_overflow();
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

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct GridLayout {
    pub square_size: f32,
    pub width: usize,
    pub height: usize,
    pub origin: Vec2,
    pub padding: f32,
}

impl GridLayout {
    pub fn grid_to_world(&self, grid_pos: &GridPosition) -> Vec2 {
        Vec2::new(
            self.origin.x + grid_pos.coordinates.x * self.square_size + self.padding + (grid_pos.offset.x * self.square_size),
            self.origin.y + grid_pos.coordinates.y * self.square_size + self.padding + (grid_pos.offset.y * self.square_size),
        )
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
    /// If the offset is more than a whole cell, then update the coordinates (and bring the offset back within 0..1)
    pub fn fix_offset_overflow(&mut self) {
        if self.offset.x > 1. {
            self.coordinates.x += 1.;
            self.offset.x -= 1.;
        }
        if self.offset.y > 1. {
            self.coordinates.y += 1.;
            self.offset.y -= 1.;
        }
        if self.offset.x < 0. {
            self.coordinates.x -= 1.;
            self.offset.x += 1.;
        }
        if self.offset.y < 0. {
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

fn update_transform_for_entities_on_grid(
    mut query: Query<(&GridPosition, &mut Transform), Changed<GridPosition>>,
    grid: Res<GridLayout>,
) {
    for (grid_pos, mut transform) in query.iter_mut() {
        let world_pos: Vec2 = grid.grid_to_world(grid_pos);
        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
enum DebugOverlays {
    _Disabled,
    #[default] // change this to disable all the debug grid drawing
    Enabled,
}

#[cfg(test)]
mod tests {
    use crate::assert_vec2_close;
    use super::*;

    #[test]
    fn gridposition_subtraction() {
        let a = GridPosition { coordinates: Vec2::new(1., 1.), offset: Vec2::new(0.3, 0.3) };
        let b = GridPosition { coordinates: Vec2::new(2., 2.), offset: Vec2::new(0.7, 0.7) };

        let aminb = a - b;
        assert_vec2_close!(aminb.coordinates, Vec2::new(-2., -2.));
        assert_vec2_close!(aminb.offset, Vec2::new(0.6, 0.6));

        let bmina = b - a;
        assert_vec2_close!(bmina.coordinates, Vec2::new(1., 1.));
        assert_vec2_close!(bmina.offset, Vec2::new(0.4, 0.4));
    }
}
