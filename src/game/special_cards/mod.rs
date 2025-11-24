use bevy::prelude::*;
use crate::game::gamestate::AppState;

pub mod resource;
mod effect;
mod system;
use system::{detect_special_card, handle_special_effects};
use effect::{reveal_effect, shuffle_effect, swap_effect};
pub struct SpecialCardsPlugin;

impl Plugin for SpecialCardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (detect_special_card, handle_special_effects, reveal_effect, shuffle_effect, swap_effect)
        .run_if(in_state(AppState::PlayerTurn)));
    }
}