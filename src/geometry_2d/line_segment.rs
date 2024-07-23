use std::cmp::Ordering;
use bevy::prelude::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct LineSegment {
    pub segment2d: Segment2d,
    pub center: Vec2,
}

impl LineSegment {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        let seg = Segment2d::from_points(a, b);
        Self { segment2d: seg.0, center: seg.1 }
    }

    pub fn start(&self) -> Vec2 {
        self.segment2d.point1() + self.center
    }

    pub fn end(&self) -> Vec2 {
        self.segment2d.point2() + self.center
    }

    pub fn do_intersect(&self, other: &LineSegment) -> bool { do_intersect(self, other) }

    pub fn intersection_point(&self, other: &LineSegment) -> Option<Vec2> { intersection_point(self, other) }
}

pub fn do_intersect(line1: &LineSegment, line2: &LineSegment) -> bool {
    let p1 = line1.start();
    let q1 = line1.end();
    let p2 = line2.start();
    let q2 = line2.end();

    let o1 = orientation(p1, q1, p2);
    let o2 = orientation(p1, q1, q2);
    let o3 = orientation(p2, q2, p1);
    let o4 = orientation(p2, q2, q1);

    if o1 != o2 && o3 != o4 {
        return true;
    }

    if o1 == Ordering::Equal && on_segment(p1, p2, q1) { return true; }
    if o2 == Ordering::Equal && on_segment(p1, q2, q1) { return true; }
    if o3 == Ordering::Equal && on_segment(p2, p1, q2) { return true; }
    if o4 == Ordering::Equal && on_segment(p2, q1, q2) { return true; }

    false
}

pub fn intersection_point(line1: &LineSegment, line2: &LineSegment) -> Option<Vec2> {
    let p1 = line1.start();
    let q1 = line1.end();
    let p2 = line2.start();
    let q2 = line2.end();

    let o1 = orientation(p1, q1, p2);
    let o2 = orientation(p1, q1, q2);
    let o3 = orientation(p2, q2, p1);
    let o4 = orientation(p2, q2, q1);

    if o1 != o2 && o3 != o4 {
        // Lines intersect, calculate intersection point
        let x1 = p1.x;
        let y1 = p1.y;
        let x2 = q1.x;
        let y2 = q1.y;
        let x3 = p2.x;
        let y3 = p2.y;
        let x4 = q2.x;
        let y4 = q2.y;

        let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        if denom.abs() < f32::EPSILON {
            return None; // Lines are parallel
        }

        let x = ((x1 * y2 - y1 * x2) * (x3 - x4) - (x1 - x2) * (x3 * y4 - y3 * x4)) / denom;
        let y = ((x1 * y2 - y1 * x2) * (y3 - y4) - (y1 - y2) * (x3 * y4 - y3 * x4)) / denom;

        return Some(Vec2::new(x, y));
    }

    // Check for special cases where endpoints lie on the other segment
    if o1 == Ordering::Equal && on_segment(p1, p2, q1) { return Some(p2); }
    if o2 == Ordering::Equal && on_segment(p1, q2, q1) { return Some(q2); }
    if o3 == Ordering::Equal && on_segment(p2, p1, q2) { return Some(p1); }
    if o4 == Ordering::Equal && on_segment(p2, q1, q2) { return Some(q1); }

    None
}

fn orientation(p: Vec2, q: Vec2, r: Vec2) -> Ordering {
    let val = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);
    if val.abs() < f32::EPSILON {
        Ordering::Equal
    } else if val > 0.0 {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

fn on_segment(p: Vec2, q: Vec2, r: Vec2) -> bool {
    q.x <= p.x.max(r.x) && q.x >= p.x.min(r.x) &&
        q.y <= p.y.max(r.y) && q.y >= p.y.min(r.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(
        Vec2::new(1.0, 1.0), Vec2::new(5.0, 5.0),
        Vec2::new(1.0, 5.0), Vec2::new(5.0, 1.0),
        Some(Vec2::new(3.0, 3.0));
        "Intersecting lines"
    )]
    #[test_case(
        Vec2::new(1.0, 1.0), Vec2::new(5.0, 5.0),
        Vec2::new(6.0, 6.0), Vec2::new(10.0, 10.0),
        None;
        "Non-intersecting lines"
    )]
    #[test_case(
        Vec2::new(1.0, 1.0), Vec2::new(5.0, 5.0),
        Vec2::new(3.0, 3.0), Vec2::new(7.0, 7.0),
        Some(Vec2::new(3.0, 3.0));
        "Overlapping lines"
    )]
    #[test_case(
        Vec2::new(1.0, 1.0), Vec2::new(5.0, 1.0),
        Vec2::new(3.0, 0.0), Vec2::new(3.0, 5.0),
        Some(Vec2::new(3.0, 1.0));
        "Perpendicular lines"
    )]
    #[test_case(
        Vec2::new(1.0, 1.0), Vec2::new(5.0, 5.0),
        Vec2::new(1.0, 1.0), Vec2::new(3.0, 3.0),
        Some(Vec2::new(3.0, 3.0));
        "Shared endpoint"
    )]
    #[test_case(
        Vec2::new(1.0, 1.0), Vec2::new(2.0, 2.0),
        Vec2::new(2.0, 2.0), Vec2::new(3.0, 3.0),
        Some(Vec2::new(2.0, 2.0));
        "Touching Endpoints"
    )]
    #[test_case(
        Vec2::new(1.0, 1.0), Vec2::new(5.0, 5.0),
        Vec2::new(2.0, 2.0), Vec2::new(4.0, 4.0),
        Some(Vec2::new(2.0, 2.0));
        "One line segment fully contained within the other"
    )]
    #[test_case(
        Vec2::new(1.0, 1.0), Vec2::new(5.0, 1.0),
        Vec2::new(2.0, 1.0), Vec2::new(4.0, 1.0),
        Some(Vec2::new(2.0, 1.0));
        "Collinear overlapping segments"
    )]
    #[test_case(
        Vec2::new(1.0, 1.0), Vec2::new(5.0, 1.0),
        Vec2::new(6.0, 1.0), Vec2::new(10.0, 1.0),
        None;
        "Collinear non-overlapping segments"
    )]
    fn test_line_intersection(
        start1: Vec2, end1: Vec2,
        start2: Vec2, end2: Vec2,
        expected: Option<Vec2>,
    ) {
        let line1 = LineSegment::new(start1, end1);
        let line2 = LineSegment::new(start2, end2);
        let result = line1.intersection_point(&line2);

        match (result, expected) {
            (Some(point1), Some(point2)) => {
                assert!((point1 - point2).length() < f32::EPSILON,
                        "Expected {:?}, but got {:?}", point2, point1);
            }
            (None, None) => {}
            _ => panic!("Expected {:?}, but got {:?}", expected, result),
        }
    }
}