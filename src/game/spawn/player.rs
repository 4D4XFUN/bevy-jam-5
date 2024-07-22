//! Spawn the player.

use crate::{
    game::{
        animation::PlayerAnimation,
        assets::{ImageAsset, ImageAssets},
        camera::CanBeFollowedByCamera,
        movement::{Movement, MovementController},
        spawn::ldtk::LdtkEntityBundle,
    },
    screen::Screen,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;
use crate::game::grid::collision::GridCollider;
use crate::game::grid::GridPosition;
use crate::game::grid::movement::GridMovement;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_player);
    app.register_type::<Player>();
    app.register_ldtk_entity::<LdtkEntityBundle>("Player");
}

#[derive(Event, Debug)]
pub struct SpawnPlayer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

fn spawn_player(
    _trigger: Trigger<SpawnPlayer>,
    mut commands: Commands,
    images: Res<ImageAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // A texture atlas is a way to split one image with a grid into multiple sprites.
    // By attaching it to a [`SpriteBundle`] and providing an index, we can specify which section of the image we want to see.
    // We will use this to animate our player character. You can learn more about texture atlases in this example:
    // https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 1, 1, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    println!("spawn player");
    let mut player_transform = Transform::from_scale(Vec2::splat(0.5).extend(1.0));
    player_transform.translation.z = 10.; // ensure player goes above level

    commands.spawn((
        Name::new("Player"),
        Player,
        CanBeFollowedByCamera,
        SpriteBundle {
            texture: images[&ImageAsset::Crab].clone_weak(),
            transform: player_transform,
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: player_animation.get_atlas_index(),
        },
        GridPosition::new(45., 24.,),
        GridMovement::default(),
        GridCollider::default(),
        StateScoped(Screen::Playing),
    ));
}
