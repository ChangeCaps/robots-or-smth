pub mod animation;
pub mod robots;
pub mod spawnable;
pub mod unit;

pub use animation::*;
use bevy::prelude::*;
pub use robots::*;
pub use spawnable::*;
pub use unit::*;

fn main() {
    App::build()
        // plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationPlugin)
        .add_plugin(SpawnPlugin)
        // systems
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    commands: &mut Commands,
    spawn_resource: Res<SpawnResource>,
    asset_server: Res<AssetServer>,
) {
    asset_server.watch_for_changes().unwrap();

    let rk550 = rk550::RK550Spawnable {
        texture: "sprites/sheet.png".into(),
        animation_set: "animations/robot.anim".into(),
    };

    spawn_resource.spawn(rk550);

    commands.spawn(Camera2dBundle {
        transform: Transform::from_scale(Vec3::new(0.75, 0.75, 0.75)),
        ..Default::default()
    });
}
