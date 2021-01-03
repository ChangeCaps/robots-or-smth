use crate::*;
use std::net::ToSocketAddrs;

#[derive(Clap)]
pub struct Client {
    #[clap(short, long, default_value = "mmo-tech.ddns.net:35566")]
    ip: String,
}

impl Client {
    pub fn run(self) {
        App::build()
            // resources
            .init_resource::<SelectedUnits>()
            .init_resource::<NetworkEntityRegistry>()
            .init_resource::<Option<PlayerId>>()
            .add_resource(NetworkSettings::client())
            .add_resource(self)
            .add_resource(WindowDescriptor {
                cursor_locked: true,
                mode: bevy::window::WindowMode::Fullscreen { use_size: false },
                ..Default::default()
            })
            // plugins
            .add_plugins(DefaultPlugins)
            .add_plugin(UnitPlugin::client())
            .add_plugin(SpawnPlugin::client())
            .add_plugin(AnimationPlugin::client())
            .add_plugin(TileMapPlugin::client())
            .add_plugin(ConnectionPlugin::client())
            .add_plugin(PositionPlugin::client())
            .add_plugin(InputPlugin)
            .add_plugin(NetworkingPlugin)
            .add_plugin(SpriteShaderPlugin)
            // assets
            // loaders
            // startup systems
            .add_startup_system(network_setup.system())
            .add_startup_system(setup.system())
            // systems
            .add_system(mouse_position_system.system())
            // run
            .run();
    }
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    client: Res<Client>,
    mut net: ResMut<NetworkResource>,
) {
    asset_server.watch_for_changes().unwrap();

    let input_config_handle: Handle<InputConfig> = asset_server.load("default_input.input");
    commands.insert_resource(InputResource(input_config_handle));

    let mut camera_transform = Transform::from_scale(Vec3::new(1.0, 1.0, 1.0));
    camera_transform.translation.z = 10.0;

    let camera = commands
        .spawn(Camera2dBundle {
            transform: camera_transform,
            ..Default::default()
        })
        .current_entity()
        .unwrap();

    commands.insert_resource(MousePosition::new(camera));

    let addr = client.ip.to_socket_addrs().unwrap().next().unwrap();

    info!("connecting at {}", addr);

    net.connect(addr);
}
