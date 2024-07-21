use bevy::core::Name;
use bevy::prelude::{App, Camera2dBundle, Commands, IsDefaultUiCamera, Startup};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera);
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 1.8;
    camera.transform.translation.x += 1280.0 / 2.2;
    camera.transform.translation.y += 720.0 / 1.3;
    commands.spawn((Name::new("Camera"), camera, IsDefaultUiCamera));
}

