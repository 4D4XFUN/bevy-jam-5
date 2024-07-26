use std::collections::VecDeque;

use bevy::prelude::*;

use crate::game::movement::Roll;
use crate::{
    game::{animation::PlayerAnimation, assets::ImageAsset},
    screen::Screen,
};
use crate::game::line_of_sight::vision::VisionArchetype;
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
    app.insert_resource(Time::<Fixed>::from_hz(30.0));
    app.insert_resource(CurrentRecords(Vec::new()));
    app.insert_resource(GhostQueue {
        ghosts: VecDeque::new(),
        max_ghosts: 30,
    });
    app.add_systems(
        FixedUpdate,
        (record_intent, replay_ghost, animate_ghost).run_if(in_state(Screen::Playing)),
    );
    app.observe(spawn_ghost);
    app.observe(reset_ghosts);
    app.observe(clean_up);
}

#[derive(Resource)]
struct GhostQueue {
    ghosts: VecDeque<Entity>,
    max_ghosts: usize,
}

#[derive(Resource)]
struct CurrentRecords(Vec<(Vec2, Vec2, PlayerAnimationState)>);

#[derive(Component)]
struct Ghost;

fn record_intent(
    mut current_velocities: ResMut<CurrentRecords>,
    query: Query<(&GridPosition, &PlayerAnimation), With<Player>>,
) {
    let Ok((position, animation)) = query.get_single() else {
        return;
    };
    let (coordinates, offset) = position.get_values();
    current_velocities
        .0
        .push((coordinates, offset, animation.get_current_state()));
}

#[derive(Component)]
struct PositionRecord {
    positions: Vec<(Vec2, Vec2, PlayerAnimationState)>,
    current_record: usize,
}

fn spawn_ghost(
    _trigger: Trigger<OnDeath>,
    mut ghost_queue: ResMut<GhostQueue>,
    mut current_records: ResMut<CurrentRecords>,
    spawn_points: Query<&SpawnPointGridPosition>,
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
    if let Some(mut entry) = current_records.0.pop() {
        entry.2 = PlayerAnimationState::Idling;
        current_records.0.push(entry);
    }

    let new_ghost = commands
        .spawn((
            Name::new("Ghost"),
            Ghost,
            SpriteBundle {
                texture: images[&ImageAsset::Player].clone_weak(),
                sprite: Sprite {
                    color: Color::srgb(0.5, 0.5, 0.5),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..Default::default()
            },
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: player_animation.get_atlas_index(),
            },
            GridPosition::new(spawn_point.0.x, spawn_point.0.y),
            Roll::default(),
            PositionRecord {
                positions: current_records.0.clone(),
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

    current_records.0.clear();
}

fn reset_ghosts(
    _trigger: Trigger<OnDeath>,
    spawn_points: Query<&SpawnPointGridPosition>,
    mut query: Query<(&mut GridPosition, &mut PositionRecord), With<Ghost>>,
) {
    let Ok(spawn_point) = spawn_points.get_single() else {
        return;
    };
    for (mut pos, mut velocities) in &mut query {
        velocities.current_record = 0;
        pos.coordinates.x = spawn_point.0.x;
        pos.coordinates.y = spawn_point.0.y;
    }
}

fn replay_ghost(mut query: Query<(&mut PositionRecord, &mut GridPosition)>) {
    for (mut position_record, mut position) in &mut query {
        if position_record.current_record < position_record.positions.len() {
            let (coordinates, offset, _) =
                position_record.positions[position_record.current_record];
            position.set(coordinates, offset);
            position_record.current_record += 1;
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

fn animate_ghost(mut query: Query<(&PositionRecord, &mut Sprite, &mut PlayerAnimation)>) {
    for (position_record, mut sprite, mut animation) in &mut query {
        if position_record.current_record == 0
            || position_record.current_record >= position_record.positions.len()
        {
            continue;
        }

        let (current_pos, current_offset, animation_state) =
            position_record.positions[position_record.current_record];

        let (previous_pos, previous_offset, _) =
            position_record.positions[position_record.current_record - 1];

        let current = current_pos + current_offset;
        let previous = previous_pos + previous_offset;

        let diff = current - previous;
        sprite.flip_x = diff.x < 0.0;

        animation.update_state(animation_state);
    }
}
