use bevy::prelude::*;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct AnimationSetLoader;

impl AssetLoader for AnimationSetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let animation_set = ron::de::from_bytes::<AnimationSet>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(animation_set));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["anim"]
    }
}

#[derive(Serialize, Deserialize)]
pub struct Animation {
    start: u32,
    end: u32,
    frame_length: f32,
}

impl Animation {
    pub fn new(frames: std::ops::Range<u32>, frame_length: f32) -> Self {
        Self {
            start: frames.start,
            end: frames.end,
            frame_length,
        }
    }
}

#[derive(TypeUuid, Serialize, Deserialize)]
#[uuid = "53494558-e8f6-447f-94bc-7010a8979781"]
pub struct AnimationSet {
    animations: HashMap<String, Animation>,
}

impl AnimationSet {
    pub fn get(&self, name: impl Into<String>) -> Option<&Animation> {
        self.animations.get(&name.into())
    }
}

#[derive(Reflect)]
pub struct Animator {
    animation_set: Handle<AnimationSet>,
    playing_animation: String,
    play_time: f32,
}

impl Animator {
    pub fn new(animation_set: Handle<AnimationSet>, playing_animation: impl Into<String>) -> Self {
        Self {
            animation_set,
            playing_animation: playing_animation.into(),
            play_time: 0.0,
        }
    }

    pub fn play(&mut self, name: impl Into<String>) {
        self.playing_animation = name.into();
        self.play_time = 0.0;
    }

    pub fn set_playing(&mut self, name: impl Into<String>) {
        self.playing_animation = name.into();
    }
}

pub fn animator_system(
    time: Res<Time>,
    animation_sets: Res<Assets<AnimationSet>>,
    mut query: Query<(&mut Animator, &mut TextureAtlasSprite)>,
) {
    for (mut animator, mut sprite) in query.iter_mut() {
        let Animator {
            animation_set,
            playing_animation,
            play_time,
            ..
        } = &mut *animator;

        if let Some(animation_set) = animation_sets.get(animation_set.clone()) {
            if let Some(animation) = animation_set.get(playing_animation.clone()) {
                *play_time += time.delta_seconds();

                let curret_frame = (*play_time / animation.frame_length).floor() as u32
                    % (animation.end - animation.start);

                sprite.index = curret_frame + animation.start;
            }
        }
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder
            .add_system(animator_system.system())
            .add_asset::<AnimationSet>()
            .add_asset_loader(AnimationSetLoader)
            .register_type::<Animator>();
    }
}
