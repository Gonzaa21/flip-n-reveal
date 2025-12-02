use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::game::card::component::{Card, CardPosition, CardHandles, CardBack, Suit, Selected, DoubleClick};
use crate::game::special_cards::resource::SpecialEffect;
use crate::game::{graveyard::component::Graveyard, turn_player::component::Turn, deck::component::Deck, hand::component::Hand, player::component::Player, special_cards::resource::SpecialCardEffect};
use crate::game::card::handles::{handle_deck_click, handle_card_click, handle_graveyard_click};
use crate::ui::card_animation::component::{AnimationType, CardAnimation};
use crate::ui::soundtrack::event::{PlayCardDraw, PlayCardPlace};
use bevy::asset::Assets;
use bevy::image::{Image, ImageSampler};
use rand::seq::SliceRandom;
use rand::rng;

// spawn card values after setup and allocate suit/value and textures
// iterate and mix all cards front/back
pub fn setup_cards(mut commands: Commands, asset_server: Res<AssetServer>) {
    let suits = ["coarse", "cup", "gold", "sword"];
    let mut card_routes = Vec::new();

    for suit_name in suits.iter() {
        for value in 1..=12 {
            let route = format!("textures/deck/{}/{}.PNG", suit_name, value);
            card_routes.push(route);
        }
    }

    // load textures
    let card_handles: Vec<Handle<Image>> = card_routes
        .into_iter()
        .map(|route| asset_server.load(route))
        .collect();

    // card back texture
    let card_back_handle = asset_server.load("textures/deck/back/back.PNG");

    // insert handles resources
    commands.insert_resource(CardHandles(card_handles.clone()));
    commands.insert_resource(CardBack(card_back_handle));

    let suits_logic = [Suit::Coarse, Suit::Cup, Suit::Gold, Suit::Sword];
    let mut cards_data: Vec<(Suit, u8)> = suits_logic
        .iter()
        .flat_map(|suit| (1..=12).map(move |value| (suit.clone(), value)))
        .collect();

    cards_data.shuffle(&mut rng());
}


pub fn card_face(
    card_back: Option<Res<CardBack>>,
    mut query: Query<(&Card, &mut Sprite, Option<&CardAnimation>)>
) {
    let Some(card_back) = card_back else { return; };

    for (card, mut sprite, animation) in query.iter_mut() {
        if let Some(anim) = animation {
            if anim.animation_type == AnimationType::Flip {
                if anim.progress < 0.5 {
                    // first half: keep down
                    sprite.image = card_back.0.clone();
                    continue;
                } else {
                    // second half: keep up
                    sprite.image = card.front_face.clone();
                    continue;
                }
            }
        }
        
        if card.face_up {
            // asign front
            sprite.image = card.front_face.clone();
        } else {
            // Asign back
            sprite.image = card_back.0.clone();
        }
    }
}

// card_selection - detect clicks
//      detect_card_click
//          handle_card_click - manage cards
//              discard_card() - drawn card to graveyard
//              card_swap() - swap drawn card to hand card
//
//      detect_deck_click
//          handle_deck_click - draw card of the deck
//
//      detect_graveyard_click
//          handle_graveyard_click - draw of the graveyard

// card selection system
pub fn card_selection(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut card_query: Query<(Entity, &mut Transform, &mut Card), With<Card>>,
    selected_query: Query<Entity, With<Selected>>,
    mut double_click: ResMut<DoubleClick>,
    time: Res<Time>,
    turn_query: ResMut<Turn>,
    hand_query: Query<&mut Hand>,
    mut graveyard_query: Query<&mut Graveyard>,
    deck_query: Query<&mut Deck>,
    player_query: Query<(Entity, &Player)>,
    special_effect: Option<ResMut<SpecialCardEffect>>,
    draw_message: MessageWriter<PlayCardDraw>,
    place_message: MessageWriter<PlayCardPlace>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    // obtain window, camera, cursor/world position
    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return; };

    // verify if it has special effect, run if awaiting_target = true, also awating_own_card = true
    if let Some(mut effect) = special_effect {
        if effect.awaiting_target {
            if let Some(clicked_entity) = detect_card_click(&card_query, world_pos) {
                // verify if clicked card belongs to the rival
                if let Ok((_card_entity, _card_tr, card)) = card_query.get(clicked_entity) {
                    if let CardPosition::Hand(hand_owner) = card.position {
                        if hand_owner != turn_query.current_player {
                            // save card selected
                            effect.target_card = Some(clicked_entity);

                            // verify effect (Swap or Shuffle)
                            if matches!(effect.effect_type, Some(SpecialEffect::Swap)) {
                                effect.awaiting_target = false;
                                effect.awaiting_own_card = true;
                                info!(target: "mygame", "Now select one of your cards to swap");
                            } else {
                                effect.target_player = Some(hand_owner);
                                effect.awaiting_target = false;
                            }

                            info!(target: "mygame", "Target player selected for special effect");
                        } else {
                            info!(target: "mygame", "Cannot target your own cards");
                        }
                    }
                }
                info!(target: "mygame", "Click on opponent's cards to select target");
                return;
            }
        } else if effect.awaiting_own_card {
            if let Some(clicked_entity) = detect_card_click(&card_query, world_pos) {
                // verify if clicked card is yours
                if let Ok((_card_entity, _card_tr, card)) = card_query.get(clicked_entity) {
                    if let CardPosition::Hand(owner) = card.position {
                        if owner == turn_query.current_player {
                            effect.own_card = Some(clicked_entity);
                            effect.awaiting_own_card = false;
                        } else {
                            info!(target: "mygame", "Select one of your cards");
                        }
                    }
                }
                info!(target: "mygame", "Click on opponent's cards to select target");
                return;
            }
        }
    }

    // detect click in deck
    if detect_deck_click(world_pos, window) {
        handle_deck_click(deck_query, turn_query, &mut card_query, draw_message);
        return;
    }

    // detect click in graveyard 
    if detect_graveyard_click(world_pos, window) {
        handle_graveyard_click(graveyard_query, turn_query, &mut card_query);
        return;
    }

    // detect click in hand
    if let Some(clicked_entity) = detect_card_click(&card_query, world_pos) {
        handle_card_click(
            clicked_entity, &mut commands, &selected_query, &mut double_click,
            &time, turn_query, &mut card_query, &mut graveyard_query, &player_query, hand_query, windows, place_message
        );
        return;
    }

    // if is not detected any entity
    if detect_card_click(&card_query, world_pos).is_none() && !detect_deck_click(world_pos, window) && !detect_graveyard_click(world_pos, window) {
        // deselect all cards
        for selected_entity in selected_query.iter() {
            commands.entity(selected_entity).remove::<Selected>();
        }
    }
}

