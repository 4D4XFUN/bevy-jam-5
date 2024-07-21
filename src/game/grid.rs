use std::ops::Sub;
use bevy::app::App;
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::screen::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<GridLayout>()
        .add_systems(Startup, init_grid)
        .add_systems(OnEnter(Screen::Playing), setup_grid)
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
            self.origin.x + grid_pos.x * self.square_size,
            self.origin.y + grid_pos.y * self.square_size,
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
            padding: 20.,
        }
    }
}

fn init_grid(mut grid: ResMut<GridLayout>, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.single();

    let available_width = window.width() - (2.0 * grid.padding);
    let available_height = window.height() - (2.0 * grid.padding);

    let square_size = 16. * 3.;

    let square_width = (available_width / square_size) as usize;
    let actual_grid_width = (square_width as f32) * square_size;
    let square_height = (available_height / square_size) as usize;
    let actual_grid_height = (square_height as f32) * square_size;

    grid.width = square_width;
    grid.height = square_height;
    grid.square_size = square_size;

    grid.origin = Vec2::new(
        (0. - actual_grid_width + square_size) / 2.,
        (0. - actual_grid_height + square_size) / 2.,
    );

    println!("Grid initialized: {:?}", grid);
}

fn setup_grid(mut commands: Commands, grid: Res<GridLayout>) {
    let grid_entity = commands
        .spawn((
            GridSprite,
            SpatialBundle::default(),
            StateScoped(Screen::Playing), // despawn when we stop playing, along with all children
        ))
        .id();

    // Spawn child sprites for each grid cell
    for y in 0..grid.height {
        for x in 0..grid.width {
            let position =
                grid.origin + Vec2::new(x as f32 * grid.square_size, y as f32 * grid.square_size);

            let color = if (x + y) % 2 == 0 {
                Color::srgb(0.9, 0.9, 0.9)
            } else {
                Color::srgb(0.8, 0.8, 0.8)
            };

            // Spawn the child sprite and parent it to the GridSprite
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::splat(grid.square_size)),
                        ..default()
                    },
                    transform: Transform::from_translation(position.extend(-100.0)),
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