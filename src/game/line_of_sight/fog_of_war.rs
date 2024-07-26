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
    app.init_resource::<FogOfWarGpuHandles>();
    app.add_plugins(Material2dPlugin::<FogOfWarMaterial>::default());
    app.add_systems(
        Update,
        (
            setup_fog_of_war,
            // update_grid_fog_of_war_overlay,
            recover_fog_of_war,
            reveal_fog_of_war,
        )
            .chain()
            .in_set(AppSet::UpdateFog),
    );

    // reflection
    app.register_type::<FogOfWarOverlay>();
}

pub fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {}

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
        info!("Tried to make grid with dimensions {} x {}, skipping because it's 0 in a dimension.", width, height);
        return;
    }

    // Create a single quad mesh for the entire grid
    // todo can we just use Rectangle::default()?
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));

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
        color: Color::linear_rgba(0.,0.,0.5,1.,).to_linear(),
        fog_texture: fog_texture_handle.clone(),
    });

    // Spawn the fog of war entity
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material,
            transform: Transform::from_scale(Vec3::new(grid.width as f32 * grid.square_size, grid.height as f32 * grid.square_size, 1.0))
                .with_translation(Vec3::new(0.0, 0.0, 10.0)),
            ..default()
        },
        FogOfWar {
            width,
            height,
            data: vec![1.0; (width * height) as usize],
        },
    ));
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct FogOfWarOverlay {
    fog_of_war_grid_sprites: Vec<Entity>,
    width: usize,
    height: usize,
    resolution: f32,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct FogOfWarOverlayVoxel;

impl FogOfWarOverlay {
    pub(crate) fn insert_at(&mut self, x: usize, y: usize, e: Entity) {
        self.fog_of_war_grid_sprites[x + y * self.width] = e;
    }

    pub fn get_at(&self, x: usize, y: usize) -> Entity {
        let index = x + y * self.width;
        self.fog_of_war_grid_sprites[index]
    }
}

impl FogOfWarOverlay {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut fog_of_war_grid_sprites = Vec::new();
        fog_of_war_grid_sprites.resize(size, Entity::PLACEHOLDER);
        Self {
            fog_of_war_grid_sprites,
            width,
            height,
            resolution: 1.0,
        }
    }
}

#[derive(Resource)]
struct FogOfWarGpuHandles {
    mesh: Handle<Mesh>,
    mat_transparent: Handle<ColorMaterial>,
    mat_revealed: Handle<ColorMaterial>,
    mat_hidden: Handle<ColorMaterial>,
}

impl FromWorld for FogOfWarGpuHandles {
    // called when init_resource is used on the app
    fn from_world(world: &mut World) -> Self {
        // meshes
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        // hardcoded grid size :(
        let mesh_handle = meshes.add(Rectangle::new(16., 16.));

        // materials
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let mat_hidden = materials.add(ColorMaterial::from(Color::srgba(0.0, 0.0, 0.0, 1.0)));
        let mat_revealed = materials.add(ColorMaterial::from(Color::srgba(0.0, 0.0, 0.0, 0.0)));
        let mat_transparent = materials.add(ColorMaterial::from(Color::srgba(0.0, 0.0, 0.0, 0.5)));

        FogOfWarGpuHandles {
            mesh: mesh_handle,
            mat_hidden,
            mat_revealed,
            mat_transparent,
        }
    }
}
fn update_grid_fog_of_war_overlay(
    mut commands: Commands,
    grid: Res<GridLayout>,
    fog_of_war_gpu_handles: Res<FogOfWarGpuHandles>,
    existing_overlays: Query<Entity, With<FogOfWarOverlay>>,
) {
    if !grid.is_changed() {
        return;
    }

    for e in existing_overlays.iter() {
        commands.entity(e).despawn_recursive();
    }

    let mut overlay = FogOfWarOverlay::new(grid.width, grid.height);

    let mut child_ids = vec![];
    // Spawn child sprites for each grid cell
    for y in 0..grid.height {
        for x in 0..grid.width {
            let position = grid.grid_to_world(&GridPosition::new(x as f32, y as f32));

            let alpha = 1.0;
            let color = Color::srgba(0.0, 0.0, 0.0, alpha);

            // Spawn the child sprite and parent it to the GridSprite
            let child_id = commands
                .spawn((
                    FogOfWarOverlayVoxel,
                    MaterialMesh2dBundle {
                        mesh: fog_of_war_gpu_handles.mesh.clone().clone().into(),
                        material: fog_of_war_gpu_handles.mat_hidden.clone(),
                        transform: Transform::from_translation(position.extend(10.0)),
                        ..default()
                    },
                ))
                .id();

            overlay.insert_at(x, y, child_id);
            child_ids.push(child_id);
        }
    }

    let parent_overlay_entity = commands
        .spawn((
            Name::new("FogOfWarOverlay"),
            overlay,
            SpatialBundle::default(),
        ))
        .id();

    for e in child_ids.iter() {
        commands.entity(*e).set_parent(parent_overlay_entity);
    }
}

fn reveal_fog_of_war(
    grid: Res<GridLayout>,
    line_of_sight_query: Query<&VisibleSquares, With<CanRevealFog>>,
    mut commands: Commands,
    fog_of_war_query: Query<&FogOfWarOverlay>,
    fog_of_war_gpu_handles: Res<FogOfWarGpuHandles>,
    mut fog_of_war_sprite_query: Query<Entity, With<FogOfWarOverlayVoxel>>,
) {
    let Ok(fog) = fog_of_war_query.get_single() else {
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

        // info!("Found {} neighbors of {} squares", with_neighbors.len() - without_neighbors.len(), without_neighbors.len());

        for coordinate in with_neighbors {
            let fog_entity = fog.get_at(coordinate.x as usize, coordinate.y as usize);
            commands.entity(fog_entity).insert(fog_of_war_gpu_handles.mat_revealed.clone());
        }
    }
}

fn recover_fog_of_war(
    mut commands: Commands,
    fog_of_war_query: Query<&FogOfWarOverlay>,
    fog_of_war_gpu_handles: Res<FogOfWarGpuHandles>,
) {
    for mut s in fog_of_war_query.iter() {
        for &x in s.fog_of_war_grid_sprites.iter() {
            commands.entity(x).insert(fog_of_war_gpu_handles.mat_hidden.clone());
        }
    }
}
