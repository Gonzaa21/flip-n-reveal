use bevy::prelude::*;

#[derive(Component, PartialEq)]
pub struct CardAnimation {
    pub animation_type: AnimationType,
    pub progress: f32,
    pub duration: f32,
    pub state: AnimationState,
    pub original_scale: Vec3,
    pub original_position: Vec3,
    pub original_rotation: Quat,
    pub target_position: Option<Vec3>, 
    pub delay: f32,
    pub delay_elapsed: f32,
}

#[derive(PartialEq)]
pub enum AnimationType {
    Flip,
    Movement,
    Deal,
}

#[derive(PartialEq)]
pub enum AnimationState {
    Animating,
    WaitingToStart, // wait to animate
}