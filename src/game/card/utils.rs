use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::game::card::component::{Card, CardPosition, Selected};
use crate::game::{graveyard::component::Graveyard, turn_player::component::Turn, hand::component::Hand, player::component::Player};
use crate::ui::soundtrack::event::PlayCardPlace;

// AUXILIAR SYSTEMS
pub fn discard_card(
    clicked_entity: Entity,
    card_query: &mut Query<(Entity, &mut Transform, &mut Card), With<Card>>,
    graveyard_query: &mut Query<&mut Graveyard>,
    turn_query: ResMut<Turn>,
    player_query: &Query<(Entity, &Player)>,
    commands: &mut Commands,
    selected_query: &Query<Entity, With<Selected>>,
    mut place_message: MessageWriter<PlayCardPlace>,
) {
    if let Ok((_, _transform, card)) = card_query.get_mut(clicked_entity) {
        if matches!(card.position, CardPosition::DrawnCard(player_id) if player_id == turn_query.current_player) {
            if let Ok((_, _, mut card)) = card_query.get_mut(clicked_entity) {
                card.position = CardPosition::Graveyard;
                card.face_up = true;
                card.is_being_dealt = true;

                // update graveyard
                if let Ok(mut graveyard) = graveyard_query.single_mut() {
                    graveyard.cards.push(clicked_entity);
                    
                    place_message.write(PlayCardPlace);
                    info!(target: "mygame", "Card discarded directly to graveyard: {:?}", clicked_entity);
                }

                // remove card selection
                for selected_entity in selected_query.iter() {
                    commands.entity(selected_entity).remove::<Selected>();
                }

                change_turn(turn_query, player_query);
            }
            return;
        }
    }
}

pub fn card_swap(
    clicked_entity: Entity,
    card_query: &mut Query<(Entity, &mut Transform, &mut Card), With<Card>>,
    graveyard_query: &mut Query<&mut Graveyard>,
    turn_query: ResMut<Turn>,
    hand_query: &mut Query<&mut Hand>,
    player_query: &Query<(Entity, &Player)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    commands: &mut Commands,
    selected_query: &Query<Entity, With<Selected>>,
) {
    let clicked_card = card_query.iter()
        .find(|(entity, _, _)| *entity == clicked_entity)
        .map(|(_, _, card)| card);

    if let Some(clicked_card) = clicked_card {
        // verify: card is inside the hand
        if !matches!(clicked_card.position, CardPosition::Hand(player_id) if player_id == turn_query.current_player) {
            info!(target: "mygame", "Card is not in current player's hand");
            return;
        }
    
        // search the drawn card
        let drawn_card = card_query.iter()
            .find(|(_, _, card)| matches!(card.position, CardPosition::DrawnCard(player_id) if player_id == turn_query.current_player));
        
        if drawn_card.is_none() { // if it was not found
            info!(target: "mygame", "No drawn card found for current player");
            return;
        }
    
        // obtain drawn_card entity and original position card
        let drawn_card_entity = drawn_card.unwrap().0; // .0 indicate the first parameter (Entity)
        
        // exchange positions
        if let Ok([(_, mut drawn_transform, mut drawn_card), (_, mut clicked_transform, mut clicked_card)]) = 
            card_query.get_many_mut([drawn_card_entity, clicked_entity]) {
            
            let clicked_pos = clicked_transform.translation; // obtain position
            
            let Ok(window) = windows.single() else { return; };
            let is_sec_player = clicked_pos.y > 0.0;
            let base_y = if is_sec_player {
                window.height() * 0.15
            } else {
                window.height() * -0.15
            };

            drawn_card.position = CardPosition::Hand(turn_query.current_player);
            drawn_card.face_up = false;
            drawn_transform.translation = Vec3::new(clicked_pos.x, base_y, 10.0);
            clicked_card.position = CardPosition::Graveyard; // card of hand selected to graveyard
            clicked_card.face_up = true; // card front
            
            let Ok(window) = windows.single() else { return; };
            if let Ok(mut graveyard) = graveyard_query.single_mut() {
                graveyard.cards.push(clicked_entity); // update changes
                
                let stack_index = (graveyard.cards.len() - 1) as f32;
                let max_stack = 8.0;
                        
                let (offset_x, offset_y) = if stack_index < max_stack {
                    (stack_index * 0.5, stack_index * 0.3)
                } else {
                    (max_stack * 0.5, max_stack * 0.3)
                };
                
                // small rotation
                let random_rotation = (rand::random::<f32>() - 0.5) * 0.15;
                clicked_transform.rotation = Quat::from_rotation_z(random_rotation);
                
                clicked_transform.translation = Vec3::new(
                    window.width() * -0.06 + offset_x,
                    window.height() * 0.0 + offset_y,
                    10.0 + stack_index
                );
                
                if let Some((_, player)) = player_query.iter().find(|(entity, _)| *entity == turn_query.current_player) {
                    if let Ok(mut hand) = hand_query.get_mut(player.hand) {
                        // remove hand
                        hand.cards.retain(|&card_entity| card_entity != clicked_entity);
                        // add new card in hand
                        hand.cards.push(drawn_card_entity);
                    }
                }
                info!(target: "mygame", "Card swap completed: {:?} -> Hand, {:?} -> Graveyard", drawn_card_entity, clicked_entity);
                
                // remove card selection
                for selected_entity in selected_query.iter() {
                    commands.entity(selected_entity).remove::<Selected>();
                }

                // automatic turn
                change_turn(turn_query, &player_query);
            }
        }
    }
}

// system to change turn automatically
fn change_turn(
    mut turn: ResMut<Turn>,
    players: &Query<(Entity, &Player)>,
) {
    // obtain player list
    let players: Vec<Entity> = players.iter()
        .map(|(entity,_)| entity)
        .collect();

    if let Some(pos) = players.iter().position(|&p| p == turn.current_player) {
        let next_index = (pos + 1) % players.len();
        turn.current_player = players[next_index];
        turn.has_drawn_card = false;

        info!(target: "mygame", "Turn automatically changed to player: {:?}", turn.current_player);
    }
}