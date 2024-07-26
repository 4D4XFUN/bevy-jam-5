use bevy::prelude::*;
use std::collections::HashSet;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::render::texture::{ImageSampler, ImageSamplerDescriptor};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2d};
use bevy::utils::info;
use crate::game::grid::grid_layout::GridLayout;
use crate::game::grid::GridPosition;
use crate::game::line_of_sight::vision::VisibleSquares;
use crate::game::line_of_sight::CanRevealFog;
use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    //systems
    app.add_plugins(Material2dPlugin::<FogOfWarMaterial>::default());
    app.add_systems(
        Update,
        (
            setup_fog_of_war,
            // update_grid_fog_of_war_overlay,
            // recover_fog_of_war,
            // reveal_fog_of_war,
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

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct FogOfWarMaterial {
    #[uniform(0)]
    color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    fog_texture: Option<Handle<Image>>,
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
        info!("Tried to make grid with dimensions {} x {}, skipping because it's 0 in a dimension.", width, height);
        return;
    }

    // Create a single quad mesh for the entire grid
    let mut mesh = Rectangle::default();

    // Create a texture for fog of war data
    let mut fog_texture = Image::new_fill(
        bevy::render::render_resource::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &vec![255; (width * height) as usize],
        bevy::render::render_resource::TextureFormat::R8Unorm,
        RenderAssetUsages::all(),
    );
    fog_texture.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());

    let fog_texture_handle = images.add(fog_texture);

    // Create the material
    let material = materials.add(FogOfWarMaterial {
        color: LinearRgba::BLACK,
        fog_texture: Some(fog_texture_handle.clone()),
    });

    let mesh_transform_grid_center = grid.center_worldpos();

    // Spawn the fog of war entity
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material,
            transform: Transform::from_scale(Vec3::new(grid.width as f32 * grid.square_size, grid.height as f32 * grid.square_size, 1.0))
                .with_translation(mesh_transform_grid_center.extend(10.)),
            ..default()
        },
        FogOfWar {
            width,
            height,
            data: vec![1.0; (width * height) as usize],
        },
    ));
}
