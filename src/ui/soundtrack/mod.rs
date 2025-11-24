use bevy::prelude::*;
use crate::game::gamestate::AppState;
use crate::ui::soundtrack::music_system::{load_audio, play_menu, play_game, stop_music};
use crate::ui::soundtrack::resource::CurrentMusic;
use crate::ui::soundtrack::event::{PlayCardDraw, PlayCardPlace, PlayButtonClick};
use crate::ui::soundtrack::effect_system::{play_card_draw, play_card_place, button_effect};

pub mod resource;
mod music_system;
mod effect_system;
pub mod event;
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app
        // insert currentMusic resource
        .init_resource::<CurrentMusic>()

        // register messages of sound effects
        .add_message::<PlayCardDraw>()
        .add_message::<PlayCardPlace>()
        .add_message::<PlayButtonClick>()
        
        // load audio at first
        .add_systems(PreStartup, load_audio)
        
        // menu music - play in MainMenu
        .add_systems(Update, play_menu.run_if(in_state(AppState::MainMenu)))
        .add_systems(OnExit(AppState::MainMenu), stop_music)
        
        // game music - play in Setup, PlayerTurn and RoundEnd
        .add_systems(Update, play_game.run_if(in_state(AppState::Setup)
            .or(in_state(AppState::PlayerTurn))
            .or(in_state(AppState::RoundEnd))
        ))
        
        // stop when leave the game
        .add_systems(OnEnter(AppState::MainMenu), stop_music)
        
        // sound effects
        .add_systems(Update, (play_card_draw, play_card_place, button_effect));
    }
}