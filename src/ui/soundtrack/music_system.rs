use bevy::prelude::*;
use bevy::audio::{PlaybackMode, Volume};
use crate::ui::soundtrack::resource::{CurrentMusic, GameAudio};

// load audio assets
pub fn load_audio(
    mut commands: Commands<'_, '_>,
    asset_server: Res<AssetServer>,
) {
    let card_place_path = vec![
        asset_server.load("audio/deal/card-place-1.wav"),
        asset_server.load("audio/deal/card-place-2.wav"),
        asset_server.load("audio/deal/card-place-3.wav"),
        asset_server.load("audio/deal/card-place-4.ogg"),
        asset_server.load("audio/deal/card-place-5.ogg"),
    ];

    let sound = GameAudio {
        menu: asset_server.load("audio/background/background_menu.wav"),
        game: asset_server.load("audio/background/background_game.wav"),
        card_place: card_place_path,
        button: asset_server.load("audio/button_effect.wav"),
    };

    commands.insert_resource(sound);
    commands.insert_resource(CurrentMusic::default());
}

// play music in menu
pub fn play_menu(
    mut commands: Commands,
    audio: Option<Res<GameAudio>>,
    mut current_music: ResMut<CurrentMusic>,
    asset_server: Res<AssetServer>,
) {
    // verify if audio resource exists
    let Some(audio) = audio else {
        return;
    };

    // verify if game audio is loaded
    if !asset_server.is_loaded(&audio.menu) {
        return;
    }

    if current_music.entity.is_some() {
        return;
    }

    // spawn music
    let entity = commands.spawn((
        AudioPlayer::new(audio.menu.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Loop, // loop
            volume: Volume::Linear(0.1), // 30% volume
            ..default()
        },
    )).id();

    current_music.entity = Some(entity);
}

// play music in game
pub fn play_game(
    mut commands: Commands,
    audio: Option<Res<GameAudio>>,
    mut current_music: ResMut<CurrentMusic>,
    asset_server: Res<AssetServer>,
) {
    // verify if audio resource exists
    let Some(audio) = audio else {
        return;
    };

    // verify if game audio is loaded
    if !asset_server.is_loaded(&audio.menu) {
        return;
    }
    
    if current_music.entity.is_some() {
        return;
    }

    // spawn music
    let entity = commands.spawn((
        AudioPlayer::new(audio.game.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Loop, // loop
            volume: Volume::Linear(0.4), // 30% volume
            ..default()
        },
    )).id();

    current_music.entity = Some(entity);
}

// stop music
pub fn stop_music(
    mut commands: Commands,
    mut current_music: ResMut<CurrentMusic>,
) {
    if let Some(entity) = current_music.entity {
        commands.entity(entity).despawn();
        current_music.entity = None;
    }
}