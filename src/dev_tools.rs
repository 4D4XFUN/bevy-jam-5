//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{dev_tools::states::log_transitions, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use leafwing_input_manager::prelude::*;

use crate::game::grid::DebugOverlaysState;
use crate::input::DevAction;
use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    // Print state transitions in dev builds
    app.add_systems(Update, log_transitions::<Screen>);
    app.add_systems(Update, log_transitions::<WorldInspectorState>);
    app.add_systems(Update, log_transitions::<DebugOverlaysState>);
    app.add_systems(Startup, spawn_dev_input_manager);

    // press F1 in dev builds to open an entity inspector
    app.init_state::<WorldInspectorState>()
        .add_systems(Update, toggle_world_inspector_state)
        .add_plugins(WorldInspectorPlugin::new().run_if(in_state(WorldInspectorState::Enabled)));
}

/// listens for dev-only keybinds
fn spawn_dev_input_manager(mut commands: Commands) {
    commands.spawn(InputManagerBundle::with_map(DevAction::default_input_map()));
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
enum WorldInspectorState {
    #[default]
    Disabled,
    Enabled,
}

fn toggle_world_inspector_state(
    current_state: Res<State<WorldInspectorState>>,
    query: Query<&ActionState<DevAction>>,
    mut set_next_state: ResMut<NextState<WorldInspectorState>>,
) {
    for act in query.iter() {
        if act.just_pressed(&DevAction::ToggleWorldInspector) {
            set_next_state.set(match current_state.get() {
                WorldInspectorState::Disabled => WorldInspectorState::Enabled,
                WorldInspectorState::Enabled => WorldInspectorState::Disabled,
            });
        }
    }
}
