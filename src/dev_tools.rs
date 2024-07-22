//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{dev_tools::states::log_transitions, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::game::grid::DebugOverlaysState;
use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    // Print state transitions in dev builds
    app.add_systems(Update, log_transitions::<Screen>);
    app.add_systems(Update, log_transitions::<WorldInspectorState>);
    app.add_systems(Update, log_transitions::<DebugOverlaysState>);

    // press F1 in dev builds to open an entity inspector
    app.init_state::<WorldInspectorState>()
        .add_systems(Update, update_world_inspector_state)
        .add_plugins(WorldInspectorPlugin::new().run_if(in_state(WorldInspectorState::Enabled)));
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
enum WorldInspectorState {
    #[default]
    Disabled,
    Enabled,
}

fn update_world_inspector_state(
    current_state: Res<State<WorldInspectorState>>,
    keypress: Res<ButtonInput<KeyCode>>,
    mut set_next_state: ResMut<NextState<WorldInspectorState>>,
) {
    if keypress.just_pressed(KeyCode::F1) {
        set_next_state.set(match current_state.get() {
            WorldInspectorState::Disabled => WorldInspectorState::Enabled,
            WorldInspectorState::Enabled => WorldInspectorState::Disabled,
        });
    }
}
