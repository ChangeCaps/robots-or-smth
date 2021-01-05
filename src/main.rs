pub mod animation;
pub mod asset_loading;
pub mod bar;
pub mod behaviour;
mod client;
pub mod command;
pub mod connection;
pub mod input;
pub mod isometric;
pub mod map;
pub mod mouse_position;
pub mod network;
pub mod position;
pub mod robots;
pub mod selection;
pub mod server;
pub mod size;
pub mod spawnable;
pub mod sprite_shader;
pub mod tile_map;
pub mod tile_map_spawnable;
pub mod unit;
pub mod unit_animation;
pub mod unit_spawnable;

pub use bar::*;
pub use behaviour::*;
pub use command::*;
pub use map::*;
pub use sprite_shader::*;
pub use tile_map_spawnable::*;
pub use unit_animation::*;
pub use unit_spawnable::*;
#[macro_use]
pub use asset_loading::*;
pub use animation::*;
pub use bevy::prelude::*;
pub use bevy_networking_turbulence::*;
use clap::Clap;
use client::*;
pub use connection::*;
pub use input::*;
pub use isometric::*;
pub use mouse_position::*;
pub use network::*;
pub use position::*;
pub use rand::prelude::*;
pub use robots::*;
pub use selection::*;
pub use serde::{Deserialize, Serialize};
pub use server::*;
pub use size::*;
pub use spawnable::*;
pub use tile_map::*;
pub use unit::*;

#[derive(Clap)]
enum RunMode {
    Server(Server),
    Client(Client),
}

#[derive(Clap)]
#[clap(author = "Hjalte C. Nannestad", version = clap::crate_version!())]
struct Options {
    #[clap(subcommand)]
    run_mode: RunMode,
}

fn main() {
    simple_logger::SimpleLogger::from_env()
        .init()
        .expect("A logger was already initialized");

    let options = Options::parse();

    match options.run_mode {
        RunMode::Server(server) => {
            server.run();
        }
        RunMode::Client(client) => {
            client.run();
        }
    }
}
