use bevy::prelude::*;
use bevy::utils::HashMap;
use leafwing_input_manager::action_state::ActionState;
use crate::dev_tools::DebugOverlaysState;
use crate::input::DevActionToggles;

pub enum ZLayers {
    Background = 0,
    Level = 1,
    Enemy = 2,
    Ghost = 3,
    Player = 5,
    Fog = 6,
    DebugOverlays = 7,
    Camera = 20,
}
impl ZLayers {
    pub fn f32(self) -> f32 {
        (self as usize) as f32
    }

    pub fn transform(self) -> Transform {
        Transform::from_translation(Vec2::default().extend(self.f32()))
    }
}

pub fn debug_z_layers(
    query: Query<(Entity, Option<&Name>, &Transform)>,
    input: Query<&ActionState<DevActionToggles>>,
) {
    if input.iter()
        .all(|act| !act.just_pressed(&DevActionToggles::ZLayerDump)) { return; }

    let mut items = HashMap::<i32, Vec<String>>::new();
    for (entity, maybe_name, transform) in query.iter() {

        if let None = maybe_name { continue; } // remove to log everything (noisy)

        let id = if let Some(name) = maybe_name { format!("{:?}", name) } else { format!("{:?}", entity) };

        let z = transform.translation.z.floor() as i32;
        items.entry(z).or_insert_with(Vec::new).push(id);
    }

    // sort
    let mut sorted_items: Vec<_> = items.into_iter().collect();
    sorted_items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    //print
    for (z, ids) in sorted_items {
        info!("{}: {}", z, ids.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity_check() {
        assert!(ZLayers::Player.f32() > ZLayers::Background.f32());

        assert!(ZLayers::Player.f32() == 100.);
    }
}