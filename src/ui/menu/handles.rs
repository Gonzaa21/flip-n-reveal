use bevy::prelude::*;

use bevy::window::PrimaryWindow;
use crate::ui::menu::component::{ButtonState, ButtonImages, ExitButton, PlayButton, MainMenuUI};
use crate::game::gamestate::AppState;
use crate::ui::soundtrack::event::PlayButtonClick;

// detect click in PLAY and change to Setup, detect click in EXIT and close game
pub fn handle_button_clicks(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut button_query: Query<(&Transform, &mut ButtonState, Option<&PlayButton>, Option<&ExitButton>, Entity)>,
    images: Res<Assets<Image>>,
    sprites: Query<&Sprite>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit_query: MessageWriter<AppExit>,
    mut button_click_message: MessageWriter<PlayButtonClick>,
) {
    // only if click left mouse button
    if mouse_input.just_pressed(MouseButton::Left) {
        // obtain window, camera, cursor/world position
        let Ok(window) = windows.single() else { return; };
        let Ok((camera, camera_transform)) = camera_query.single() else { return; };
        let Some(cursor_pos) = window.cursor_position() else { return; };
        let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return; };

        // verify colision
        for (transform, mut state, _play, _exit, entity) in &mut button_query {
            let Ok(sprite) = sprites.get(entity) else { continue; };
            let handle: &Handle<Image> = &sprite.image; // obtain img sprite
            if let Some(image) = images.get(handle) {
                // if have colision, change to Pressed state
                if detect_button(world_pos, transform, image) {
                    *state = ButtonState::Pressed;
                    return;
                }
            }
        }
    }

    // only when the click is released from the button
    if mouse_input.just_released(MouseButton::Left) {
        
        // obtain window, camera, cursor/world position
        let Ok(window) = windows.single() else { return; };
        let Ok((camera, camera_transform)) = camera_query.single() else { return; };
        let Some(cursor_pos) = window.cursor_position() else { return; };
        let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return; };
        
        // For each button in Pressed state:
        for (transform, mut state, play, exit, entity) in &mut button_query {
            let Ok(sprite) = sprites.get(entity) else { continue; };
            let handle: &Handle<Image> = &sprite.image; // obtain img sprite
            if let Some(image) = images.get(handle) {
                // if the cursor remains over the button: execute action
                if detect_button(world_pos, transform, image) {
                    if play.is_some() {
                        button_click_message.write(PlayButtonClick);
                        next_state.set(AppState::Setup);
                        info!(target: "mygame", "Starting game...");
                    } else if exit.is_some() {
                        exit_query.write(AppExit::Success);
                    }
                    *state = ButtonState::Normal;
                } else {
                    // else, change to normal/hovered state
                    *state = ButtonState::Normal;
                }
            }
        }
    }
}

// auxiliar function - verify if click is inside the button
fn detect_button(
    cursor_pos: Vec2,
    button_transform: &Transform,
    image: &Image,
) -> bool {
    let size = image.size_f32() * button_transform.scale.truncate(); // obtain img size discarding z
    let half_size = size / 2.0; // obtain half size

    let button_pos = button_transform.translation.truncate(); // obtain button position

    cursor_pos.x >= button_pos.x - half_size.x &&
    cursor_pos.x <= button_pos.x + half_size.x &&
    cursor_pos.y >= button_pos.y - half_size.y &&
    cursor_pos.y <= button_pos.y + half_size.y
}

pub fn update_button_visuals(
    mut button_query: Query<(&ButtonState, &ButtonImages, &mut Sprite), With<MainMenuUI>>,
) {
    // verify state of all buttons and update sprite
    for (state, images, mut sprite) in &mut button_query {
        sprite.image = match state {
            ButtonState::Normal => images.normal.clone(),
            ButtonState::Hovered => images.hovered.clone(),
            ButtonState::Pressed => images.pressed.clone(),
        };
    }
}

// detect if mouse is on the button
pub fn update_button_hover(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut button_query: Query<(&Transform, &mut ButtonState, Entity)>,
    sprites: Query<&Sprite>,
    images: Res<Assets<Image>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    // obtain cursor position
    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return; };
    
    // for each button, verify if mouse is on top
    for (transform, mut state, entity) in &mut button_query {
        // execute loop if mouse button left is pressed and have Pressed state
        if mouse_input.pressed(MouseButton::Left) && *state == ButtonState::Pressed {
            continue;
        }

        // obtain button img/sprite
        let Ok(sprite) = sprites.get(entity) else { continue; };
        let Some(image) = images.get(&sprite.image) else { continue; };

        // detect if is hovered
        let is_hovered = detect_button(world_pos, transform, image);

        // update state
        // change state to Hover if is on top
        // change state to Normal if is not on top
        if is_hovered && *state == ButtonState::Normal {
            *state = ButtonState::Hovered;
        } else if !is_hovered && *state == ButtonState::Hovered {
            *state = ButtonState::Normal;
        }
    }
}