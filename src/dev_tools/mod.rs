//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{dev_tools::states::log_transitions, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use leafwing_input_manager::prelude::*;

use crate::input::DevActionToggles;
use crate::screen::Screen;

pub mod grid_overlay;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<DebugOverlaysState>();
    app.add_plugins((grid_overlay::plugin, enemy_vision::plugin));

    // Print state transitions in dev builds
    app.add_systems(Update, log_transitions::<Screen>);
    app.add_systems(Update, log_transitions::<WorldInspectorState>);
    app.add_systems(Update, log_transitions::<DebugOverlaysState>);
    app.add_systems(Startup, spawn_dev_input_manager);

    app.add_systems(Update, toggle_debug_overlays);

    // press F1 in dev builds to open an entity inspector
    app.init_state::<WorldInspectorState>()
        .add_systems(Update, toggle_world_inspector_state)
        .add_plugins(WorldInspectorPlugin::new().run_if(in_state(WorldInspectorState::Enabled)));
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum DebugOverlaysState {
    #[default]
    Disabled,
    Enabled,
}

/// listens for dev-only keybinds
fn spawn_dev_input_manager(mut commands: Commands) {
    commands.spawn(InputManagerBundle::with_map(
        DevActionToggles::default_input_map(),
    ));
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
enum WorldInspectorState {
    #[default]
    Disabled,
    Enabled,
}

fn toggle_world_inspector_state(
    current_state: Res<State<WorldInspectorState>>,
    query: Query<&ActionState<DevActionToggles>>,
    mut set_next_state: ResMut<NextState<WorldInspectorState>>,
) {
    for act in query.iter() {
        if act.just_pressed(&DevActionToggles::WorldInspector) {
            set_next_state.set(match current_state.get() {
                WorldInspectorState::Disabled => WorldInspectorState::Enabled,
                WorldInspectorState::Enabled => WorldInspectorState::Disabled,
            });
        }
    }
}

pub fn toggle_debug_overlays(
    current_state: Res<State<DebugOverlaysState>>,
    query: Query<&ActionState<DevActionToggles>>,
    mut set_next_state: ResMut<NextState<DebugOverlaysState>>,
) {
    for act in query.iter() {
        if act.just_pressed(&DevActionToggles::DebugOverlays) {
            set_next_state.set(match current_state.get() {
                DebugOverlaysState::Disabled => DebugOverlaysState::Enabled,
                DebugOverlaysState::Enabled => DebugOverlaysState::Disabled,
            });
        }
    }
}

mod enemy_vision {
    use bevy::app::App;
    use bevy::prelude::*;

    use crate::dev_tools::DebugOverlaysState;
    use crate::game::ai::Hunter;
    use crate::game::grid::grid_layout::GridLayout;
    use crate::game::grid::GridPosition;
    use crate::game::line_of_sight::vision::VisibleSquares;
    use crate::AppSet;

    pub(super) fn plugin(app: &mut App) {
        app.add_systems(
            Update,
            render_enemy_vision_cones
                .in_set(AppSet::UpdateFog)
                .run_if(in_state(DebugOverlaysState::Enabled)),
        );
    }

    pub fn render_enemy_vision_cones(
        mut gizmos: Gizmos,
        query: Query<(&GridPosition, &VisibleSquares), With<Hunter>>,
        grid: Res<GridLayout>,
    ) {
        for (position, h) in query.iter() {
            let squares: Vec<_> = h
                .visible_squares
                .iter()
                .map(GridPosition::from_ivec)
                .collect();
            for square in squares {
                let distance_from_enemy = position.coordinates.distance(square.coordinates);
                gizmos.rect_2d(
                    grid.grid_to_world(&square),
                    0.,
                    Vec2::splat(4.),
                    Color::srgba(
                        0.2 + 1.0 / distance_from_enemy,
                        0.1 + distance_from_enemy / 20.0,
                        0.2,
                        1.0,
                    ),
                )
            }
        }
    }
}
