use bevy::prelude::*;

#[derive(Resource)]
pub struct Turn {
    pub current_player: Entity, // current player's turn
    pub has_drawn_card: bool, // if player has drawn a card
}