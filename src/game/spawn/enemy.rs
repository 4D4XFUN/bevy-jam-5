use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;
use bevy_ecs_ldtk::{GridCoords, LdtkEntity, LdtkSpriteSheetBundle};
use rand::Rng;

use crate::game::ai::Hunter;
use crate::game::grid::GridPosition;
use crate::game::line_of_sight::vision::{
    Facing, VisibleSquares, VisionAbility, VisionArchetype, VisionBundle,
};
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
        (rotate_facing, return_to_post, detect_player, follow_player).chain(),
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

#[derive(Component, Reflect, Copy, Clone, Default)]
#[reflect(Component)]
pub struct SpawnCoords(pub GridPosition);

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
    marker: Enemy,
    vision: VisionBundle,
    role: Hunter,
}

impl EnemyBundle {
    pub fn new(x: i32, y: i32) -> Self {
        // todo delete this it's for testing - randomize types of enemies
        let mut rng = rand::thread_rng();
        let is_sniper = rng.gen_ratio(1, 3);
        let vision_archetype = if is_sniper {
            VisionArchetype::Sniper
        } else {
            VisionArchetype::Patrol
        };

        Self {
            marker: Enemy,
            can_damage: CanApplyDamage,
            spawn_coords: SpawnCoords(GridPosition::new(x as f32, y as f32)),
            grid_position: GridPosition::new(x as f32, y as f32),
            grid_movement: GridMovement::default(),
            vision: VisionBundle {
                vision_ability: VisionAbility::of(vision_archetype),
                ..default()
            },
            role: Hunter,
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

fn rotate_facing(
    mut query: Query<&mut Facing, (With<Enemy>, Without<CanSeePlayer>)>,
    time: Res<Time>,
) {
    const SECONDS_TO_ROTATE: f32 = 10.;
    const RADIANS_PER_SEC: f32 = 2.0 * std::f32::consts::PI / SECONDS_TO_ROTATE;
    for mut facing in query.iter_mut() {
        let dt = time.delta_seconds();
        let mut f: Vec2 = facing.direction;

        let angle = RADIANS_PER_SEC * dt;
        f = Vec2::new(
            f.x * angle.cos() - f.y * angle.sin(),
            f.x * angle.sin() + f.y * angle.cos(),
        );

        f = f.normalize();

        facing.direction = f;
    }
}

fn detect_player(
    aware_enemies: Query<(Entity, &Transform, &VisibleSquares), (With<Enemy>, With<CanSeePlayer>)>,
    unaware_enemies: Query<(Entity, &VisibleSquares), (With<Enemy>, Without<CanSeePlayer>)>,
    player: Query<(&GridPosition, &Transform), With<Player>>,
    mut commands: Commands,
) {
    let Ok((player_grid_pos, player_transform)) = player.get_single() else {
        return;
    };

    for (enemy_entity, enemy_transform, enemy_vision) in &aware_enemies {
        if !enemy_vision.contains(player_grid_pos)
            && enemy_transform
                .translation
                .distance(player_transform.translation)
                > ENEMY_CHASE_RANGE
        {
            commands.entity(enemy_entity).remove::<CanSeePlayer>();
        }
    }
    for (enemy_entity, enemy_vision) in &unaware_enemies {
        if enemy_vision.contains(player_grid_pos) {
            commands.entity(enemy_entity).insert(CanSeePlayer);
        }
    }
}

const ENEMY_CHASE_SPEED: f32 = 100.0;
const ENEMY_RETURN_TO_POST_SPEED: f32 = 21.0;
const ENEMY_CHASE_RANGE: f32 = 100.0;

fn return_to_post(
    mut unaware_enemies: Query<
        (&mut GridMovement, &GridPosition, &SpawnCoords),
        (With<Enemy>, Without<CanSeePlayer>),
    >,
) {
    for (mut movement, &position, spawn) in &mut unaware_enemies {
        let direction = position.direction_to(&spawn.0);
        if direction.length() < 1.0 {
            movement.acceleration_player_force = Vec2::ZERO;
        } else {
            movement.acceleration_player_force = direction.normalize() * ENEMY_RETURN_TO_POST_SPEED;
        }
    }
}

pub(crate) fn follow_player(
    mut enemy_movement_controllers: Query<
        (&mut GridMovement, &mut Facing, &GridPosition),
        (With<Enemy>, With<CanSeePlayer>),
    >,
    player: Query<&GridPosition, With<Player>>,
) {
    let Ok(player_pos) = player.get_single() else {
        return;
    };

    for (mut controller, mut facing, enemy_pos) in &mut enemy_movement_controllers {
        let direction = enemy_pos.direction_to(player_pos);
        facing.direction = direction;
        controller.acceleration_player_force = direction.normalize() * ENEMY_CHASE_SPEED;
    }
}

fn on_death_reset_enemies(
    _trigger: Trigger<OnDeath>,
    mut query: Query<(Entity, &mut GridPosition, &SpawnCoords, &mut Facing), With<Enemy>>,
    mut commands: Commands,
) {
    for (enemy, mut pos, spawn_point, mut facing) in &mut query {
        *pos = spawn_point.0;
        facing.direction = Vec2::new(1., 0.);
        commands.entity(enemy).remove::<CanSeePlayer>();
    }
}
