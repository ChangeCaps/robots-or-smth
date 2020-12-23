use bevy::prelude::*;
use std::collections::VecDeque;

pub enum Action {
    Move { target_position: Vec2 },
}

pub struct Unit {
    actions: VecDeque<Action>,
}

pub fn unit_system(time: Res<Time>, mut query: Query<(&mut Unit, &mut Transform)>) {
    for (mut unit, mut transform) in query.iter_mut() {
        let mut completed = false;

        if let Some(action) = unit.actions.back() {
            match action {
                Action::Move { target_position } => {}
            }
        }
    }
}
