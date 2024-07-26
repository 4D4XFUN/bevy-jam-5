use crate::game::grid::grid_layout::GridLayout;
use crate::game::grid::GridPosition;
use crate::game::line_of_sight::vision::VisibleSquares;
use crate::game::line_of_sight::CanRevealFog;
use crate::AppSet;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::render::texture::{ImageSampler, ImageSamplerDescriptor};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle};
use std::collections::HashSet;

pub(super) fn plugin(app: &mut App) {
    //systems
    app.add_plugins(Material2dPlugin::<FogOfWarMaterial>::default());
    app.add_systems(
        Update,
        (
            setup_fog_of_war,
            recover_fog_of_war,
            reveal_fog_of_war,
            copy_data_to_texture,
        )
            .chain()
            .in_set(AppSet::UpdateFog),
    );
}

#[derive(Component)]
struct FogOfWar {
    width: u32,
    height: u32,
    data: Vec<f32>,
}

impl FogOfWar {
    pub fn index(&self, x: u32, y: u32) -> u32 {
        let row = self.height - y - 1;

        self.width * row + x
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct FogOfWarMaterial {
    #[uniform(0)]
    color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    fog_texture: Handle<Image>,
}

impl Material2d for FogOfWarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fog_of_war.wgsl".into()
    }
}

fn setup_fog_of_war(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FogOfWarMaterial>>,
    mut images: ResMut<Assets<Image>>,
    old_fog: Query<(Entity, &FogOfWar)>,
    grid: Res<GridLayout>,
) {
    // only rerun if the grid has changed
    if !grid.is_changed() {
        return;
    }

    // despawn old fogs of war
    for (e, fow) in old_fog.iter() {
        info!("Despawning old {} x {} fog", fow.width, fow.height);
        commands.entity(e).despawn_recursive();
    }

    let width = grid.width as u32;
    let height = grid.height as u32;

    if width == 0 || height == 0 {
        info!(
            "Tried to make grid with dimensions {} x {}, skipping because it's 0 in a dimension.",
            width, height
        );
        return;
    }

    // Create a single quad mesh for the entire grid
    let mesh = Rectangle::default();

    // Create a texture for fog of war data
    let num_grid_squares = width * height;

    let pixels: Vec<u8> = (0..num_grid_squares)
        .map(|i| {
            let step = 255. / num_grid_squares as f32;
            let value = step * i as f32;
            value.floor() as u8
        })
        .collect();

    let mut fog_texture = Image::new_fill(
        bevy::render::render_resource::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        // &vec![255; num_grid_squares as usize],
        &pixels[..],
        bevy::render::render_resource::TextureFormat::R8Unorm,
        RenderAssetUsages::all(),
    );
    fog_texture.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());

    let fog_texture_handle = images.add(fog_texture);

    // Create the material
    let material = materials.add(FogOfWarMaterial {
        color: LinearRgba::BLACK,
        fog_texture: fog_texture_handle.clone(),
    });

    let mesh_transform_grid_center = grid.center_worldpos();

    // Spawn the fog of war entity
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material,
            transform: Transform::from_scale(Vec3::new(
                grid.width as f32 * grid.square_size,
                grid.height as f32 * grid.square_size,
                1.0,
            ))
                .with_translation(mesh_transform_grid_center.extend(10.)),
            ..default()
        },
        FogOfWar {
            width,
            height,
            data: vec![1.0; num_grid_squares as usize],
        },
    ));

    info!(
        "Initialized fog of war with {} grid positions",
        num_grid_squares
    );
}

fn copy_data_to_texture(
    mut fog_query: Query<(&mut FogOfWar, &Handle<FogOfWarMaterial>)>,
    mut images: ResMut<Assets<Image>>,
    mut fog_materials: ResMut<Assets<FogOfWarMaterial>>,
) {
    for (fog, material_handle) in fog_query.iter_mut() {
        if let Some(material) = fog_materials.get_mut(material_handle) {
            if let Some(texture) = images.get_mut(&material.fog_texture) {
                for (i, value) in fog.data.iter().enumerate() {
                    texture.data[i] = (*value * 255.0) as u8;
                }
            }
        }
    }
}

fn reveal_fog_of_war(
    grid: Res<GridLayout>,
    line_of_sight_query: Query<&VisibleSquares, With<CanRevealFog>>,
    mut fog_of_war_query: Query<&mut FogOfWar>,
) {
    let Ok(mut fog) = fog_of_war_query.get_single_mut() else {
        return;
    };

    for component in line_of_sight_query.iter() {
        let without_neighbors = &component.visible_squares;
        let mut with_neighbors = HashSet::<IVec2>::new();
        for coordinate in without_neighbors.iter() {
            for x in grid
                .neighbors(&GridPosition::new(coordinate.x as f32, coordinate.y as f32))
                .into_iter()
                .map(|v| IVec2::new(v.x as i32, v.y as i32))
            {
                with_neighbors.insert(x);
            }
        }

        for square in with_neighbors.iter() {
            let index = fog.index(square.x as u32, square.y as u32);
            fog.data[index as usize] = (if without_neighbors.contains(square) {
                0.0f32
            } else {
                0.2f32
            })
                .min(fog.data[index as usize]);
        }
    }
}

fn recover_fog_of_war(mut fog_of_war_query: Query<&mut FogOfWar>) {
    const RECOVERY_SPEED: f32 = 0.1;
    for mut s in fog_of_war_query.iter_mut() {
        let data = &mut s.data;
        for d in data.iter_mut() {
            *d = (*d + RECOVERY_SPEED).min(1.0);
        }
    }
}
