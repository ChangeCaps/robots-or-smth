use crate::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Position {
    pub position: Vec3,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PositionMessage {
    position: Position,
    network_entity: NetworkEntity,
}

pub fn position_system(mut query: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation = *isometric::ISO_TO_SCREEN * position.position;
        transform.translation.z = transform.translation.y / -256.0;
    }
}

pub fn server_network_position_system(
    mut net: ResMut<NetworkResource>,
    query: Query<(&Position, &NetworkEntity)>,
) {
    for (position, network_entity) in query.iter() {
        let message = PositionMessage {
            position: position.clone(),
            network_entity: network_entity.clone(),
        };

        net.broadcast_message(message);
    }
}

pub fn client_network_position_system(
    mut net: ResMut<NetworkResource>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    mut query: Query<&mut Position>,
) {
    for (_handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(position_message) = channels.recv::<PositionMessage>() {
            if let Some(entity) = network_entity_registry.get(&position_message.network_entity) {
                *query.get_mut(*entity).unwrap() = position_message.position;
            } else {
                /*warn!(
                    "Got position mesage for unregistered network entity: {:?}",
                    position_message.network_entity
                );*/
            }
        }
    }
}

pub struct PositionPlugin(pub bool);

impl PositionPlugin {
    pub fn server() -> Self {
        Self(true)
    }

    pub fn client() -> Self {
        Self(false)
    }
}

impl Plugin for PositionPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        if self.0 {
            app_builder.add_system(server_network_position_system.system());
        } else {
            app_builder.add_system(client_network_position_system.system());
            app_builder.add_system(position_system.system());
        }
    }
}
