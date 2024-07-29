use bevy::prelude::*;
use rand::seq::SliceRandom;
use std::collections::HashMap;
const GRID_SIZE: f32 = 16.0;
const DIALOG_OFFSET: Vec3 = Vec3::new(0.0, GRID_SIZE, 0.0);

pub fn plugin(app: &mut App) {
    app.observe(handle_show_dialog_event)
        .add_systems(Startup, setup_dialog_system)
        .add_systems(Update, update_dialog);
}

#[derive(Component)]
pub struct Dialog {
    pub _content: String,
    pub time_to_show: Timer,
}

pub struct DialogLines {
    // this will get loaded from a file, or instantiated in-place
    pub lines: Vec<String>,
    pub index: usize,
}

impl DialogLines {
    pub fn from_string(blob: &str) -> Self {
        DialogLines {
            lines: blob.lines().map(String::from).collect(),
            index: 0,
        }
    }

    pub fn from(lines: Vec<String>) -> Self {
        DialogLines { lines, index: 0 }
    }

    /// Picks a line at random from all the lines
    pub fn random_line(&self) -> String {
        self.lines.choose(&mut rand::thread_rng()).unwrap().clone()
    }

    /// Get a specific line, by index
    pub fn line(&self, index: usize) -> String {
        let index = index & self.lines.len();
        self.lines[index].clone()
    }

    /// Gets the next line, advancing the pointer. Once we're out of lines, returns nothing.
    pub fn next_line(&mut self) -> Option<String> {
        let index = self.index;
        if index >= self.lines.len() {
            return None;
        }
        let line = self.lines[index].clone();
        self.index += 1;
        Some(line)
    }
}

#[allow(dead_code)]
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum DialogLineType {
    PlayerSpawn,
    EnemySpotsPlayer,
    EnemyLosesPlayer,
    PlayerFindsKey,
    PlayerUnlocksDoor,
}

/// Stores all the line sets for various characters or situations that arise
#[derive(Resource)]
pub struct DialogLineResource {
    store: HashMap<DialogLineType, DialogLines>,
}

