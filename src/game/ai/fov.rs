use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, MaterialMesh2dBundle};

pub fn plugin(app: &mut App) {
    app
        .add_plugins(MaterialPlugin::<FovMaterial>::default())
        .add_systems(Update, update_fov);
}

// Custom shader material for the FOV arc
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FovMaterial {
    #[uniform(0)]
    pub(crate) color: Color,
    #[uniform(0)]
    pub(crate) arc_params: Vec4, // x: start_angle, y: end_angle, z: inner_radius, w: outer_radius
}

impl Material2d for FovMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fov_material.wgsl".into()
    }
}

// Component to hold FOV parameters
#[derive(Component)]
pub struct Fov {
    pub(crate) angle: f32,
    pub(crate) radius: f32,
}

pub fn update_fov(
    mut fov_query: Query<(&Fov, &mut Transform, &Children)>,
    mut materials: ResMut<Assets<FovMaterial>>,
    material_handles: Query<&Handle<FovMaterial>>,
) {
    for (fov, mut transform, children) in fov_query.iter_mut() {
        // Update sprite rotation (assuming it rotates)
        transform.rotation = Quat::from_rotation_z(transform.rotation.to_euler(EulerRot::XYZ).2 + 0.01);

        // Update FOV material
        if let Some(child) = children.first() {
            if let Ok(material_handle) = material_handles.get(*child) {
                if let Some(material) = materials.get_mut(material_handle) {
                    let start_angle = -fov.angle / 2.0;
                    let end_angle = fov.angle / 2.0;
                    material.arc_params.x = start_angle;
                    material.arc_params.y = end_angle;
                }
            }
        }
    }
}