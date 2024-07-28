use bevy::prelude::*;

use crate::game::line_of_sight::front_facing_edges::FacingWallsCache;
use crate::game::line_of_sight::vision::{Facing, VisibleSquares, VisionAbility, VisionArchetype};

pub mod fog_of_war;
pub mod vision;

pub mod front_facing_edges;
pub mod vision_cones;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        fog_of_war::plugin,
        front_facing_edges::plugin,
        vision::plugin,
        vision_cones::plugin,
    ));
}

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
