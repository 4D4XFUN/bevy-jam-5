use bevy::{
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    utils::HashMap,
};

#[derive(PartialEq, Eq, Hash, Reflect)]
pub enum ImageAsset {
    Ducky,
    Crab,
    RoboCrab,
    Player,
    Gargoyle,
    Decoration,
}

#[derive(Resource, Reflect, Deref, DerefMut)]
pub struct ImageAssets(HashMap<ImageAsset, Handle<Image>>);

impl ImageAssets {
    pub fn new(asset_server: &AssetServer) -> Self {
        let mut assets = HashMap::new();

        for (path, image_asset_tag) in [
            ("images/ducky.png", ImageAsset::Ducky),
            ("images/crab.png", ImageAsset::Crab),
            ("images/robo_crab.png", ImageAsset::RoboCrab),
            ("images/character_animated.png", ImageAsset::Player),
            ("images/gargoyle.png", ImageAsset::Gargoyle),
            ("atlas/Dungeon_Tileset_v2.png", ImageAsset::Decoration),
        ] {
            assets.insert(
                image_asset_tag,
                asset_server.load_with_settings(path, |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                }),
            );
        }

        info!("Done loading assets");

        Self(assets)
    }

    pub fn all_loaded(&self, assets: &Assets<Image>) -> bool {
        self.0.iter().all(|(_, handle)| assets.contains(handle))
    }
}

#[derive(PartialEq, Eq, Hash, Reflect)]
pub enum SfxAsset {
    ButtonHover,
    ButtonPress,
    Step1,
    Step2,
    Step3,
    Step4,
    Roll,
    Death,
    Detected,
}

#[derive(Resource, Reflect, Deref, DerefMut)]
pub struct SfxAssets(HashMap<SfxAsset, Handle<AudioSource>>);

impl SfxAssets {
    pub fn new(asset_server: &AssetServer) -> Self {
        let mut assets = HashMap::new();

        assets.insert(
            SfxAsset::ButtonHover,
            asset_server.load("audio/sfx/button_hover.ogg"),
        );
        assets.insert(
            SfxAsset::ButtonPress,
            asset_server.load("audio/sfx/button_press.ogg"),
        );
        assets.insert(SfxAsset::Step1, asset_server.load("audio/sfx/step1.ogg"));
        assets.insert(SfxAsset::Step2, asset_server.load("audio/sfx/step2.ogg"));
        assets.insert(SfxAsset::Step3, asset_server.load("audio/sfx/step3.ogg"));
        assets.insert(SfxAsset::Step4, asset_server.load("audio/sfx/step4.ogg"));

        assets.insert(SfxAsset::Roll, asset_server.load("audio/sfx/roll.ogg"));
        assets.insert(SfxAsset::Death, asset_server.load("audio/sfx/death.ogg"));
        assets.insert(SfxAsset::Detected, asset_server.load("audio/sfx/enemy_detect.ogg"));

        Self(assets)
    }

    pub fn all_loaded(&self, assets: &Assets<AudioSource>) -> bool {
        self.0.iter().all(|(_, handle)| assets.contains(handle))
    }
}

#[derive(PartialEq, Eq, Hash, Reflect)]
pub enum SoundtrackAsset {
    Credits,
    Gameplay,
}

#[derive(Resource, Reflect, Deref, DerefMut)]
pub struct SoundtrackAssets(HashMap<SoundtrackAsset, Handle<AudioSource>>);

impl SoundtrackAssets {
    pub fn new(asset_server: &AssetServer) -> Self {
        let mut assets = HashMap::new();
        assets.insert(
            SoundtrackAsset::Credits,
            asset_server.load("audio/soundtracks/Monkeys Spinning Monkeys.ogg"),
        );
        assets.insert(
            SoundtrackAsset::Gameplay,
            asset_server.load("audio/soundtracks/Fluffing A Duck.ogg"),
        );
        Self(assets)
    }

    pub fn all_loaded(&self, assets: &Assets<AudioSource>) -> bool {
        self.0.iter().all(|(_, handle)| assets.contains(handle))
    }
}
