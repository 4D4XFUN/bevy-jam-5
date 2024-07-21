use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use crate::game::assets::{ImageAsset, ImageAssets};
use crate::game::movement::{Movement, MovementController, WrapWithinWindow};
use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_ai_proving_grounds)
        .add_systems(Update, update_detection_arc);
}

#[derive(Event, Debug)]
pub struct SpawnAiProvingGrounds;

#[derive(Component)]
pub struct Fov {
    pub(crate) angle: f32,
    pub(crate) radius: f32,
}

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
    t.rotate_z(1.0 * std::f32::consts::FRAC_PI_2);
    let robocrab = commands.spawn((
        Name::new("Robot"),
        SpriteBundle {
            texture: images[&ImageAsset::RoboCrab].clone_weak(),
            transform: t,
            ..Default::default()
        },
        Fov {
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
            material: materials.add(ColorMaterial::from(Color::srgba(1.0, 0.0, 0.0, 0.2))),
            transform: t,
            ..default()
        },
        DetectionArc,
    )).set_parent(robocrab);
}

#[derive(Component)]
struct DetectionArc;

// TODO this seems to have tanked framerate on my machine - might want to rethink how we do detection arcs
fn update_detection_arc(
    mut arc_query: Query<(&Parent, &mut Handle<ColorMaterial>), With<DetectionArc>>,
    player_query: Query<&Transform, (With<MovementController>, Without<DetectionArc>)>,
    robot_query: Query<(&Transform, &Fov)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (arc_parent, mut arc_material) in arc_query.iter_mut() {
        if let Ok((robot_transform, fov)) = robot_query.get(arc_parent.get()) {
            if let Ok(player_transform) = player_query.get_single() {
                let to_player = player_transform.translation - robot_transform.translation;
                let distance = to_player.length();
                let angle = to_player.y.atan2(to_player.x); //- robot_transform.rotation.to_euler(EulerRot::XYZ).2;
                let angle = angle.abs().rem_euclid(std::f32::consts::TAU);

                let is_detected = distance <= fov.radius && angle <= fov.angle / 2.0;

                let color = if is_detected {
                    Color::srgba(0.0, 1.0, 0.0, 0.2)
                } else {
                    Color::srgba(1.0, 0.0, 0.0, 0.2)
                };

                *arc_material = materials.add(ColorMaterial::from(color));
            }
        }
    }
}