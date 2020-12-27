use crate::*;
use bevy::prelude::*;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub enum Action {
    Move { target_position: Vec2 },
}

pub struct ActionQueue {
    pub actions: VecDeque<Action>,
}

#[derive(Serialize, Deserialize)]
pub enum MovementSpeed {
    Constant(f32),
    FrameWise(Vec<f32>),
}

#[derive(TypeUuid, Serialize, Deserialize)]
#[uuid = "9193fc48-7a91-4163-988c-6a3ef47fe53a"]
pub struct Unit {
    pub size: f32,
    pub selection_size: f32,
    pub movement_speed: MovementSpeed,
}

pub fn unit_action_system(
    mouse_position: Res<MousePosition>,
    mouse_input: Res<Input<MouseButton>>,
    selected_units: Res<SelectedUnits>,
    mut query: Query<&mut ActionQueue>,
) {
    if mouse_input.just_pressed(MouseButton::Right) {
        for entity in &selected_units.units {
            let mut action_queue = query.get_mut(*entity).unwrap();

            action_queue.actions = vec![Action::Move {
                target_position: mouse_position.position(),
            }]
            .into();
        }
    }
}

pub fn unit_action_execution_system(
    time: Res<Time>,
    units: Res<Assets<Unit>>,
    mut query: Query<(
        &mut ActionQueue,
        &mut Position,
        &mut Animator,
        &Handle<Unit>,
    )>,
) {
    for (mut action_queue, mut position, mut animator, unit_handle) in query.iter_mut() {
        let unit = units.get(unit_handle).unwrap();

        let mut completed = false;

        if let Some(action) = action_queue.actions.back() {
            match action {
                Action::Move { target_position } => {
                    let mut movement_step = *target_position - position.position.truncate();

                    let movement_speed = match &unit.movement_speed {
                        MovementSpeed::Constant(speed) => *speed,
                        MovementSpeed::FrameWise(frame_mods) => {
                            frame_mods[animator.current_frame() as usize]
                        }
                    };

                    if movement_step.length() <= movement_speed * time.delta_seconds() {
                        completed = true;
                    } else {
                        let angle =
                            movement_step.y.atan2(movement_step.x) / std::f32::consts::PI * 4.0;

                        movement_step =
                            movement_step.normalize() * movement_speed * time.delta_seconds();

                        if angle < -3.5 {
                            animator.set_playing("walk_left");
                        } else if angle < -2.5 {
                            animator.set_playing("walk_down_left");
                        } else if angle < -1.5 {
                            animator.set_playing("walk_down");
                        } else if angle < -0.5 {
                            animator.set_playing("walk_down_right");
                        } else if angle < 0.5 {
                            animator.set_playing("walk_right");
                        } else if angle < 1.5 {
                            animator.set_playing("walk_up_right");
                        } else if angle < 2.5 {
                            animator.set_playing("walk_up");
                        } else if angle < 3.5 {
                            animator.set_playing("walk_up_left");
                        } else {
                            animator.set_playing("walk_left");
                        }

                        position.position += movement_step.extend(0.0);
                    }
                }
            }
        }

        if completed {
            action_queue.actions.pop_back();
        }
    }
}

pub struct UnitLoader;

impl AssetLoader for UnitLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let unit = ron::de::from_bytes::<Unit>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(unit));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["unit"]
    }
}
