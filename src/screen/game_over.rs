use bevy::prelude::*;

use super::Screen;

use crate::{game::end_game::EndGameCondition, ui::prelude::*};

pub fn plugin(app: &mut App) {
    app.observe(end_game);
    app.add_systems(OnEnter(Screen::GameOver), enter_game_over);
    app.add_systems(OnExit(Screen::GameOver), exit_game_over);

    app.add_systems(
        Update,
        handle_game_over_action.run_if(in_state(Screen::GameOver)),
    );
}

fn end_game(_trigger: Trigger<EndGameCondition>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::GameOver);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum GameOverAction {
    Back,
}

fn enter_game_over(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::GameOver))
        .with_children(|children| {
            children.header("You dieded");

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
