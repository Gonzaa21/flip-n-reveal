use bevy::prelude::*;
pub mod component;
mod decision;
mod system;

use decision::*;
use system::*;

use crate::game::hand::system::deal_initial_hands;
use crate::game::gamestate::AppState;

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(
            Update,
            update_ai_memory.run_if(in_state(AppState::PlayerTurn)).before(ai_turn_controller)
        )
        .add_systems(
            Update,
            ai_turn_controller.run_if(in_state(AppState::PlayerTurn))
        )
        .add_systems(OnEnter(AppState::PlayerTurn), initialize_ai_memory.after(deal_initial_hands));
    }
}