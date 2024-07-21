use bevy::core::Name;
use bevy::prelude::*;
use crate::game::spawn::player::Player;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera);
    //app.add_systems(Update, camera_follow);
    app.add_systems(Update, camera_follows_player); // rudimentary player-following camera
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct CameraTarget;

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 0.5;
    commands.spawn((Name::new("Camera"), camera, IsDefaultUiCamera));
}

fn camera_follows_player(
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Changed<Transform>, Without<Camera>)>,
) {
    if let Ok(player_transform) = player.get_single() {
        if let Ok(mut camera_transform) = camera.get_single_mut() {
            camera_transform.translation = player_transform.translation;
        }
    }
}

fn camera_follow(
    time: Res<Time>,
    cameras: Query<(&mut Camera2d)>,
    targets: Query<(&CameraTarget, &Transform)>
) {
    
}