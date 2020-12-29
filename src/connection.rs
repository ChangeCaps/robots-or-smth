use crate::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u64);

#[derive(Clone, Debug)]
pub struct Owner(pub PlayerId);

#[derive(Default)]
pub struct Players {
    pub player_ids: HashMap<ConnectionHandle, PlayerId>,
    pub connection_handles: HashMap<PlayerId, ConnectionHandle>,
}

impl Players {
    pub fn insert(&mut self, player_id: PlayerId, connection_handle: ConnectionHandle) {
        self.player_ids.insert(connection_handle, player_id);
        self.connection_handles.insert(player_id, connection_handle);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConnectionMessage {
    Client,
    Server(PlayerId),
}

fn client_connection_system(
    mut timer: Local<Timer>,
    time: Res<Time>,
    mut net: ResMut<NetworkResource>,
    mut player_id: ResMut<Option<PlayerId>>,
) {
    timer.set_duration(1.0);
    timer.set_repeating(true);
    timer.tick(time.delta_seconds());

    if timer.just_finished() {
        net.broadcast_message(ConnectionMessage::Client);
    }

    for (_handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(connection_message) = channels.recv::<ConnectionMessage>() {
            match connection_message {
                ConnectionMessage::Client => {
                    error!("Server connected with {:?}", connection_message);
                }
                ConnectionMessage::Server(id) => {
                    info!("Server responded and assigned player_id: {:?}", id);

                    *player_id = Some(id);
                }
            }
        }
    }
}

fn server_connection_system(mut net: ResMut<NetworkResource>) {
    for (_handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(connection_message) = channels.recv::<ConnectionMessage>() {
            match connection_message {
                ConnectionMessage::Client => {}
                ConnectionMessage::Server(_) => {
                    error!("Client connected with {:?}", connection_message);
                }
            }
        }
    }
}

pub struct ConnectionPlugin(pub bool);

impl ConnectionPlugin {
    pub fn server() -> Self {
        Self(true)
    }

    pub fn client() -> Self {
        Self(false)
    }
}

impl Plugin for ConnectionPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        if self.0 {
            app_builder.init_resource::<Players>();
            app_builder.add_system(server_connection_system.system());
        } else {
            app_builder.add_system(client_connection_system.system());
        }
    }
}
