use crate::*;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
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
    AddCommand(Command),
    SetCommand(Command),
    ClearCommands,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandMessage {
    pub operation: CommandQueueOperation,
    pub network_entity: NetworkEntity,
}

/// A list of [`Command`]s, an instance of which exists for every unit.
/// The unit will automatically go through every command in the queue, until it's empty.
/// When the [`CommandQueue`] is *locked*, [`Command`]s can only be added,
/// if set_command is tried, it will in stead be put into *awaiting*,
/// and the queue will only be set when *unlocked* again.
pub struct CommandQueue {
    commands: VecDeque<Command>,
    locked: bool,
    awaiting: Option<Command>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new(),
            locked: false,
            awaiting: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn current(&self) -> Option<&Command> {
        self.commands.back()
    }

    pub fn current_mut(&mut self) -> Option<&mut Command> {
        self.commands.back_mut()
    }

    pub fn add_command(&mut self, command: Command) {
        self.commands.push_front(command);
    }

    pub fn set_command(&mut self, command: Command) {
        if self.locked {
            self.awaiting = Some(command);
        } else {
            self.commands = vec![command].into();
        }
    }

    pub fn apply(&mut self, operation: CommandQueueOperation) {
        match operation {
            CommandQueueOperation::AddCommand(command) => {
                self.commands.push_front(command);
            }
            CommandQueueOperation::SetCommand(command) => {
                self.set_command(command);
            }
            CommandQueueOperation::ClearCommands => {
                self.commands.clear();
            }
        }
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn unlock(&mut self) {
        self.locked = false;

        if let Some(awaiting) = std::mem::replace(&mut self.awaiting, None) {
            self.commands = vec![awaiting].into();
        }
    }

    /// Completes the current [`Command`], and unlocks the *self*.
    pub fn complete(&mut self) {
        self.commands.pop_front();
        self.unlock();
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
    pub animation_set: String,
    pub unit_animation_set: String,
    pub size: f32,
    pub height: f32,
    pub width: f32,
    pub selection_size: f32,
    pub movement_priority: f32,
    /// Attacks are optional
    pub attack: Option<Attack>,
    pub movement_speed: MovementSpeed,
    pub max_health: f32,
}

impl Unit {
    pub fn instance(&self) -> UnitInstance {
        UnitInstance {
            health: self.max_health,
            movement: None,
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
    pub movement: Option<Vec2>,
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
            let entity = network_entity_registry
                .get(&unit_instance_message.target)
                .unwrap();

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

fn operation(command: Command, keyboard_input: &Input<KeyCode>) -> CommandQueueOperation {
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

    let move_command = input_config
        .move_command
        .just_pressed(&keyboard_input, &mouse_input);

    let attack_move = input_config
        .attack_move_command
        .just_pressed(&keyboard_input, &mouse_input);

    match () {
        _ if move_command || attack_move => {
            match target {
                CommandTarget::Ally(target) => {
                    for network_entity in &selected_units.network_entities {
                        if attack_move && target == *network_entity {
                            continue;
                        }

                        let message = CommandMessage {
                            operation: operation(
                                if attack_move {
                                    Command::Attack {
                                        target_position: None,
                                        target_unit: Some(target),
                                        persue: true,
                                    }
                                } else {
                                    Command::Move {
                                        target: CommandTarget::Ally(target),
                                        precise: selected_units.network_entities.len() == 1,
                                    }
                                },
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
                                Command::Attack {
                                    target_position: None,
                                    target_unit: Some(target),
                                    persue: true,
                                },
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

                            let command = if attack_move {
                                Command::Attack {
                                    target_position: Some(target_position + relative_position),
                                    target_unit: None,
                                    persue: true,
                                }
                            } else {
                                Command::Move {
                                    target: CommandTarget::Position(
                                        target_position + relative_position,
                                    ),
                                    precise: true,
                                }
                            };

                            let message = CommandMessage {
                                operation: operation(command.clone(), &keyboard_input),
                                network_entity: *network_entity,
                            };

                            net.broadcast_message(message);
                        }
                    } else {
                        for network_entity in &selected_units.network_entities {
                            let message = CommandMessage {
                                operation: operation(
                                    if attack_move {
                                        Command::Attack {
                                            target_position: Some(target_position),
                                            target_unit: None,
                                            persue: true,
                                        }
                                    } else {
                                        Command::Move {
                                            target: CommandTarget::Position(target_position),
                                            precise: false,
                                        }
                                    },
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
            let entity =
                if let Some(e) = network_entity_registry.get(&action_message.network_entity) {
                    e
                } else {
                    continue;
                };

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
            app_builder.add_system(attack_command_system.system());
            app_builder.add_system(move_command_system.system());
            app_builder.add_system(idle_command_system.system());
            app_builder.add_system(unit_collision_system.system());
            app_builder.add_system(network_unit_action_system.system());
            app_builder.add_system(server_unit_instance_system.system());
            app_builder.add_system(server_unit_health_system.system());
            app_builder.add_system(unit_movement_system.system());
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
