use crate::*;
use std::net::ToSocketAddrs;

#[derive(Clap)]
pub struct Client {
    #[clap(short, long, default_value = "mmo-tech.ddns.net:35566")]
    ip: String,
    #[clap(long)]
    free_cursor: bool,
}

impl Client {
    pub fn run(self) {
        let grab_cursor = !self.free_cursor;

        App::build()
            // resources
            .init_resource::<SelectedUnits>()
            .init_resource::<NetworkEntityRegistry>()
            .init_resource::<Option<PlayerId>>()
            .init_resource::<MousePosition>()
            .add_resource(NetworkSettings::client())
            .add_resource(self)
            .add_resource(WindowDescriptor {
                cursor_locked: grab_cursor,
                mode: bevy::window::WindowMode::BorderlessFullscreen,
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
            .add_plugin(UnitAnimationPlugin::client())
            .add_plugin(NetworkAudioPlugin::client())
            .add_plugin(InputPlugin)
            .add_plugin(NetworkingPlugin)
            .add_plugin(SpriteShaderPlugin)
            .add_plugin(BarPlugin)
            // assets
            // loaders
            // startup systems
            .add_startup_system(network_setup.system())
            .add_startup_system(setup.system())
            // systems
            .add_system(mouse_position_system.system())
            .add_system(camera_movement_system.system())
            // run
            .run();
    }
}

pub struct MainCamera(pub Entity);

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    client: Res<Client>,
    input_config: Res<Assets<InputConfig>>,
    mut net: ResMut<NetworkResource>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    // we store these so they wont be automatically freed
    let handles = asset_server.load_folder(".").unwrap();
    commands.insert_resource(handles);
    asset_server.watch_for_changes().unwrap();

    let input_config_handle = input_config.get_handle("default_input.input");
    commands.insert_resource(InputResource(input_config_handle));

    let mut camera_transform = Transform::from_scale(Vec3::new(1.0, 1.0, 1.0));
    camera_transform.translation.z = 500.0;

    commands.spawn(CameraUiBundle {
        ..Default::default()
    });

    let camera = commands
        .spawn(Camera2dBundle {
            transform: camera_transform,
            ..Default::default()
        })
        .current_entity()
        .unwrap();

    commands.insert_resource(MainCamera(camera));

    let selection_box = commands
        .spawn(SpriteBundle {
            visible: Visible {
                is_transparent: true,
                is_visible: false,
            },
            material: color_materials.add(Color::rgba(0.0, 1.0, 0.0, 0.4).into()),
            ..Default::default()
        })
        .current_entity()
        .unwrap();

    commands.insert_resource(SelectionBox(selection_box));

    let addr = client.ip.to_socket_addrs().unwrap().next().unwrap();

    info!("connecting at {}", addr);

    net.connect(addr);
}

fn camera_movement_system(
    time: Res<Time>,
    input_config: Res<Assets<InputConfig>>,
    input_resource: Res<InputResource>,
    mouse_position: Res<MousePosition>,
    main_camera: Res<MainCamera>,
    mut query: Query<&mut Transform, With<bevy::render::camera::Camera>>,
) {
    let input_config = if let Some(i) = input_config.get(&input_resource.0) {
        i
    } else {
        return;
    };

    if let Ok(mut transform) = query.get_mut(main_camera.0) {
        let mp = mouse_position.normalized_screen_position();
        let mut movement = Vec3::zero();

        if mp.x.abs() > mouse_position.aspect_ratio() - 0.05 {
            movement += Vec3::new(mp.x, 0.0, 0.0).normalize();
        }

        if mp.y.abs() > 1.0 - 0.05 {
            movement += Vec3::new(0.0, mp.y, 0.0).normalize();
        }

        if movement.length() > 0.0 {
            transform.translation +=
                movement.normalize() * time.delta_seconds() * input_config.camera_scroll_speed;
        }
    }
}
