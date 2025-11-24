use bevy::prelude::*;

// audio resources
#[derive(Resource, Default)]
pub struct GameAudio {
    pub menu: Handle<AudioSource>,
    pub game: Handle<AudioSource>,
    pub card_place: Vec<Handle<AudioSource>>,
    pub button: Handle<AudioSource>,
}

// manage if music is playing, for default: none
#[derive(Resource)]
pub struct CurrentMusic {
    pub entity: Option<Entity>,
}

impl Default for CurrentMusic {
    fn default() -> Self {
        Self { entity: None }
    }
}