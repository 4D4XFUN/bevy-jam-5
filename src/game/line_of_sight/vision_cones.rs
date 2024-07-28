use std::collections::{HashMap, HashSet};

use bevy::app::App;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

use crate::AppSet;
use crate::game::grid::grid_layout::GridLayout;
use crate::game::grid::GridPosition;
use crate::game::line_of_sight::vision::VisibleSquares;
use crate::game::line_of_sight::vision_cones::handles::VisionConeRenderingHandles;

pub fn plugin(app: &mut App) {
    // plugins
    app.add_plugins(handles::plugin);

    // systems
    app.add_systems(Startup, setup);
    app.add_systems(Update, draw_line_of_sight.in_set(AppSet::UpdateFog));

    // reflection
    app.register_type::<IlluminatedSquares>();
}

#[derive(Component, Clone, Copy, Default)]
pub struct RenderedFieldOfView;

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct IlluminatedSquares {
    pub squares: HashMap<IVec2, Entity>,
}

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("IlluminatedSquares"),
        IlluminatedSquares::default(),
        SpatialBundle {
            transform: Transform::from_translation(Vec3::default().with_z(7.)),
            ..default()
        },
    ));
}

pub fn draw_line_of_sight(
    mut commands: Commands,
    visible_query: Query<&VisibleSquares, With<RenderedFieldOfView>>,
    mut illuminated_query: Query<(Entity, &mut IlluminatedSquares)>,
    grid: Res<GridLayout>,
    handles: Res<VisionConeRenderingHandles>,
) {
    let Ok((overlay_entity, mut overlay)) = illuminated_query.get_single_mut() else {
        return;
    };

    let mut visible_squares = HashSet::<IVec2>::new();
    for h in visible_query.iter() {
        for xy in h.visible_squares.iter() {
            visible_squares.insert(*xy);
        }
    }

    let illuminated_positions = HashSet::from_iter(overlay.squares.keys().copied());

    let squares_to_remove = illuminated_positions.difference(&visible_squares);
    for s in squares_to_remove.into_iter() {
        // info!("Square {:?} is no longer visible", &s);
        let Some(e) = overlay.squares.get(s) else {
            continue;
        };
        commands.entity(*e).despawn();
        overlay.squares.remove(s);
    }

    let squares_to_add = visible_squares.difference(&illuminated_positions);
    for s in squares_to_add.into_iter() {
        // info!("Square {:?} is visible!", &s);
        let position = GridPosition::from_ivec(s);
        let transform = grid.grid_to_world(&position);
        let child_id = commands
            .spawn((MaterialMesh2dBundle {
                mesh: handles.mesh.clone().clone().into(),
                material: handles.material.clone(),
                transform: Transform::from_translation(transform.extend(0.0)),
                ..default()
            },))
            .id();
        overlay.squares.insert(*s, child_id);
        commands.entity(overlay_entity).add_child(child_id);
    }
}

/// Cache handles to a mesh/material that is instanced and used to render all cones of vision to save on performance vs. making new sprites for each one.
mod handles {
    use bevy::prelude::*;

    use crate::game::spawn::level::GRID_SIZE;

    pub fn plugin(app: &mut App) {
        app.init_resource::<VisionConeRenderingHandles>();

        app.register_type::<VisionConeRenderingHandles>();
    }

    #[derive(Resource, Reflect)]
    #[reflect(Resource)]
    pub struct VisionConeRenderingHandles {
        pub mesh: Handle<Mesh>,
        pub material: Handle<ColorMaterial>,
    }

    impl FromWorld for VisionConeRenderingHandles {
        // called when init_resource is used on the app
        fn from_world(world: &mut World) -> Self {
            // meshes
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            let grid_size = GRID_SIZE as f32;
            let mesh_handle = meshes.add(Rectangle::new(grid_size, grid_size));

            // materials
            let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
            let material = materials.add(ColorMaterial::from(Color::srgba(1.0, 1.0, 0.0, 0.015)));

            VisionConeRenderingHandles {
                mesh: mesh_handle,
                material,
            }
        }
    }
}
