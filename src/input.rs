use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<PlayerAction>::default());

    #[cfg(feature = "dev")]
    app.add_plugins(InputManagerPlugin::<DevAction>::default());
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlayerAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Roll,
    ZoomIn,
    ZoomOut,
    ZoomToOverview,
}
impl PlayerAction {
    /// Define the default bindings to the input
    pub fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        input_map.insert(Self::MoveUp, KeyCode::KeyW);
        input_map.insert(Self::MoveUp, KeyCode::ArrowUp);

        input_map.insert(Self::MoveDown, KeyCode::KeyS);
        input_map.insert(Self::MoveDown, KeyCode::ArrowDown);

        input_map.insert(Self::MoveLeft, KeyCode::KeyA);
        input_map.insert(Self::MoveLeft, KeyCode::ArrowLeft);

        input_map.insert(Self::MoveRight, KeyCode::KeyD);
        input_map.insert(Self::MoveRight, KeyCode::ArrowRight);

        // camera
        input_map.insert(Self::ZoomToOverview, KeyCode::Space);

        // roll
        input_map.insert(Self::Roll, KeyCode::ShiftLeft);

        input_map
    }
}

#[cfg(feature = "dev")]
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum DevAction {
    ToggleWorldInspector,
    ToggleDebugOverlays,
}

#[cfg(feature = "dev")]
impl DevAction {
    pub fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        input_map.insert(Self::ToggleWorldInspector, KeyCode::F1);
        input_map.insert(Self::ToggleDebugOverlays, KeyCode::F3);

        input_map
    }
}
