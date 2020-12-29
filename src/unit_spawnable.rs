use crate::*;
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnitSpawnable {
    pub position: Vec3,
    pub owner: PlayerId,
    pub unit: String,
    pub texture: String,
    pub animation_set: String,
}

#[typetag::serde]
impl Spawnable for UnitSpawnable {
    fn spawn(&self, commands: &mut Commands, resources: &Resources) -> Entity {
        let asset_server = resources.get::<AssetServer>().unwrap();
        let network_settings = resources.get::<NetworkSettings>().unwrap();

        let unit: Handle<Unit> = asset_server.load(self.unit.as_str());
        let animation_set = asset_server.load(self.animation_set.as_str());

        let animator = Animator::new(animation_set, "walk_up");

        commands
            .spawn((ActionQueue {
                actions: VecDeque::new(),
            },))
            .with(animator)
            .with(unit)
            .with(Position {
                position: self.position,
            })
            .with(Owner(self.owner));

        if !network_settings.is_server {
            let mut texture_atlases = resources.get_mut::<Assets<TextureAtlas>>().unwrap();
            let mut textures = resources.get_mut::<Assets<ColorMaterial>>().unwrap();

            let texture = asset_server.load(self.texture.as_str());
            let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(96.0, 128.0), 24 * 8, 1);
            let selection_circle: Handle<Texture> = asset_server.load("sprites/selection.png");

            commands
                .with_bundle(SpriteSheetBundle {
                    texture_atlas: texture_atlases.add(texture_atlas),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(SpriteBundle {
                        material: textures.add(selection_circle.into()),
                        ..Default::default()
                    });
                });
        }

        commands.current_entity().unwrap()
    }
}
