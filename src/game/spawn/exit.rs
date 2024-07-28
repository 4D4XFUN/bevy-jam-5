use bevy::{prelude::*, render::primitives::Aabb};

use crate::{
    game::{
        assets::{ImageAsset, ImageAssets},
        end_game::EndGameCondition,
        grid::GridPosition,
    },
    screen::Screen,
};

use super::{
    keys::{CanPickup, Key},
    player::Player,
};

pub fn plugin(app: &mut App) {
    app.observe(spawn_exit);

    app.add_systems(Update, check_exit);
}

#[derive(Event)]
pub struct SpawnExitTrigger;

#[derive(Component)]
struct Exit;

#[derive(Component)]
pub struct CanBeUnlocked;

const LADDER_INDEX: usize = 6 + 9 * 23;

fn spawn_exit(
    _trigger: Trigger<SpawnExitTrigger>,
    images: Res<ImageAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 23, 21, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    commands.spawn((
        Name::new("Exit"),
        StateScoped(Screen::Playing),
        Exit,
        SpriteBundle {
            texture: images[&ImageAsset::Decoration].clone_weak(),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: LADDER_INDEX,
        },
        GridPosition::new(20., 52.),
        CanBeUnlocked,
        // GridCollider::default(),
    ));
}

fn check_exit(
    exits: Query<(&Transform, &Aabb), With<Exit>>,
    players: Query<(&Transform, &Aabb), (With<Player>, (With<Key>, Without<CanPickup>))>,
    mut commands: Commands,
) {
    let Ok((exit_transform, exit)) = exits.get_single() else {
        return;
    };

    let Ok((player_transform, player)) = players.get_single() else {
        return;
    };

    let exit_min =
        Vec3::from(exit.center) - Vec3::from(exit.half_extents) + exit_transform.translation;
    let exit_max =
        Vec3::from(exit.center) + Vec3::from(exit.half_extents) + exit_transform.translation;

    let player_min =
        Vec3::from(player.center) - Vec3::from(player.half_extents) + player_transform.translation;
    let player_max =
        Vec3::from(player.center) + Vec3::from(player.half_extents) + player_transform.translation;

    let x_min = player_min.x >= exit_min.x && player_min.x <= exit_max.x;
    let x_max = player_max.x >= exit_min.x && player_max.x <= exit_max.x;

    let y_min = player_min.y >= exit_min.y && player_min.y <= exit_max.y;
    let y_max = player_max.y >= exit_min.y && player_max.y <= exit_max.y;

    if (x_min || x_max) && (y_min || y_max) {
        commands.trigger(EndGameCondition::Win);
    }
}
