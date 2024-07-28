use bevy::app::{App, Update};
use bevy::core::Name;
use bevy::math::Vec3;
use bevy::prelude::{
    Bundle, Commands, Component, Entity, Query, Reflect, Transform, Trigger, With, Without,
};
use bevy::render::primitives::Aabb;
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;
use bevy_ecs_ldtk::{GridCoords, LdtkEntity, LdtkSpriteSheetBundle};

use crate::game::audio::sfx::Sfx;
use crate::game::grid::GridPosition;
use crate::game::spawn::enemy::SpawnCoords;
use crate::game::spawn::health::OnDeath;

use super::player::Player;

pub(super) fn plugin(app: &mut App) {
    // spawning
    app.register_ldtk_entity::<LdtkKeyBundle>("Key");

    // systems
    app.add_systems(Update, fix_loaded_ldtk_entities);
    app.add_systems(Update, pickup_key);
    // reflection
    app.register_type::<Key>();
    app.observe(on_death_reset_keys);
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
                Name::new("LdtkKey"),
                KeyBundle::new(grid_coords.x as f32, grid_coords.y as f32),
            ));
    }
}

fn on_death_reset_keys(
    _trigger: Trigger<OnDeath>,
    mut query: Query<(&mut GridPosition, &SpawnCoords), With<Key>>,
) {
    for (mut pos, spawn_point) in &mut query {
        *pos = spawn_point.0;
    }
}

fn pickup_key(
    player: Query<(&Transform, &Aabb), (With<Player>, Without<Key>)>,
    mut keys: Query<(Entity, &mut GridPosition, &Aabb), With<CanPickup>>,
    mut commands: Commands,
) {
    let Ok((player_transform, player)) = player.get_single() else {
        return;
    };

    let player_min =
        Vec3::from(player.center) - Vec3::from(player.half_extents) + player_transform.translation;
    let player_max =
        Vec3::from(player.center) + Vec3::from(player.half_extents) + player_transform.translation;

    let Ok((_key_entity, _key_transform, _key)) = keys.get_single() else {
        return;
    };

    for (key_entity, key_transform, key) in keys.iter_mut() {
        let key_min =
            Vec3::from(key.center) - Vec3::from(key.half_extents) + key_transform.coordinates.extend(0.0);
        let key_max =
            Vec3::from(key.center) + Vec3::from(key.half_extents) + key_transform.coordinates.extend(0.0);

        let x_min = player_min.x >= key_min.x && player_min.x <= key_max.x;
        let x_max = player_max.x >= key_min.x && player_max.x <= key_max.x;

        let y_min = player_min.y >= key_min.y && player_min.y <= key_max.y;
        let y_max = player_max.y >= key_min.y && player_max.y <= key_max.y;

        if (x_min || x_max) && (y_min || y_max) {
            commands.trigger(Sfx::KeyPickup);
            commands.entity(key_entity).remove::<CanPickup>();
        }
    }
}

pub fn on_death_drop_key(
    _trigger: Trigger<OnDeath>,
    mut keys: Query<(Entity, &mut Transform), (With<Key>, Without<CanPickup>)>,
    player_pos: Query<&GridPosition, With<Player>>,
    mut commands: Commands,
) {
    let player_pos = player_pos.single();
    for (mut key_entity, mut transform) in keys.iter_mut() {
        key_entity = commands
            .spawn(KeyBundle::new(
                player_pos.coordinates.x,
                player_pos.coordinates.y,
            ))
            .id();

        commands.entity(key_entity).insert(CanPickup);

        transform.scale = Vec3::splat(1.0);
        transform.translation =
            player_pos.actual_coordinates().extend(0.0) - Vec3::new(0., 0.5, 0.);

        commands.trigger(Sfx::KeyDrop);
    }
}


