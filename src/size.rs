use crate::*;
use bevy::prelude::*;

pub fn unit_size_system(
    units: Res<Assets<Unit>>,
    entities: Query<Entity, (With<Position>, With<Handle<Unit>>)>,
    mut query: Query<(&mut Position, &Handle<Unit>)>,
) {
    for entity in entities.iter() {
        for other in entities.iter() {
            if entity == other {
                continue;
            }

            let (other_position, other_unit_handle) = {
                let x = query.get_mut(other).unwrap();

                (x.0.clone(), x.1.clone())
            };

            let other_unit = units.get(other_unit_handle).unwrap();

            let (mut position, unit_handle) = query.get_mut(entity).unwrap();

            let unit = units.get(unit_handle).unwrap();

            let diff = position.position - other_position.position;
            let dist = diff.length();

            if dist == 0.0 {
                continue;
            }

            let overlap = (unit.size + other_unit.size) - dist;

            // check overlap
            if overlap <= 0.0 {
                continue;
            }

            position.position += diff.normalize() * overlap;
        }
    }
}
