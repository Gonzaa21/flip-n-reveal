use bevy::prelude::*;
use crate::game::card::component::{Card, CardPosition};
use crate::game::{graveyard::component::Graveyard, turn_player::component::Turn, deck::component::Deck};
use crate::ui::soundtrack::event::{PlayCardDraw};

// Same code as in card/handles.rs, focusing in the logic of draw in graveyard or deck for use in AI player

pub fn handle_graveyard_logic(
    graveyard_query: &mut Query<&mut Graveyard>,
    turn_query: &mut Turn,
    card_query: &mut Query<(Entity, &mut Transform, &mut Card), With<Card>>,
    ai_entity: Entity,
) {
    let Ok(mut graveyard) = graveyard_query.single_mut() else { return; };
    
    if graveyard.cards.is_empty() { return; }
    
    let drawn_card_entity = graveyard.cards.pop().unwrap();
    
    if let Ok((_, _, mut card)) = card_query.get_mut(drawn_card_entity) {
        card.position = CardPosition::DrawnCard(ai_entity);
        card.owner_id = Some(ai_entity);
        card.face_up = true;
        card.from_deck = false;
        turn_query.has_drawn_card = true;
    }
    info!(target: "mygame", "AI drawing from graveyard");
}

pub fn handle_deck_logic(
    mut deck_query: Query<&mut Deck>,
    mut turn_query: ResMut<Turn>,
    mut card_query: Query<(Entity, &mut Transform, &mut Card), With<Card>>,
    mut draw_message: MessageWriter<PlayCardDraw>,
    ai_entity: Entity,
) {
    let Ok(mut deck) = deck_query.single_mut() else { return; };
    
    if deck.cards_values.is_empty() { return; }
    
    let drawn_card_entity = deck.cards_values.remove(0);
    
    if let Ok((_, _, mut card)) = card_query.get_mut(drawn_card_entity) {
        card.is_being_dealt = true;
        card.position = CardPosition::DrawnCard(ai_entity);
        card.owner_id = Some(ai_entity);
        card.face_up = true;
        card.from_deck = true;
        turn_query.has_drawn_card = true;
        draw_message.write(PlayCardDraw);
    }
    
    info!(target: "mygame", "AI drawing from deck");
}
