use crate::*;
use bevy::prelude::*;

#[derive(Serialize, Deserialize)]
pub enum InputType {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

impl InputType {
    pub fn just_pressed(&self, keyboard: &Input<KeyCode>, mouse: &Input<MouseButton>) -> bool {
        match self {
            InputType::Keyboard(keycode) => keyboard.just_pressed(keycode.clone()),
            InputType::Mouse(mouse_button) => mouse.just_pressed(mouse_button.clone()),
        }
    }

    pub fn pressed(&self, keyboard: &Input<KeyCode>, mouse: &Input<MouseButton>) -> bool {
        match self {
            InputType::Keyboard(keycode) => keyboard.pressed(keycode.clone()),
            InputType::Mouse(mouse_button) => mouse.pressed(mouse_button.clone()),
        }
    }

    pub fn just_released(&self, keyboard: &Input<KeyCode>, mouse: &Input<MouseButton>) -> bool {
        match self {
            InputType::Keyboard(keycode) => keyboard.just_released(keycode.clone()),
            InputType::Mouse(mouse_button) => mouse.just_released(mouse_button.clone()),
        }
    }
}

#[derive(bevy::reflect::TypeUuid, serde::Serialize, serde::Deserialize)]
#[uuid = "f1a219cb-f50d-460f-b688-c55e2ebd28ee"]
pub struct InputConfig {
    pub select: InputType,
    pub add_to_selection: InputType,
    pub queue_actions: InputType,
    pub move_command: InputType,
    pub attack_move_command: InputType,
    pub camera_scroll_speed: f32,
}

#[derive(bevy::reflect::TypeUuid)]
#[uuid = "ab6dc80e-61cd-4d9e-9757-384f3b7da22b"]
pub struct InputResource(pub Handle<InputConfig>);

pub struct InputConfigLoader;

ron_loader!(InputConfigLoader, "input" => InputConfig);

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.add_asset::<InputConfig>();
        app_builder.add_asset_loader(InputConfigLoader);
    }
}
