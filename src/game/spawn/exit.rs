use bevy::{prelude::*, render::primitives::Aabb};
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;
use bevy_ecs_ldtk::{EntityInstance, GridCoords, LdtkEntity, LdtkSpriteSheetBundle};

use super::player::Player;
use crate::game::dialog::{DialogLineType, ShowDialogEvent, ShowDialogType};
use crate::game::line_of_sight::BlocksVision;
use crate::game::spawn::keys::{CanPickup, Key};
use crate::game::spawn::level::BlocksMovement;
use crate::game::{grid::GridPosition, utilities::intersect};

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
    player_query: Query<(Entity, &Transform, &Aabb), With<Player>>,
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
    let Ok((player_ent, player_transform, player_aabb)) = player_query.get_single() else {
        return;
    };

    if picked_up_keys.0 > 0 {
        for (entity, mut visibility, transform, aabb) in &mut door_query {
            if intersect((player_transform, player_aabb), (transform, aabb)) {
                commands.entity(entity).remove::<BlocksMovement>();
                commands.entity(entity).remove::<BlocksVision>();
                *visibility = Visibility::Hidden;
                picked_up_keys.0 -= 1;

                commands.trigger(ShowDialogEvent {
                    entity: player_ent,
                    dialog_type: ShowDialogType::NextLine(DialogLineType::PlayerUnlocksDoor),
                });

                let keys: Vec<_> = key.iter().collect();
                if let Some(key) = keys.first() {
                    info!("Despawning key {:?}", &key);
                    commands.entity(*key).despawn();
                }
            }
        }
    }
}
