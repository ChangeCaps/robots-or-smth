use crate::*;
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, bevy::reflect::TypeUuid)]
#[uuid = "9b805d6c-a848-4999-ab7e-c657791c139b"]
pub struct Map {
    pub players: HashSet<PlayerId>,
    pub spawns: Vec<Box<dyn Spawnable>>,
    pub player_spawns: HashMap<PlayerId, Vec<Box<dyn Spawnable>>>,
}

impl Map {
    pub fn get_unused(&self, players: &Players) -> Option<PlayerId> {
        for map_player in &self.players {
            if !players.connection_handles.contains_key(map_player) {
                return Some(*map_player);
            }
        }

        None
    }

    pub fn all_connected(&self, players: &Players) -> bool {
        for map_player in &self.players {
            if !players.connection_handles.contains_key(map_player) {
                return false;
            }
        }

        true
    }

    pub fn spawn(&self, players: &Players, spawn_resource: &SpawnResource) {
        for spawn in &self.spawns {
            spawn_resource.spawn(Spawner::new(spawn.clone()));
        }

        for (_, player_id) in &players.player_ids {
            for player_spawn in &self.player_spawns[player_id] {
                spawn_resource.spawn(Spawner::new(player_spawn.clone()));
            }
        }
    }
}

pub struct MapLoader;

ron_loader!(MapLoader, "map" => Map);

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.add_asset_loader(MapLoader);
        app_builder.add_asset::<Map>();
    }
}
