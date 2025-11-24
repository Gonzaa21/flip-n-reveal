use bevy::prelude::*;
use crate::ui::back_button::component::{ButtonState, BackButton, SetupUI, ButtonImages};
use crate::ui::soundtrack::event::PlayButtonClick;
use crate::game::gamestate::AppState;
use bevy::window::PrimaryWindow;

pub fn spawn_button(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    existing_buttons: Query<Entity, With<BackButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = windows.single() else { return; };

    // spawn if not exists now
    if !existing_buttons.is_empty() {
        return;
    }

    let image_back_normal: Handle<Image> = asset_server.load("textures/ui/buttons/back1.png");
    let image_back_pressed: Handle<Image> = asset_server.load("textures/ui/buttons/back2.png");
    let image_back_hover: Handle<Image> = asset_server.load("textures/ui/buttons/back3.png");

    // Spawn with default scale
    commands.spawn((
        Sprite::from_image(image_back_normal.clone()),
        Transform::from_xyz(window.width() * -0.44, window.height() * 0.4, 10.0).with_scale(Vec3::splat(0.7)),
        ButtonImages {normal: image_back_normal, pressed: image_back_pressed, hovered: image_back_hover},
        ButtonState::Normal,
        BackButton,
        SetupUI,
    ));
}

// clean_menu
pub fn clean_button(
    mut commands: Commands,
    menu_query: Query<Entity, With<SetupUI>>,
) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn();
    }
}



// Handle and detect button
pub fn handle_button(
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    images: Res<Assets<Image>>,
    sprites: Query<&Sprite>,
    mut button_query: Query<(&Transform, &mut ButtonState, Option<&BackButton>, Entity)>,
    mut next_state: ResMut<NextState<AppState>>,
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
        for (transform, mut state, _back, entity) in &mut button_query {
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
        for (transform, mut state, back, entity) in &mut button_query {
            let Ok(sprite) = sprites.get(entity) else { continue; };
            let handle: &Handle<Image> = &sprite.image; // obtain img sprite
            if let Some(image) = images.get(handle) {
                // if the cursor remains over the button: execute action
                if detect_button(world_pos, transform, image) {
                    if back.is_some() {
                        button_click_message.write(PlayButtonClick);
                        next_state.set(AppState::MainMenu);
                        info!(target: "mygame", "Return to menu...");
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

pub fn button_visuals(
    mut button_query: Query<(&ButtonState, &ButtonImages, &mut Sprite), With<BackButton>>,
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
pub fn button_hover(
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