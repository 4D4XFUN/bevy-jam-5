use bevy::app::App;

pub mod fov;

pub fn plugin(_app: &mut App) {
    _app.add_plugins(fov::plugin);
}
