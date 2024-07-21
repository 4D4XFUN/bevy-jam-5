use bevy::prelude::*;
use crate::game::animation::PlayerAnimation;
use crate::game::assets::{ImageAsset, ImageAssets};
use crate::game::movement::{Movement, MovementController, WrapWithinWindow};
use crate::game::spawn::player::{Player, SpawnPlayer};
use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_ai_proving_grounds);
}

#[derive(Event, Debug)]
pub struct SpawnAiProvingGrounds;

fn spawn_ai_proving_grounds(_trigger: Trigger<SpawnAiProvingGrounds>,
                            mut commands: Commands,
                            images: Res<ImageAssets>,
                            mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,

) {

    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();
    commands.spawn((
        Name::new("Robot"),
        SpriteBundle {
            texture: images[&ImageAsset::Ducky].clone_weak(),
            transform: Transform::from_scale(Vec2::splat(8.0).extend(1.0)),
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: player_animation.get_atlas_index(),
        },
        MovementController::default(),
        Movement { speed: 420.0 },
        WrapWithinWindow,
        player_animation,
        StateScoped(Screen::AiProvingGrounds),
    ));
}

