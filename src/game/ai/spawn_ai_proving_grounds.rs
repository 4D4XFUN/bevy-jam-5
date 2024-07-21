use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
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
                            mut meshes: ResMut<Assets<Mesh>>,
                            mut materials: ResMut<Assets<ColorMaterial>>,
) {

    let scale = 4.0;

    // spawn player
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

    // spawn robot
    let mut t = Transform::from_scale(Vec2::splat(scale).extend(1.0));
    t.translation = Vec2::splat(100.).extend(1.);
    t.rotate_z(-1.0 * std::f32::consts::FRAC_PI_2);
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

    // Spawn detection circle
    let mut t = Transform::from_xyz(0.0, 0.0, 0.0);
    t.rotate_z(std::f32::consts::PI);
    let shape = Mesh2dHandle(meshes.add(CircularSector::new(50.0, 1.0)));
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: shape,
            material: materials.add(ColorMaterial::from(Color::rgba(1.0, 0.0, 0.0, 0.2))),
            transform: t,
            ..default()
        },
    )).set_parent(robocrab);
}