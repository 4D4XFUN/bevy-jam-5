//! AI proving grounds

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::game::ai::spawn_ai_proving_grounds::SpawnAiProvingGrounds;

use super::{playing, Screen};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::AiProvingGrounds), enter_ai_proving_grounds);
    app.add_systems(OnExit(Screen::AiProvingGrounds), exit_ai_proving_grounds);

    app.add_systems(
        Update,
        playing::return_to_title_screen.run_if(
            in_state(Screen::AiProvingGrounds).and_then(input_just_pressed(KeyCode::Escape)),
        ),
    );

    // add a secret way to get here from the main menu - F2
    app.add_systems(
        Update,
        transition_to_ai_proving_grounds
            .run_if(in_state(Screen::Title).and_then(input_just_pressed(KeyCode::F2))),
    );
}

pub fn transition_to_ai_proving_grounds(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::AiProvingGrounds);
}

fn enter_ai_proving_grounds(mut commands: Commands) {
    commands.trigger(SpawnAiProvingGrounds);
}

fn exit_ai_proving_grounds(mut _commands: Commands) {}
