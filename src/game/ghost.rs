use std::collections::VecDeque;

use bevy::prelude::*;

use crate::game::line_of_sight::vision::VisionArchetype;
use crate::game::line_of_sight::CanRevealFog;
use crate::game::movement::Roll;
use crate::{
    game::{animation::PlayerAnimation, assets::ImageAsset},
    screen::Screen,
};

use super::{
    animation::PlayerAnimationState,
    assets::ImageAssets,
    end_game::EndGameCondition,
    grid::GridPosition,
    line_of_sight::PlayerLineOfSightBundle,
    spawn::{
        health::{OnDeath, SpawnPointGridPosition},
        player::Player,
    },
};

///Handles ghosts.
///
/// Records ghost data (player movement intent) during FixedUpdate.
/// Replays
pub fn plugin(app: &mut App) {
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.insert_resource(CurrentRecordQueue::new());
    app.insert_resource(GhostQueue {
        ghosts: VecDeque::new(),
        max_ghosts: 30,
    });
    app.add_systems(
        FixedUpdate,
        (record_intent, replay_ghost, animate_ghost, ghost_visibility)
            .run_if(in_state(Screen::Playing)),
    );
    app.observe(on_death_spawn_new_ghost);
    app.observe(on_death_reset_ghosts);
    app.observe(clean_up);
}

fn record_intent(
    mut ghost_records: ResMut<CurrentRecordQueue>,
    query: Query<(&GridPosition, &PlayerAnimation), With<Player>>,
) {
    let Ok((position, animation)) = query.get_single() else {
        return;
    };
    let (coord, offset) = position.get_values();
    ghost_records.0.records.push(GhostRecord {
        coord,
        offset,
        anim_state: animation.get_current_state(),
        is_alive: true,
    });
}

const DEAD_GHOST_FADE_SPEED: f32 = 0.03;
fn ghost_visibility(
    mut query: Query<(Entity, &mut Sprite, &GhostRecordQueue), With<Ghost>>,
    mut commands: Commands,
) {
    for (entity, mut sprite, ghost_record_queue) in query.iter_mut() {
        if ghost_record_queue.current_record > 0
            && ghost_record_queue.records[ghost_record_queue.current_record - 1].is_alive
        {
            sprite.color = sprite.color.with_alpha(GHOST_DEFAULT_ALPHA);
        } else if sprite.color.alpha() > 0.0 {
            sprite.color = sprite
                .color
                .with_alpha(sprite.color.alpha() - DEAD_GHOST_FADE_SPEED);
            commands.entity(entity).remove::<CanRevealFog>();
        }
    }
}

#[derive(Resource)]
struct GhostQueue {
    ghosts: VecDeque<Entity>,
    max_ghosts: usize,
}

#[derive(Resource)]
struct CurrentRecordQueue(GhostRecordQueue);

impl CurrentRecordQueue {
    pub fn new() -> Self {
        Self(GhostRecordQueue {
            records: vec![],
            current_record: 0,
        })
    }
}

#[derive(Component)]
struct Ghost;

#[derive(Component)]
pub struct GhostRecordQueue {
    records: Vec<GhostRecord>,
    current_record: usize,
}

#[derive(Reflect, Debug, Clone, PartialEq)]
struct GhostRecord {
    coord: Vec2,
    offset: Vec2,
    anim_state: PlayerAnimationState,
    is_alive: bool,
}

impl GhostRecord {
    pub fn new() -> Self {
        Self {
            coord: Default::default(),
            offset: Default::default(),
            anim_state: PlayerAnimationState::Idling,
            is_alive: true,
        }
    }
}

const GHOST_DEFAULT_ALPHA: f32 = 0.3;

