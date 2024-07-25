use crate::game::line_of_sight::{
    CalculatedLineOfSight, FacingWallsCache, LineOfSightMeshHandle,
};
use crate::AppSet;
use bevy::prelude::*;
use crate::dev_tools::DebugOverlaysState;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            redraw_front_facing_edges,
            // draw_rays,
            draw_debug_triangles,
        )
            .in_set(AppSet::UpdateWorld)
            .run_if(in_state(DebugOverlaysState::Enabled)),
    );
}

/// Every update, if the grid coords changed, redraw the overlay of edges facing the player
pub fn redraw_front_facing_edges(
    mut gizmos: Gizmos,
    front_facing_edges_query: Query<&FacingWallsCache>,
) {
    let near_color = Color::srgb(1., 1., 0.);
    let far_color = Color::srgb(0., 1., 1.);
    for wall_cache in front_facing_edges_query.iter() {
        let steps = wall_cache.facing_wall_edges.len() as f32;

        for (i, edge) in wall_cache.facing_wall_edges.iter().enumerate() {
            let c = near_color.mix(&far_color, i as f32 / steps);
            let a = edge.segment2d.point1() + edge.center;
            let b = edge.segment2d.point2() + edge.center;
            gizmos.line_2d(a, b, c);
        }
    }
}

pub fn _draw_rays(mut gizmos: Gizmos, query: Query<&CalculatedLineOfSight>) {
    let color = Color::srgb(0., 1., 0.);
    for ray_cache in query.iter() {
        for ray in ray_cache.rays.iter() {
            gizmos.line_2d(ray.start(), ray.end(), color);
        }
    }
}

pub fn draw_debug_triangles(mut gizmos: Gizmos, query: Query<&LineOfSightMeshHandle>) {
    for mesh in query.iter() {
        let near_color = Color::srgb(1., 1., 0.);
        let far_color = Color::srgb(0., 1., 1.);
        for (i, tri) in mesh.triangle_indices.windows(3).enumerate() {
            let (a, b, c) = (
                mesh.triangle_vertices[tri[0] as usize],
                mesh.triangle_vertices[tri[1] as usize],
                mesh.triangle_vertices[tri[2] as usize],
            );

            let prim = Triangle2d {
                vertices: [
                    Vec2::new(a[0], a[1]),
                    Vec2::new(b[0], b[1]),
                    Vec2::new(c[0], c[1]),
                ],
            };

            let color = if i % 2 == 0 { near_color } else { far_color };
            gizmos.primitive_2d(&prim, Vec2::ZERO, 0., color);
        }
    }
}
