use crate::*;
use bevy::prelude::*;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionQueueOperation {
    AddAction(Action),
    SetAction(Action),
    ClearActions,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionMessage {
    operation: ActionQueueOperation,
    network_entity: NetworkEntity,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    Move { target_position: Vec2 },
}

pub struct ActionQueue {
    pub actions: VecDeque<Action>,
}

impl ActionQueue {
    pub fn apply(&mut self, operation: ActionQueueOperation) {
        match operation {
            ActionQueueOperation::AddAction(action) => {
                self.actions.push_front(action);
            }
            ActionQueueOperation::SetAction(action) => {
                self.actions = vec![action].into();
            }
            ActionQueueOperation::ClearActions => {
                self.actions.clear();
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum MovementSpeed {
    Constant(f32),
    FrameWise(Vec<f32>),
}

#[derive(TypeUuid, Serialize, Deserialize)]
#[uuid = "ed8cb018-707f-4b10-959a-2f2920bb0d2a"]
pub struct Unit {
    pub size: f32,
    pub selection_size: f32,
    pub movement_speed: MovementSpeed,
}

pub fn unit_action_system(
    mouse_position: Res<MousePosition>,
    mouse_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    selected_units: Res<SelectedUnits>,
    mut net: ResMut<NetworkResource>,
    query: Query<&NetworkEntity>,
) {
    let action = match () {
        _ if mouse_input.just_pressed(MouseButton::Right) => Some(Action::Move {
            target_position: mouse_position.position(),
        }),
        _ => None,
    };

    let operation = match action {
        Some(action) if keyboard_input.pressed(KeyCode::LShift) => {
            Some(ActionQueueOperation::AddAction(action))
        }
        Some(action) => Some(ActionQueueOperation::SetAction(action)),
        None => None,
    };

    if let Some(operation) = operation {
        for entity in &selected_units.units {
            let network_entity = query.get(*entity).unwrap();

            let message = ActionMessage {
                operation: operation.clone(),
                network_entity: *network_entity,
            };

            net.broadcast_message(message);
        }
    }
}

pub fn network_unit_action_system(
    mut net: ResMut<NetworkResource>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    mut query: Query<&mut ActionQueue>,
) {
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(action_message) = channels.recv::<ActionMessage>() {
            let entity = network_entity_registry
                .get(&action_message.network_entity)
                .unwrap();
            let mut action_queue = query.get_mut(*entity).unwrap();

            info!(
                "Got action operation from [{:?}]: {:?}",
                handle, action_message.operation
            );

            action_queue.apply(action_message.operation);
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
                        let angle_step = *isometric::ISO_TO_SCREEN * movement_step.extend(0.0);

                        let angle = angle_step.y.atan2(angle_step.x) / std::f32::consts::PI * 4.0;

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

ron_loader!(UnitLoader, "unit" => Unit);

pub struct UnitPlugin(bool);

impl UnitPlugin {
    pub fn server() -> Self {
        Self(true)
    }

    pub fn client() -> Self {
        Self(false)
    }
}

impl Plugin for UnitPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.add_asset_loader(UnitLoader);
        app_builder.add_asset::<Unit>();

        if self.0 {
            app_builder.add_system(unit_action_execution_system.system());
            app_builder.add_system(unit_size_system.system());
            app_builder.add_system(network_unit_action_system.system());
        } else {
            app_builder.add_system(unit_action_system.system());
            app_builder.add_system(unit_selection_system.system());
            app_builder.add_system(unit_selection_ring_system.system());
        }
    }
}
