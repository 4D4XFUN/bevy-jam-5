use bevy::{prelude::*, render::primitives::Aabb};
use bevy_ecs_ldtk::{EntityInstance, GridCoords, LdtkEntity, LdtkSpriteSheetBundle};
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;

use crate::game::{grid::GridPosition, utilities::intersect};
use crate::game::line_of_sight::BlocksVision;
use crate::game::spawn::keys::{CanPickup, Key};
use crate::game::spawn::level::BlocksMovement;

use super::player::Player;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, open_locked_doors);
    app.register_ldtk_entity::<LdtkLockedDoorBundle>("DoorLocked");
    app.init_resource::<NumKeysPickedUp>();

    app.register_type::<NumKeysPickedUp>();
}

#[derive(Event)]
pub struct SpawnExitTrigger;

#[derive(Component, Default, Clone, Copy)]
struct Exit;

#[derive(Component, Default, Clone, Copy)]
pub struct CanBeUnlocked;

#[derive(Component, Default, Copy, Clone)]
pub struct LockedDoor;

#[derive(Default, Bundle, LdtkEntity)]
struct LdtkLockedDoorBundle {
    tag: crate::game::spawn::bars::LdtkBars,
    #[sprite_sheet_bundle]
    sprite_bundle: LdtkSpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
    #[with(fix_loaded_ldtk_entities)]
    locked_door_bundle: LockedDoorBundle,
}

/// Takes all ldtk enemy entities, and adds all the components we need for them to work in our game.
fn fix_loaded_ldtk_entities(instance: &EntityInstance) -> LockedDoorBundle {
    LockedDoorBundle::new(instance)
}

// This is what our game needs to make an enemy work, separate from LDTK
// Keeping the stuff we need to work separate from LDTK lets us instantiate enemies in code, if we want/need to.
#[derive(Bundle, Default, Clone)]
struct LockedDoorBundle {
    name: Name,
    grid_position: GridPosition,
    exit: Exit,
    wall: BlocksMovement,
    vision: BlocksVision,
    can_be_unlocked: CanBeUnlocked,
}

impl LockedDoorBundle {
    pub fn new(instance: &EntityInstance) -> Self {
        let grid_position =
            GridPosition::new(instance.grid.x as f32, 64.0 - instance.grid.y as f32 - 1.0);

        Self {
            name: Name::new("Exit"),
            grid_position,
            exit: Default::default(),
            wall: Default::default(),
            vision: Default::default(),
            can_be_unlocked: Default::default(),
        }
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct NumKeysPickedUp(pub i32);

fn open_locked_doors(
    mut picked_up_keys: ResMut<NumKeysPickedUp>,
    player_query: Query<(&Transform, &Aabb), With<Player>>,
    mut door_query: Query<
        (Entity, &mut Visibility, &Transform, &Aabb),
        (
            With<CanBeUnlocked>,
            With<BlocksMovement>,
            With<BlocksVision>,
        ),
    >,
    key: Query<Entity, (With<Key>, Without<CanPickup>)>,
    mut commands: Commands,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };

    if picked_up_keys.0 > 0 {
        for (entity, mut visibility, transform, aabb) in &mut door_query {
            if intersect(player, (transform, aabb)) {
                commands.entity(entity).remove::<BlocksMovement>();
                commands.entity(entity).remove::<BlocksVision>();
                *visibility = Visibility::Hidden;
                picked_up_keys.0 -= 1;
                if let Ok(key) = key.get_single() {
                    commands.entity(key).despawn();
                }
            }
        }
    }
}

const LADDER_INDEX: usize = 4 * 12;

// fn spawn_exit(
//     _trigger: Trigger<SpawnExitTrigger>,
//     images: Res<ImageAssets>,
//     mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
//     mut commands: Commands,
// ) {
//     let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 12, 5, None, None);
//     let texture_atlas_layout = texture_atlas_layouts.add(layout);
//     commands.spawn((
//         Name::new("Exit"),
//         StateScoped(Screen::Playing),
//         Exit,
//         SpriteBundle {
//             texture: images[&ImageAsset::Decoration].clone_weak(),
//             transform: Transform::from_xyz(0.0, 0.0, 2.0),
//             ..Default::default()
//         },
//         TextureAtlas {
//             layout: texture_atlas_layout.clone(),
//             index: LADDER_INDEX,
//         },
//         GridPosition::new(59., 31.),
//     ));
// }

// fn lose_unlockability_on_death(
//     mut commands: Commands,
//     q_keys: Query<Entity, (With<Key>, With<CanPickup>)>,
//     mut q_exit: Query<Entity, (With<Exit>, With<CanBeUnlocked>)>,
// ) {
//     if q_keys.iter().count() == 0 {
//         for exit_entity in q_exit.iter_mut() {
//             commands.entity(exit_entity).remove::<CanBeUnlocked>();
//         }
//     }
// }

// fn gain_unlockability_on_pickup(
//     mut commands: Commands,
//     q_keys: Query<Entity, (With<Key>, Without<CanPickup>)>,
//     mut q_exit: Query<Entity, (With<Exit>, Without<CanBeUnlocked>)>,
// ) {
//     for _ in q_keys.iter() {
//         for exit_entity in q_exit.iter_mut() {
//             commands.entity(exit_entity).insert(CanBeUnlocked);
//         }
//     }
// }

// fn check_exit(
//     exits: Query<(&Transform, &Aabb), (With<Exit>, With<CanBeUnlocked>)>,
//     players: Query<(&Transform, &Aabb), With<Player>>,
//     mut commands: Commands,
// ) {
//     let Ok(exit) = exits.get_single() else {
//         return;
//     };
//
//     let Ok(player) = players.get_single() else {
//         return;
//     };
//
//     if intersect(exit, player) {
//         commands.trigger(EndGameCondition::Win);
//     }
// }
