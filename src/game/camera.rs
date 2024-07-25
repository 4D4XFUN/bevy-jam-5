use crate::game::spawn::level::{LevelWalls, GRID_SIZE};
use crate::game::spawn::player::Player;
use crate::input::PlayerAction;
use crate::postprocessing::PostProcessSettings;
use bevy::core::Name;
use bevy::input::mouse::MouseScrollUnit;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::prelude::Keyframes::Translation;
use leafwing_input_manager::action_state::ActionState;
use crate::z_layers::ZLayers;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CameraProperties>();
    app.add_systems(Startup, spawn_camera);
    app.add_systems(
        Update,
        (
            record_binary_zoom_input,
            record_smooth_zoom_input,
            apply_camera_zoom,
            camera_follow,
        )
            .chain(),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct CameraFollowTarget;

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
pub struct CanZoomSmoothly(f32);

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle {
        transform: ZLayers::Camera.transform(),
        ..default()
    };
    camera.projection.scale = INITIAL_CAMERA_ZOOM;
    commands.spawn((
        Name::new("Camera"),
        camera,
        IsDefaultUiCamera,
        CameraProperties {
            initial_camera_zoom: INITIAL_CAMERA_ZOOM,
            camera_zoom_snappiness: 20.0,
            zoom_sensitivity: 1.0,
            mouse_wheel_sensitivity_multiplier: 5.0,
            camera_zoom_max: 1.1,
            camera_zoom_min: 0.2,
            camera_zoom_buffer: 0.01,
            camera_follow_snappiness: 7.0,
        },
        CanZoomSmoothly(INITIAL_CAMERA_ZOOM),
        PostProcessSettings { intensity: 0.00005 },
    ));
}

const INITIAL_CAMERA_ZOOM: f32 = 0.3;

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
pub struct CameraProperties {
    initial_camera_zoom: f32,
    camera_zoom_snappiness: f32,
    zoom_sensitivity: f32,
    mouse_wheel_sensitivity_multiplier: f32,
    camera_zoom_max: f32,
    camera_zoom_min: f32,
    camera_zoom_buffer: f32,
    camera_follow_snappiness: f32,
}

fn record_smooth_zoom_input(
    mut evr_scroll: EventReader<MouseWheel>,
    time: Res<Time>,
    mut query: Query<(&mut CanZoomSmoothly, &CameraProperties), With<Camera>>,
) {
    if let Ok((mut zoom_destination, camera_properties)) = query.get_single_mut() {
        // handle scroll wheel input
        for ev in evr_scroll.read() {
            let dist = camera_properties.zoom_sensitivity * time.delta().as_secs_f32();
            let mut log_scale = zoom_destination.0.ln();
            match ev.unit {
                MouseScrollUnit::Line => {
                    log_scale -= ev.y * dist * camera_properties.mouse_wheel_sensitivity_multiplier;
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
    }
}

fn record_binary_zoom_input(
    // input: Res<ButtonInput<KeyCode>>,
    action: Query<&ActionState<PlayerAction>>,
    mut query: Query<(&mut CanZoomSmoothly, &CameraProperties), With<Camera>>,
) {
    if let Ok((mut zoom_destination, camera_properties)) = query.get_single_mut() {
        for act in action.iter() {
            if act.pressed(&PlayerAction::ZoomToOverview) {
                zoom_destination.0 = camera_properties.camera_zoom_max;
            } else if act.just_released(&PlayerAction::ZoomToOverview) {
                zoom_destination.0 = camera_properties.initial_camera_zoom;
            }
        }
    }
}

fn apply_camera_zoom(
    time: Res<Time>,
    mut query: Query<
        (
            &mut OrthographicProjection,
            &CanZoomSmoothly,
            &CameraProperties,
        ),
        With<Camera>,
    >,
) {
    if let Ok((mut projection, zoom_destination, camera_properties)) = query.get_single_mut() {
        // handle smooth zoom over time
        if projection.scale < zoom_destination.0 - camera_properties.camera_zoom_buffer
            || projection.scale > zoom_destination.0 + camera_properties.camera_zoom_buffer
        {
            projection.scale += (zoom_destination.0 - projection.scale)
                * (camera_properties.camera_zoom_snappiness)
                * time.delta_seconds();
        }
        projection.scale = projection.scale.clamp(
            camera_properties.camera_zoom_min,
            camera_properties.camera_zoom_max,
        );
    }
}

fn camera_follow(
    mut camera: Query<
        (&mut Transform, &OrthographicProjection, &CameraProperties),
        (With<Camera>, Without<Player>),
    >,
    target: Query<&Transform, (With<CameraFollowTarget>, Without<Camera>)>,
    level_walls: Res<LevelWalls>,
    time: Res<Time>,
) {
    let Ok(target_transform) = target.get_single() else {
        return;
    };
    let Ok((mut camera_transform, orthographic, properties)) = camera.get_single_mut() else {
        return;
    };
    //calculate bounds
    let vertical_bounds = orthographic.area.width();
    let horizontal_bounds = orthographic.area.height();
    let level_width = (level_walls.level_width * GRID_SIZE) as f32;
    let level_height = (level_walls.level_height * GRID_SIZE) as f32;
    let min_x = (horizontal_bounds / 2.0).min(level_width / 2.0);
    let max_x = (level_width - horizontal_bounds / 2.0).max(level_width / 2.0);
    let min_y = (vertical_bounds / 2.0).min(level_height / 2.0);
    let max_y = (level_height - vertical_bounds / 2.0).max(level_height / 2.0);

    let bounded_target_position = Vec3::new(
        target_transform.translation.x.clamp(min_x, max_x),
        target_transform.translation.y.clamp(min_y, max_y),
        camera_transform.translation.z,
    );

    //smoothly interpolate camera position to target position
    let translation = camera_transform.translation.lerp(
        bounded_target_position,
        time.delta_seconds() * properties.camera_follow_snappiness,
    );

    //and hard clam that camera's position if it is out of bounds
    camera_transform.translation = Vec3::new(
        translation.x.clamp(min_x, max_x),
        translation.y.clamp(min_y, max_y),
        camera_transform.translation.z,
    );
}
