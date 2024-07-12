use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                // to avoid blurry pixels
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // just draw something simple on screen so we can verify it's working on itch
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::srgb(0.8, 0.2, 0.2),
            custom_size: Some(Vec2::new(100.0, 100.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::ZERO),
        ..default()
    });
}