fn on_death_spawn_new_ghost(
    _trigger: Trigger<OnDeath>,
    mut ghost_queue: ResMut<GhostQueue>,
    mut current_record_queue: ResMut<CurrentRecordQueue>,
    spawn_points: Query<&SpawnPointGridPosition, With<Player>>,
    images: Res<ImageAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let Ok(spawn_point) = spawn_points.get_single() else {
        return;
    };
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 7, 6, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    // if you die rolling, your ghost rolls infinitely, so we reset the last frame to Idling
    if let Some(mut entry) = current_record_queue.0.records.pop() {
        entry.anim_state = PlayerAnimationState::Idling;
        entry.is_alive = false;
        current_record_queue.0.records.push(entry);
    }

    let new_ghost = commands
        .spawn((
            Name::new("Ghost"),
            Ghost,
            SpriteBundle {
                texture: images[&ImageAsset::Player].clone_weak(),
                sprite: Sprite {
                    color: Color::srgba(0.5, 0.5, 0.5, GHOST_DEFAULT_ALPHA),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 2.0),
                ..Default::default()
            },
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: player_animation.get_atlas_index(),
            },
            GridPosition::new(spawn_point.0.x, spawn_point.0.y),
            Roll::default(),
            GhostRecordQueue {
                records: current_record_queue.0.records.clone(),
                current_record: 0,
            },
            PlayerLineOfSightBundle::default().with_vision_archetype(VisionArchetype::Ghost),
            player_animation,
        ))
        .id();

    ghost_queue.ghosts.push_back(new_ghost);
    if ghost_queue.ghosts.len() > ghost_queue.max_ghosts {
        if let Some(old_ghost) = ghost_queue.ghosts.pop_front() {
            commands.entity(old_ghost).despawn_recursive();
        }
    }

    current_record_queue.0.records.clear();
}

fn on_death_reset_ghosts(
    _trigger: Trigger<OnDeath>,
    spawn_points: Query<&SpawnPointGridPosition>,
    mut alive_ghosts: Query<
        (&mut GridPosition, &mut GhostRecordQueue),
        (With<Ghost>, With<CanRevealFog>),
    >,
    mut dead_ghosts: Query<
        (Entity, &mut GridPosition, &mut GhostRecordQueue),
        (With<Ghost>, Without<CanRevealFog>),
    >,
    mut commands: Commands,
) {
    let Ok(spawn_point) = spawn_points.get_single() else {
        return;
    };

    for (mut pos, mut velocities) in &mut alive_ghosts {
        velocities.current_record = 0;
        pos.coordinates.x = spawn_point.0.x;
        pos.coordinates.y = spawn_point.0.y;
    }

    for (ghost, mut pos, mut velocities) in &mut dead_ghosts {
        velocities.current_record = 0;
        pos.coordinates.x = spawn_point.0.x;
        pos.coordinates.y = spawn_point.0.y;
        commands.entity(ghost).insert(CanRevealFog);
    }
}

fn replay_ghost(mut query: Query<(&mut GhostRecordQueue, &mut GridPosition)>) {
    for (mut ghost_record_queue, mut position) in &mut query {
        if ghost_record_queue.current_record < ghost_record_queue.records.len() {
            let ghost_record =
                ghost_record_queue.records[ghost_record_queue.current_record].clone();
            position.set(ghost_record.coord, ghost_record.offset);
            ghost_record_queue.current_record += 1;
        }
    }
}

fn clean_up(
    _trigger: Trigger<EndGameCondition>,
    query: Query<Entity, With<Ghost>>,
    mut commands: Commands,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn animate_ghost(mut query: Query<(&GhostRecordQueue, &mut Sprite, &mut PlayerAnimation)>) {
    for (ghost_record, mut sprite, mut animation) in &mut query {
        if ghost_record.current_record == 0
            || ghost_record.current_record >= ghost_record.records.len()
        {
            continue;
        }

        let current_ghost_record = ghost_record.records[ghost_record.current_record].clone();

        let previous_ghost_record = ghost_record.records[ghost_record.current_record - 1].clone();

        let current = current_ghost_record.coord + current_ghost_record.offset;
        let previous = previous_ghost_record.coord + previous_ghost_record.offset;

        let diff = current - previous;
        sprite.flip_x = diff.x < 0.0;

        animation.update_state(current_ghost_record.anim_state);
    }
}
