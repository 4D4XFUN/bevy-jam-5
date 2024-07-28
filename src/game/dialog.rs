use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
struct Dialog {
    content: String,
    time_to_show: Timer,
}

struct DialogLines {
    // this will get loaded from a file, or instantiated in-place
    lines: Vec<String>,
}

impl DialogLines {
    pub fn from_string(blob: String) -> Self {
        todo!("load a list of lines from a file using the rust include! macro to build them into the binary rather than loading at runtime")
    }

    pub fn from(lines: Vec<String>) -> Self {
        todo!("create an instance of dialog lines from a rust vec of strings")
    }

    pub fn random_line(&self) -> Dialog {
        todo!("return a random line from this resource's line collection")
    }

    pub fn next_line(&self, index: usize) -> Dialog {
        todo!("return the next line in order from this dialog's line collection")
    }
}

enum DialogLineType {
    PlayerSpawn,
    EnemySpotsPlayer,
    EnemyLosesPlayer,
    PlayerFindsKey,
    PlayerUnlocksDoor,
}

/// Stores all the line sets for various characters or situations that arise
#[derive(Resource)]
struct DialogLineResource {
    store: HashMap<DialogLineType, DialogLines>,
}
