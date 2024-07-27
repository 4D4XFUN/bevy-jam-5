//! Represents the global grid and provides mapping functions from grid-coordinate space to world space and back

use std::ops::Range;

use bevy::math::Vec2;
use bevy::prelude::*;

use crate::game::grid::GridPosition;
use crate::geometry_2d::line_segment::LineSegment;

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct GridLayout {
    pub square_size: f32,
    pub width: usize,
    pub height: usize,
    pub origin: Vec2,
    pub padding: Vec2,
}

impl Default for GridLayout {
    fn default() -> Self {
        GridLayout {
            square_size: 32.,
            width: 20,
            height: 10,
            origin: Vec2::ZERO,
            padding: Vec2::ZERO,
        }
    }
}

impl GridLayout {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            square_size: 16.,
            origin: Vec2::default(),
            padding: Vec2::default(),
        }
    }

    pub fn grid_to_world(&self, grid_pos: &GridPosition) -> Vec2 {
        Vec2::new(
            self.origin.x
                + self.padding.x
                + grid_pos.coordinates.x * self.square_size
                + (grid_pos.offset.x * self.square_size),
            self.origin.y
                + self.padding.y
                + grid_pos.coordinates.y * self.square_size
                + (grid_pos.offset.y * self.square_size),
        )
    }
    pub fn center_worldpos(&self) -> Vec2 {
        let half_width_px = self.width as f32 * self.square_size / 2.;
        let half_height_px = self.height as f32 * self.square_size / 2.;

        let offset = self.origin;

        offset + Vec2::new(half_width_px, half_height_px)
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

    pub fn bounding_box(
        &self,
        grid_position: &GridPosition,
        radius_in_grid_squares: f32,
    ) -> GridBoundingBox {
        // info!("bounding_box grid: {},{}; {:?} {}", self.width, self.height, grid_position, radius_in_grid_squares);

        let (x, y) = (
            grid_position.coordinates.x as u32,
            grid_position.coordinates.y as u32,
        );
        let (width, height) = (self.width as u32, self.height as u32);
        let radius_in_grid_squares = radius_in_grid_squares as u32;

        let x_min = x.saturating_sub(radius_in_grid_squares).clamp(0, width);
        let x_max = x.saturating_add(radius_in_grid_squares + 1).clamp(0, width);

        let y_min = y.saturating_sub(radius_in_grid_squares).clamp(0, height);
        let y_max = y
            .saturating_add(radius_in_grid_squares + 1)
            .clamp(0, height);

        GridBoundingBox {
            origin: UVec2::new(x_min, y_min),
            dimensions: UVec2::new(x_max - x_min, y_max - y_min),
        }
    }

    pub fn neighbors(&self, pos: &GridPosition) -> Vec<Vec2> {
        let bb = self.bounding_box(pos, 1.);
        bb.coords_range()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GridBoundingBox {
    pub origin: UVec2,
    pub dimensions: UVec2,
}

impl GridBoundingBox {
    pub fn xrange(&self) -> Range<u32> {
        self.origin.x..(self.origin.x + self.dimensions.x)
    }
    pub fn yrange(&self) -> Range<u32> {
        self.origin.y..(self.origin.y + self.dimensions.y)
    }

    pub fn coords_range(&self) -> Vec<Vec2> {
        let mut coords = vec![];
        for x in self.xrange() {
            for y in self.yrange() {
                coords.push(Vec2::new(x as f32, y as f32));
            }
        }
        coords
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

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(
        1,
        UVec2::new(2, 2),
        UVec2::new(3, 3);
        "Simple contained box"
    )]
    #[test_case(
        5,
        UVec2::new(0, 0),
        UVec2::new(9, 9);
        "Overflowing bounds"
    )]
    #[test_case(
        9000,
        UVec2::new(0, 0),
        UVec2::new(10, 10);
        "Huge"
    )]
    fn bounding_box(radius: usize, expected_origin: UVec2, expected_dimensions: UVec2) {
        let grid = GridLayout::new(10, 10);
        let origin = GridPosition::new(3., 3.);

        let act = grid.bounding_box(&origin, radius as f32);

        assert_eq!(act.origin, expected_origin);
        assert_eq!(act.dimensions, expected_dimensions);
    }

    #[test]
    fn neighbors() {
        let grid = GridLayout::new(10, 10);
        let origin = GridPosition::new(3., 3.);
        let neighbors = grid.neighbors(&origin);

        assert_eq!(9, neighbors.len(), "{:?}", neighbors);
    }
}
