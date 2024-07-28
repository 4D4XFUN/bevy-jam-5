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
    pub content: String,
    pub time_to_show: Timer,
}

pub struct DialogLines {
    // this will get loaded from a file, or instantiated in-place
    pub lines: Vec<String>,
}

impl DialogLines {
    pub fn from_string(blob: String) -> Self {
        DialogLines {
            lines: blob.lines().map(String::from).collect(),
        }
    }

    pub fn from(lines: Vec<String>) -> Self {
        DialogLines { lines }
    }

    pub fn random_line(&self) -> String {
        self.lines.choose(&mut rand::thread_rng()).unwrap().clone()
    }

    pub fn next_line(&self, index: usize) -> String {
        self.lines[index % self.lines.len()].clone()
    }
}

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

#[derive(Debug)]
pub enum ShowDialogType {
    Custom(String, f32),
    NextLine(DialogLineType, usize),
    RandomLine(DialogLineType),
}

fn setup_dialog_system(mut commands: Commands) {
    let mut dialog_line_resource = DialogLineResource {
        store: HashMap::new(),
    };

    // Add some example dialog lines
    dialog_line_resource.store.insert(
        DialogLineType::PlayerSpawn,
        DialogLines::from(vec![
            "Where am I?".to_string(),
            "A new adventure begins!".to_string(),
        ]),
    );
    dialog_line_resource.store.insert(
        DialogLineType::EnemySpotsPlayer,
        DialogLines::from(vec!["Intruder alert!".to_string(), "Get them!".to_string()]),
    );

    commands.insert_resource(dialog_line_resource);
}

fn handle_show_dialog_event(
    mut trigger: Trigger<ShowDialogEvent>,
    mut commands: Commands,
    dialog_line_resource: Res<DialogLineResource>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/m3x6.ttf");

    let ANTI_BLURINESS_MULTIPLIER = 4.;
    let GRID_SIZE_FONT_SCALE_FRACTION = 0.5;

    let text_style = TextStyle {
        font,
        font_size: GRID_SIZE * ANTI_BLURINESS_MULTIPLIER,
        color: Color::WHITE,
    };

    let event = trigger.event();

    let (content, duration) = match &event.dialog_type {
        ShowDialogType::Custom(text, duration) => (text.clone(), *duration),
        ShowDialogType::NextLine(dialog_type, index) => {
            if let Some(dialog_lines) = dialog_line_resource.store.get(dialog_type) {
                (dialog_lines.next_line(*index), 3.0)
            } else {
                info!("Can't find dialog lines for {:?}", dialog_type);
                return;
            }
        }
        ShowDialogType::RandomLine(dialog_type) => {
            if let Some(dialog_lines) = dialog_line_resource.store.get(dialog_type) {
                (dialog_lines.random_line(), 3.0)
            } else {
                info!("Can't find dialog lines for {:?}", dialog_type);
                return;
            }
        }
    };

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
                content,
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
