use bevy::prelude::*;

#[derive(Event)]
pub enum EndGameCondition {
    // Win,
    TimeOut,
}
