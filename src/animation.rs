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
    texture: u32,
}

impl Animation {
    pub fn new(frames: std::ops::Range<u32>, frame_length: f32) -> Self {
        Self {
            start: frames.start,
            end: frames.end,
            frame_length,
            texture: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AnimationTexture {
    pub rows: usize,
    pub columns: usize,
    pub size: Vec2,
    pub path: String,
}

#[derive(TypeUuid, Serialize, Deserialize)]
#[uuid = "53494558-e8f6-447f-94bc-7010a8979781"]
pub struct AnimationSet {
    pub default_texture: u32,
    pub animation_textures: HashMap<u32, AnimationTexture>,
    pub animations: HashMap<String, Animation>,
}

impl AnimationSet {
    pub fn get(&self, name: impl Into<String>) -> Option<&Animation> {
        self.animations.get(&name.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AnimatorOperation {
    Play(String),
    SetPlaying(String, u32),
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
    current_frame_set: bool,
    #[reflect(ignore)]
    current_frame_changed: bool,
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
            current_frame_set: false,
            current_frame_changed: true,
            operations: Vec::new(),
        }
    }

    pub fn play(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.playing_animation = name.clone();
        self.play_time = 0.0;
        self.current_frame = 0;
        self.current_frame_changed = true;
        self.operations.push(AnimatorOperation::Play(name));
    }

    pub fn playing(&self) -> &String {
        &self.playing_animation
    }

    pub fn set_playing(&mut self, name: impl Into<String>) {
        let name = name.into();

        if self.playing_animation == name {
            return;
        }

        self.playing_animation = name.clone();
        self.operations
            .push(AnimatorOperation::SetPlaying(name, self.current_frame()));
    }

    pub fn current_frame(&self) -> u32 {
        self.current_frame
    }

    pub fn set_current_frame(&mut self, frame: u32) {
        if frame != self.current_frame() {
            self.current_frame = frame;
            self.current_frame_set = true;
            self.current_frame_changed = true;
        }
    }

    pub fn frame_just_changed(&self) -> bool {
        self.current_frame_changed
    }

    pub fn apply(&mut self, message: AnimatorOperation) {
        match message {
            AnimatorOperation::Play(anim) => {
                self.playing_animation = anim;
                self.play_time = 0.0;
                self.current_frame = 0;
            }
            AnimatorOperation::SetPlaying(anim, frame) => {
                self.playing_animation = anim;
                self.current_frame = frame;
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
            current_frame_set,
            current_frame_changed,
            ..
        } = &mut *animator;

        if let Some(animation_set) = animation_sets.get(animation_set.clone()) {
            if let Some(animation) = animation_set.get(&*playing_animation) {
                *current_frame_changed = false;

                if *current_frame_set {
                    *current_frame_set = false;

                    *play_time = *current_frame as f32 * animation.frame_length;
                }

                *play_time += time.delta_seconds();

                let new_frame = (*play_time / animation.frame_length).floor() as u32
                    % (animation.end - animation.start + 1);

                if new_frame != *current_frame {
                    *current_frame = new_frame;
                    *current_frame_changed = true;
                }
            }
        }
    }
}

pub fn animator_sprite_system(
    animation_sets: Res<Assets<AnimationSet>>,
    textures: Res<Assets<Texture>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut query: Query<(&Animator, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (animator, mut sprite, texture_atlas_handle) in query.iter_mut() {
        if let Some(animation_set) = animation_sets.get(&animator.animation_set) {
            if let Some(animation) = animation_set.get(&animator.playing_animation) {
                if let Some(animation_texture) =
                    animation_set.animation_textures.get(&animation.texture)
                {
                    let texture_atlas = texture_atlases.get_mut(&*texture_atlas_handle).unwrap();

                    // TODO: optimize, dont set on every update for every unit
                    *texture_atlas = TextureAtlas::from_grid(
                        textures.get_handle(animation_texture.path.as_str()),
                        animation_texture.size,
                        animation_texture.columns,
                        animation_texture.rows,
                    );

                    sprite.index = animator.current_frame() + animation.start;
                } else {
                    error!("AnimationTexture not found");
                }
            } else {
                error!("Animation not found");
            }
        } else {
            error!("AnimationSet not found");
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
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(animator_message) = channels.recv::<AnimatorMessage>() {
            let entity =
                if let Some(e) = network_entity_registry.get(&animator_message.network_entity) {
                    e
                } else {
                    warn!(
                        "Recieved animation message for unregistered entity {}",
                        handle
                    );
                    continue;
                };

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
