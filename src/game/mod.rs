use bevy::prelude::*;
use crate::game::gamestate::AppState;
use round_end::system::cleanup_game_entities;

pub mod gamestate;
pub mod player;
pub mod hand;
pub mod graveyard;
pub mod deck;
pub mod card;
pub mod turn_player;
pub mod round_end;
pub mod special_cards;
pub mod ai;

use gamestate::GameStatePlugin;
use hand::HandPlugin;
use graveyard::GraveyardPlugin;
use deck::DeckPlugin;
use card::CardPlugin;
use turn_player::TurnPlugin;
use round_end::RoundEndPlugin;
use special_cards::SpecialCardsPlugin;
use ai::AIPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(GameStatePlugin)
        .add_plugins(HandPlugin)
        .add_plugins(GraveyardPlugin)
        .add_plugins(DeckPlugin)
        .add_plugins(CardPlugin)
        .add_plugins(TurnPlugin)
        .add_plugins(AIPlugin)
        .add_plugins(RoundEndPlugin)
        .add_plugins(SpecialCardsPlugin)

        // clean all entities on exit
        .add_systems(OnEnter(AppState::MainMenu), cleanup_game_entities);
    }
}