use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;
use bevy_ecs_ldtk::{GridCoords, LdtkEntity, LdtkSpriteSheetBundle};

use crate::game::grid::GridPosition;
use crate::game::movement::GridMovement;
use crate::game::spawn::health::{CanApplyDamage, OnDeath};
use crate::game::spawn::player::Player;

pub(super) fn plugin(app: &mut App) {
    // spawning
    app.register_ldtk_entity::<LdtkEnemyBundle>("Enemy");

    #[cfg(feature = "dev")]
    app.observe(spawn_oneshot_enemy);

    // systems
    app.add_systems(Update, fix_loaded_ldtk_entities);
    app.add_systems(
        Update,
        (return_to_post, detect_player, follow_player).chain(),
    );

    // reflection
    app.register_type::<Enemy>();
    app.register_type::<CanSeePlayer>();
    app.register_type::<SpawnCoords>();
    app.observe(on_death_reset_enemies);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Enemy;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct CanSeePlayer;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SpawnCoords(GridPosition);

#[derive(Component, Default, Copy, Clone)]
pub struct LdtkEnemy;

// This is ldtk-specific stuff for loading enemy assets.
// These should be transformed into an enemy type internal to our app
#[derive(Default, Bundle, LdtkEntity)]
struct LdtkEnemyBundle {
    tag: LdtkEnemy,
    #[sprite_sheet_bundle]
    sprite_bundle: LdtkSpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
}

// This is what our game needs to make an enemy work, separate from LDTK
// Keeping the stuff we need to work separate from LDTK lets us instantiate enemies in code, if we want/need to.
#[derive(Bundle)]
struct EnemyBundle {
    spawn_coords: SpawnCoords,
    grid_position: GridPosition,
    grid_movement: GridMovement,
    can_damage: CanApplyDamage,
    can_see: CanSeePlayer,
    marker: Enemy,
}

impl EnemyBundle {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            marker: Enemy,
            can_see: CanSeePlayer,
            can_damage: CanApplyDamage,
            spawn_coords: SpawnCoords(GridPosition::new(x as f32, y as f32)),
            grid_position: GridPosition::new(x as f32, y as f32),
            grid_movement: GridMovement::default(),
        }
    }
}

/// Takes all ldtk enemy entities, and adds all the components we need for them to work in our game.
fn fix_loaded_ldtk_entities(
    query: Query<(Entity, &GridCoords), With<LdtkEnemy>>,
    mut commands: Commands,
) {
    for (ldtk_entity, grid_coords) in query.iter() {
        commands
            .entity(ldtk_entity)
            .remove::<LdtkEnemy>() // we have to remove it because it's used as the query for this function
            .insert((
                Name::new("LdtkEnemy"),
                EnemyBundle::new(grid_coords.x, grid_coords.y),
            ));
    }
}

#[derive(Event, Debug)]
pub struct SpawnEnemyTrigger;

#[cfg(feature = "dev")]
fn spawn_oneshot_enemy(
    _trigger: Trigger<SpawnEnemyTrigger>,
    mut commands: Commands,
    images: Res<crate::game::assets::ImageAssets>,
) {
    info!("Spawning a dev-only gargoyle next to player to test non-ldtk enemy functionality");
    commands.spawn((
        Name::new("custom_gargoyle"),
        EnemyBundle::new(42, 24), // right next to player
        SpriteBundle {
            texture: images[&crate::game::assets::ImageAsset::Gargoyle].clone_weak(),
            transform: Transform::from_translation(Vec3::default().with_z(100.)),
            ..Default::default()
        },
    ));
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

const ENEMY_CHASE_SPEED: f32 = 10.0;
const ENEMY_RETURN_TO_POST_SPEED: f32 = 30.0;

fn return_to_post(
    mut unaware_enemies: Query<
        (&mut GridMovement, &Transform, &SpawnCoords),
        (With<Enemy>, Without<CanSeePlayer>),
    >,
) {
    for (mut controller, transform, coords) in &mut unaware_enemies {
        let spawn_translation = Vec2::new(
            coords.0.coordinates.x * 16.0,
            1024.0 - coords.0.coordinates.y * 16.0,
        );
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

fn on_death_reset_enemies(
    _trigger: Trigger<OnDeath>,
    mut query: Query<(&mut GridPosition, &SpawnCoords), With<Enemy>>,
) {
    for (mut pos, spawn_point) in &mut query {
        *pos = spawn_point.0;
    }
}
