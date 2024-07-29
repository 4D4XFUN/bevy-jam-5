use std::time::Duration;

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_ldtk::ldtk::FieldValue;
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;
use bevy_ecs_ldtk::{EntityInstance, GridCoords, LdtkEntity, LdtkSpriteSheetBundle};

use crate::game::ai::patrol::{PatrolBundle, PatrolMode, PatrolRoute, PatrolState, PatrolWaypoint};
use crate::game::ai::AiState::{Chasing, ReturnedToPost};
use crate::game::ai::{AiState, HasAiState, Hunter};
use crate::game::animation::{PlayerAnimation, PlayerAnimationState};
use crate::game::assets::{ImageAsset, ImageAssets};
use crate::game::audio::sfx::Sfx;
use crate::game::dialog::{DialogLineType, ShowDialogEvent, ShowDialogType};
use crate::game::grid::GridPosition;
use crate::game::line_of_sight::vision::{
    Facing, VisibleSquares, VisionAbility, VisionArchetype, VisionBundle,
};
use crate::game::line_of_sight::vision_cones::RenderedFieldOfView;
use crate::game::movement::GridMovement;
use crate::game::spawn::health::{CanApplyDamage, OnDeath};
use crate::game::spawn::player::Player;
use crate::game::threat::{ThreatTimer, ThreatTimerSettings};
use crate::screen::Screen;
use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    // spawning
    app.register_ldtk_entity::<LdtkEnemyBundle>("Enemy");

    // systems
    app.add_systems(
        Update,
        (detect_player, return_to_post, follow_player)
            .chain()
            .run_if(in_state(Screen::Playing))
            .in_set(AppSet::Update),
    );

    // reflection
    app.register_type::<Enemy>();
    app.register_type::<CanSeePlayer>();
    app.register_type::<SpawnCoords>();
    app.observe(on_death_reset_enemies);
    app.observe(attach_anim_component);
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
pub struct LdtkEnemyBundle {
    tag: LdtkEnemy,
    #[sprite_sheet_bundle]
    sprite_bundle: LdtkSpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
    #[with(fix_loaded_ldtk_entities)]
    enemy_bundle: EnemyBundle,
}

/// Takes all ldtk enemy entities, and adds all the components we need for them to work in our game.
fn fix_loaded_ldtk_entities(instance: &EntityInstance) -> EnemyBundle {
    EnemyBundle::new(instance)
}

// This is what our game needs to make an enemy work, separate from LDTK
// Keeping the stuff we need to work separate from LDTK lets us instantiate enemies in code, if we want/need to.
#[derive(Bundle, Default, Clone)]
struct EnemyBundle {
    name: Name,
    spawn_coords: SpawnCoords,
    grid_position: GridPosition,
    grid_movement: GridMovement,
    can_damage: CanApplyDamage,
    marker: Enemy,
    vision: VisionBundle,
    rendered_field_of_view: RenderedFieldOfView,
    role: Hunter,
    ai_state: HasAiState,
    patrol_bundle: PatrolBundle,
}

