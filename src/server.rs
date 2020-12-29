use crate::*;

#[derive(Clap)]
pub struct Server {
    #[clap(short, long, default_value = "maps/default_map.map")]
    map: String,
    #[clap(short, long, default_value = "35566")]
    port: u16,
}

impl Server {
    pub fn run(self) {
        App::build()
            // resources
            .init_resource::<NetworkEntityRegistry>()
            .add_resource(self)
            .add_resource(NetworkSettings::server())
            .add_resource(bevy::app::ScheduleRunnerSettings::run_loop(
                std::time::Duration::from_secs_f64(1.0 / 60.0),
            ))
            // plugins
            .add_plugins(MinimalPlugins)
            .add_plugin(bevy::asset::AssetPlugin)
            .add_plugin(bevy::transform::TransformPlugin)
            .add_plugin(bevy::reflect::ReflectPlugin)
            .add_plugin(bevy::log::LogPlugin)
            .add_plugin(SpawnPlugin::server())
            .add_plugin(AnimationPlugin::server())
            .add_plugin(TileMapPlugin::server())
            .add_plugin(UnitPlugin::server())
            .add_plugin(ConnectionPlugin::server())
            .add_plugin(PositionPlugin::server())
            .add_plugin(MapPlugin)
            .add_plugin(NetworkingPlugin)
            // startup systems
            .add_startup_system(network_setup.system())
            .add_startup_system(setup.system())
            // systems
            .add_system(handle_connection_system.system())
            .run();
    }
}

fn setup(
    commands: &mut Commands,
    mut net: ResMut<NetworkResource>,
    server: Res<Server>,
    asset_server: Res<AssetServer>,
) {
    asset_server.watch_for_changes().unwrap();

    let map_handle: Handle<Map> = asset_server.load(server.map.as_str());
    commands.insert_resource(map_handle);

    let ip_address = bevy_networking_turbulence::find_my_ip_address().unwrap();
    let socket_address = std::net::SocketAddr::new(ip_address, server.port);
    info!("Starting server at: {}", socket_address);
    net.listen(socket_address);
}

fn handle_connection_system(
    mut net: ResMut<NetworkResource>,
    mut reader: Local<EventReader<NetworkEvent>>,
    mut players: ResMut<Players>,
    maps: Res<Assets<Map>>,
    map_handle: Res<Handle<Map>>,
    spawn_resource: Res<SpawnResource>,
    events: Res<Events<NetworkEvent>>,
) {
    for event in reader.iter(&events) {
        match event {
            NetworkEvent::Connected(handle) => {
                info!("connection at {}", handle);

                let map = maps.get(&*map_handle).unwrap();

                if let Some(player_id) = map.get_unused(&players) {
                    players.insert(player_id, *handle);
                    net.send_message(*handle, ConnectionMessage::Server(player_id))
                        .unwrap();

                    if map.all_connected(&players) {
                        info!("all players connected, spawning map");
                        map.spawn(&players, &spawn_resource);
                    }
                } else {
                    warn!("not enough players available for map");
                }
            }
            NetworkEvent::Disconnected(handle) => {
                warn!("disconnected at {}", handle);
            }
            _ => {}
        }
    }
}
