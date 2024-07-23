//! Represents the global grid and provides mapping functions from grid-coordinate space to world space and back

use crate::game::grid::GridPosition;
use bevy::math::Vec2;
use bevy::prelude::*;
use std::hash::{Hash, Hasher};

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
            self.origin.x
                + grid_pos.coordinates.x * self.square_size
                + self.padding
                + (grid_pos.offset.x * self.square_size),
            self.origin.y
                + grid_pos.coordinates.y * self.square_size
                + self.padding
                + (grid_pos.offset.y * self.square_size),
        )
    }

    /// Get the positions of the corners of a position on the grid, in world (pixel) coordinates
    pub fn corners(&self, grid_pos: &GridPosition) -> Corners {
        let sw = self.grid_to_world(grid_pos);
        let ne = sw
            .with_x(sw.x + self.square_size)
            .with_y(sw.y + self.square_size);
        Corners {
            southwest: sw,
            northwest: Vec2::new(sw.x, ne.y),
            northeast: ne,
            southeast: Vec2::new(ne.x, sw.y),
        }
    }

    /// Gets line segments for each side of the box that forms the grid position passed in
    pub fn sides(&self, grid_pos: &GridPosition) -> Sides {
        let corners = self.corners(grid_pos);

        Sides {
            north: LineSegment::new(corners.northwest, corners.northeast),
            south: LineSegment::new(corners.southwest, corners.southeast),
            east: LineSegment::new(corners.southeast, corners.northeast),
            west: LineSegment::new(corners.southwest, corners.northwest),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LineSegment {
    pub a: Vec2,
    pub b: Vec2,
}

impl PartialEq for LineSegment {
    fn eq(&self, other: &Self) -> bool {
        (self.a.x.floor() as i32 == other.a.x.floor() as i32
            && self.a.y.floor() as i32 == other.a.y.floor() as i32
            && self.b.x.floor() as i32 == other.b.x.floor() as i32
            && self.b.y.floor() as i32 == other.b.y.floor() as i32)
            || (self.a.x.floor() as i32 == other.b.x.floor() as i32
                && self.a.y.floor() as i32 == other.b.y.floor() as i32
                && self.b.x.floor() as i32 == other.a.x.floor() as i32
                && self.b.y.floor() as i32 == other.a.y.floor() as i32)
    }
}

impl Eq for LineSegment {}

impl LineSegment {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Self { a, b }
    }
}
impl Hash for LineSegment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let (min_x, max_x) = if self.a.x < self.b.x {
            (self.a.x, self.b.x)
        } else {
            (self.b.x, self.a.x)
        };
        let (min_y, max_y) = if self.a.y < self.b.y {
            (self.a.y, self.b.y)
        } else {
            (self.b.y, self.a.y)
        };

        (min_x.floor() as i32).hash(state);
        (min_y.floor() as i32).hash(state);
        (max_x.floor() as i32).hash(state);
        (max_y.floor() as i32).hash(state);
    }
}
#[derive(Copy, Clone, Debug)]
pub struct Corners {
    pub southwest: Vec2,
    pub northwest: Vec2,
    pub northeast: Vec2,
    pub southeast: Vec2,
}

#[derive(Copy, Clone, Debug)]
pub struct Sides {
    pub north: LineSegment,
    pub east: LineSegment,
    pub south: LineSegment,
    pub west: LineSegment,
}

impl Sides {
    pub fn _all(&self) -> Vec<LineSegment> {
        vec![self.north, self.east, self.south, self.west]
    }
}