// feedback visual when selecting
pub fn card_visual(
    mut card_query: Query<(&mut Transform, Option<&Selected>, &Card), With<Card>>,
    turn_query: Res<Turn>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = windows.single() else { return; };

    for (mut transform, selected, card) in card_query.iter_mut() {
        if let CardPosition::Hand(owner) = card.position {
            if owner == turn_query.current_player {
                let is_sec_player = transform.translation.y > 0.0;
                
                if selected.is_some() {
                    if is_sec_player {
                        transform.translation.y = window.height() * 0.13; // if is second player
                    } else {
                        transform.translation.y = window.height() * -0.13; // if is local player
                    }
                    transform.translation.z = 50.0;
                } else {
                    // return default position
                    let base_y = if is_sec_player {
                        window.height() * 0.15
                    } else {
                        window.height() * -0.15
                    };
                    transform.translation.y = base_y;
                    transform.translation.z = 10.0;
                }
            } else {
                transform.translation.z = 10.0;
            }
        } else {
            if transform.translation.z >= 50.0 {
                transform.translation.z = 10.0;
            }
        }
    }
}

// sets the front and back sprite cards
pub fn configure_texture(
    mut images: ResMut<Assets<Image>>,
    card_handles: Option<Res<CardHandles>>,
    card_back: Option<Res<CardBack>>,
) {
    let Some(card_handles) = card_handles else { return; };
    let Some(card_back) = card_back else { return; };
    
    // front cards
    for handle in &card_handles.0 {
        if let Some(image) = images.get_mut(handle) {
            image.sampler = ImageSampler::nearest();
        }
    }
    
    // back card
    if let Some(image) = images.get_mut(&card_back.0) {
        image.sampler = ImageSampler::nearest();
    }
}

// DETECTION CLICK SYSTEMS
fn detect_card_click(
    card_query: &Query<(Entity, &mut Transform, &mut Card), With<Card>>,
    world_pos: Vec2
) -> Option<Entity> {
    for (card_entity, card_transform, _card_comp) in card_query.iter() {
        let card_pos = card_transform.translation;
        let card_size = Vec2::new(80.0 * 0.7, 120.0 * 0.7);
        
        // Check if the click is inside the card
        if world_pos.x >= card_pos.x - card_size.x / 2.0 
        && world_pos.x <= card_pos.x + card_size.x / 2.0
        && world_pos.y >= card_pos.y - card_size.y / 2.0
        && world_pos.y <= card_pos.y + card_size.y / 2.0 {
            return Some(card_entity);
        }
    }
    None
}

fn detect_deck_click(world_pos: Vec2, window: &Window) -> bool {
    let deck_x = window.width() * 0.15;
    let deck_y = window.height() * 0.0;
    let deck_pos = Vec3::new(deck_x, deck_y, 0.0);
    let deck_size = Vec2::new(80.0, 120.0);
    
    world_pos.x >= deck_pos.x - deck_size.x / 2.0 
    && world_pos.x <= deck_pos.x + deck_size.x / 2.0
    && world_pos.y >= deck_pos.y - deck_size.y / 2.0
    && world_pos.y <= deck_pos.y + deck_size.y / 2.0
}

fn detect_graveyard_click(world_pos: Vec2, window: &Window) -> bool {
    let graveyard_x = window.width() * -0.06;
    let graveyard_y = window.height() * 0.0;
    let graveyard_pos = Vec3::new(graveyard_x, graveyard_y, 0.0);
    let graveyard_size = Vec2::new(80.0, 120.0);
    
    world_pos.x >= graveyard_pos.x - graveyard_size.x / 2.0 
    && world_pos.x <= graveyard_pos.x + graveyard_size.x / 2.0
    && world_pos.y >= graveyard_pos.y - graveyard_size.y / 2.0
    && world_pos.y <= graveyard_pos.y + graveyard_size.y / 2.0
}