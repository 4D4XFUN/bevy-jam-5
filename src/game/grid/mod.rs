pub mod grid_layout;

pub mod collision;

use std::ops::{Add, Sub};

use crate::game::grid::grid_layout::GridLayout;
use crate::game::spawn::level::LevelWalls;
use crate::AppSet;
use bevy::app::App;
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy_ecs_ldtk::GridCoords;

pub fn plugin(app: &mut App) {
    app.init_resource::<GridLayout>();
    app.add_systems(Update, update_grid_when_level_changes);
    app.observe(fix_grid_position_system);

    app.add_plugins(collision::plugin);
    app.add_systems(
        Update,
        set_real_position_based_on_grid.in_set(AppSet::UpdateWorld),
    );

    app.register_type::<(GridPosition, GridLayout)>();
}

#[derive(Component)]
pub struct GridSprite;

#[derive(Component, Reflect, Default, Debug, Copy, Clone, PartialEq)]
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

    /// retrieves current coordinate and offset
    pub fn get_values(&self) -> (Vec2, Vec2) {
        (self.coordinates, self.offset)
    }

    /// sets the position to a fixed coordinate and offset
    pub fn set(&mut self, coordinates: Vec2, offset: Vec2) {
        self.coordinates = coordinates;
        self.offset = offset;
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

fn update_grid_when_level_changes(
    mut grid: ResMut<GridLayout>,
    level_walls: Res<LevelWalls>,
    mut commands: Commands,
) {
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
    commands.trigger(GridChangedEvent)
}

#[derive(Event)]
struct GridChangedEvent;

/// Reconciles LDTK's y-position (top of screen is 0) with ours/bevy's (bottom of screen is 0)
/// I'm not sure if this fits better in grid/ or ldtk/ module
fn fix_grid_position_system(
    _trigger: Trigger<GridChangedEvent>,
    mut query: Query<(&mut GridPosition, &GridCoords)>,
) {
    println!("Fixing grid positions");
    for (mut grid_pos, ldtk_grid_coords) in query.iter_mut() {
        let ldtk_y = ldtk_grid_coords.y;
        let fixed_y = ldtk_y;

        // println!("Fixing position {:?}, to have y={}. Current ldtk coords: {:?}", &grid_pos.coordinates, &fixed_y, &ldtk_grid_coords);
        grid_pos.coordinates.y = fixed_y as f32;
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