impl EnemyBundle {
    pub fn new(instance: &EntityInstance) -> Self {
        const DEFAULT_WAYPOINT_WAIT_TIME: Duration = Duration::new(5, 0);
        // todo delete this it's for testing - randomize types of enemies
        //let mut rng = rand::thread_rng();
        let is_sniper = false; //rng.gen_ratio(1, 3);
        let vision_archetype = if is_sniper {
            VisionArchetype::Sniper
        } else {
            VisionArchetype::Patrol
        };

        let grid_position =
            GridPosition::new(instance.grid.x as f32, 64.0 - instance.grid.y as f32 - 1.0);

        let mut ai = AiState::Idle;

        let mut patrol_nodes: Vec<PatrolWaypoint> = vec![];
        for field in instance.field_instances.clone() {
            if let FieldValue::Points(points) = field.value {
                if points.is_empty() {
                    ai = AiState::Idle;
                    break;
                }
                for (i, point) in points.iter().enumerate() {
                    let p = point.unwrap();
                    let next_waypoint = if i == points.len() - 1 {
                        points[0]
                    } else {
                        points[i + 1]
                    };
                    let facing = match next_waypoint {
                        None => Facing::default(),
                        Some(next_point) => {
                            let direction = IVec2::new(next_point.x - p.x, p.y - next_point.y);
                            Facing(direction.as_vec2())
                        }
                    };
                    patrol_nodes.push(PatrolWaypoint {
                        position: GridPosition::new(p.x as f32, 64.0 - p.y as f32 - 1.),
                        facing,
                        wait_time: DEFAULT_WAYPOINT_WAIT_TIME,
                    });

                    ai = AiState::Patrolling;
                }
            }
        }
        Self {
            name: Name::new("LdtkEnemy"),
            marker: Enemy,
            can_damage: CanApplyDamage,
            spawn_coords: SpawnCoords(grid_position),
            grid_position,
            grid_movement: GridMovement::default(),
            vision: VisionBundle {
                vision_ability: VisionAbility::of(vision_archetype),
                ..default()
            },
            rendered_field_of_view: RenderedFieldOfView,
            role: Hunter,
            ai_state: HasAiState {
                current_state: ai,
                previous_state: Default::default(),
                can_patrol: !patrol_nodes.is_empty(),
                is_away_from_post: false,
            },
            patrol_bundle: PatrolBundle {
                state: PatrolState {
                    current_waypoint: 0,
                    wait_timer: Timer::new(DEFAULT_WAYPOINT_WAIT_TIME, TimerMode::Once),
                    direction: 1,
                },
                route: PatrolRoute {
                    waypoints: patrol_nodes,
                    mode: PatrolMode::Cycle,
                },
            },
        }
    }
}

#[derive(Event, Debug)]
pub struct SpawnEnemyTrigger;

#[allow(dead_code)]
fn rotate_facing(
    mut query: Query<(&mut Facing, &HasAiState), (With<Enemy>, Without<CanSeePlayer>)>,
    time: Res<Time>,
) {
    const SECONDS_TO_ROTATE: f32 = 10.;
    const RADIANS_PER_SEC: f32 = 2.0 * std::f32::consts::PI / SECONDS_TO_ROTATE;
    for (mut facing, ai) in query.iter_mut() {
        if ai.current_state == AiState::Idle {
            let dt = time.delta_seconds();
            let mut f: Vec2 = facing.0;

            let angle = RADIANS_PER_SEC * dt;
            f = Vec2::new(
                f.x * angle.cos() - f.y * angle.sin(),
                f.x * angle.sin() + f.y * angle.cos(),
            );

            f = f.normalize();

            facing.0 = f;
        }
    }
}

fn detect_player(
    aware_enemies: Query<(Entity, &Transform, &VisibleSquares), (With<Enemy>, With<CanSeePlayer>)>,
    unaware_enemies: Query<(Entity, &VisibleSquares), (With<Enemy>, Without<CanSeePlayer>)>,
    player: Query<(&GridPosition, &Transform), With<Player>>,
    threat_timer: Res<ThreatTimer>,
    threat_settings: Res<ThreatTimerSettings>,
    mut commands: Commands,
) {
    let Ok((player_grid_pos, player_transform)) = player.get_single() else {
        return;
    };

    if threat_timer.current_level >= threat_settings.levels - 1 {
        for (enemy_entity, _) in &unaware_enemies {
            commands.entity(enemy_entity).insert(CanSeePlayer);
        }
        return;
    }

    for (enemy_entity, enemy_transform, enemy_vision) in &aware_enemies {
        if !enemy_vision.contains(player_grid_pos)
            && enemy_transform
                .translation
                .distance(player_transform.translation)
                > ENEMY_CHASE_RANGE
        {
            commands.entity(enemy_entity).remove::<CanSeePlayer>();
            commands.trigger(Sfx::LostPlayer);
            commands.trigger(ShowDialogEvent {
                entity: enemy_entity,
                dialog_type: ShowDialogType::RandomLine(DialogLineType::EnemyLosesPlayer),
            });
        }
    }
    for (enemy_entity, enemy_vision) in &unaware_enemies {
        if enemy_vision.contains(player_grid_pos) {
            commands.entity(enemy_entity).insert(CanSeePlayer);
            commands.trigger(Sfx::Detected);
            commands.trigger(ShowDialogEvent {
                entity: enemy_entity,
                dialog_type: ShowDialogType::RandomLine(DialogLineType::EnemySpotsPlayer),
            });
        }
    }
}

