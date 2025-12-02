use bevy::prelude::*;
use crate::game::ai::component::AIMemory;
use crate::game::graveyard::component::Graveyard;
use crate::game::hand::component::Hand;
use crate::game::card::component::Card;

/*
ANALYSIS OF SCORES - calculation functions
make decisions depending of the estimated own score and the opponent
*/
pub fn estimate_own_score(
    ai_memory: &AIMemory,
    hand: &Hand,
) -> f32 {
    let mut total = 0.0;
    
    for &card_entity in &hand.cards {
        if let Some(&value) = ai_memory.known_cards.get(&card_entity) {
            total += value as f32;
        } else {
            total += calculate_expected_value(ai_memory);
        }
    }
    total
}

pub fn estimate_opponent_score(
    ai_memory: &AIMemory,
    hand: &Hand,
) -> f32 {
    let mut total = 0.0;
    
    for &card_entity in &hand.cards {
        if let Some(&value) = ai_memory.opponent_known_cards.get(&card_entity) {
            total += value as f32;
        } else {
            total += calculate_expected_value(ai_memory);
        }
    }
    total
}

fn calculate_expected_value(ai_memory: &AIMemory,) -> f32 {
    // number of known cards
    let known_cards_count = ai_memory.known_cards.len(); // known cards
    let graveyard_cards_count = ai_memory.seen_discards.len(); // cards seen in graveyard
    let opponent_cards_count = ai_memory.opponent_known_cards.len(); // known opponent cards

    // sum known card values
    let known_cards_values = ai_memory.known_cards.values().copied().sum::<u8>() as f32; // known cards
    let graveyard_cards_values = ai_memory.seen_discards.iter().sum::<u8>() as f32; // cards seen in graveyard
    let opponent_cards_values = ai_memory.opponent_known_cards.values().copied().sum::<u8>() as f32; // known opponent cards

    // total number of cards and values
    let total_count = known_cards_count + graveyard_cards_count + opponent_cards_count;
    let total_values = known_cards_values + graveyard_cards_values + opponent_cards_values;

    let remaining_cards = (48 - total_count) as f32;
    let remaining_values = 312.0 - total_values;

    if remaining_cards == 0.0 {return 6.5;};

    remaining_values as f32 / remaining_cards as f32
}


/*
DECISIONS - draw and swap
make decisions depending of the estimated known card values of hands and probabilities
*/
// decide where should draw card (from graveyard or deck)
pub fn should_draw(
    ai_memory: &AIMemory,
    graveyard_query: &Query<&mut Graveyard>,
    worst_card: Option<u8>,
    card_query: &Query<(Entity, &mut Transform, &mut Card), With<Card>>,
) -> bool {
    
    // obtain graveyard and verify if it is empty
    let Ok(graveyard) = graveyard_query.single() else {
        return false;
    };
    if graveyard.cards.is_empty() {
        return false;
    }

    // obtain last card in graveyard
    let Some(&top_card_entity) = graveyard.cards.last() else {
        return false;
    };
    
    // obtain the value of that card
    let Ok((_, _, top_card)) = card_query.get(top_card_entity) else {
        return false;
    };
    let graveyard_value = top_card.value as f32;

    // calculate expected total value of the hand
    let expected_value = calculate_expected_value(ai_memory);

    // decision
    if let Some(worst) = worst_card {
        let worst_value = worst as f32;

        graveyard_value < worst_value && graveyard_value < expected_value
    } else {
        graveyard_value < expected_value
    }

}

// obtain the worst card in hand
pub fn get_worst_known_card_hand(
    ai_memory: &AIMemory,
    hand: &Hand,
) -> Option<(Entity, u8)> {
    
    let mut worst_card: Option<(Entity, u8)> = None;
    
    // iterate all cards in hand
    for &card_entity in &hand.cards {
        // verify if card is stored in the memory (known_cards)
        if let Some(&value) = ai_memory.known_cards.get(&card_entity) {
            // if worst_card is None, update new card value, 
            // else if worst_card have cards, verify if new card value is grater than worst_value
            worst_card = match worst_card {
                None => Some((card_entity, value)),
                Some((_, worst_value)) if value > worst_value => Some((card_entity, value)),
                other => other,  
            };
        }
    }
    worst_card
}

// decide which card should swap for drawn card (swap or discard)
pub fn should_swap(
    drawn_card: u8,
    ai_memory: &AIMemory,
    hand: &Hand,
) -> bool {
    // obtain worst card
    let worst_card = get_worst_known_card_hand(ai_memory, hand);
    
    // search worst card value, if drawn card value is smaller than worst card, swap
    if let Some((_, worst_value)) = worst_card {
        if drawn_card < worst_value {
            return true;
        }
    }

    // if drawn card value is <= 4, always swap
    if drawn_card <= 4 {
        return true;
    }
    
    // if drawn_card is <= 6 and known cards are smaller than 4, swap
    if drawn_card <= 6 && ai_memory.known_cards.len() < 4 {
        return true;
    }

    false // if it doesn't achieve the conditions, discard
}

// decide the best card to swap
pub fn get_best_card_swap(
    drawn_card_value: u8,
    ai_memory: &AIMemory,
    hand: &Hand,
) -> Option<Entity> {

    // obtain worst card
    let worst_card = get_worst_known_card_hand(ai_memory, hand);
    
    // search the worst card, if drawn card is smaller than the worst card, swap
    if let Some((entity, worst_value)) = worst_card {
        if drawn_card_value < worst_value {
            return Some(entity);
        }
    }

    // if drawn card value <= 5, swap card for a unknown one
    if drawn_card_value <= 5 {
        return get_unknown_card_hand(ai_memory, hand);
    }

    None // if it doesn't achieve the conditions, discard
}

// obtain the first unknown card in hand
fn get_unknown_card_hand(
    ai_memory: &AIMemory,
    hand: &Hand,
) -> Option<Entity> {
    // iterate all cards in hand
    for &card_entity in &hand.cards {
        // search the first unknown card in hand
        if !ai_memory.known_cards.contains_key(&card_entity) {
            return Some(card_entity);
        }
    }
    None
}

// decide when shound end the round
pub fn should_end_round(
    ai_memory: &AIMemory,
    my_hand: &Hand,
    opponent_hand: &Hand,
    turn_count: u32, 
)-> bool {
    // calculate estimated scores
    let own_score = estimate_own_score(ai_memory, my_hand);
    let opponent_score = estimate_opponent_score(ai_memory, opponent_hand);

    // can finish after the first 4 turns
    if turn_count <= 4 {
        return false;
    }

    let margin_score = own_score - opponent_score;

    // conditions using opponent known cards
    if ai_memory.opponent_known_cards.is_empty() {
        // finish it if: the estimated own score is ≤ (small or equal) 20 and if turns played >= 6
        if own_score <= 20.0 && turn_count >= 6 {
            return true;
        }
        
        if own_score <= 15.0 && turn_count >= 5 {
            return true;
        }
    } else {
        // when finish: when estimated own/opponent score have a difference of 4 points
        if margin_score <= -3.0 {
            return true;
        }
    }

    // do not finish if it have known cards greater than or equal 10
    for &card_entity in &my_hand.cards {
        let known_cards = ai_memory.known_cards.get(&card_entity);
        if let Some(value) = known_cards {
            if *value >= 10 {
                return false;
            }
        }
    }

    false
}