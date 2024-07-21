use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

use crate::game::ai::fov::FovMaterial;
use crate::game::assets::{ImageAsset, ImageAssets};
use crate::game::movement::{Movement, MovementController, WrapWithinWindow};
use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_ai_proving_grounds);
}

#[derive(Event, Debug)]
pub struct SpawnAiProvingGrounds;

fn spawn_ai_proving_grounds(_trigger: Trigger<SpawnAiProvingGrounds>,
                            mut commands: Commands,
                            images: Res<ImageAssets>,
                            mut materials: ResMut<Assets<FovMaterial>>,
                            mut meshes: ResMut<Assets<Mesh>>,
) {
    let scale = 4.0;
    commands.spawn((
        Name::new("Player"),
        SpriteBundle {
            texture: images[&ImageAsset::Crab].clone_weak(),
            transform: Transform::from_scale(Vec2::splat(scale).extend(1.0)),
            ..Default::default()
        },
        MovementController::default(),
        Movement { speed: 420.0 },
        WrapWithinWindow,
        StateScoped(Screen::AiProvingGrounds),
    ));

    let mut t = Transform::from_scale(Vec2::splat(scale).extend(1.0));
    t.translation = Vec2::splat(100.).extend(1.);
    let robocrab = commands.spawn((
        Name::new("Robot"),
        SpriteBundle {
            texture: images[&ImageAsset::RoboCrab].clone_weak(),
            transform: t,
            ..Default::default()
        },
        crate::game::ai::fov::Fov {
            angle: std::f32::consts::PI / 2.0, // 90 degrees
            radius: 200.0,
        },
        StateScoped(Screen::AiProvingGrounds),
    )).id();

    // FOV arc
    let fov_mesh: Handle<Mesh> = meshes.add(Circle::new(1.0).into());
    let fov_material = materials.add(FovMaterial {
        color: Color::rgba(1.0, 1.0, 0.0, 0.5), // Semi-transparent yellow
        arc_params: Vec4::new(0.0, std::f32::consts::PI / 2.0, 0.0, 1.0),
    });
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: fov_mesh.into(),
            material: fov_material,
            transform: Transform::from_xyz(0.0, 0.0, -1.0).with_scale(Vec3::splat(200.0)),
            ..default()
        },
    )).set_parent(robocrab);
}

