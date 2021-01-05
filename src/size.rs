use crate::*;
use bevy::prelude::*;

pub fn unit_collision_system(
    units: Res<Assets<Unit>>,
    entities: Query<Entity, (With<Position>, With<Handle<Unit>>)>,
    mut query: Query<(&mut Position, &Behaviour, &Handle<Unit>)>,
) {
    for a_entity in entities.iter() {
        let (a_position, a_priority, a_unit) = {
            let (position, behaviour, unit_handle) = query.get_mut(a_entity).unwrap();
            let unit = if let Some(unit) = units.get(&*unit_handle) {
                unit
            } else {
                warn!("Invalid handle to unit");
                continue;
            };

            let priority = if let Behaviour::Move { .. } = &*behaviour {
                unit.movement_priority
            } else {
                0.0
            };

            (position.position.clone(), priority, unit.clone())
        };

        for b_entity in entities.iter() {
            if a_entity == b_entity {
                continue;
            }

            let (b_position, b_priority, b_unit) = {
                let (position, behaviour, unit_handle) = query.get_mut(b_entity).unwrap();
                let unit = units.get(&*unit_handle).unwrap();

                let priority = if let Behaviour::Move { .. } = &*behaviour {
                    unit.movement_priority
                } else {
                    0.0
                };

                (position.position.clone(), priority, unit.clone())
            };

            let a = a_position.truncate();
            let b = b_position.truncate();

            let diff = a - b;
            let dist = diff.length();

            if dist == 0.0 {
                continue;
            }

            let overlap = (a_unit.size + b_unit.size) - dist;

            // check overlap
            if overlap <= 0.0 {
                continue;
            }

            if a_priority == 0.0 && b_priority == 0.0 {
                query.get_mut(a_entity).unwrap().0.position +=
                    diff.normalize().extend(0.0) * 0.5 * overlap;
                query.get_mut(b_entity).unwrap().0.position -=
                    diff.normalize().extend(0.0) * 0.5 * overlap;

                continue;
            }

            let a_mod = b_priority / (a_priority + b_priority) * 0.25;
            let b_mod = a_priority / (a_priority + b_priority) * 0.25;

            // apply
            let (mut a_position, _, _) = query.get_mut(a_entity).unwrap();
            a_position.position += diff.normalize().extend(0.0) * a_mod * overlap;

            // apply
            let (mut b_position, _, _) = query.get_mut(b_entity).unwrap();
            b_position.position -= diff.normalize().extend(0.0) * b_mod * overlap;
        }
    }
}
