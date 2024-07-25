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
    app.insert_resource(CurrentVelocities(Vec::new()));
    app.add_systems(
        FixedUpdate,
        (record_intent, replay_ghost).run_if(in_state(Screen::Playing)),
    );
    app.observe(spawn_ghost);
    app.observe(reset_ghosts);
    app.observe(clean_up);
}

#[derive(Resource)]
struct CurrentVelocities(Vec<Vec2>);

#[derive(Component)]
struct Ghost;

fn record_intent(
    mut current_velocities: ResMut<CurrentVelocities>,
    query: Query<&GridMovement, With<Player>>,
) {
    let Ok(movement) = query.get_single() else {
        return;
    };
    current_velocities.0.push(movement.velocity);
}

#[derive(Component)]
struct Velocities {
    velocities: Vec<Vec2>,
    current_velocity: usize,
}

fn spawn_ghost(
    _trigger: Trigger<OnDeath>,
    mut current_velocities: ResMut<CurrentVelocities>,
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

    commands.spawn((
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
        Velocities {
            velocities: current_velocities.0.clone(),
            current_velocity: 0,
        },
        LineOfSightBundle::default(),
        player_animation,
    ));

    current_velocities.0.clear();
}

fn reset_ghosts(
    _trigger: Trigger<OnDeath>,
    spawn_points: Query<&SpawnPointGridPosition>,
    mut query: Query<(&mut GridPosition, &mut Velocities), With<Ghost>>,
) {
    let Ok(spawn_point) = spawn_points.get_single() else {
        return;
    };
    for (mut pos, mut velocities) in &mut query {
        velocities.current_velocity = 0;
        pos.coordinates.x = spawn_point.0.x;
        pos.coordinates.y = spawn_point.0.y;
    }
}

fn replay_ghost(mut query: Query<(&mut Velocities, &mut GridMovement)>) {
    for (mut velocities, mut movement) in &mut query {
        if velocities.current_velocity < velocities.velocities.len() {
            movement.velocity = velocities.velocities[velocities.current_velocity];
            velocities.current_velocity += 1;
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
