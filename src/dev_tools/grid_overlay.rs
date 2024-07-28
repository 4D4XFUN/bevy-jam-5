use bevy::color::Color;
use bevy::core::Name;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;

use crate::dev_tools::DebugOverlaysState;
use crate::game::grid::{GridPosition, GridSprite};
use crate::game::grid::grid_layout::GridLayout;
use crate::game::spawn::player::Player;
use crate::screen::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_grid_debug_overlay, update_player_grid_debug_overlay)
            .run_if(in_state(DebugOverlaysState::Enabled)),
    );
}

#[derive(Component, Reflect, Debug, Copy, Clone, PartialEq)]
#[reflect(Component)]
struct GridOverlay;

fn update_player_grid_debug_overlay(
    mut gizmos: Gizmos,
    grid: Res<GridLayout>,
    query: Query<&GridPosition, (With<Player>,)>,
) {
    for player_pos in query.iter() {
        gizmos.rect_2d(
            grid.grid_to_world(&GridPosition::new(
                player_pos.coordinates.x,
                player_pos.coordinates.y,
            )),
            0.,
            Vec2::splat(15.),
            Color::srgba(1.0, 0.0, 0.0, 1.0),
        );
    }
}

fn update_grid_debug_overlay(
    mut commands: Commands,
    grid: Res<GridLayout>,
    existing_overlays: Query<Entity, With<GridOverlay>>,
) {
    if !grid.is_changed() {
        return;
    }

    // despawn old overlays
    for e in existing_overlays.into_iter() {
        commands.entity(e).despawn_recursive()
    }

    // spawn a new overlay
    let name = format!("GridOverlay_{}x{}", grid.width, grid.height);
    let grid_entity = commands
        .spawn((
            GridOverlay,
            Name::new(name),
            GridSprite,
            SpatialBundle::default(),
            StateScoped(Screen::Playing),
        ))
        .id();

    // Spawn child sprites for each grid cell
    for y in 0..grid.height {
        for x in 0..grid.width {
            let position = grid.grid_to_world(&GridPosition::new(x as f32, y as f32));

            let alpha = 0.1;
            let color = if (x + y) % 2 == 0 {
                Color::srgba(0.9, 0.9, 0.9, alpha)
            } else {
                Color::srgba(0.8, 0.8, 0.8, alpha)
            };

            // Spawn the child sprite and parent it to the GridSprite
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::splat(grid.square_size)),
                        ..default()
                    },
                    transform: Transform::from_translation(position.extend(10.0)),
                    ..default()
                })
                .set_parent(grid_entity);
        }
    }
}
