use bevy::prelude::*;
use std::sync::{Arc, Mutex};

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
