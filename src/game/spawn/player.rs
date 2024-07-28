//! Spawn the player.
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;
use leafwing_input_manager::InputManagerBundle;

use crate::{
    game::{
        animation::PlayerAnimation,
        assets::{ImageAsset, ImageAssets},
        camera::CameraFollowTarget,
        spawn::ldtk::LdtkEntityBundle,
    },
    screen::Screen,
};
use crate::game::grid::GridPosition;
use crate::game::line_of_sight::PlayerLineOfSightBundle;
use crate::game::movement::GridMovement;
use crate::game::movement::RollState;
use crate::game::spawn::health::{CanReceiveDamage, SpawnPointGridPosition};
use crate::input::PlayerAction;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_player);
    app.register_type::<Player>();
    app.register_ldtk_entity::<LdtkEntityBundle>("Player");
}

#[derive(Event, Debug)]
pub struct SpawnPlayerTrigger;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

fn spawn_player(
    _trigger: Trigger<SpawnPlayerTrigger>,
    mut commands: Commands,
    images: Res<ImageAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // A texture atlas is a way to split one image with a grid into multiple sprites.
    // By attaching it to a [`SpriteBundle`] and providing an index, we can specify which section of the image we want to see.
    // We will use this to animate our player character. You can learn more about texture atlases in this example:
    // https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 7, 6, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::default();

    let mut player_transform = Transform::from_scale(Vec2::splat(1.).extend(1.0));
    player_transform.translation.z = 10.; // ensure player goes above level

    commands.spawn((
        Name::new("Player"),
        StateScoped(Screen::Playing),
        Player,
        CameraFollowTarget,
        SpriteBundle {
            texture: images[&ImageAsset::Player].clone_weak(),
            transform: player_transform,
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: player_animation.get_atlas_index(),
        },
        SpawnPointGridPosition(Vec2::new(32., 64. - 33.)),
        CanReceiveDamage,
        GridPosition::new(32., 64. - 33.),
        GridMovement::default(),
        RollState::default(),
        InputManagerBundle::with_map(PlayerAction::default_input_map()),
        player_animation,
        PlayerLineOfSightBundle { ..default() },
    ));
}
