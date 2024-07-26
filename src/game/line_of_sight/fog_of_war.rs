use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::grid::grid_layout::GridLayout;
use crate::game::grid::GridPosition;
use crate::game::line_of_sight::vision::VisibleSquares;
use crate::game::line_of_sight::CanRevealFog;
use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    //systems
    app.add_systems(
        Update,
        (
            update_grid_fog_of_war_overlay,
            recover_fog_of_war,
            reveal_fog_of_war,
        )
            .chain()
            .in_set(AppSet::UpdateFog),
    );

    // reflection
    app.register_type::<FogOfWarOverlay>();
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct FogOfWarOverlay {
    fog_of_war_grid_sprites: Vec<Entity>,
    width: usize,
    height: usize,
    resolution: f32,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct FogOfWarOverlayVoxel;

impl FogOfWarOverlay {
    pub(crate) fn insert_at(&mut self, x: usize, y: usize, e: Entity) {
        self.fog_of_war_grid_sprites[x + y * self.width] = e;
    }

    pub fn get_at(&self, x: usize, y: usize) -> Entity {
        let index = x + y * self.width;
        self.fog_of_war_grid_sprites[index]
    }
}

impl FogOfWarOverlay {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut fog_of_war_grid_sprites = Vec::new();
        fog_of_war_grid_sprites.resize(size, Entity::PLACEHOLDER);
        Self {
            fog_of_war_grid_sprites,
            width,
            height,
            resolution: 1.0,
        }
    }
}

fn update_grid_fog_of_war_overlay(
    mut commands: Commands,
    grid: Res<GridLayout>,
    existing_overlays: Query<Entity, With<FogOfWarOverlay>>,
) {
    if !grid.is_changed() {
        return;
    }

    for e in existing_overlays.iter() {
        commands.entity(e).despawn_recursive();
    }

    let mut overlay = FogOfWarOverlay::new(grid.width, grid.height);

    let mut child_ids = vec![];
    // Spawn child sprites for each grid cell
    for y in 0..grid.height {
        for x in 0..grid.width {
            let position = grid.grid_to_world(&GridPosition::new(x as f32, y as f32));

            let alpha = 1.0;
            let color = Color::srgba(0.0, 0.0, 0.0, alpha);

            // Spawn the child sprite and parent it to the GridSprite
            let child_id = commands
                .spawn((
                    FogOfWarOverlayVoxel,
                    SpriteBundle {
                        sprite: Sprite {
                            color,
                            custom_size: Some(Vec2::splat(grid.square_size)), // todo resolution
                            ..default()
                        },
                        transform: Transform::from_translation(position.extend(10.0)),
                        ..default()
                    },
                ))
                .id();

            overlay.insert_at(x, y, child_id);
            child_ids.push(child_id);
        }
    }

    let parent_overlay_entity = commands
        .spawn((
            Name::new("FogOfWarOverlay"),
            overlay,
            SpatialBundle::default(),
        ))
        .id();

    for e in child_ids.iter() {
        commands.entity(*e).set_parent(parent_overlay_entity);
    }
}

fn reveal_fog_of_war(
    grid: Res<GridLayout>,
    line_of_sight_query: Query<&VisibleSquares, With<CanRevealFog>>,
    fog_of_war_query: Query<&FogOfWarOverlay>,
    mut fog_of_war_sprite_query: Query<&mut Sprite, With<FogOfWarOverlayVoxel>>,
) {
    let Ok(fog) = fog_of_war_query.get_single() else {
        return;
    };

    for component in line_of_sight_query.iter() {
        let without_neighbors = &component.visible_squares;
        let mut with_neighbors = HashSet::<IVec2>::new();
        for coordinate in without_neighbors.iter() {
            for x in grid
                .neighbors(&GridPosition::new(coordinate.x as f32, coordinate.y as f32))
                .into_iter()
                .map(|v| IVec2::new(v.x as i32, v.y as i32))
            {
                with_neighbors.insert(x);
            }
        }

        // info!("Found {} neighbors of {} squares", with_neighbors.len() - without_neighbors.len(), without_neighbors.len());

        for coordinate in with_neighbors {
            let Ok(mut sprite) = fog_of_war_sprite_query
                .get_mut(fog.get_at(coordinate.x as usize, coordinate.y as usize))
            else {
                warn!("Couldn't find fog sprite at {:?}", coordinate);
                continue;
            };

            let is_neighbor = !without_neighbors.contains(&coordinate);

            if is_neighbor {
                sprite.color.set_alpha(0.5); // neighbors slightly dimmer
            } else {
                sprite.color.set_alpha(0.0);
            }
        }
    }
}

fn recover_fog_of_war(mut fog_of_war_sprite_query: Query<&mut Sprite, With<FogOfWarOverlayVoxel>>) {
    let recovery_alpha_change = 1.0 / 15.0;
    for mut s in fog_of_war_sprite_query.iter_mut() {
        let alpha = s.color.alpha();
        if alpha < 1.0 - recovery_alpha_change {
            // it'll never fully recover
            s.color.set_alpha(alpha + recovery_alpha_change);
        } else {
            s.color.set_alpha(1.0);
        }
    }
}
