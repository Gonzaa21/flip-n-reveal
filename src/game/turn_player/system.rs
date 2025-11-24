use bevy::prelude::*;
use crate::game::{player::component::Player, gamestate::AppState};
use super::component::Turn;

// start first player turn
pub fn start_turn_system(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Some(first_player) = players.iter().next() {
        commands.insert_resource(Turn {
            current_player: first_player,
            has_drawn_card: false,
        });
        next_state.set(AppState::PlayerTurn);
        info!(target: "mygame","Shift started for player: {:?}", first_player);
    } else {
        warn!(target: "mygame","There are no players");
    }
}

// finish game round
pub fn end_round_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
            next_state.set(AppState::RoundEnd);
            info!(target: "mygame", "Round ended by player!");
    }
}