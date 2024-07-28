use bevy::prelude::*;

use crate::{
    game::{end_game::EndGameCondition, threat::ThreatTimer},
    ui::prelude::*,
};

use super::Screen;

pub fn plugin(app: &mut App) {
    app.observe(end_game);
    app.add_systems(OnEnter(Screen::GameOver), enter_game_over);
    app.add_systems(OnExit(Screen::GameOver), exit_game_over);

    app.add_systems(
        Update,
        handle_game_over_action.run_if(in_state(Screen::GameOver)),
    );
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(Screen = Screen::GameOver)]
pub enum EndGame {
    #[default]
    Win,
}

fn end_game(
    _trigger: Trigger<EndGameCondition>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_substate: ResMut<NextState<EndGame>>,
) {
    next_screen.set(Screen::GameOver);
    let substate = match _trigger.event() {
        EndGameCondition::Win => EndGame::Win,
    };
    next_substate.set(substate);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum GameOverAction {
    Back,
}

fn enter_game_over(mut commands: Commands, substate: Res<State<EndGame>>, time: Res<ThreatTimer>) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::GameOver))
        .with_children(|children| {
            let text = "You won!";
            children.header(text);
            if *substate.get() == EndGame::Win {
                children.label(format!("Your time was {}s", time.timer.elapsed_secs()));
            }

            children.button("Back").insert(GameOverAction::Back);
        });
}

fn exit_game_over() {}

fn handle_game_over_action(
    mut next_screen: ResMut<NextState<Screen>>,
    mut button_query: InteractionQuery<&GameOverAction>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                GameOverAction::Back => next_screen.set(Screen::Title),
            }
        }
    }
}
