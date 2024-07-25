use std::collections::VecDeque;

use bevy::prelude::*;

use super::{
    assets::ImageAssets,
    end_game::EndGameCondition,
    grid::GridPosition,
    line_of_sight::LineOfSightBundle,
    spawn::{
        health::{OnDeath, SpawnPointGridPosition},
        player::Player,
    },
};
use crate::game::movement::{GridMovement, Roll};
use crate::{
    game::{animation::PlayerAnimation, assets::ImageAsset},
    screen::Screen,
};

///Handles ghosts.
///
/// Records ghost data (player movement intent) during FixedUpdate.
/// Replays
pub fn plugin(app: &mut App) {
    app.insert_resource(Time::<Fixed>::from_hz(30.0));
    app.insert_resource(CurrentGridPosition(Vec::new()));
    app.insert_resource(GhostQueue {
        ghosts: VecDeque::new(),
        max_ghosts: 3,
    });
    app.add_systems(
        FixedUpdate,
        (record_intent, replay_ghost).run_if(in_state(Screen::Playing)),
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
struct CurrentGridPosition(Vec<(Vec2, Vec2)>);

#[derive(Component)]
struct Ghost;

fn record_intent(
    mut current_velocities: ResMut<CurrentGridPosition>,
    query: Query<&GridPosition, With<Player>>,
) {
    let Ok(position) = query.get_single() else {
        return;
    };
    current_velocities.0.push(position.get_values());
}

#[derive(Component)]
struct PositionRecord {
    positions: Vec<(Vec2, Vec2)>,
    current_record: usize,
}

fn spawn_ghost(
    _trigger: Trigger<OnDeath>,
    mut ghost_queue: ResMut<GhostQueue>,
    mut current_velocities: ResMut<CurrentGridPosition>,
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

    let new_ghost = commands.spawn((
        Name::new("Ghost"),
        Ghost,
        SpriteBundle {
            texture: images[&ImageAsset::Player].clone_weak(),
            sprite: Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                ..default()
            },
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: player_animation.get_atlas_index(),
        },
        GridPosition::new(spawn_point.0.x, spawn_point.0.y),
        GridMovement::default(),
        Roll::default(),
        PositionRecord {
            positions: current_velocities.0.clone(),
            current_record: 0,
        },
        LineOfSightBundle::default(),
        player_animation,
    )).id();

    ghost_queue.ghosts.push_back(new_ghost);
    if ghost_queue.ghosts.len() > ghost_queue.max_ghosts {
        if let Some(old_ghost) = ghost_queue.ghosts.pop_front() {
            commands.entity(old_ghost).despawn_recursive();
        }
    }

    current_velocities.0.clear();
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
            let (coordinates, offset) = position_record.positions[position_record.current_record];
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
