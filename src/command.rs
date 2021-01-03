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
        network_entity_registry: &NetworkEntityRegistry,
        query: &Query<(&Position)>,
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
pub struct MoveCommand {
    pub target: CommandTarget,
    pub precise: bool,
}

#[typetag::serde]
impl Command for MoveCommand {
    fn execute(
        &mut self,
        entity: Entity,
        unit: &Unit,
        network_entity_registry: &NetworkEntityRegistry,
        query: &Query<(&Position)>,
    ) -> CommandControlFlow {
        let target_position = match &self.target {
            CommandTarget::Position(p) => *p,
            CommandTarget::Unit(n) => {
                let entity = network_entity_registry.get(n).unwrap();
                let position = query.get(*entity).unwrap();

                position.position.truncate()
            }
        };

        let position = query.get(entity).unwrap();
        let dist = (target_position - position.position.truncate()).length();

        if self.precise {
            if dist < 0.1 {
                CommandControlFlow::Completed
            } else {
                CommandControlFlow::Behaviour(Behaviour::Move {
                    target: target_position.clone(),
                })
            }
        } else {
            if dist < unit.size * 1.1 {
                CommandControlFlow::Completed
            } else {
                CommandControlFlow::Behaviour(Behaviour::Move {
                    target: target_position.clone(),
                })
            }
        }
    }
}
