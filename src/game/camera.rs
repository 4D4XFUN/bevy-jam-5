use bevy::core::Name;
use bevy::prelude::*;
use crate::game::spawn::player::Player;
use bevy::input::mouse::MouseWheel;
use bevy::input::mouse::MouseScrollUnit;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera);
    app.add_systems(Update, (camera_zoom, camera_follow)); // rudimentary player-following camera
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct CanBeFollowedByCamera;

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
pub struct CanZoomSmoothly(f32);

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = INITIAL_CAMERA_ZOOM;
    commands.spawn((Name::new("Camera"), camera, IsDefaultUiCamera, CanZoomSmoothly(INITIAL_CAMERA_ZOOM)));
}

const INITIAL_CAMERA_ZOOM: f32 = 0.3;
const CAMERA_ZOOM_SNAPPINESS: f32 = 0.025;
const MOUSE_WHEEL_SENSITIVITY: f32 = 0.5;
const CAMERA_ZOOM_MAX: f32 = 1.5;
const CAMERA_ZOOM_MIN: f32 = 0.3;
const CAMERA_ZOOM_BUFFER: f32 = 0.01;

fn camera_zoom(
    mut evr_scroll: EventReader<MouseWheel>,
    time: Res<Time>,
    mut query: Query<(&mut OrthographicProjection, &mut CanZoomSmoothly), With<Camera>>,
) {
    if let Ok ((mut projection, mut zoom_destination)) = query.get_single_mut() {
        // handle scroll wheel input
        for ev in evr_scroll.read() {
            let dist = MOUSE_WHEEL_SENSITIVITY * time.delta().as_secs_f32();
            let mut log_scale = zoom_destination.0.ln();
            match ev.unit {
                MouseScrollUnit::Line => {
                    log_scale -= ev.y * dist;
                }
                MouseScrollUnit::Pixel => {
                    log_scale -= ev.y * dist;
                }
            }
            if log_scale.exp() > CAMERA_ZOOM_MAX {
                zoom_destination.0 = CAMERA_ZOOM_MAX;
            } else if log_scale.exp() < CAMERA_ZOOM_MIN {
                zoom_destination.0 = CAMERA_ZOOM_MIN;
            } else {
                zoom_destination.0 = log_scale.exp();
            }
        }

        // handle smooth zoom over time
        if projection.scale < zoom_destination.0 - CAMERA_ZOOM_BUFFER
            || projection.scale > zoom_destination.0 + CAMERA_ZOOM_BUFFER {
            projection.scale += (zoom_destination.0 - projection.scale) * (CAMERA_ZOOM_SNAPPINESS) ;
        }
        projection.scale = projection.scale.clamp(CAMERA_ZOOM_MIN, CAMERA_ZOOM_MAX);
    }
}

fn camera_follow(
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    target: Query<&Transform, (With<CanBeFollowedByCamera>, Without<Camera>)>,
) {
    if let Ok(target_transform) = target.get_single() {
        if let Ok(mut camera_transform) = camera.get_single_mut() {
            let lerp = (target_transform.translation - camera_transform.translation) * 0.05;
            camera_transform.translation += lerp;
        }
    }
}