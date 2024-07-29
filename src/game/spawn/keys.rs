use std::f32::consts;

use bevy::app::{App, Update};
use bevy::core::Name;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_ecs_ldtk::{GridCoords, LdtkEntity, LdtkSpriteSheetBundle};
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;

use crate::game::audio::sfx::Sfx;
use crate::game::dialog::{DialogLineType, ShowDialogEvent, ShowDialogType};
use crate::game::end_game::EndGameCondition;
use crate::game::grid::GridPosition;
use crate::game::line_of_sight::{CanRevealFog, PlayerLineOfSightBundle};
use crate::game::line_of_sight::vision::VisionAbility;
use crate::game::spawn::enemy::SpawnCoords;
use crate::game::spawn::exit::NumKeysPickedUp;
use crate::game::spawn::health::OnDeath;
use crate::game::utilities::intersect;

use super::player::Player;

pub(super) fn plugin(app: &mut App) {
    // spawning
    app.register_ldtk_entity::<LdtkKeyBundle>("Key");

    // systems
    app.add_systems(Update, fix_loaded_ldtk_entities);
    app.add_systems(Update, pickup_key);
    // reflection
    app.register_type::<Key>();
    app.observe(on_end_game_reset_keys);
    app.observe(on_death_respawn_key);
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
    player_line_of_sight_bundle: PlayerLineOfSightBundle,
}

impl KeyBundle {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            spawn_coords: SpawnCoords(GridPosition::new(x, y)),
            grid_position: GridPosition::new(x, y),
            can_pickup: CanPickup,
            player_line_of_sight_bundle: PlayerLineOfSightBundle {
                facing: Default::default(),
                can_reveal_fog: CanRevealFog,
                vision_ability: VisionAbility {
                    field_of_view_radians: 2. * consts::PI,
                    range_in_grid_units: 1.0,
                },
                facing_walls_cache: Default::default(),
                visible_squares: Default::default(),
            },
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
    player: Query<(Entity, &Transform, &Aabb), (With<Player>, Without<Key>)>,
    mut keys: Query<(Entity, &mut Transform, &Aabb), (With<Key>, With<CanPickup>)>,
    mut commands: Commands,
    mut picked_up_keys: ResMut<NumKeysPickedUp>,
) {
    let Ok((player_ent, player_transform, player_aabb)) = player.get_single() else {
        return;
    };

    for (key_entity, mut key_transform, key) in &mut keys {
        if intersect((player_transform, player_aabb), (&key_transform, key)) {
            commands.trigger(Sfx::KeyPickup);
            commands.entity(key_entity).remove::<CanPickup>();
            key_transform.translation.z = -10.;

            commands.trigger(ShowDialogEvent {
                entity: player_ent,
                dialog_type: ShowDialogType::NextLine(DialogLineType::PlayerFindsKey),
            });
            picked_up_keys.0 += 1;
        }
    }
}

pub fn on_death_respawn_key(
    _trigger: Trigger<OnDeath>,
    mut keys: Query<(Entity, &mut Transform, &SpawnCoords), With<Key>>,
    mut commands: Commands,
    mut num_keys_picked_up: ResMut<NumKeysPickedUp>,
) {
    for (key_entity, mut transform, spawn_coords) in &mut keys {
        let coordinates = spawn_coords.0.coordinates;
        commands
            .entity(key_entity)
            .insert(KeyBundle::new(coordinates.x, coordinates.y));

        transform.translation = Vec3::ZERO;

        commands.trigger(Sfx::KeyDrop);
        num_keys_picked_up.0 = 0;
    }
}
