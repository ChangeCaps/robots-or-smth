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

#[derive(TypeUuid)]
#[uuid = "66f796f4-fdac-40a3-a600-54cc920ad367"]
pub struct Spawner {
    spawnable: Box<dyn Spawnable>,
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
pub trait Spawnable: Send + Sync + 'static {
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

        for spawnable in spawn_resource.clear() {
            spawnable.spawn(&mut commands, resources);
        }
    }

    commands.apply(world, resources);
}

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder
            .add_resource(SpawnResource {
                spawnables: Arc::new(Mutex::new(Vec::new())),
            })
            .add_system_to_stage(bevy::app::stage::POST_UPDATE, spawn_system.system());
    }
}
