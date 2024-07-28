use bevy::{prelude::*, render::primitives::Aabb};

use crate::{
    game::{
        assets::{ImageAsset, ImageAssets},
        end_game::EndGameCondition,
        grid::GridPosition,
        utilities::intersect,
    },
    screen::Screen,
};

use super::{
    keys::{CanPickup, Key},
    player::Player,
};

pub fn plugin(app: &mut App) {
    app.observe(spawn_exit);

    app.add_systems(Update, check_exit);
    app.add_systems(Update, lose_unlockability_on_death);
    app.add_systems(Update, gain_unlockability_on_pickup);
}

#[derive(Event)]
pub struct SpawnExitTrigger;

#[derive(Component)]
struct Exit;

#[derive(Component)]
pub struct CanBeUnlocked;

const LADDER_INDEX: usize = 4 * 12;

fn spawn_exit(
    _trigger: Trigger<SpawnExitTrigger>,
    images: Res<ImageAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 12, 5, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    commands.spawn((
        Name::new("Exit"),
        StateScoped(Screen::Playing),
        Exit,
        SpriteBundle {
            texture: images[&ImageAsset::Decoration].clone_weak(),
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: LADDER_INDEX,
        },
        GridPosition::new(59., 31.),
    ));
}

fn lose_unlockability_on_death(
    mut commands: Commands,
    q_keys: Query<Entity, (With<Key>, With<CanPickup>)>,
    mut q_exit: Query<Entity, (With<Exit>, With<CanBeUnlocked>)>,
) {
    if q_keys.iter().count() == 0 {
        for exit_entity in q_exit.iter_mut() {
            commands.entity(exit_entity).remove::<CanBeUnlocked>();
        }
    }
}

fn gain_unlockability_on_pickup(
    mut commands: Commands,
    q_keys: Query<Entity, (With<Key>, Without<CanPickup>)>,
    mut q_exit: Query<Entity, (With<Exit>, Without<CanBeUnlocked>)>,
) {
    for _ in q_keys.iter() {
        for exit_entity in q_exit.iter_mut() {
            commands.entity(exit_entity).insert(CanBeUnlocked);
        }
    }
}

fn check_exit(
    exits: Query<(&Transform, &Aabb), (With<Exit>, With<CanBeUnlocked>)>,
    players: Query<(&Transform, &Aabb), With<Player>>,
    mut commands: Commands,
) {
    let Ok(exit) = exits.get_single() else {
        return;
    };

    let Ok(player) = players.get_single() else {
        return;
    };

    if intersect(exit, player) {
        commands.trigger(EndGameCondition::Win);
    }
}
