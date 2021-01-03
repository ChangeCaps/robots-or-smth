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
pub enum CommandTarget {
    Position(Vec2),
    Unit(NetworkEntity),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CommandQueueOperation {
    AddCommand(Box<dyn Command>),
    SetCommand(Box<dyn Command>),
    ClearCommands,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandMessage {
    operation: CommandQueueOperation,
    network_entity: NetworkEntity,
}

pub struct CommandQueue {
    pub commands: VecDeque<Box<dyn Command>>,
}

impl CommandQueue {
    pub fn apply(&mut self, operation: CommandQueueOperation, unit: &Unit) {
        match operation {
            CommandQueueOperation::AddCommand(action) => {
                self.commands.push_front(action);
            }
            CommandQueueOperation::SetCommand(action) => {
                self.commands = vec![action].into();
            }
            CommandQueueOperation::ClearCommands => {
                self.commands.clear();
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum MovementSpeed {
    Constant(f32),
    FrameWise { speed: f32, frame_mods: Vec<f32> },
}

#[derive(TypeUuid, Serialize, Deserialize)]
#[uuid = "ed8cb018-707f-4b10-959a-2f2920bb0d2a"]
pub struct Unit {
    pub size: f32,
    pub selection_size: f32,
    pub movement_priority: f32,
    pub movement_speed: MovementSpeed,
}

fn operation(command: Box<dyn Command>, keyboard_input: &Input<KeyCode>) -> CommandQueueOperation {
    match () {
        _ if keyboard_input.pressed(KeyCode::LShift) => CommandQueueOperation::AddCommand(command),
        _ => CommandQueueOperation::SetCommand(command),
    }
}

pub fn unit_command_system(
    mouse_position: Res<MousePosition>,
    mouse_input: Res<Input<MouseButton>>,
    input_config: Res<Assets<InputConfig>>,
    input_resource: Res<InputResource>,
    keyboard_input: Res<Input<KeyCode>>,
    selected_units: Res<SelectedUnits>,
    units: Res<Assets<Unit>>,
    mut net: ResMut<NetworkResource>,
    query: Query<(&Position, &Handle<Unit>, &NetworkEntity)>,
) {
    let input_config = match input_config.get(&input_resource.0) {
        Some(i) => i,
        None => return,
    };

    let mut target = CommandTarget::Position(mouse_position.position());

    for (position, unit_handle, network_entity) in query.iter() {
        let unit = units.get(&*unit_handle).unwrap();

        if (mouse_position.position() - position.position.truncate()).length() < unit.selection_size
        {
            target = CommandTarget::Unit(*network_entity);
        }
    }

    match () {
        _ if input_config
            .move_command
            .just_pressed(&keyboard_input, &mouse_input) =>
        {
            let mut diameter = 0.0;
            let mut center = Vec2::zero();
            let mut center_of_mass = Vec2::zero();
            let mut area = 0.0;

            // calculate MEC
            for a in &selected_units.units {
                let (a_position, a_unit_handle, _) = query.get(*a).unwrap();
                let a_unit = units.get(&*a_unit_handle).unwrap();

                area += a_unit.size.powi(2) * std::f32::consts::PI;
                center_of_mass += a_position.position.truncate();

                for b in &selected_units.units {
                    if a == b {
                        continue;
                    }

                    let (b_position, b_unit_handle, _) = query.get(*b).unwrap();
                    let b_unit = units.get(&*b_unit_handle).unwrap();

                    let diff = a_position.position.truncate() - b_position.position.truncate();
                    let d = diff.length() + a_unit.size + b_unit.size;

                    if d > diameter {
                        diameter = d;

                        center =
                            (a_position.position.truncate() + b_position.position.truncate()) / 2.0;
                        center += diff.normalize() * a_unit.size;
                        center -= diff.normalize() * b_unit.size;
                    }
                }
            }

            center_of_mass /= selected_units.units.len() as f32;

            if area > (diameter / 2.0).powi(2) * std::f32::consts::PI * 0.4 {
                for entity in &selected_units.units {
                    let (position, _, network_entity) = query.get(*entity).unwrap();

                    let relative_position = position.position.truncate() - center_of_mass;

                    let command = MoveCommand {
                        target: match &target {
                            CommandTarget::Position(position) => {
                                CommandTarget::Position(*position + relative_position)
                            }
                            CommandTarget::Unit(_) => target.clone(),
                        },
                        precise: match &target {
                            CommandTarget::Position(_) => true,
                            CommandTarget::Unit(_) => false,
                        },
                    };

                    let message = CommandMessage {
                        operation: operation(Box::new(command.clone()), &keyboard_input),
                        network_entity: *network_entity,
                    };

                    net.broadcast_message(message);
                }
            } else {
                for network_entity in &selected_units.network_entities {
                    let message = CommandMessage {
                        operation: operation(
                            Box::new(MoveCommand {
                                target: target.clone(),
                                precise: false,
                            }),
                            &keyboard_input,
                        ),
                        network_entity: *network_entity,
                    };

                    net.broadcast_message(message);
                }
            }
        }
        _ => (),
    }
}

pub fn network_unit_action_system(
    mut net: ResMut<NetworkResource>,
    units: Res<Assets<Unit>>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    players: Res<Players>,
    mut query: Query<(&mut CommandQueue, &Handle<Unit>, &Owner)>,
) {
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(action_message) = channels.recv::<CommandMessage>() {
            let entity = network_entity_registry
                .get(&action_message.network_entity)
                .unwrap();
            let (mut action_queue, unit_handle, owner) = query.get_mut(*entity).unwrap();

            let player = players.player_ids.get(handle).unwrap();

            if *player != owner.0 {
                warn!("Recieved action from wrong owner {:?}", player);
                continue;
            }

            let unit = units.get(&*unit_handle).unwrap();
            action_queue.apply(action_message.operation.clone(), unit);
        }
    }
}

pub fn unit_command_execution_system(
    units: Res<Assets<Unit>>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    command_query: Query<(&Position)>,
    mut query: Query<(Entity, &mut CommandQueue, &mut Behaviour, &Handle<Unit>)>,
) {
    for (entity, mut command_queue, mut behaviour, unit_handle) in query.iter_mut() {
        let unit = units.get(&*unit_handle).unwrap();

        if let Some(command) = command_queue.commands.back_mut() {
            match command.execute(entity, &unit, &network_entity_registry, &command_query) {
                CommandControlFlow::Wait => {
                    // bib bob, do nothing
                }
                CommandControlFlow::Behaviour(new_behaviour) => {
                    *behaviour = new_behaviour;
                }
                CommandControlFlow::Completed => {
                    command_queue.commands.pop_back();
                    *behaviour = Behaviour::Idle;
                }
            }
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
            app_builder.add_system(unit_command_execution_system.system());
            app_builder.add_system(unit_command_behaviour_system.system());
            app_builder.add_system(unit_collision_system.system());
            app_builder.add_system(network_unit_action_system.system());
        } else {
            app_builder.add_system(unit_command_system.system());
            app_builder.add_system(unit_selection_system.system());
            app_builder.add_system(unit_selection_ring_system.system());
        }
    }
}
