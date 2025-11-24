use bevy::prelude::*;

// Eventos para reproducir efectos de sonido
#[derive(Event, Message)]
pub struct PlayCardDraw;

#[derive(Event, Message)]
pub struct PlayCardPlace;

#[derive(Event, Message)]
pub struct PlayButtonClick;