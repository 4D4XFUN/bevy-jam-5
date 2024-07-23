use bevy::{
    prelude::*,
    render::{camera::RenderTarget, view::RenderLayers},
};
use bevy_magic_light_2d::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_type::<LightOccluder2D>()
        .register_type::<OmniLightSource2D>()
        .register_type::<SkylightMask2D>()
        .register_type::<SkylightLight2D>()
        .register_type::<BevyMagicLight2DSettings>()
        .register_type::<LightPassParams>()
        .insert_resource(BevyMagicLight2DSettings {
            light_pass_params: LightPassParams {
                reservoir_size: 16,
                smooth_kernel_size: (2, 1),
                direct_light_contrib: 0.2,
                indirect_light_contrib: 0.8,
                ..default()
            },
            ..default()
        })
        .insert_resource(ClearColor(Color::srgba_u8(0, 0, 0, 0)));
    app.add_plugins(BevyMagicLight2DPlugin);
    app.add_systems(Startup, setup.after(setup_post_processing_camera));
}

const CAMERA_SCALE: f32 = 1.0;

fn setup(camera_targets: Res<CameraTargets>, mut commands: Commands) {
    // Add roofs.
    commands
        .spawn(SpatialBundle {
            transform: Transform {
                translation: Vec3::new(30.0, -180.0, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("skylight_mask_1"))
        .insert(SkylightMask2D {
            h_size: Vec2::new(430.0, 330.0),
        });
    commands
        .spawn(SpatialBundle {
            transform: Transform {
                translation: Vec3::new(101.6, -989.4, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("skylight_mask_2"))
        .insert(SkylightMask2D {
            h_size: Vec2::new(163.3, 156.1),
        });

    // Add skylight light.
    commands.spawn((
        SkylightLight2D {
            color: Color::srgb_u8(93, 158, 179),
            intensity: 0.025,
        },
        Name::new("global_skylight"),
    ));

    let projection = OrthographicProjection {
        scale: CAMERA_SCALE,
        near: -2000.0,
        far: 2000.0,
        ..default()
    };
    commands
        .spawn(Camera2dBundle {
            camera: Camera {
                hdr: false,
                target: RenderTarget::Image(camera_targets.floor_target.clone()),
                ..default()
            },
            projection: projection.clone(),
            ..default()
        })
        .insert(Name::new("floors_target_camera"))
        .insert(SpriteCamera)
        .insert(FloorCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_FLOOR));

    commands
        .spawn(Camera2dBundle {
            camera: Camera {
                hdr: false,
                target: RenderTarget::Image(camera_targets.walls_target.clone()),
                ..default()
            },
            projection: projection.clone(),
            ..default()
        })
        .insert(Name::new("walls_target_camera"))
        .insert(SpriteCamera)
        .insert(WallsCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_WALLS));

    commands
        .spawn(Camera2dBundle {
            camera: Camera {
                hdr: false,
                target: RenderTarget::Image(camera_targets.objects_target.clone()),
                ..default()
            },
            projection: projection.clone(),
            ..default()
        })
        .insert(Name::new("objects_target_camera"))
        .insert(SpriteCamera)
        .insert(ObjectsCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS));
}
