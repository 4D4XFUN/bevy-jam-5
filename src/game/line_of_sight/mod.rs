pub mod fog_of_war;

use crate::game::grid::grid_layout::GridLayout;
use crate::game::grid::GridPosition;
use crate::geometry_2d::line_segment::LineSegment;
// use crate::AppSet;
use crate::dev_tools::line_of_sight_debug;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::sprite::Mesh2dHandle;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((front_facing_edges::plugin, fog_of_war::plugin));

    // Temporarily disabled since mesh gen wasn't working
    // app.add_systems(
    //     Update,
    //     (
    //         calculate_vision_extent_by_sweeping_in_a_circle,
    //         update_line_of_sight_mesh,
    //     )
    //         .chain()
    //         .in_set(AppSet::Update),
    // );

    // Reflection registrations
    app.register_type::<LineOfSightSource>();
}

#[derive(Bundle)]
pub struct LineOfSightBundle {
    pub line_of_sight_source: LineOfSightSource,
    pub facing_walls_cache: FacingWallsCache,
    pub calculated_line_of_sight: CalculatedLineOfSight,
    pub los_mesh_handle: LineOfSightMeshHandle,
}

impl Default for LineOfSightBundle {
    fn default() -> Self {
        Self {
            line_of_sight_source: LineOfSightSource {
                max_distance_in_grid_units: 7.,
                max_rays_to_cast: 60,
            },
            facing_walls_cache: FacingWallsCache::new(),
            calculated_line_of_sight: CalculatedLineOfSight::default(),
            los_mesh_handle: LineOfSightMeshHandle::new(),
        }
    }
}

#[derive(Component, Debug, Reflect, Copy, Clone)]
#[reflect(Component)]
pub struct LineOfSightSource {
    pub max_distance_in_grid_units: f32,
    pub max_rays_to_cast: usize,
}

#[derive(Component, Debug, Clone)]
pub struct FacingWallsCache {
    // previous grid position, used to prevent recomputing LOS if tile hasn't changed
    pub last_grid_position: Vec2,
    pub facing_wall_edges: Vec<LineSegment>,
}

