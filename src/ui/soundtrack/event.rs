use bevy::prelude::*;

// messages/events for play effects
#[derive(Event, Message)]
pub struct PlayCardDraw;

#[derive(Event, Message)]
pub struct PlayCardPlace;

#[derive(Event, Message)]
pub struct PlayButtonClick;