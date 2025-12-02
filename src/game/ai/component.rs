use bevy::{ecs::entity::Entity, prelude::Component};
use std::collections::HashMap;

// select what player is AI
#[derive(Component)]
pub struct AIPlayer {
    pub difficulty: AIDifficulty,
}

// remember revealed cards
#[derive(Component)]
pub struct AIMemory {
    pub known_cards: HashMap<Entity, u8>, // cards that AI know during the game
    pub initial_cards: Vec<(Entity, u8)>, // two initial cards
    pub seen_discards: Vec<u8>, // cards discarded in graveyard
    pub opponent_known_cards: HashMap<Entity, u8>, // opponent cards revealed by using special actions
    pub turns_played: u32, // count of turns have played, 0 for default
}

// for default, create Vectors and HashMaps
impl Default for AIMemory {
    fn default() -> Self {
        Self {
            known_cards: HashMap::new(),
            initial_cards: Vec::new(),
            seen_discards: Vec::new(),
            opponent_known_cards: HashMap::new(),
            turns_played: 0,
        }
    }
}

// AI state
#[derive(Component, Debug)]
pub enum AIState {
    Idle,                                  // wait turn
    Thinking { timer: f32 },               // simulate think with delay
    DecidingDraw,                          // decide where to draw (graveyard or deck)
    ExecutingDraw,                         // draw card
    ThinkingSwap { timer: f32, drawn_card: Entity }, // delay before decide swap
    ActivatingSpecial { drawn_card: Entity }, // active special card
    DecidingSwap { drawn_card: Entity },   // decide whether to swap
    ExecutingSwap { drawn_card_entity: Entity, target_card_entity: Option<Entity> }, // swap card
}

// for default, idle state
impl Default for AIState {
    fn default() -> Self {
        Self::Idle
    }
}

// AI difficult
#[derive(Clone, Copy)]
pub enum AIDifficulty {
    Hard,
}