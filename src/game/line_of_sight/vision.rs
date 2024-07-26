use std::f32::consts;
use bevy::prelude::*;
use crate::game::ai::{Hunter, Prey};
use crate::game::grid::GridPosition;
use crate::game::line_of_sight::FacingWallsCache;
use crate::game::spawn::level::LevelWalls;

pub fn plugin(app: &mut App) {

    // reflect
    app.register_type::<Facing>();
    app.register_type::<VisionAbility>();
}

#[derive(Bundle, Default)]
pub struct VisionBundle {
    pub facing: Facing,
    pub vision_ability: VisionAbility,
}

/// Which direction an enemy is looking
#[derive(Component, Reflect, Debug, Copy, Clone, Default)]
#[reflect(Component)]
pub struct Facing {
    pub angle_radians: f32, // iirc +x is 0
}

#[derive(Component, Reflect, Debug, Copy, Clone)]
#[reflect(Component)]
pub struct VisionAbility {
    pub field_of_view_radians: f32, // angle of cone of vision (total)
    pub range_in_grid_units: f32,
}

impl Default for VisionAbility {
    fn default() -> Self {
        Self::of(VisionArchetype::default())
    }
}

impl VisionAbility {
    pub fn of(archetype: VisionArchetype) -> Self {
        match archetype {
            VisionArchetype::Sniper => VisionAbility {
                field_of_view_radians: consts::FRAC_PI_8,
                range_in_grid_units: 12.0,
            },
            VisionArchetype::Patrol => VisionAbility {
                field_of_view_radians: consts::FRAC_PI_2,
                range_in_grid_units: 6.0,
            },
            VisionArchetype::Ghost => VisionAbility {
                field_of_view_radians: 2. * consts::PI,
                range_in_grid_units: 5.0,
            },
            VisionArchetype::Player => VisionAbility {
                field_of_view_radians: 2. * consts::PI,
                range_in_grid_units: 30.0,
            },
        }
    }
}

// maybe we can figure out a way to encode these in LDTK for easy enemy design
#[derive(Default)]
pub enum VisionArchetype {
    /// Very narrow FOV, Long range, short detection time
    #[default]
    Sniper,

    /// Medium FOV, Medium Range, Medium detection time
    Patrol,

    /// Like the player but less range
    Ghost,

    /// This is the player's FOV
    Player,
}