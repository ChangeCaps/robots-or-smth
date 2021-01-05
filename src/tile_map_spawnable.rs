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
        let network_settings = resources.get::<NetworkSettings>().unwrap();

        let tile_maps = resources.get::<Assets<TileMap>>().unwrap();
        let tile_sets = resources.get::<Assets<TileSet>>().unwrap();

        let tile_map = tile_maps.get_handle(self.tile_map.as_str());
        let tile_set = tile_sets.get_handle(self.tile_set.as_str());

        commands.spawn(TileMapBundle {
            tile_map,
            tile_set,
            ..Default::default()
        });

        if network_settings.is_client() {
            let textures = resources.get::<Assets<Texture>>().unwrap();
            let mut color_materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
            let texture = textures.get_handle(self.texture.as_str());
            let material: Handle<ColorMaterial> = color_materials.add(texture.into());

            commands.with(material);
        }

        commands.current_entity().unwrap()
    }
}
