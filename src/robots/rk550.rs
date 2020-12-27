use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Clone)]
pub struct RK550Spawnable {
    pub unit: String,
    pub texture: String,
    pub animation_set: String,
}

#[typetag::serde]
impl Spawnable for RK550Spawnable {
    fn spawn(&self, commands: &mut Commands, resources: &Resources) -> Entity {
        let asset_server = resources.get::<AssetServer>().unwrap();
        let mut texture_atlases = resources.get_mut::<Assets<TextureAtlas>>().unwrap();
        let mut textures = resources.get_mut::<Assets<ColorMaterial>>().unwrap();

        let selection_circle: Handle<Texture> = asset_server.load("sprites/selection.png");

        let unit: Handle<Unit> = asset_server.load(self.unit.as_str());
        let texture = asset_server.load(self.texture.as_str());
        let animation_set = asset_server.load(self.animation_set.as_str());

        let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(96.0, 128.0), 24 * 8, 1);
        let animator = Animator::new(animation_set, "walk_up");

        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: texture_atlases.add(texture_atlas),
                ..Default::default()
            })
            .with(ActionQueue {
                actions: VecDeque::new(),
            })
            .with(animator)
            .with(unit)
            .with(Position {
                position: Vec3::zero(),
            })
            .with_children(|parent| {
                parent.spawn(SpriteBundle {
                    material: textures.add(selection_circle.into()),
                    ..Default::default()
                });
            })
            .current_entity()
            .unwrap()
    }
}