impl FacingWallsCache {
    pub fn new() -> Self {
        Self {
            last_grid_position: Vec2::splat(-69420.),
            facing_wall_edges: vec![],
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct CalculatedLineOfSight {
    // rays we've cast from player
    pub rays: Vec<LineSegment>,

    // where all the rays originate from
    pub origin: Vec2,
}

#[allow(dead_code)]
pub fn calculate_vision_extent_by_sweeping_in_a_circle(
    mut query: Query<(
        &GridPosition,
        &LineOfSightSource,
        &FacingWallsCache,
        &mut CalculatedLineOfSight,
    )>,
    grid: Res<GridLayout>,
) {
    for (grid_pos, los_source, facing_walls, mut calculated_points) in query.iter_mut() {
        let steps = los_source.max_rays_to_cast; // how many steps to take around the circle
        let total_angle = std::f32::consts::PI * 2.; // the total angle to sweep
        let step_angle = total_angle / steps as f32;
        let max_range = los_source.max_distance_in_grid_units * grid.square_size;

        // let mut points = vec![];
        let mut rays = vec![];

        let ray_start = grid.grid_to_world(grid_pos);
        calculated_points.origin = ray_start;
        for i in 0..steps {
            let theta = step_angle * i as f32;

            // construct a segment at the given an
            let ray_end = Vec2::new(
                theta.cos() * max_range + ray_start.x,
                theta.sin() * max_range + ray_start.y,
            );

            let mut ray = LineSegment::new(ray_start, ray_end);

            for wall_segment in facing_walls.facing_wall_edges.iter() {
                if ray.do_intersect(wall_segment) {
                    let intersection_point = ray.intersection_point(wall_segment).unwrap();
                    ray = LineSegment::new(ray_start, intersection_point);
                    break;
                }
            }

            rays.push(ray);
        }

        calculated_points.rays = rays;
    }
}

#[allow(dead_code)]
#[derive(Component)]
pub struct LineOfSightMeshHandle {
    pub mesh_handle: Entity,
    pub refresh_timer: Timer,

    pub triangle_vertices: Vec<[f32; 3]>,
    pub triangle_indices: Vec<u32>,
}

impl LineOfSightMeshHandle {
    pub fn new() -> LineOfSightMeshHandle {
        Self {
            mesh_handle: Entity::PLACEHOLDER,
            refresh_timer: Timer::new(Duration::from_millis(50), TimerMode::Repeating),
            triangle_vertices: vec![],
            triangle_indices: vec![],
        }
    }
}

#[allow(dead_code)]
pub fn update_line_of_sight_mesh(
    mut commands: Commands,
    mut query: Query<(&mut CalculatedLineOfSight, &mut LineOfSightMeshHandle)>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    // raycasting is done and we have a list of points where rays collided with walls, in angle order.
    // now we construct a mesh from those points, by drawing triangles between the center point and each consecutive ray's intersection with a wall
    for (calculated_points, mut los_mesh_handle) in query.iter_mut() {
        los_mesh_handle.refresh_timer.tick(time.delta());
        if !los_mesh_handle.refresh_timer.just_finished() {
            continue;
        }

        let z: f32 = 500.; // todo we need a z-layers const somewhere
        let origin = calculated_points.origin;
        let mut vertices: Vec<[f32; 3]> = vec![[origin.x, origin.y, z]];
        for ray in calculated_points.rays.iter() {
            let pt = ray.end();
            vertices.push([pt.x, pt.y, 0.0]);
        }

        // build our triangles
        let mut triangle_indices: Vec<u32> = vec![0, (calculated_points.rays.len() - 1) as u32, 1]; // this is the last tri that closes the circle
        for i in 1..calculated_points.rays.len() - 1 {
            triangle_indices.push(0);
            triangle_indices.push(i as u32);
            triangle_indices.push(i as u32 + 1);
        }

        // todo remove this it's for debugging
        los_mesh_handle.triangle_vertices.clone_from(&vertices);
        los_mesh_handle
            .triangle_indices
            .clone_from(&triangle_indices);
        // println!("{:?}", &vertices);
        // println!("{:?}", &triangle_indices);

        // color on mesh
        let mut v_color: Vec<[f32; 4]> = vec![];
        v_color.resize(vertices.len(), [0., 0., 1., 1.]); // blue?

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_indices(Indices::U32(triangle_indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

        let mesh_id = commands
            .spawn((
                Name::new("LineOfSightMesh"),
                // ColorMesh2dBundle {
                //     mesh: Mesh2dHandle(meshes.add(mesh)),
                //     material: materials.add(ColorMaterial::from(Color::srgba(1., 1., 0., 0.5))),
                //     transform: Transform::from_xyz(origin.x, origin.y, z),
                //     ..default()
                // },

                // The `Handle<Mesh>` needs to be wrapped in a `Mesh2dHandle` to use 2d rendering instead of 3d
                Mesh2dHandle(meshes.add(mesh)),
                // This bundle's components are needed for something to be rendered
                SpatialBundle::INHERITED_IDENTITY,
            ))
            .id();

        let old_id = los_mesh_handle.mesh_handle;
        los_mesh_handle.mesh_handle = mesh_id;

        // despawn the old one
        if old_id != Entity::PLACEHOLDER {
            commands.entity(old_id).despawn();
        }
    }
}

/// Finds front facing edges of walls (from player's perspective)
pub mod front_facing_edges {
    use bevy::prelude::*;

    use crate::game::grid::grid_layout::GridLayout;
    use crate::game::grid::GridPosition;
    use crate::game::line_of_sight::{FacingWallsCache, LineOfSightSource};
    use crate::game::spawn::level::LevelWalls;
    use crate::geometry_2d::line_segment::LineSegment;
    use crate::AppSet;

    pub fn plugin(app: &mut App) {
        // Systems
        app.add_systems(
            Update,
            update_front_facing_edges_when_grid_pos_changes.in_set(AppSet::Update),
        );
    }

    /// Whenever the player moves a whole tile, we have to recompute which parts of walls are facing them
    pub fn update_front_facing_edges_when_grid_pos_changes(
        mut query: Query<
            (&GridPosition, &mut LineOfSightSource, &mut FacingWallsCache),
            Changed<GridPosition>,
        >,
        walls: Res<LevelWalls>,
        grid: Res<GridLayout>,
    ) {
        for (&player_position, player_los, mut facing_walls_cache) in query.iter_mut() {
            // skip if we haven't actually changed tiles
            if player_position.coordinates == facing_walls_cache.last_grid_position {
                continue;
            }
            facing_walls_cache.last_grid_position = player_position.coordinates;

            // compute nearest edges for every wall
            let mut edges: Vec<LineSegment> = vec![];
            let pc = player_position.coordinates;
            for wall in walls.wall_locations.iter() {
                let wall_pos = GridPosition::new(wall.x as f32, wall.y as f32)
                    .with_offset(Vec2::new(-0.5, -0.5));

                // skip walls that are further than our LOS distance
                if (player_position - wall_pos).coordinates.length()
                    > player_los.max_distance_in_grid_units
                {
                    continue;
                }

                // add all near-facing edges of walls to a list
                let sides = grid.sides(&wall_pos);
                if pc.x > wall_pos.coordinates.x && !walls.collides(wall.x + 1, wall.y) {
                    edges.push(sides.east)
                } else if pc.x < wall_pos.coordinates.x && !walls.collides(wall.x - 1, wall.y) {
                    edges.push(sides.west)
                }
                if pc.y > wall_pos.coordinates.y && !walls.collides(wall.x, wall.y + 1) {
                    edges.push(sides.north)
                } else if pc.y < wall_pos.coordinates.y && !walls.collides(wall.x, wall.y - 1) {
                    edges.push(sides.south)
                }
            }

            // sort edges based on distance from player
            let player_world_pos = grid.grid_to_world(&player_position);
            edges.sort_by(|a, b| {
                let dist_to_a = (player_world_pos - a.center).length();
                let dist_to_b = (player_world_pos - b.center).length();
                dist_to_a.total_cmp(&dist_to_b)
            });

            facing_walls_cache.facing_wall_edges = edges;
        }
    }
}
