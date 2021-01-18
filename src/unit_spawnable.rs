use crate::*;
use bevy::render::pipeline::{RenderPipeline, RenderPipelines};
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnitSpawnable {
    pub position: Vec3,
    pub owner: PlayerId,
    pub unit: String,
}

#[typetag::serde]
impl Spawnable for UnitSpawnable {
    fn spawn(&self, commands: &mut Commands, resources: &Resources) -> Entity {
        let network_settings = resources.get::<NetworkSettings>().unwrap();

        let units = resources.get::<Assets<Unit>>().unwrap();
        let animation_sets = resources.get::<Assets<AnimationSet>>().unwrap();

        let unit_handle = units.get_handle(self.unit.as_str());
        let unit = units.get(&unit_handle).unwrap();
        let animation_set_handle = animation_sets.get_handle(unit.animation_set.as_str());
        let animation_set = animation_sets.get(&animation_set_handle).unwrap();

        let animator = Animator::new(animation_set_handle, "idle_up");

        commands.spawn((
            CommandQueue::new(),
            animator,
            unit_handle,
            unit.instance(),
            Position {
                position: self.position,
            },
            UnitDirection::Down,
            Owner(self.owner),
        ));

        if network_settings.is_server {
            let unit_animation_sets = resources.get::<Assets<UnitAnimationSet>>().unwrap();
            let unit_animation_set =
                unit_animation_sets.get_handle(unit.unit_animation_set.as_str());

            commands
                .with(unit_animation_set)
                .with(UnitAnimator::new("idle".into()));
        } else {
            let textures = resources.get::<Assets<Texture>>().unwrap();
            let mut texture_atlases = resources.get_mut::<Assets<TextureAtlas>>().unwrap();
            let mut color_materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();

            // FIXME: this is bad, will panic if no texture 0 is present
            let animation_texture =
                &animation_set.animation_textures[&0];

            let texture = textures.get_handle(animation_texture.path.as_str());
            let texture_atlas = TextureAtlas::from_grid(
                texture,
                animation_texture.size,
                animation_texture.columns,
                animation_texture.rows,
            );
            let selection_circle = textures.get_handle("sprites/selection.png");

            commands
                .with_bundle(SpriteSheetBundle {
                    texture_atlas: texture_atlases.add(texture_atlas),
                    visible: Visible {
                        is_transparent: true,
                        ..Default::default()
                    },
                    render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        SPRITE_SHEET_PIPELINE_HANDLE.typed(),
                    )]),
                    ..Default::default()
                })
                .with(HealthBar(1))
                .with_children(|parent| {
                    parent.spawn(SpriteBundle {
                        material: color_materials.add(selection_circle.into()),
                        render_pipelines: RenderPipelines::from_pipelines(vec![
                            RenderPipeline::new(SPRITE_PIPELINE_HANDLE.typed()),
                        ]),
                        ..Default::default()
                    });

                    parent.spawn(BarBundle {
                        bar: Bar {
                            size: Vec2::new(unit.width, 6.0),
                            max_value: unit.max_health,
                            current_value: unit.max_health,
                            ..Default::default()
                        },
                        transform: Transform::from_translation(Vec3::new(
                            0.0,
                            unit.height + 16.0,
                            0.0,
                        )),
                        ..Default::default()
                    });
                });
        }

        commands.current_entity().unwrap()
    }
}
