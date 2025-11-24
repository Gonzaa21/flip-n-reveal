use bevy::prelude::*;
use bevy::audio::{PlaybackMode, Volume};
use rand::Rng;
use crate::ui::soundtrack::resource::GameAudio;
use crate::ui::soundtrack::event::*;

// Replay card steal effect
pub fn play_card_draw(
    mut commands: Commands,
    audio: Option<Res<GameAudio>>,
    mut events: MessageReader<PlayCardDraw>,
) {
    let Some(audio) = audio else { return; };
    
    for _ in events.read() {
        if !audio.card_place.is_empty() {
            // iterate and randomize sound effect card place
            let mut rng = rand::rng();
            let random_index = rng.random_range(0..audio.card_place.len());
            let selected_sound = audio.card_place[random_index].clone();

            commands.spawn((
                AudioPlayer::new(selected_sound),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::Linear(0.5),
                    ..default()
                },
            ));
        }
    }
}

// play effect of placing/discarding card
pub fn play_card_place(
    mut commands: Commands,
    audio: Option<Res<GameAudio>>,
    mut events: MessageReader<PlayCardPlace>,
) {
    let Some(audio) = audio else { return; };
    
    for _ in events.read() {
        if !audio.card_place.is_empty() {
            let mut rng = rand::rng();
            let random_index = rng.random_range(0..audio.card_place.len());
            let selected_sound = audio.card_place[random_index].clone();

            commands.spawn((
                AudioPlayer::new(selected_sound),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::Linear(0.5),
                    ..default()
                },
            ));
        }
    }
}

// reproduce effect when clicking UI buttons
pub fn button_effect(
    mut commands: Commands,
    audio: Option<Res<GameAudio>>,
    mut events: MessageReader<PlayButtonClick>,
) {
    let Some(audio) = audio else { return; };
    
    for _ in events.read() {
        commands.spawn((
            AudioPlayer::new(audio.button.clone()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::Linear(0.5),
                ..default()
            },
        ));
    }
}