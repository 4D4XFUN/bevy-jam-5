use bevy::prelude::*;

use crate::game::grid::grid_layout::GridLayout;
use crate::game::grid::GridPosition;
use crate::game::line_of_sight::{FacingWallsCache, LineOfSightSource};
use crate::geometry_2d::line_segment::LineSegment;
use crate::AppSet;
use crate::z_layers::ZLayers;

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
                        transform: ZLayers::Fog.transform(),
                        sprite: Sprite {
                            color,
                            custom_size: Some(Vec2::splat(grid.square_size)), // todo resolution
                            ..default()
                        },
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
            SpatialBundle::from_transform(ZLayers::Fog.transform()),
        ))
        .id();

    for e in child_ids.iter() {
        commands.entity(*e).set_parent(parent_overlay_entity);
    }
}

fn reveal_fog_of_war(
    grid: Res<GridLayout>,
    line_of_sight_query: Query<(&GridPosition, &LineOfSightSource, &FacingWallsCache)>,
    fog_of_war_query: Query<&FogOfWarOverlay>,
    mut fog_of_war_sprite_query: Query<&mut Sprite, With<FogOfWarOverlayVoxel>>,
) {
    let Ok(fog) = fog_of_war_query.get_single() else {
        return;
    };

    // for each LOS source, iterate through the nearest fog of war squares and reduce their alpha
    for (position, source, walls) in line_of_sight_query.iter() {
        for x in 0..fog.width {
            for y in 0..fog.height {
                let fog_coords = Vec2::new(x as f32, y as f32);
                let dist = position.coordinates.distance(fog_coords);

                // special case for the square we're standing on
                if dist <= 1.0 {
                    if let Ok(mut s) = fog_of_war_sprite_query.get_mut(fog.get_at(x, y)) {
                        s.color.set_alpha(0.0);
                    }
                    continue;
                }

                // don't look too far
                if dist > source.max_distance_in_grid_units {
                    continue;
                }

                let ray_start = grid.grid_to_world(position);
                let ray_end = grid.grid_to_world(&GridPosition::new(x as f32, y as f32));

                // shorten the ray slightly so we can "see into" walls
                let penetration_factor = 1.0;
                let direction = (ray_end - ray_start).normalize();
                // info!("{} {} {} {} {slope}", ray_start.x, ray_start.y, ray_end.x, ray_end.y);
                let ray_end = ray_end - direction * penetration_factor;

                let ray = LineSegment::new(ray_start, ray_end);

                let can_see = walls.facing_wall_edges.iter().all(|w| !ray.do_intersect(w));

                if can_see {
                    // set surrounding tiles
                    let max_x = usize::clamp(x.saturating_add(1), 0, fog.width - 1);
                    let min_x = usize::clamp(x.saturating_sub(1), 0, fog.width - 1);
                    let max_y = usize::clamp(y.saturating_add(1), 0, fog.height - 1);
                    let min_y = usize::clamp(y.saturating_sub(1), 0, fog.height - 1);

                    // if level_walls.collides(x as i32, y as i32) {
                    //
                    // } else {
                    //     s.color.set_alpha(0.0);
                    // }
                    for x_index in min_x..=max_x {
                        for y_index in min_y..=max_y {
                            let Ok(mut adjacent_sprite) =
                                fog_of_war_sprite_query.get_mut(fog.get_at(x_index, y_index))
                            else {
                                continue;
                            };
                            adjacent_sprite.color.set_alpha(0.0);
                        }
                    }
                }
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
