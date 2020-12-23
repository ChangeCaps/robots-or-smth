use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RK550Spawnable {
    pub texture: String,
    pub animation_set: String,
}

#[typetag::serde]
impl Spawnable for RK550Spawnable {
    fn spawn(&self, commands: &mut Commands, resources: &Resources) -> Entity {
        let asset_server = resources.get::<AssetServer>().unwrap();
        let mut texture_atlases = resources.get_mut::<Assets<TextureAtlas>>().unwrap();

        let texture = asset_server.load(self.texture.as_str());
        let animation_set = asset_server.load(self.animation_set.as_str());

        let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(96.0, 96.0), 24 * 8, 1);
        let animator = Animator::new(animation_set, "walk_up");

        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: texture_atlases.add(texture_atlas),
                ..Default::default()
            })
            .with(animator)
            .current_entity()
            .unwrap()
    }
}
