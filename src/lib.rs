use bevy::window::WindowResolution;
use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    prelude::*,
};
use bevy_ecs_ldtk::{LdtkPlugin, LdtkWorldBundle, LevelSelection};

#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod screen;
mod ui;

pub mod geometry_2d;

mod input;
#[cfg(test)]
pub mod testing;

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
                AppSet::UpdateFog,
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
                        resolution: WindowResolution::new(1280.0, 1280.0),
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(AudioPlugin {
                    global_volume: GlobalVolume {
                        volume: Volume::new(0.3), // quiet audio
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

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
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
    /// Fog updates happen last
    UpdateFog,
}

fn spawn_ldtk_world_bundle(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading LDTK assets");
    commands.spawn((
        Name::new("LdtkWorld"),
        LdtkWorldBundle {
            ldtk_handle: asset_server.load("tile-based-game.ldtk"),
            ..Default::default()
        },
    ));
}
