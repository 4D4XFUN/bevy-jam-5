use bevy::app::{App, Update};
use bevy::core::Name;
use bevy::math::IVec2;
use bevy::prelude::{Bundle, Commands, Component, Entity, Query, Reflect, Trigger, With};
use bevy::utils::HashSet;
use bevy_ecs_ldtk::{GridCoords, LdtkEntity, LdtkSpriteSheetBundle};
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;

use crate::game::grid::GridPosition;
use crate::game::line_of_sight::CanRevealFog;
use crate::game::line_of_sight::vision::VisibleSquares;
use crate::game::spawn::enemy::SpawnCoords;
use crate::game::spawn::health::OnDeath;

pub(super) fn plugin(app: &mut App) {
    // spawning
    app.register_ldtk_entity::<LdtkKeyBundle>("Key");

    // systems
    app.add_systems(Update, fix_loaded_ldtk_entities);
    // reflection
    app.register_type::<Key>();
    app.observe(on_death_reset_keys);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub struct Key;

#[derive(Component, Default, Copy, Clone)]
pub struct LdtkKey;

#[derive(Default, Bundle, LdtkEntity)]
struct LdtkKeyBundle {
    tag: LdtkKey,
    #[sprite_sheet_bundle]
    sprite_bundle: LdtkSpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Bundle)]
struct KeyBundle {
    spawn_coords: SpawnCoords,
    grid_position: GridPosition,
    visible_squares: VisibleSquares,
    can_reveal_fog: CanRevealFog,
}

impl KeyBundle {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            spawn_coords: SpawnCoords(GridPosition::new(x, y)),
            grid_position: GridPosition::new(x, y),
            visible_squares: VisibleSquares::default(),
            can_reveal_fog: CanRevealFog,
        }
    }
}

fn fix_loaded_ldtk_entities(
    mut query: Query<(Entity, &GridCoords, &mut VisibleSquares), With<LdtkKey>>,
    mut commands: Commands,
) {
    for (ldtk_entity, grid_coords, mut visible_squares) in query.iter_mut() {
        let mut hash_set = HashSet::new();
        hash_set.insert(IVec2::new(grid_coords.x, grid_coords.y));
        visible_squares.visible_squares = hash_set;
        commands
            .entity(ldtk_entity)
            .remove::<LdtkKey>() // we have to remove it because it's used as the query for this function
            .insert((
                Name::new("LdtkKey"),
                KeyBundle::new(grid_coords.x as f32, grid_coords.y as f32),
            ));
    }
}

fn on_death_reset_keys(
    _trigger: Trigger<OnDeath>,
    mut query: Query<(&mut GridPosition, &SpawnCoords), With<Key>>,
) {
    for (mut pos, spawn_point) in &mut query {
        *pos = spawn_point.0;
    }
}
