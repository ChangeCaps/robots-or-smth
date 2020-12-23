use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct AnimationSetAsset {}

#[derive(Serialize, Deserialize)]
pub struct AnimationAsset {}

pub struct Animation {
    frames: std::ops::Range<u32>,
    frame_length: f32,
}

impl Animation {
    pub fn new(frames: std::ops::Range<u32>, frame_length: f32) -> Self {
        Self {
            frames,
            frame_length,
        }
    }
}

#[derive(TypeUuid)]
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
    changed: bool,
}

impl Animator {
    pub fn new(animation_set: Handle<AnimationSet>, playing_animation: String) -> Self {
        Self {
            animation_set,
            playing_animation,
            play_time: 0.0,
            changed: true,
        }
    }

    pub fn changed(&mut self) -> bool {
        std::mem::replace(&mut self.changed, false)
    }

    pub fn play(&mut self, name: impl Into<String>) {
        self.playing_animation = name.into();
        self.play_time = 0.0;
        self.changed = true;
    }

    pub fn set_playing(&mut self, name: impl Into<String>) {
        self.playing_animation = name.into();
        self.changed = true;
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
                    % (animation.frames.end - animation.frames.start);

                sprite.index = curret_frame + animation.frames.start;
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
            .register_type::<Animator>();
    }
}
