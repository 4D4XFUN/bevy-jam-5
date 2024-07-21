use bevy::prelude::*;

use crate::game::animation::PlayerAnimation;
use crate::game::assets::{ImageAsset, ImageAssets};
use crate::game::movement::{Movement, MovementController, WrapWithinWindow};
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
    let scale = 4.0;
    commands.spawn((
        Name::new("Player"),
        SpriteBundle {
            texture: images[&ImageAsset::Crab].clone_weak(),
            transform: Transform::from_scale(Vec2::splat(scale).extend(1.0)),
            ..Default::default()
        },
        MovementController::default(),
        Movement { speed: 420.0 },
        WrapWithinWindow,
        StateScoped(Screen::AiProvingGrounds),
    ));

    let mut t = Transform::from_scale(Vec2::splat(scale).extend(1.0));
    t.translation = Vec2::splat(100.).extend(1.);
    commands.spawn((
        Name::new("Robot"),
        SpriteBundle {
            texture: images[&ImageAsset::RoboCrab].clone_weak(),
            transform: t,
            ..Default::default()
        },
        StateScoped(Screen::AiProvingGrounds),
    ));


}

