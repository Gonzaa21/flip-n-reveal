use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::game::card::component::{Card, PreviousCardPosition, CardPosition, PreviousTranslation};
use crate::ui::card_animation::component::{CardAnimation, AnimationState, AnimationType};
use crate::game::graveyard::component::Graveyard;

// detect card movement
pub fn detect_movement(
    mut commands: Commands,
    mut card_query: Query<(Entity, &Card, &mut Transform, Option<&PreviousCardPosition>, Option<&PreviousTranslation>)>,
    animation_query: Query<&CardAnimation>,
    windows: Query<&Window, With<PrimaryWindow>>,
    graveyard_query: Query<&Graveyard>,
) {
    for (entity, card, mut transform, previous_pos, previous_translation) in card_query.iter_mut() {
        // verify if is animating already
        if animation_query.get(entity).is_ok() {
            continue;
        }
        
        // obtain previous card position
        let prev_pos = match previous_pos {
            Some(p) => &p.0,
            None => {
                // save position
                commands.entity(entity).insert(PreviousCardPosition(card.position.clone()));
                commands.entity(entity).insert(PreviousTranslation(transform.translation));
                continue;
            }
        };

        // obtain previous card translation
        let prev_trans = match previous_translation {
            Some(p) => p.0,
            None => {
                commands.entity(entity).insert(PreviousTranslation(transform.translation));
                continue;
            }
        };
        
        // verify if have changes
        let position_changed = prev_pos != &card.position;
        let translation_changed = prev_trans != transform.translation;

        if !position_changed && !translation_changed {
            continue; // do nothing if not changed
        }

        // detect when the card must animate
        let should_animate_movement = match (prev_pos, &card.position) {
            (CardPosition::Deck, CardPosition::DrawnCard(_)) => true,
            (CardPosition::DrawnCard(_), CardPosition::Graveyard) => true,
            (CardPosition::DrawnCard(_), CardPosition::Hand(_)) => true,
            (CardPosition::Hand(_), CardPosition::Graveyard) => true,
            (CardPosition::Graveyard, CardPosition::DrawnCard(_)) => true,
            _ => false,
        };
    
        if should_animate_movement {
            let Ok(window) = windows.single() else { continue; };

            let target_pos = match &card.position {
                CardPosition::DrawnCard(_) => Vec3::new(window.width() * 0.1, 0.0, 30.0),
                CardPosition::Graveyard => {
                    // Calculate stacking offset for graveyard
                    if let Ok(graveyard) = graveyard_query.single() {
                        let stack_index = graveyard.cards.iter()
                            .position(|&e| e == entity)
                            .unwrap_or(graveyard.cards.len() - 1) as f32;
                        
                        let max_stack = 15.0; // max cards to stack
                        
                        // stack offset effect
                        let (offset_x, offset_y) = if stack_index < max_stack {
                            (stack_index * 0.5, stack_index * 0.3)
                        } else {
                            (max_stack * 0.5, max_stack * 0.3)
                        };
                        
                        Vec3::new(
                            window.width() * -0.06 + offset_x,
                            window.height() * 0.0 + offset_y,
                            10.0 + stack_index
                        )
                    } else {
                        // keep position if it is not among the first
                        Vec3::new(window.width() * -0.06, 0.0, 10.0)
                    }
                }
                CardPosition::Hand(_) => {
                    // to hand, keep original position
                    transform.translation
                }
                _ => transform.translation,
                
            };

            transform.translation = prev_trans; // move the card to original position before animating

            // insert movement animation
            commands.entity(entity).insert(CardAnimation {
                animation_type: AnimationType::Movement,
                progress: 0.0,
                duration: 0.25,
                state: AnimationState::Animating,
                original_position: prev_trans,
                original_scale: transform.scale,
                original_rotation: transform.rotation,
                target_position: Some(target_pos),
                delay: 0.0,
                delay_elapsed: 0.0,
            });
        }

        // update previous position/translation
        if position_changed {
            commands.entity(entity).insert(PreviousCardPosition(card.position.clone()));
        }
        if translation_changed {
            commands.entity(entity).insert(PreviousTranslation(transform.translation));
        }
    }
}

pub fn animate_movement(
    mut commands: Commands,
    mut card_query: Query<(Entity, &mut Transform, &mut CardAnimation, &mut Card)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut animation, mut card) in card_query.iter_mut() {
        if animation.animation_type == AnimationType::Movement {
            if let Some(target) = animation.target_position {
                
                // Interpolate from original_position to new position
                let t = animation.progress;
                transform.translation = animation.original_position.lerp(target, t);

                // update progress
                animation.progress += time.delta_secs() / animation.duration;
                
                // when the progress has finished, remove card animation
                if animation.progress >= 1.0 {
                    card.is_being_dealt = false;
                    transform.translation = target;

                    // if card position is graveyard, use random rotation
                    if matches!(card.position, CardPosition::Graveyard) {
                        let random_rotation = (rand::random::<f32>() - 0.5) * 0.15;
                        transform.rotation = Quat::from_rotation_z(random_rotation);
                    }

                    commands.entity(entity).remove::<CardAnimation>();
                }
            }
        }
    }
}