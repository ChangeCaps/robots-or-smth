pub mod animation;
pub mod spawnable;

pub use animation::*;
use bevy::prelude::*;
pub use spawnable::*;

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

fn setup(commands: &mut Commands) {}
