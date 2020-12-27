pub mod animation;
pub mod asset_loading;
pub mod input;
pub mod isometric;
pub mod mouse_position;
pub mod position;
pub mod robots;
pub mod selection;
pub mod size;
pub mod spawnable;
pub mod tilemap;
pub mod unit;

#[macro_use]
pub use asset_loading::*;
pub use animation::*;
use bevy::prelude::*;
pub use input::*;
pub use isometric::*;
pub use mouse_position::*;
pub use position::*;
pub use robots::*;
pub use selection::*;
pub use size::*;
pub use spawnable::*;
pub use tilemap::*;
pub use unit::*;

fn main() {
    App::build()
        // plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationPlugin)
        .add_plugin(SpawnPlugin)
        .add_plugin(TileMapPlugin)
        .add_plugin(InputPlugin)
        // resources
        .init_resource::<SelectedUnits>()
        // assets
        .add_asset::<Unit>()
        // loaders
        .add_asset_loader(UnitLoader)
        // systems
        .add_system(unit_action_system.system())
        .add_system(unit_action_execution_system.system())
        .add_system(unit_size_system.system())
        .add_system(unit_selection_system.system())
        .add_system(mouse_position_system.system())
        .add_system(position_system.system())
        .add_system(unit_selection_ring_system.system())
        .add_startup_system(setup.system())
        // run
        .run();
}

fn setup(
    commands: &mut Commands,
    spawn_resource: Res<SpawnResource>,
    asset_server: Res<AssetServer>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    asset_server.watch_for_changes().unwrap();

    let rk550 = rk550::RK550Spawnable {
        unit: "robots/rk550.unit".into(),
        texture: "sprites/sheet.png".into(),
        animation_set: "animations/robot.anim".into(),
    };

    spawn_resource.spawn(rk550.clone());
    spawn_resource.spawn(rk550);

    let mut camera_transform = Transform::from_scale(Vec3::new(1.0, 1.0, 1.0));
    camera_transform.translation.z = 10.0;

    let tile_set_texture: Handle<Texture> = asset_server.load("sprites/grass_tile.png");
    let tilemap: Handle<TileMap> = asset_server.load("maps/default_map.map");
    let tile_set: Handle<TileSet> = asset_server.load("maps/default_tile_set.tile_set");

    let input_config: Handle<InputConfig> = asset_server.load("default_input.input");
    commands.insert_resource(InputResource(input_config));

    commands.spawn(TileMapBundle {
        tile_set,
        tilemap,
        material: color_materials.add(tile_set_texture.into()),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
        ..Default::default()
    });

    let camera = commands
        .spawn(Camera2dBundle {
            transform: camera_transform,
            ..Default::default()
        })
        .current_entity()
        .unwrap();

    commands.insert_resource(MousePosition::new(camera));
}
