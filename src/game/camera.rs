use crate::game::spawn::player::Player;
use crate::postprocessing::PostProcessSettings;
use bevy::core::Name;
use bevy::input::mouse::MouseScrollUnit;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CameraProperties>();
    app.add_systems(Startup, spawn_camera);
    app.add_systems(Update, (camera_zoom, camera_follow));
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
    commands.spawn((
        Name::new("Camera"),
        camera,
        IsDefaultUiCamera,
        CameraProperties {
            initial_camera_zoom: INITIAL_CAMERA_ZOOM,
            camera_zoom_snappiness: 0.2,
            mouse_wheel_sensitivity: 30.,
            camera_zoom_max: 1.5,
            camera_zoom_min: 0.2,
            camera_zoom_buffer: 0.01,
        },
        CanZoomSmoothly(INITIAL_CAMERA_ZOOM),
        PostProcessSettings {
            intensity: 0.00005,
        },
    ));
}

const INITIAL_CAMERA_ZOOM: f32 = 0.3;

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
pub struct CameraProperties {
    initial_camera_zoom: f32,
    camera_zoom_snappiness: f32,
    mouse_wheel_sensitivity: f32,
    camera_zoom_max: f32,
    camera_zoom_min: f32,
    camera_zoom_buffer: f32,
}

fn camera_zoom(
    mut evr_scroll: EventReader<MouseWheel>,
    time: Res<Time>,
    mut query: Query<
        (
            &mut OrthographicProjection,
            &mut CanZoomSmoothly,
            &CameraProperties,
        ),
        With<Camera>,
    >,
) {
    if let Ok((mut projection, mut zoom_destination, camera_properties)) = query.get_single_mut() {
        // handle scroll wheel input
        for ev in evr_scroll.read() {
            let dist = camera_properties.mouse_wheel_sensitivity * time.delta().as_secs_f32();
            let mut log_scale = zoom_destination.0.ln();
            match ev.unit {
                MouseScrollUnit::Line => {
                    log_scale -= ev.y * dist;
                }
                MouseScrollUnit::Pixel => {
                    log_scale -= ev.y * dist;
                }
            }
            if log_scale.exp() > camera_properties.camera_zoom_max {
                zoom_destination.0 = camera_properties.camera_zoom_max;
            } else if log_scale.exp() < camera_properties.camera_zoom_min {
                zoom_destination.0 = camera_properties.camera_zoom_min;
            } else {
                zoom_destination.0 = log_scale.exp();
            }
        }

        // handle smooth zoom over time
        if projection.scale < zoom_destination.0 - camera_properties.camera_zoom_buffer
            || projection.scale > zoom_destination.0 + camera_properties.camera_zoom_buffer
        {
            projection.scale += (zoom_destination.0 - projection.scale)
                * (camera_properties.camera_zoom_snappiness);
        }
        projection.scale = projection.scale.clamp(
            camera_properties.camera_zoom_min,
            camera_properties.camera_zoom_max,
        );
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
