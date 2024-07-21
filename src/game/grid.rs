use crate::game::spawn::level::LevelWalls;
use crate::screen::Screen;
use bevy::app::App;
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use log::log;
use std::fmt::format;
use std::ops::Sub;

pub fn plugin(app: &mut App) {
    app.init_resource::<GridLayout>()
        .add_systems(Update, update_grid_when_level_changes)
        .add_systems(Update, update_grid_debug_overlay)
        .add_systems(Update, update_transform_for_entities_on_grid);

    app.register_type::<(GridPosition, GridLayout)>();
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
    pub fn grid_to_world(&self, grid_pos: Vec2) -> Vec2 {
        Vec2::new(
            self.origin.x + grid_pos.x * self.square_size + self.padding,
            self.origin.y + grid_pos.y * self.square_size + self.padding,
        )
    }
}

#[derive(Component)]
pub struct GridSprite;

#[derive(Component, Reflect, Debug, Copy, Clone, PartialEq)]
#[reflect(Component)]
pub struct GridPosition(pub Vec2);
impl GridPosition {
    pub fn new(x: f32, y: f32) -> Self {
        GridPosition(Vec2::new(x, y))
    }
}

impl Sub for GridPosition {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.sub(rhs.0))
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

fn update_grid_when_level_changes(mut grid: ResMut<GridLayout>, mut level_walls: Res<LevelWalls>) {
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
    mut existing_overlays: Query<(Entity), (With<GridOverlay>)>,
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
            let position = grid.grid_to_world(Vec2::new(x as f32, y as f32));

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
        let world_pos: Vec2 = grid.grid_to_world(grid_pos.0);
        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gridposition_subtraction() {
        let a = GridPosition(Vec2::new(1., 1.));
        let b = GridPosition(Vec2::new(2., 2.));

        let aminb = a - b;
        assert_eq!(aminb.0, Vec2::new(-1., -1.));

        let bmina = b - a;
        assert_eq!(bmina.0, Vec2::new(1., 1.));
    }
}
