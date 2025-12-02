use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::game::ai::component::{AIPlayer, AIMemory, AIState};
use crate::game::ai::decision::should_end_round;
use crate::game::card::utils::{card_swap, discard_card};
use crate::game::player::component::Player;
use crate::game::hand::component::Hand;
use crate::game::card::component::{Card, CardPosition, Selected};
use crate::game::turn_player::component::Turn;
use crate::game::graveyard::component::Graveyard;
use crate::game::deck::component::Deck;
use crate::game::AppState;
use crate::ui::soundtrack::event::{PlayCardDraw, PlayCardPlace};
use crate::game::special_cards::resource::{SpecialCardEffect, SpecialEffect};

use crate::game::ai::{estimate_own_score, estimate_opponent_score};
use crate::game::card::refactor_handles::{handle_graveyard_logic, handle_deck_logic};
use crate::game::ai::{should_draw, should_swap, get_worst_known_card_hand, get_best_card_swap};

// start ai memory
pub fn initialize_ai_memory(
    mut commands: Commands,
    ai_query: Query<(Entity, &Player), With<AIPlayer>>,
    hand_query: Query<&Hand>,
    card_query: Query<&Card>,
) {
    info!(target: "mygame", "initialize_ai_memory called!");
    for (ai_entity, player) in ai_query.iter() {
        info!(target: "mygame", "Found AI player: {:?}", ai_entity);

        // obtain player hand
        let Ok(hand) = hand_query.get(player.hand) else { 
            warn!(target: "mygame", "Could not get AI hand");
            continue; 
        };
        
        // leave memory as default
        let mut memory = AIMemory::default();
        
        // iterate first two cards and save in the memory
        for (i, &card_entity) in hand.cards.iter().take(2).enumerate() {
            if let Ok(card) = card_query.get(card_entity) {
                memory.known_cards.insert(card_entity, card.value);
                memory.initial_cards.push((card_entity, card.value));
                info!(target: "mygame", "AI memorized initial card {}: value {}", i, card.value);
            }
        }
        
        // select Idle state
        commands.entity(ai_entity).insert((
            memory,
            AIState::Idle,
        ));
        
        info!(target: "mygame", "AI memory initialized for player: {:?}", ai_entity);
    }
}

// update ai memory during the game
pub fn update_ai_memory(
    mut ai_query: Query<(Entity, &Player, &mut AIMemory), With<AIPlayer>>,
    card_query: Query<(Entity, &Card)>,
    hand_query: Query<&Hand>,
) {
    let Ok((ai_entity, player, mut memory)) = ai_query.single_mut() else { return; };

    // iterate OWN cards with face_up = true and save in memory
    if let Ok(hand) = hand_query.get(player.hand) {
        for &card_entity in &hand.cards {
            if let Ok((entity, card)) = card_query.get(card_entity) {
                if card.face_up {
                    memory.known_cards.insert(entity, card.value);
                }
            }
        }
    }
        
    // save discarded cards of graveyard to memory
    for (_, card) in card_query.iter() {
        if matches!(card.position, CardPosition::Graveyard) {
            // verify if have card values in graveyard
            if !memory.seen_discards.contains(&card.value) {
                memory.seen_discards.push(card.value);
            }
        }
    }
    
    // save opponent cards to memory when it turns over (face_up = true)
    for (entity, card) in card_query.iter() {
        if let CardPosition::Hand(owner) = card.position {
            if owner != ai_entity && card.face_up {
                memory.opponent_known_cards.insert(entity, card.value);
            }
        }
    }
}

