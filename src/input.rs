use crate::*;
use bevy::prelude::*;

#[derive(bevy::reflect::TypeUuid, serde::Serialize, serde::Deserialize)]
#[uuid = "f1a219cb-f50d-460f-b688-c55e2ebd28ee"]
pub struct InputConfig {
    pub add_to_selection: KeyCode,
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
