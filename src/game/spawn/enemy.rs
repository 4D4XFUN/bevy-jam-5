use crate::game::grid::movement::GridMovement;
use crate::game::grid::GridPosition;
use crate::game::spawn::health::CanApplyDamage;
use crate::game::spawn::player::Player;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;
use bevy_ecs_ldtk::{EntityInstance, GridCoords, LdtkEntity, LdtkSpriteSheetBundle};

pub(super) fn plugin(app: &mut App) {
    // spawning
    app.register_ldtk_entity::<EnemyBundle>("Enemy");

    // systems
    app.add_systems(
        Update,
        (return_to_post, detect_player, follow_player).chain(),
    );

    // reflection
    app.register_type::<Enemy>();
    app.register_type::<CanSeePlayer>();
    app.register_type::<SpawnCoords>();
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Enemy;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct CanSeePlayer;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SpawnCoords(IVec2);

#[derive(Default, Bundle, LdtkEntity)]
struct EnemyBundle {
    enemy: Enemy,
    can_apply_damage: CanApplyDamage,
    #[sprite_sheet_bundle]
    sprite_bundle: LdtkSpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
    #[with(enemy_initial_components)]
    enemy_defaults_bundle: EnemySettingsBundle,
}

#[derive(Default, Bundle)]
struct EnemySettingsBundle {
    spawn_coords: SpawnCoords,
    grid_position: GridPosition,
    grid_movement: GridMovement,
}

fn enemy_initial_components(instance: &EntityInstance) -> EnemySettingsBundle {
    EnemySettingsBundle {
        spawn_coords: SpawnCoords(instance.grid),

        grid_position: GridPosition::new(instance.grid.x as f32, instance.grid.y as f32),
        grid_movement: GridMovement::immobile(), // todo revert to default
    }
}

const ENEMY_SIGHT_RANGE: f32 = 100.0;

fn detect_player(
    aware_enemies: Query<(Entity, &Transform), (With<Enemy>, With<CanSeePlayer>)>,
    unaware_enemies: Query<(Entity, &Transform), (With<Enemy>, Without<CanSeePlayer>)>,
    player: Query<&Transform, With<Player>>,
    mut commands: Commands,
) {
    for (enemy_entity, enemy_transform) in &aware_enemies {
        if let Ok(player_transform) = player.get_single() {
            if enemy_transform
                .translation
                .distance(player_transform.translation)
                > ENEMY_SIGHT_RANGE
            {
                commands.entity(enemy_entity).remove::<CanSeePlayer>();
            }
        }
    }
    for (enemy_entity, enemy_transform) in &unaware_enemies {
        if let Ok(player_transform) = player.get_single() {
            if enemy_transform
                .translation
                .distance(player_transform.translation)
                <= ENEMY_SIGHT_RANGE
            {
                commands.entity(enemy_entity).insert(CanSeePlayer);
            }
        }
    }
}

const ENEMY_CHASE_SPEED: f32 = 2.0;
const ENEMY_RETURN_TO_POST_SPEED: f32 = 1.0;

fn return_to_post(
    mut unaware_enemies: Query<
        (&mut GridMovement, &Transform, &SpawnCoords),
        (With<Enemy>, Without<CanSeePlayer>),
    >,
) {
    for (mut controller, transform, coords) in &mut unaware_enemies {
        let spawn_translation =
            Vec2::new(coords.0.x as f32 * 16.0, 1024.0 - coords.0.y as f32 * 16.0);
        let direction = spawn_translation - transform.translation.truncate();

        controller.acceleration_player_force = direction.normalize() * ENEMY_RETURN_TO_POST_SPEED;
    }
}

fn follow_player(
    mut enemy_movement_controllers: Query<
        (&mut GridMovement, &Transform),
        (With<Enemy>, With<CanSeePlayer>),
    >,
    player: Query<&Transform, With<Player>>,
) {
    for (mut controller, entity_transform) in &mut enemy_movement_controllers {
        if let Ok(player_transform) = player.get_single() {
            let direction = player_transform.translation - entity_transform.translation;

            controller.acceleration_player_force =
                direction.truncate().normalize() * ENEMY_CHASE_SPEED;
        }
    }
}
