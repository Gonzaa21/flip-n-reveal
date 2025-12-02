use bevy::prelude::*;
use crate::game::player::component::Player;
use crate::game::hand::component::Hand;
use crate::game::gamestate::GameEntity;
use crate::game::ai::component::{AIDifficulty, AIMemory, AIPlayer, AIState};

pub fn spawn_player(mut commands: Commands) {
    let player_names = ["Player 1", "Player 2"];

    for (i, name) in player_names.iter().enumerate() {
        // create hand
        let hand = commands.spawn((
            Hand { cards: Vec::new() },
            GameEntity,
        )).id();

        let player_entity = commands.spawn((
            Player {
                name: name.to_string(),
                hand: hand,
                is_local_player: i == 0
            },
            GameEntity,
        )).id();
        
        // add AIPlayer to second player
        if i == 1 {
            commands.entity(player_entity).insert((
                AIPlayer {
                    difficulty: AIDifficulty::Hard,
                },
                AIMemory::default(),
                AIState::Idle,
            ));
        }
    }
}