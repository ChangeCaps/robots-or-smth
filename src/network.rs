use crate::*;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
pub struct NetworkEntity(pub u64);

pub struct NetworkEntityRegistry {
    entities: HashMap<NetworkEntity, Entity>,
    next_entity: NetworkEntity,
}

impl Default for NetworkEntityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkEntityRegistry {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_entity: NetworkEntity(0),
        }
    }

    pub fn get(&self, network_entity: &NetworkEntity) -> Option<&Entity> {
        self.entities.get(network_entity)
    }

    pub fn generate_entity(&mut self) -> NetworkEntity {
        let network_entity = self.next_entity;
        self.next_entity.0 += 1;
        network_entity
    }

    pub fn insert(&mut self, network_entity: NetworkEntity, entity: Entity) {
        self.entities.insert(network_entity, entity);
    }

    pub fn add(&mut self, entity: Entity) -> NetworkEntity {
        let network_entity = self.generate_entity();
        self.entities.insert(network_entity, entity);
        network_entity
    }
}

#[derive(bevy::reflect::TypeUuid)]
#[uuid = "0c0366f6-d3f3-43e3-b6cb-6b45bbe12627"]
pub struct NetworkSettings {
    pub is_server: bool,
}

impl NetworkSettings {
    pub fn server() -> Self {
        Self { is_server: true }
    }

    pub fn client() -> Self {
        Self { is_server: false }
    }
}

pub fn network_setup(mut net: ResMut<NetworkResource>) {
    net.set_channels_builder(|builder: &mut ConnectionChannelsBuilder| {
        builder
            .register::<ConnectionMessage>(CONNECTION_MESSAGE_SETTINGS)
            .unwrap();

        builder
            .register::<AnimatorMessage>(ANIMATOR_MESSAGE_SETTINGS)
            .unwrap();

        builder
            .register::<SpawnMessage>(SPAWNER_MESSAGE_SETTINGS)
            .unwrap();

        builder
            .register::<ActionMessage>(ACTION_MESSAGE_SETTINGS)
            .unwrap();

        builder
            .register::<PositionMessage>(POSITION_MESSAGE_SETTINGS)
            .unwrap();
    });
}

pub const CONNECTION_MESSAGE_SETTINGS: MessageChannelSettings = MessageChannelSettings {
    channel: 0,
    channel_mode: MessageChannelMode::Reliable {
        reliability_settings: ReliableChannelSettings {
            bandwidth: 4096,
            recv_window_size: 1024,
            send_window_size: 1024,
            burst_bandwidth: 1024,
            init_send: 512,
            wakeup_time: Duration::from_millis(100),
            initial_rtt: Duration::from_millis(200),
            max_rtt: Duration::from_secs(2),
            rtt_update_factor: 0.1,
            rtt_resend_factor: 1.5,
        },
        max_message_len: 1024,
    },
    message_buffer_size: 64,
    packet_buffer_size: 64,
};

const ANIMATOR_MESSAGE_SETTINGS: MessageChannelSettings = MessageChannelSettings {
    channel: 1,
    channel_mode: MessageChannelMode::Unreliable,
    message_buffer_size: 64,
    packet_buffer_size: 64,
};

const SPAWNER_MESSAGE_SETTINGS: MessageChannelSettings = MessageChannelSettings {
    channel: 2,
    channel_mode: MessageChannelMode::Reliable {
        reliability_settings: ReliableChannelSettings {
            bandwidth: 4096,
            recv_window_size: 1024,
            send_window_size: 1024,
            burst_bandwidth: 1024,
            init_send: 512,
            wakeup_time: Duration::from_millis(100),
            initial_rtt: Duration::from_millis(200),
            max_rtt: Duration::from_secs(2),
            rtt_update_factor: 0.1,
            rtt_resend_factor: 1.5,
        },
        max_message_len: 1024,
    },
    message_buffer_size: 64,
    packet_buffer_size: 64,
};

const ACTION_MESSAGE_SETTINGS: MessageChannelSettings = MessageChannelSettings {
    channel: 3,
    channel_mode: MessageChannelMode::Reliable {
        reliability_settings: ReliableChannelSettings {
            bandwidth: 4096,
            recv_window_size: 1024,
            send_window_size: 1024,
            burst_bandwidth: 1024,
            init_send: 512,
            wakeup_time: Duration::from_millis(100),
            initial_rtt: Duration::from_millis(200),
            max_rtt: Duration::from_secs(2),
            rtt_update_factor: 0.1,
            rtt_resend_factor: 1.5,
        },
        max_message_len: 1024,
    },
    message_buffer_size: 64,
    packet_buffer_size: 64,
};

const POSITION_MESSAGE_SETTINGS: MessageChannelSettings = MessageChannelSettings {
    channel: 4,
    channel_mode: MessageChannelMode::Unreliable,
    message_buffer_size: 64,
    packet_buffer_size: 64,
};
