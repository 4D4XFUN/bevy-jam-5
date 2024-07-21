use bevy::core::Name;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera);
    app.add_systems(Update, camera_follow);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct CameraTarget;

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 1.8;
    camera.transform.translation.x += 1280.0 / 2.2;
    camera.transform.translation.y += 720.0 / 1.3;
    commands.spawn((Name::new("Camera"), camera, IsDefaultUiCamera));
}

fn camera_follow(
    time: Res<Time>,
    cameras: Query<(&mut Camera2d)>,
    targets: Query<(&CameraTarget, &Transform)>
) {
    
}