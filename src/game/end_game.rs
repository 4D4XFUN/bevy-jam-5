use bevy::prelude::*;

#[derive(Event)]
#[allow(dead_code)]
pub enum EndGameCondition {
    Win,
}
