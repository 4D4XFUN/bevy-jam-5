use bevy::app::{App, Update};
use bevy::core::Name;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;
use bevy_ecs_ldtk::{GridCoords, LdtkEntity, LdtkSpriteSheetBundle};

use crate::game::audio::sfx::Sfx;
use crate::game::end_game::EndGameCondition;
use crate::game::ghost::Ghost;
use crate::game::grid::GridPosition;
use crate::game::spawn::enemy::SpawnCoords;
use crate::game::spawn::health::OnDeath;
use crate::game::utilities::intersect;

use super::player::Player;

pub(super) fn plugin(app: &mut App) {
    // spawning
    app.register_ldtk_entity::<LdtkKeyBundle>("Key");

    // systems
    app.add_systems(Update, fix_loaded_ldtk_entities);
    app.add_systems(Update, pickup_key);
    app.add_systems(Update, follow_player);
    // reflection
    app.register_type::<Key>();
    app.observe(on_end_game_reset_keys);
    app.observe(on_death_drop_key);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub struct Key;

#[derive(Component)]
pub struct CanPickup;

#[derive(Component, Default, Copy, Clone)]
pub struct LdtkKey;

#[derive(Default, Bundle, LdtkEntity)]
struct LdtkKeyBundle {
    tag: LdtkKey,
    #[sprite_sheet_bundle]
    sprite_bundle: LdtkSpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Bundle)]
struct KeyBundle {
    spawn_coords: SpawnCoords,
    grid_position: GridPosition,
    can_pickup: CanPickup,
}

impl KeyBundle {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            spawn_coords: SpawnCoords(GridPosition::new(x, y)),
            grid_position: GridPosition::new(x, y),
            can_pickup: CanPickup,
        }
    }
}

fn fix_loaded_ldtk_entities(
    query: Query<(Entity, &GridCoords), With<LdtkKey>>,
    mut commands: Commands,
) {
    for (ldtk_entity, grid_coords) in query.iter() {
        commands
            .entity(ldtk_entity)
            .remove::<LdtkKey>() // we have to remove it because it's used as the query for this function
            .insert((
                Name::new("Key"),
                KeyBundle::new(grid_coords.x as f32, grid_coords.y as f32),
                Key,
            ));
    }
}

fn on_end_game_reset_keys(
    _trigger: Trigger<EndGameCondition>,
    mut query: Query<(&mut GridPosition, &SpawnCoords), With<Key>>,
) {
    info!("resetting keys");
    for (mut pos, spawn_point) in &mut query {
        *pos = spawn_point.0;
    }
}

fn pickup_key(
    player: Query<(&Transform, &Aabb), (With<Player>, Without<Ghost>, Without<Key>)>,
    mut keys: Query<(Entity, &Transform, &Aabb), (With<Key>, With<CanPickup>)>,
    mut commands: Commands,
) {
    let Ok(player) = player.get_single() else {
        return;
    };

    for (key_entity, key_transform, key) in &mut keys {
        if intersect(player, (&key_transform, key)) {
            commands.trigger(Sfx::KeyPickup);
            commands.entity(key_entity).remove::<CanPickup>();
        }
    }
}

pub fn follow_player(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Ghost>, Without<Key>)>,
    mut key_query: Query<&mut Transform, (With<Key>, Without<CanPickup>)>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let speed = 5.0;
        for mut key_transform in &mut key_query {
            key_transform.translation = key_transform
                .translation
                .lerp(player_transform.translation, speed * time.delta_seconds());
        }
    }
}

pub fn on_death_drop_key(
    trigger: Trigger<OnDeath>,
    mut keys: Query<(Entity, &mut Transform), (With<Key>, Without<CanPickup>)>,
    mut commands: Commands,
) {
    let death = trigger.event();

    for (key_entity, mut transform) in &mut keys {
        commands
            .entity(key_entity)
            .insert(KeyBundle::new(death.0.x, death.0.y));

        transform.translation = Vec3::ZERO;

        commands.trigger(Sfx::KeyDrop);
    }
}