// control AI turn
pub fn ai_turn_controller(
    mut commands: Commands,
    time: Res<Time>,
    turn_query: ResMut<Turn>,
    mut ai_query: Query<(Entity, &Player, &mut AIState, &mut AIMemory), With<AIPlayer>>,
    mut hand_query: Query<&mut Hand>,
    mut graveyard_query: Query<&mut Graveyard>,
    mut card_query: Query<(Entity, &mut Transform, &mut Card), With<Card>>,
    deck_query: Query<&mut Deck>,
    player_query: Query<(Entity, &Player)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut next_state: ResMut<NextState<AppState>>,
    draw_message: MessageWriter<PlayCardDraw>,
    place_message: MessageWriter<PlayCardPlace>,
    selected_query: Query<Entity, With<Selected>>,
) {
    // search AI player
    let Ok((ai_entity, ai_player, mut ai_state, mut ai_memory)) = ai_query.single_mut() else {
        info!(target: "mygame", "No AI player found in query!");
        return;  // there is no AI
    };

    // check if it's the AI's turn
    if turn_query.current_player != ai_entity {
        if !matches!(*ai_state, AIState::Idle) {
            *ai_state = AIState::Idle; // if is not turn, make sure it's in Idle
        }
        return;
    }

    // match of ai states
    match &mut *ai_state {
        AIState::Idle => {
            // save the first two cards and the cards with face up in memory
            if ai_memory.initial_cards.is_empty() {
                let Ok(ai_hand) = hand_query.get(ai_player.hand) else { return; };
                for (_, &card_entity) in ai_hand.cards.iter().take(2).enumerate() {
                    if let Ok((_, _, card)) = card_query.get(card_entity) {
                        if card.face_up {
                            ai_memory.known_cards.insert(card_entity, card.value);
                            ai_memory.initial_cards.push((card_entity, card.value));
                        }
                    }
                }
            }

            *ai_state = AIState::Thinking { timer: 1.0 }; // wait a few seconds
            info!(target: "mygame", "AI turn started, thinking...");
        }

        AIState::Thinking { timer } => {
            *timer -= time.delta_secs();

            if *timer <= 0.0 { // when timer finish, change to DecidingDraw state
                *ai_state = AIState::DecidingDraw;
                info!(target: "mygame", "AI deciding where to draw...");
            }
        }

        AIState::DecidingDraw => {
            // obtain AI's hand
            let Ok(ai_hand) = hand_query.get(ai_player.hand) else { return; };

            // obtain the worst known card
            let worst_card = get_worst_known_card_hand(&ai_memory, ai_hand)
                .map(|(_, value)| value);

            // deciding
            let should_draw = should_draw(&ai_memory, &graveyard_query, worst_card, &card_query);

            // if should_draw = true, draw from graveyard, if it's false, from deck
            if should_draw {
                handle_graveyard_logic(&mut graveyard_query, &mut turn_query.into_inner(), &mut card_query, ai_entity);
                info!(target: "mygame", "AI drawing from graveyard");
            } else {
                handle_deck_logic(deck_query, turn_query, card_query, draw_message, ai_entity);
                info!(target: "mygame", "AI drawing from deck");
            }

            *ai_state = AIState::ExecutingDraw;
        }

        AIState::ExecutingDraw => {
            // search drawn card by AI
            let drawn_card = card_query.iter()
                .find(|(_, _, card)| matches!(card.position, CardPosition::DrawnCard(player) if player == ai_entity));

            // if AI draw a card, change to DecidingSwap state (passing drawn_entity parameter)
            if let Some((drawn_entity,_, drawn_card)) = drawn_card {
                *ai_state = AIState::ThinkingSwap { timer: 1.0, drawn_card: drawn_entity };

                info!(target: "mygame", "AI drew card with value {}", drawn_card.value);
            };
        }

        AIState::ThinkingSwap { timer, drawn_card } => {
            *timer -= time.delta_secs();

            if *timer <= 0.0 {
                // obtain drawn card
                let Ok((_, _, card)) = card_query.get(*drawn_card) else {
                    return;
                };

                // verify if drawn card is special card
                if card.from_deck {
                    let special_effect_type = match card.value {
                        11 => Some(SpecialEffect::Shuffle),
                        9 => Some(SpecialEffect::Reveal), 
                        7 => Some(SpecialEffect::Swap),
                        _ => None,
                    };

                    // if it is, use special effects
                    if let Some(effect) = special_effect_type {
                        *ai_state = AIState::ActivatingSpecial { drawn_card: *drawn_card };
                        info!(target: "mygame", "AI activated special card: {:?}", effect);
                        return;
                    }
                }

                // change to decide swap
                *ai_state = AIState::DecidingSwap { drawn_card: *drawn_card };
                info!(target: "mygame", "AI deciding what to do with card");
            }
        }

        AIState::ActivatingSpecial { drawn_card } => {
            // obtain drawn card
            let Ok((_, _, card)) = card_query.get(*drawn_card) else { return; };

            // use verify card value of drawn card and include the specific effect
            match card.value {
                11 => {
                    // obtain opponent
                    let opponent = player_query.iter()
                        .find(|(entity, _)| *entity != ai_entity);

                    // insert Shuffle effect
                    if let Some((opponent_entity, _)) = opponent {
                        commands.insert_resource(SpecialCardEffect {
                            card_entity: Some(*drawn_card),
                            effect_type: Some(SpecialEffect::Shuffle),
                            awaiting_target: false,
                            target_player: Some(opponent_entity),
                            ..Default::default()
                        });

                        info!(target: "mygame", "AI shuffled opponent's hand");
                    }
                }
                7 => {
                    // obtain opponent
                    let opponent = player_query.iter()
                        .find(|(entity, _)| *entity != ai_entity);

                    if let Some((_, opponent_player)) = opponent {
                        // obtain opponent hand
                        let Ok(opponent_hand) = hand_query.get(opponent_player.hand) else { return; };
                        
                        // select a opponent card of his hand
                        let target_card = ai_memory.opponent_known_cards.iter()
                            .max_by_key(|(_, value)| *value)
                            .map(|(entity, _)| *entity)
                            .or(opponent_hand.cards.first().copied());
                        
                        // obtain ai hand
                        let Ok(ai_hand) = hand_query.get(ai_player.hand) else { return; };
                        
                        // select own worst known card in hand
                        let own_card = get_worst_known_card_hand(&ai_memory, ai_hand)
                            .map(|(entity, _)| entity);
                        
                        // insert swap effect
                        if let (Some(target), Some(own)) = (target_card, own_card) {
                            commands.insert_resource(SpecialCardEffect {
                                card_entity: Some(*drawn_card),
                                effect_type: Some(SpecialEffect::Swap),
                                awaiting_target: false,
                                awaiting_own_card: false,
                                target_card: Some(target),
                                own_card: Some(own),
                                ..Default::default()
                            });
                            info!(target: "mygame", "AI will swap cards");
                        }
                    }
                }
                9 => {
                    // insert reveal effect
                    commands.insert_resource(SpecialCardEffect {
                        card_entity: Some(*drawn_card),
                        effect_type: Some(SpecialEffect::Reveal),
                        awaiting_target: false,
                        ..Default::default()
                    });
                }
                _ => {}
            }

            // change to deciding swap state
            *ai_state = AIState::DecidingSwap { drawn_card: *drawn_card };
        }

        AIState::DecidingSwap { drawn_card } => {
            // obtain drawn card
            let Ok((_, _, drawn_card_comp)) = card_query.get(*drawn_card) else { return; };
            let drawn_value = drawn_card_comp.value;

            // obtain AI's hand
            let Ok(ai_hand) = hand_query.get(ai_player.hand) else { return; };

            // deciding
            let should_swap = should_swap(drawn_value, &ai_memory, ai_hand);

            if should_swap {
                // obtain best card to swap
                let target_card = get_best_card_swap(drawn_value, &ai_memory, ai_hand);

                // verify if have the best card to swap and change AIState to ExecutingSwap
                if let Some(target_entity) = target_card {
                    *ai_state = AIState::ExecutingSwap { drawn_card_entity: *drawn_card, target_card_entity: Some(target_entity) };
                } else {
                    // if have not card to swap, discard
                    *ai_state = AIState::ExecutingSwap { drawn_card_entity: *drawn_card, target_card_entity: None };                    
                }

            } else {
                // if should_swap = false, discard directly
                *ai_state = AIState::ExecutingSwap { drawn_card_entity: *drawn_card, target_card_entity: None };
            }
        }

        AIState::ExecutingSwap { drawn_card_entity, target_card_entity } => {
            // if have target_card, execute card_swap system, if not, discard_card system
            match target_card_entity {
                Some(target) => {
                    card_swap(*target, &mut card_query, &mut graveyard_query, turn_query, hand_query.reborrow(), &player_query, windows, &mut commands, &selected_query);
                }
                None => {
                    discard_card(*drawn_card_entity, &mut card_query, &mut graveyard_query, turn_query, &player_query, &mut commands, &selected_query, place_message);
                }
            }

            // increment turn counter
            ai_memory.turns_played += 1;
            info!(target: "mygame", "AI turn count: {}", ai_memory.turns_played);

            /*  -- CHECK TO END ROUND -- */

            // obtain drawn card
            let Ok(ai_hand) = hand_query.get(ai_player.hand) else { 
                *ai_state = AIState::Idle;
                return;
            };

            // obtain opponent player
            let opponent = player_query.iter()
                .find(|(entity, _)| *entity != ai_entity);

            if let Some((_, opponent_player)) = opponent {
                // get opponent hand
                let Ok(opponent_hand) = hand_query.get(opponent_player.hand) else {
                    *ai_state = AIState::Idle;
                    return;
                };
                
                // deciding
                let should_end = should_end_round(&ai_memory, ai_hand, opponent_hand, ai_memory.turns_played);
                
                info!(target: "mygame", "AI knows {} of its own cards", ai_memory.known_cards.len());
                info!(target: "mygame", "AI knows {} opponent cards", ai_memory.opponent_known_cards.len());

                let own_score = estimate_own_score(&ai_memory, ai_hand);
                let opponent_score = estimate_opponent_score(&ai_memory, opponent_hand);
        
                info!(target: "mygame", "AI end round check - Own: {:.1}, Opponent: {:.1}, Margin: {:.1}", 
                    own_score, opponent_score, own_score - opponent_score);

                // if should_end = true, finish round, if not, change turn
                if should_end {
                    next_state.set(AppState::RoundEnd);
                    info!(target: "mygame", "AI decided to end the round!");
                } else {
                    *ai_state = AIState::Idle;
                }
            }
            *ai_state = AIState::Idle;
        }
    }
}