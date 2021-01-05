use crate::*;
use bevy::prelude::*;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CommandTarget {
    Position(Vec2),
    Ally(NetworkEntity),
    Enemy(NetworkEntity),
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
    pub request_set: Option<Box<dyn Command>>,
}

impl CommandQueue {
    pub fn apply(&mut self, operation: CommandQueueOperation) {
        match operation {
            CommandQueueOperation::AddCommand(command) => {
                self.commands.push_front(command);
            }
            CommandQueueOperation::SetCommand(command) => {
                if self.commands.len() == 0 {
                    self.commands.push_front(command);
                } else {
                    self.request_set = Some(command);
                }
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
    pub height: f32,
    pub width: f32,
    pub selection_size: f32,
    pub movement_priority: f32,
    pub soft_attack_range: f32,
    pub hard_attack_range: f32,
    pub movement_speed: MovementSpeed,
    pub attack_damage_frames: HashMap<u32, f32>,
    pub max_health: f32,
}

impl Unit {
    pub fn instance(&self) -> UnitInstance {
        UnitInstance {
            health: self.max_health,
            operations: Vec::new(),
        }
    }
}

pub struct HealthBar(pub usize);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UnitInstanceOperation {
    SetHealth(f32),
    SubtractHealth(f32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnitInstanceMessage {
    pub operation: UnitInstanceOperation,
    pub target: NetworkEntity,
}

pub struct UnitInstance {
    pub health: f32,
    pub operations: Vec<UnitInstanceOperation>,
}

impl UnitInstance {
    pub fn apply_operations(&mut self) {
        let operations = std::mem::replace(&mut self.operations, Vec::new());

        for operation in operations.into_iter().rev() {
            self.apply_operation(operation);
        }
    }

    pub fn apply_operation(&mut self, operation: UnitInstanceOperation) {
        match operation {
            UnitInstanceOperation::SetHealth(new_health) => self.health = new_health,
            UnitInstanceOperation::SubtractHealth(damage) => self.health -= damage,
        }
    }

    pub fn operation(&mut self, operation: UnitInstanceOperation) {
        self.operations.push(operation);
    }

    pub fn set_health(&mut self, damage: f32) {
        self.operation(UnitInstanceOperation::SetHealth(damage));
    }
    
    pub fn subtract_health(&mut self, damage: f32) {
        self.operation(UnitInstanceOperation::SubtractHealth(damage));
    }
}

fn client_unit_instance_system(
    mut net: ResMut<NetworkResource>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    mut query: Query<&mut UnitInstance>,
) {
    for (_handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(unit_instance_message) = channels.recv::<UnitInstanceMessage>() {
            let entity = network_entity_registry.get(&unit_instance_message.target).unwrap();

            info!("{:?}", unit_instance_message);

            let mut unit_instance = query.get_mut(*entity).unwrap();

            unit_instance.apply_operation(unit_instance_message.operation);
        }
    }
}

fn server_unit_instance_system(
    mut net: ResMut<NetworkResource>,
    mut query: Query<(&mut UnitInstance, &NetworkEntity)>,
) {
    for (mut unit_instance, network_entity) in query.iter_mut() {
        let operations = std::mem::replace(&mut unit_instance.operations, Vec::new());

        for operation in operations.into_iter().rev() {
            unit_instance.apply_operation(operation.clone());

            let message = UnitInstanceMessage {
                operation,
                target: *network_entity,
            };
    
            net.broadcast_message(message);
        }
    }
}

fn unit_health_bar_system(
    mut bar_query: Query<&mut Bar>,
    query: Query<(&UnitInstance, &HealthBar, &Children)>,
) {
    for (unit_instance, health_bar, children) in query.iter() {
        if let Ok(mut bar) = bar_query.get_mut(children[health_bar.0]) {
            bar.current_value = unit_instance.health;
        }
    }
}

fn client_unit_health_system(
    commands: &mut Commands,
    mut selected_units: ResMut<SelectedUnits>,
    mut network_entity_registry: ResMut<NetworkEntityRegistry>,
    query: Query<(Entity, &UnitInstance, &NetworkEntity)>,
) {
    for (entity, unit_instance, network_entity) in query.iter() {
        if unit_instance.health <= 0.0 {
            network_entity_registry.remove(&network_entity);
            selected_units.units.remove(&entity);
            selected_units.network_entities.remove(&network_entity);
            commands.despawn_recursive(entity);
        }
    }
}

fn server_unit_health_system(
    commands: &mut Commands,
    mut network_entity_registry: ResMut<NetworkEntityRegistry>,
    query: Query<(Entity, &UnitInstance, &NetworkEntity)>,
) {
    for (entity, unit_instance, network_entity) in query.iter() {
        if unit_instance.health <= 0.0 {
            network_entity_registry.remove(&network_entity);
            commands.despawn_recursive(entity);
        }
    }
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
    player_id: Res<Option<PlayerId>>,
    mut net: ResMut<NetworkResource>,
    query: Query<(&Position, &Handle<Unit>, &NetworkEntity, &Owner)>,
) {
    let input_config = match input_config.get(&input_resource.0) {
        Some(i) => i,
        None => return,
    };

    let player_id = if let Some(player_id) = &*player_id {
        player_id
    } else {
        return;
    };

    let mut target = CommandTarget::Position(mouse_position.position());

    for (position, unit_handle, network_entity, owner) in query.iter() {
        let unit = units.get(&*unit_handle).unwrap();

        if (mouse_position.position() - position.position.truncate()).length() < unit.selection_size
        {
            if owner.0 == *player_id {
                target = CommandTarget::Ally(*network_entity);
            } else {
                target = CommandTarget::Enemy(*network_entity);
            }
        }
    }

    match () {
        _ if input_config
            .move_command
            .just_pressed(&keyboard_input, &mouse_input) =>
        {
            match target {
                CommandTarget::Ally(target) => {
                    for network_entity in &selected_units.network_entities {
                        let message = CommandMessage {
                            operation: operation(
                                Box::new(MoveUnitCommand { target }),
                                &keyboard_input,
                            ),
                            network_entity: *network_entity,
                        };

                        net.broadcast_message(message);
                    }
                }
                CommandTarget::Enemy(target) => {
                    for network_entity in &selected_units.network_entities {
                        let message = CommandMessage {
                            operation: operation(
                                Box::new(AttackUnitCommand { target }),
                                &keyboard_input,
                            ),
                            network_entity: *network_entity,
                        };

                        net.broadcast_message(message);
                    }
                }
                CommandTarget::Position(target_position) => {
                    let mut diameter = 0.0;
                    let mut center = Vec2::zero();
                    let mut center_of_mass = Vec2::zero();
                    let mut area = 0.0;

                    // calculate MEC
                    for a in &selected_units.units {
                        let (a_position, a_unit_handle, _, _) = query.get(*a).unwrap();
                        let a_unit = units.get(&*a_unit_handle).unwrap();

                        area += a_unit.size.powi(2) * std::f32::consts::PI;
                        center_of_mass += a_position.position.truncate();

                        for b in &selected_units.units {
                            if a == b {
                                continue;
                            }

                            let (b_position, b_unit_handle, _, _) = query.get(*b).unwrap();
                            let b_unit = units.get(&*b_unit_handle).unwrap();

                            let diff =
                                a_position.position.truncate() - b_position.position.truncate();
                            let d = diff.length() + a_unit.size + b_unit.size;

                            if d > diameter {
                                diameter = d;

                                center = (a_position.position.truncate()
                                    + b_position.position.truncate())
                                    / 2.0;
                                center += diff.normalize() * a_unit.size;
                                center -= diff.normalize() * b_unit.size;
                            }
                        }
                    }

                    center_of_mass /= selected_units.units.len() as f32;

                    if area > (diameter / 2.0).powi(2) * std::f32::consts::PI * 0.4 {
                        for entity in &selected_units.units {
                            let (position, _, network_entity, _) = query.get(*entity).unwrap();

                            let relative_position = position.position.truncate() - center_of_mass;

                            let command = MovePositionCommand {
                                target: target_position + relative_position,
                                precise: true,
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
                                    Box::new(MovePositionCommand {
                                        target: target_position,
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
            }
        }
        _ => (),
    }
}

pub fn network_unit_action_system(
    mut net: ResMut<NetworkResource>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    players: Res<Players>,
    mut query: Query<(&mut CommandQueue, &Owner)>,
) {
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(action_message) = channels.recv::<CommandMessage>() {
            let entity = network_entity_registry
                .get(&action_message.network_entity)
                .unwrap();
            let (mut action_queue, owner) = query.get_mut(*entity).unwrap();

            let player = players.player_ids.get(handle).unwrap();

            if *player != owner.0 {
                warn!("Recieved action from wrong owner {:?}", player);
                continue;
            }

            action_queue.apply(action_message.operation.clone());
        }
    }
}

pub fn unit_command_execution_system(
    units: Res<Assets<Unit>>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    command_query: Query<(&Position, &Animator)>,
    mut query: Query<(Entity, &mut CommandQueue, &mut Behaviour, &Handle<Unit>)>,
) {
    for (entity, mut command_queue, mut behaviour, unit_handle) in query.iter_mut() {
        let unit = if let Some(u) = units.get(&*unit_handle) {
            u
        } else {
            continue;
        };

        let request_set = command_queue.request_set.is_some();
        if let Some(command) = command_queue.commands.back_mut() {
            match command.execute(entity, &unit, request_set, &network_entity_registry, &command_query) {
                CommandControlFlow::Wait => {
                    // bib bob, do nothing
                }
                CommandControlFlow::Behaviour(new_behaviour) => {
                    *behaviour = new_behaviour;
                }
                CommandControlFlow::Completed => {
                    if request_set {
                        let command = std::mem::replace(&mut command_queue.request_set, None).unwrap();

                        command_queue.commands = vec![command].into();
                    } else {
                        command_queue.commands.pop_back();
                        *behaviour = Behaviour::Idle;
                    }
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
            app_builder.add_system(server_unit_instance_system.system());
            app_builder.add_system(server_unit_health_system.system());
        } else {
            app_builder.add_system(unit_command_system.system());
            app_builder.add_system(unit_selection_system.system());
            app_builder.add_system(unit_selection_ring_system.system());
            app_builder.add_system(unit_health_bar_system.system());
            app_builder.add_system(client_unit_instance_system.system());
            app_builder.add_system(client_unit_health_system.system());
        }
    }
}
