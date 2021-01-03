use crate::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TileMapSpawnable {
    tile_map: String,
    tile_set: String,
    texture: String,
}

#[typetag::serde]
impl Spawnable for TileMapSpawnable {
    fn spawn(&self, commands: &mut Commands, resources: &Resources) -> Entity {
        let asset_server = resources.get::<AssetServer>().unwrap();
        let network_settings = resources.get::<NetworkSettings>().unwrap();

        let tile_map: Handle<TileMap> = asset_server.load(self.tile_map.as_str());
        let tile_set: Handle<TileSet> = asset_server.load(self.tile_set.as_str());

        commands.spawn(TileMapBundle {
            tile_map,
            tile_set,
            ..Default::default()
        });

        if network_settings.is_client() {
            let mut color_materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
            let texture: Handle<Texture> = asset_server.load(self.texture.as_str());
            let material: Handle<ColorMaterial> = color_materials.add(texture.into());

            commands.with(material);
        }

        commands.current_entity().unwrap()
    }
}
