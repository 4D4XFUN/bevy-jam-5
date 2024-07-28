use bevy::{prelude::*, render::primitives::Aabb};

pub fn intersect(lhs: (&Transform, &Aabb), rhs: (&Transform, &Aabb)) -> bool {
    let lhs_min = Vec3::from(lhs.1.center) - Vec3::from(lhs.1.half_extents) + lhs.0.translation;
    let lhs_max = Vec3::from(lhs.1.center) + Vec3::from(lhs.1.half_extents) + lhs.0.translation;

    let rhs_min = Vec3::from(rhs.1.center) - Vec3::from(rhs.1.half_extents) + rhs.0.translation;
    let rhs_max = Vec3::from(rhs.1.center) + Vec3::from(rhs.1.half_extents) + rhs.0.translation;

    let x_min = lhs_min.x >= rhs_min.x && lhs_min.x <= rhs_max.x;
    let x_max = lhs_max.x >= rhs_min.x && lhs_max.x <= rhs_max.x;

    let y_min = lhs_min.y >= rhs_min.y && lhs_min.y <= rhs_max.y;
    let y_max = lhs_max.y >= rhs_min.y && lhs_max.y <= rhs_max.y;

    if (x_min || x_max) && (y_min || y_max) {
        return true;
    }
    false
}
