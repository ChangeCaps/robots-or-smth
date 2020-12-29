use crate::*;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct AnimationSetLoader;

ron_loader!(AnimationSetLoader, "anim" => AnimationSet);

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AnimatorOperation {
    Play(String),
    SetPlaying(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimatorMessage {
    operation: AnimatorOperation,
    network_entity: NetworkEntity,
}

#[derive(Reflect)]
pub struct Animator {
    animation_set: Handle<AnimationSet>,
    playing_animation: String,
    play_time: f32,
    current_frame: u32,
    #[reflect(ignore)]
    operations: Vec<AnimatorOperation>,
}

impl Animator {
    pub fn new(animation_set: Handle<AnimationSet>, playing_animation: impl Into<String>) -> Self {
        Self {
            animation_set,
            playing_animation: playing_animation.into(),
            play_time: 0.0,
            current_frame: 0,
            operations: Vec::new(),
        }
    }

    pub fn play(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.playing_animation = name.clone();
        self.play_time = 0.0;
        self.current_frame = 0;
        self.operations.push(AnimatorOperation::Play(name));
    }

    pub fn playing(&self) -> &String {
        &self.playing_animation
    }

    pub fn set_playing(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.playing_animation = name.clone();
        self.operations.push(AnimatorOperation::SetPlaying(name));
    }

    pub fn current_frame(&self) -> u32 {
        self.current_frame
    }

    pub fn apply(&mut self, message: AnimatorOperation) {
        match message {
            AnimatorOperation::Play(anim) => {
                self.playing_animation = anim;
                self.play_time = 0.0;
                self.current_frame = 0;
            }
            AnimatorOperation::SetPlaying(anim) => {
                self.playing_animation = anim;
            }
        }
    }
}

pub fn animator_system(
    time: Res<Time>,
    animation_sets: Res<Assets<AnimationSet>>,
    mut query: Query<&mut Animator>,
) {
    for mut animator in query.iter_mut() {
        let Animator {
            animation_set,
            playing_animation,
            play_time,
            current_frame,
            ..
        } = &mut *animator;

        if let Some(animation_set) = animation_sets.get(animation_set.clone()) {
            if let Some(animation) = animation_set.get(&*playing_animation) {
                *play_time += time.delta_seconds();

                *current_frame = (*play_time / animation.frame_length).floor() as u32
                    % (animation.end - animation.start + 1);
            }
        }
    }
}

pub fn animator_sprite_system(
    animation_sets: Res<Assets<AnimationSet>>,
    mut query: Query<(&Animator, &mut TextureAtlasSprite)>,
) {
    for (animator, mut sprite) in query.iter_mut() {
        if let Some(animation_set) = animation_sets.get(&animator.animation_set) {
            if let Some(animation) = animation_set.get(&animator.playing_animation) {
                sprite.index = animator.current_frame() + animation.start;
            }
        }
    }
}

pub fn server_network_animator_system(
    mut net: ResMut<NetworkResource>,
    mut query: Query<(&NetworkEntity, &mut Animator)>,
) {
    for (network_entity, mut animator) in query.iter_mut() {
        for operation in animator.operations.drain(..) {
            let message = AnimatorMessage {
                operation,
                network_entity: network_entity.clone(),
            };

            net.broadcast_message(message);
        }
    }
}

pub fn client_network_animator_system(
    mut net: ResMut<NetworkResource>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    mut query: Query<&mut Animator>,
) {
    for (_handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(animator_message) = channels.recv::<AnimatorMessage>() {
            let entity = network_entity_registry
                .get(&animator_message.network_entity)
                .unwrap();
            let mut animator = query.get_mut(*entity).unwrap();

            animator.apply(animator_message.operation);
        }
    }
}

pub struct AnimationPlugin(bool);

impl AnimationPlugin {
    pub fn server() -> Self {
        Self(true)
    }

    pub fn client() -> Self {
        Self(false)
    }
}

impl Plugin for AnimationPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.add_asset_loader(AnimationSetLoader);
        app_builder.add_asset::<AnimationSet>();
        app_builder.register_type::<Animator>();
        app_builder.add_system(animator_system.system());

        if self.0 {
            app_builder.add_system(server_network_animator_system.system());
        } else {
            app_builder.add_system(client_network_animator_system.system());
            app_builder.add_system(animator_sprite_system.system());
        }
    }
}
