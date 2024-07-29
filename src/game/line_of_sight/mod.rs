use bevy::prelude::*;
use bevy_ecs_ldtk::GridCoords;

use front_facing_edges::RebuildCache;

use crate::game::line_of_sight::front_facing_edges::FacingWallsCache;
use crate::game::line_of_sight::vision::{Facing, VisibleSquares, VisionAbility, VisionArchetype};

use super::spawn::level::LevelVisionBlockers;

pub mod fog_of_war;
pub mod vision;

pub mod front_facing_edges;
pub mod vision_cones;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((front_facing_edges::plugin, fog_of_war::plugin));
    app.add_plugins(vision::plugin);
    app.add_plugins(vision_cones::plugin);
    app.observe(rebuild_vision_cache_on_remove);
    app.observe(rebuild_vision_cache_on_add);
}

#[derive(Component, Default, Clone)]
pub struct BlocksVision;

#[derive(Component)]
pub struct CanRevealFog;

#[derive(Bundle)]
pub struct PlayerLineOfSightBundle {
    pub facing: Facing,
    pub can_reveal_fog: CanRevealFog,
    pub vision_ability: VisionAbility,
    pub facing_walls_cache: FacingWallsCache,
    pub visible_squares: VisibleSquares,
}

impl PlayerLineOfSightBundle {
    pub fn with_vision_archetype(mut self, archetype: VisionArchetype) -> Self {
        let vision = VisionAbility::of(archetype);
        self.vision_ability = vision;
        self
    }
}

impl Default for PlayerLineOfSightBundle {
    fn default() -> Self {
        Self {
            facing_walls_cache: FacingWallsCache::new(),
            can_reveal_fog: CanRevealFog,
            vision_ability: VisionAbility::default(),
            facing: Facing::default(),
            visible_squares: VisibleSquares::default(),
        }
    }
}

fn rebuild_vision_cache_on_remove(
    trigger: Trigger<OnRemove, BlocksVision>,
    mut vision_blocker: ResMut<LevelVisionBlockers>,
    query: Query<(Entity, &GridCoords)>,
    mut commands: Commands,
) {
    let entity = trigger.entity();
    if let Ok((_, coordinates)) = query.get(entity) {
        vision_blocker.vision_blocker_locations.remove(coordinates);
    }
    commands.trigger(RebuildCache);
}

fn rebuild_vision_cache_on_add(
    trigger: Trigger<OnAdd, BlocksVision>,
    mut vision_blocker: ResMut<LevelVisionBlockers>,
    query: Query<(Entity, &GridCoords)>,
    mut commands: Commands,
) {
    let entity = trigger.entity();
    if let Ok((_, coordinates)) = query.get(entity) {
        vision_blocker.vision_blocker_locations.insert(*coordinates);
    }
    commands.trigger(RebuildCache);
}