pub const ENEMY_CHASE_SPEED: f32 = 0.5;
pub const ENEMY_PATROL_SPEED: f32 = 0.3;
pub const ENEMY_RETURN_TO_POST_SPEED: f32 = 0.3;
pub const ENEMY_CHASE_RANGE: f32 = 100.0;

fn return_to_post(
    mut unaware_enemies: Query<
        (
            &mut GridMovement,
            &GridPosition,
            &SpawnCoords,
            &mut HasAiState,
        ),
        (With<Enemy>, Without<CanSeePlayer>),
    >,
) {
    for (mut movement, &position, spawn, mut ai) in &mut unaware_enemies {
        if ai.current_state == AiState::ReturningToPost {
            let direction = position.direction_to(&spawn.0);
            if direction.length() < 1.0 {
                movement.acceleration_player_force = Vec2::ZERO;
                ai.current_state = ReturnedToPost;
            } else {
                movement.acceleration_player_force =
                    direction.normalize() * ENEMY_RETURN_TO_POST_SPEED;
            }
        }
    }
}

pub(crate) fn follow_player(
    mut enemy_movement_controllers: Query<
        (&mut GridMovement, &mut Facing, &GridPosition, &HasAiState),
        (With<Enemy>, With<CanSeePlayer>),
    >,
    player: Query<&GridPosition, With<Player>>,
) {
    let Ok(player_pos) = player.get_single() else {
        return;
    };

    for (mut controller, mut facing, enemy_pos, ai) in &mut enemy_movement_controllers {
        if ai.current_state == Chasing {
            let direction = enemy_pos.direction_to(player_pos);
            facing.0 = direction;
            controller.acceleration_player_force = direction.normalize() * ENEMY_CHASE_SPEED;
        }
    }
}

fn on_death_reset_enemies(
    _trigger: Trigger<OnDeath>,
    mut query: Query<(Entity, &mut GridPosition, &SpawnCoords, &mut Facing, Option<&mut PatrolState>), With<Enemy>>,
    mut commands: Commands,
) {
    for (enemy, mut pos, spawn_point, mut facing, mut maybe_patrol) in &mut query {
        *pos = spawn_point.0;
        facing.0 = Vec2::new(1., 0.);
        commands.entity(enemy).remove::<CanSeePlayer>();
        if let Some(mut patrol) = maybe_patrol {
            patrol.current_waypoint = 0;
        }
    }
}

fn attach_anim_component(
    trigger: Trigger<OnAdd, Enemy>,
    images: Res<ImageAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 32, 4, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let mut frames = HashMap::new();

    frames.insert(
        PlayerAnimationState::Idling,
        (0, 3, Duration::from_millis(500)),
    );
    frames.insert(
        PlayerAnimationState::FrontIdling,
        (3, 3, Duration::from_millis(500)),
    );
    frames.insert(
        PlayerAnimationState::FrontWalking,
        (6, 3, Duration::from_millis(100)),
    );
    frames.insert(
        PlayerAnimationState::Walking,
        (12, 3, Duration::from_millis(100)),
    );
    let enemy_animation = PlayerAnimation::new(frames);
    commands.entity(trigger.entity()).insert((
        SpriteBundle {
            texture: images[&ImageAsset::Skeleton].clone_weak(),
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: enemy_animation.get_atlas_index(),
        },
        enemy_animation,
    ));
}
