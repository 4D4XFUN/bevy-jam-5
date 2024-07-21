#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod screen;
mod ui;

use crate::game::spawn::player::Player;
use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    prelude::*,
};
use bevy_ecs_ldtk::{LdtkPlugin, LdtkWorldBundle, LevelSelection};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (AppSet::TickTimers, AppSet::RecordInput, AppSet::Update).chain(),
        );

        // Spawn the main camera.
        app.add_systems(Startup, (spawn_camera, spawn_ldtk_world_bundle).chain());
        app.add_systems(Update, camera_follows_player); // rudimentary player-following camera
        app.insert_resource(LevelSelection::index(0));

        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "bevy-jam-5".to_string(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }
                        .into(),
                    ..default()
                })
                .set(AudioPlugin {
                    global_volume: GlobalVolume {
                        volume: Volume::new(0.0), // mute audio
                    },
                    ..default()
                })
                // to avoid blurry pixels
                .set(ImagePlugin::default_nearest()),
        );

        app.add_plugins(LdtkPlugin);

        // Add other plugins.
        app.add_plugins((game::plugin, screen::plugin, ui::plugin));
        app.add_plugins(game::ai::plugin);

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);

        #[cfg(feature = "dev")]
        app.add_plugins(game::ai::proving_grounds_plugin);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 0.5;
    // camera.transform.translation.x += 900.;
    // camera.transform.translation.y += 400.;
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

fn spawn_ldtk_world_bundle(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(
        (Name::new("LdtkWorld"),
         LdtkWorldBundle {
             ldtk_handle: asset_server.load("tile-based-game.ldtk"),
             ..Default::default()
         }));
}
