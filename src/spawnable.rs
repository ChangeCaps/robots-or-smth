use crate::*;
use bevy::prelude::*;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use std::sync::{Arc, Mutex};

pub struct SpawnableLoader;

impl AssetLoader for SpawnableLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let spawnable: Box<dyn Spawnable> = ron::de::from_bytes(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(Spawner::new(spawnable)));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["spn"]
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SpawnMessage {
    spawnable: Box<dyn Spawnable>,
    entity: NetworkEntity,
}

pub trait SpawnableClone {
    fn box_clone(&self) -> Box<dyn Spawnable>;
}

impl<T: Spawnable + Clone> SpawnableClone for T {
    fn box_clone(&self) -> Box<dyn Spawnable> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Spawnable> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[typetag::serde]
impl Spawnable for SpawnMessage {
    fn spawn(&self, commands: &mut Commands, resources: &Resources) -> Entity {
        let entity = self.spawnable.spawn(commands, resources);

        resources
            .get_mut::<NetworkEntityRegistry>()
            .unwrap()
            .insert(self.entity, entity);

        commands.insert_one(entity, self.entity);
        entity
    }
}

#[derive(TypeUuid, serde::Serialize, serde::Deserialize, Debug, Clone)]
#[uuid = "66f796f4-fdac-40a3-a600-54cc920ad367"]
pub struct Spawner {
    spawnable: Box<dyn Spawnable>,
}

#[typetag::serde]
impl Spawnable for Spawner {
    fn spawn(&self, commands: &mut Commands, resources: &Resources) -> Entity {
        self.spawnable.spawn(commands, resources)
    }
}

impl AsRef<Box<dyn Spawnable>> for Spawner {
    fn as_ref(&self) -> &Box<dyn Spawnable> {
        &self.spawnable
    }
}

impl Spawner {
    pub fn new(spawnable: Box<dyn Spawnable>) -> Self {
        Self { spawnable }
    }
}

#[typetag::serde(tag = "spawnable")]
pub trait Spawnable: SpawnableClone + std::fmt::Debug + Send + Sync + 'static {
    fn spawn(&self, commands: &mut Commands, resources: &Resources) -> Entity;
}

pub struct SpawnResource {
    spawnables: Arc<Mutex<Vec<Box<dyn Spawnable>>>>,
}

impl SpawnResource {
    pub fn spawn<T: Spawnable>(&self, spawnable: T) {
        self.spawnables.lock().unwrap().push(Box::new(spawnable));
    }

    pub fn clear(&self) -> Vec<Box<dyn Spawnable>> {
        std::mem::replace(&mut *self.spawnables.lock().unwrap(), Vec::new())
    }
}

pub fn spawn_system(world: &mut World, resources: &mut Resources) {
    let mut commands = Commands::default();
    commands.set_entity_reserver(world.get_entity_reserver());

    {
        let spawn_resource = resources.get::<SpawnResource>().unwrap();
        let network_settings = resources.get::<NetworkSettings>().unwrap();
        let mut net = resources.get_mut::<NetworkResource>().unwrap();

        for spawnable in spawn_resource.clear() {
            let network_entity = {
                let mut network_entity_registry =
                    resources.get_mut::<NetworkEntityRegistry>().unwrap();
                network_entity_registry.generate_entity()
            };

            let message = SpawnMessage {
                spawnable,
                entity: network_entity,
            };

            let entity = message.spawn(&mut commands, resources);

            if network_settings.is_server {
                net.broadcast_message(message);
            }

            let mut network_entity_registry = resources.get_mut::<NetworkEntityRegistry>().unwrap();
            network_entity_registry.insert(network_entity, entity);
        }
    }

    commands.apply(world, resources);
}

pub fn network_spawn_system(mut net: ResMut<NetworkResource>, spawn_resource: Res<SpawnResource>) {
    for (_handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(spawn_message) = channels.recv::<SpawnMessage>() {
            spawn_resource.spawn(spawn_message);
        }
    }
}

pub struct SpawnPlugin(pub bool);

impl SpawnPlugin {
    pub fn server() -> Self {
        Self(true)
    }

    pub fn client() -> Self {
        Self(false)
    }
}

impl Plugin for SpawnPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.add_resource(SpawnResource {
            spawnables: Arc::new(Mutex::new(Vec::new())),
        });

        app_builder.add_system_to_stage(bevy::app::stage::POST_UPDATE, spawn_system.system());

        if !self.0 {
            app_builder
                .add_system_to_stage(bevy::app::stage::PRE_UPDATE, network_spawn_system.system());
        }
    }
}