// Events
#[derive(Event)]
pub struct ShowDialogEvent {
    pub entity: Entity,
    pub dialog_type: ShowDialogType,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ShowDialogType {
    /// Just show any arbitrary string
    Custom(String, f32),

    /// Show the next line in the list, going in order. Stops showing once we reach the end.
    /// we can use this for tutorialization or longer "cutscenes"
    NextLine(DialogLineType),

    /// Just pick a line at random from the list and show it
    RandomLine(DialogLineType),

    /// Show a specific line from the pool by index
    SpecificLine(DialogLineType, usize),
}

fn setup_dialog_system(mut commands: Commands) {
    let mut dialog_line_resource = DialogLineResource {
        store: HashMap::new(),
    };

    // Here's an example of how to add dialog lines from a rust vec.
    dialog_line_resource.store.insert(
        DialogLineType::PlayerFindsKey,
        DialogLines::from(vec![
            "Now to find a door!".to_owned(),
            "Another one?".to_owned(),
        ]),
    );
    dialog_line_resource.store.insert(
        DialogLineType::PlayerUnlocksDoor,
        DialogLines::from(vec![
            "Of course it wasn't that easy..".to_owned(),
            "This place is huge!".to_owned(),
        ]),
    );

    // Another way of adding lines, this time from a file
    // (it's statically compiled into the binary so no different than a string, just easier to write out a bunch of lines)
    dialog_line_resource.store.insert(
        DialogLineType::PlayerSpawn,
        DialogLines::from_string(include_str!("lines/PlayerSpawn.txt")),
    );
    dialog_line_resource.store.insert(
        DialogLineType::PlayerFindsKey,
        DialogLines::from_string(include_str!("lines/PlayerFindsKey.txt")),
    );
    dialog_line_resource.store.insert(
        DialogLineType::PlayerUnlocksDoor,
        DialogLines::from_string(include_str!("lines/PlayerUnlocksDoor.txt")),
    );
    dialog_line_resource.store.insert(
        DialogLineType::EnemySpotsPlayer,
        DialogLines::from_string(include_str!("lines/EnemySpotsPlayer.txt")),
    );
    dialog_line_resource.store.insert(
        DialogLineType::EnemyLosesPlayer,
        DialogLines::from_string(include_str!("lines/EnemyLosesPlayer.txt")),
    );

    commands.insert_resource(dialog_line_resource);
}

fn handle_show_dialog_event(
    trigger: Trigger<ShowDialogEvent>,
    mut commands: Commands,
    mut dialog_line_resource: ResMut<DialogLineResource>,
    dialog_query: Query<(Entity, &Parent), With<Dialog>>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/m3x6.ttf");

    // fix for blurry font (since we're zooming way in on a tiny pixel font)
    // Render at larger scale, then scale down using a transform by that same amount.
    const ANTI_BLURINESS_MULTIPLIER: f32 = 4.;
    // Finally, apply a multiplier to scale further, 1.0 is our grid size
    const GRID_SIZE_FONT_SCALE_FRACTION: f32 = 0.5;

    let text_style = TextStyle {
        font,
        font_size: GRID_SIZE * ANTI_BLURINESS_MULTIPLIER,
        color: Color::WHITE,
    };

    let event = trigger.event();

    let default_duration = 3.0;
    let maybe_line: Option<(String, f32)> = match &event.dialog_type {
        ShowDialogType::Custom(text, duration) => Some((text.clone(), *duration)),
        ShowDialogType::NextLine(dialog_type) => {
            let store = &mut dialog_line_resource.store;
            if let Some(dialog_lines) = store.get_mut(dialog_type) {
                dialog_lines.next_line().map(|l| (l, default_duration))
            } else {
                info!("Can't find dialog lines for {:?}", dialog_type);
                None
            }
        }
        ShowDialogType::RandomLine(dialog_type) => {
            if let Some(dialog_lines) = dialog_line_resource.store.get(dialog_type) {
                Some((dialog_lines.random_line(), default_duration))
            } else {
                info!("Can't find dialog lines for {:?}", dialog_type);
                return;
            }
        }
        ShowDialogType::SpecificLine(dialog_type, index) => {
            if let Some(dialog_lines) = dialog_line_resource.store.get(dialog_type) {
                Some((dialog_lines.line(*index), default_duration))
            } else {
                info!("Can't find dialog lines for {:?}", dialog_type);
                return;
            }
        }
    };

    let Some((content, duration)) = maybe_line else {
        return;
    };

    // Remove existing dialog before adding more
    for (dialog_entity, parent) in dialog_query.iter() {
        if parent.get() == event.entity {
            commands.entity(dialog_entity).despawn();
        }
    }

    commands.entity(event.entity).with_children(|parent| {
        parent.spawn((
            Text2dBundle {
                text: Text::from_section(&content, text_style.clone()),
                transform: Transform::from_translation(DIALOG_OFFSET).with_scale(
                    Vec2::splat(1. / ANTI_BLURINESS_MULTIPLIER * GRID_SIZE_FONT_SCALE_FRACTION)
                        .extend(1.0),
                ),
                ..default()
            },
            Dialog {
                _content: content,
                time_to_show: Timer::from_seconds(duration, TimerMode::Once),
            },
        ));
    });
}

fn update_dialog(
    mut commands: Commands,
    time: Res<Time>,
    mut dialogs: Query<(Entity, &mut Dialog, &Parent)>,
) {
    for (entity, mut dialog, parent) in dialogs.iter_mut() {
        dialog.time_to_show.tick(time.delta());
        if dialog.time_to_show.finished() {
            commands.entity(parent.get()).remove_children(&[entity]);
            commands.entity(entity).despawn();
        }
    }
}
