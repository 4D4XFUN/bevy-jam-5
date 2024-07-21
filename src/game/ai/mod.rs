use bevy::app::App;

pub mod spawn_ai_proving_grounds;
pub mod fov;

pub fn plugin(app: &mut App) {
}

pub fn proving_grounds_plugin(app: &mut App) {
    app.add_plugins(spawn_ai_proving_grounds::plugin);
    app.add_plugins(crate::screen::ai_proving_grounds::plugin);
    app.add_plugins(fov::plugin);
}