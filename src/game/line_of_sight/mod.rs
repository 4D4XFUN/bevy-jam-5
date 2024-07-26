use bevy::prelude::*;

use crate::game::line_of_sight::front_facing_edges::FacingWallsCache;
use crate::game::line_of_sight::vision::{Facing, VisionAbility, VisionArchetype};

pub mod fog_of_war;
pub mod vision;

pub mod front_facing_edges;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((front_facing_edges::plugin, fog_of_war::plugin));
    app.add_plugins(vision::plugin);
}

#[derive(Component)]
pub struct CanRevealFog;

#[derive(Bundle)]
pub struct PlayerLineOfSightBundle {
    pub facing_walls_cache: FacingWallsCache,
    pub can_reveal_fog: CanRevealFog,
    pub vision_ability: VisionAbility,
    pub facing: Facing,
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
        }
    }
}

