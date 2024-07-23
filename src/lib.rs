#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod postprocessing;
mod screen;
mod ui;

mod input;

mod visuals;

#[cfg(test)]
pub mod testing;

use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    prelude::*,
};
use bevy_ecs_ldtk::{LdtkPlugin, LdtkWorldBundle, LevelSelection};
// use postprocessing::PostProcessing;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSet::TickTimers,
                AppSet::RecordInput,
                AppSet::UpdateVirtualGrid,
                AppSet::Update,
                AppSet::UpdateWorld,
            )
                .chain(),
        );

        // Spawn the main camera.
        app.add_systems(Startup, spawn_ldtk_world_bundle);
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
                        volume: Volume::new(0.05), // quiet audio
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
        app.add_plugins(input::plugin);
        app.add_plugins(visuals::plugin);

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);

        // app.add_plugins(PostProcessing);

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
    /// Any operations that happen on grid coordinates should happen before they get translated to pixels
    UpdateVirtualGrid,
    /// Do everything else (consider splitting this into further variants).
    Update,
    /// After all grid coordinates are settled, we translate them to real pixels in world space
    UpdateWorld,
}

fn spawn_ldtk_world_bundle(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("LdtkWorld"),
        LdtkWorldBundle {
            ldtk_handle: asset_server.load("tile-based-game.ldtk"),
            ..Default::default()
        },
    ));
}
