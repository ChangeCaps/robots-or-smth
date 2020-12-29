pub mod animation;
pub mod asset_loading;
mod client;
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
pub mod tilemap;
pub mod unit;
pub mod unit_spawnable;

pub use map::*;
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
pub use robots::*;
pub use selection::*;
pub use serde::{Deserialize, Serialize};
pub use server::*;
pub use size::*;
pub use spawnable::*;
pub use tilemap::*;
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
