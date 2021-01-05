use crate::*;

pub enum CommandControlFlow {
    Wait,
    Behaviour(Behaviour),
    Completed,
}

#[typetag::serde]
pub trait Command: std::fmt::Debug + CommandClone + Send + Sync + 'static {
    fn execute(
        &mut self,
        entity: Entity,
        unit: &Unit,
        request_cancel: bool,
        network_entity_registry: &NetworkEntityRegistry,
        query: &Query<(&Position, &Animator)>,
    ) -> CommandControlFlow;
}

pub trait CommandClone {
    fn box_clone(&self) -> Box<dyn Command>;
}

impl<T: Command + Clone> CommandClone for T {
    fn box_clone(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Command> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MovePositionCommand {
    pub target: Vec2,
    pub precise: bool,
}

#[typetag::serde]
impl Command for MovePositionCommand {
    fn execute(
        &mut self,
        entity: Entity,
        unit: &Unit,
        request_cancel: bool,
        network_entity_registry: &NetworkEntityRegistry,
        query: &Query<(&Position, &Animator)>,
    ) -> CommandControlFlow {
        if request_cancel {
            return CommandControlFlow::Completed;
        }

        let (position, _) = query.get(entity).unwrap();
        let dist = (self.target - position.position.truncate()).length();

        if self.precise {
            if dist < 0.1 {
                CommandControlFlow::Completed
            } else {
                CommandControlFlow::Behaviour(Behaviour::Move {
                    target: self.target,
                })
            }
        } else {
            if dist < unit.size * 1.1 {
                CommandControlFlow::Completed
            } else {
                CommandControlFlow::Behaviour(Behaviour::Move {
                    target: self.target,
                })
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MoveUnitCommand {
    pub target: NetworkEntity,
}

#[typetag::serde]
impl Command for MoveUnitCommand {
    fn execute(
        &mut self,
        _entity: Entity,
        _unit: &Unit,
        request_cancel: bool,
        network_entity_registry: &NetworkEntityRegistry,
        query: &Query<(&Position, &Animator)>,
    ) -> CommandControlFlow {
        if request_cancel {
            return CommandControlFlow::Completed;
        }

        match network_entity_registry.get(&self.target) {
            Some(target_entity) => {
                let (target_position, _) = query.get(*target_entity).unwrap();

                CommandControlFlow::Behaviour(Behaviour::Move {
                    target: target_position.position.truncate(),
                })
            }
            None => CommandControlFlow::Completed,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttackUnitCommand {
    pub target: NetworkEntity,
}

#[typetag::serde]
impl Command for AttackUnitCommand {
    fn execute(
        &mut self,
        entity: Entity,
        unit: &Unit,
        request_cancel: bool,
        network_entity_registry: &NetworkEntityRegistry,
        query: &Query<(&Position, &Animator)>,
    ) -> CommandControlFlow {
        match network_entity_registry.get(&self.target) {
            Some(target) => {
                let (position, animator) = query.get(entity).unwrap();
                let (target_position, _) = query.get(*target).unwrap();

                let diff = position.position.truncate() - target_position.position.truncate();
                let dist = diff.length();

                if dist > unit.soft_attack_range {
                    if request_cancel {
                        CommandControlFlow::Completed     
                    } else {
                        CommandControlFlow::Behaviour(Behaviour::Move {
                            target: target_position.position.truncate(),
                        })
                    }
                } else {
                    if request_cancel && animator.current_frame() == 0 {
                        CommandControlFlow::Completed
                    } else {
                        CommandControlFlow::Behaviour(Behaviour::Attack {
                            target_position: target_position.position.truncate(),
                            target: *target,
                            damage: unit.attack_damage_frames.clone(),
                        })
                    }
                }
            }
            None => {
                let (_, animator) = query.get(entity).unwrap();

                if animator.current_frame() == 0 {
                    CommandControlFlow::Completed
                } else {
                    CommandControlFlow::Wait
                }
            },
        }
    }
}
