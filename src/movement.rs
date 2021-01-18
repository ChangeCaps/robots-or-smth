use crate::*;
use bevy::tasks::prelude::*;

pub fn unit_movement_system(
    task_pool: Res<ComputeTaskPool>,
    time: Res<Time>,
    units: Res<Assets<Unit>>,
    mut query: Query<(
        &mut UnitDirection,
        &mut Position,
        &mut UnitInstance,
        &mut UnitAnimator,
        &Animator,
        &Handle<Unit>,
    )>,
) {
    query.par_iter_mut(16).for_each(
        &task_pool,
        |(
            mut unit_direction,
            mut position,
            mut unit_instance,
            mut unit_animator,
            animator,
            unit_handle,
        )| {
            if let Some(movement) = &unit_instance.movement {
                unit_animator.set_playing("walk");

                let unit = units.get(unit_handle).unwrap();

                let movement_speed = match &unit.movement_speed {
                    MovementSpeed::Constant(speed) => *speed,
                    MovementSpeed::FrameWise { speed, frame_mods } => {
                        frame_mods[animator.current_frame() as usize] * *speed
                    }
                };

                let diff = *movement - position.position.truncate();
                let dist = diff.length();

                if dist < 0.1 {
                    unit_animator.set_playing("idle");
                    unit_instance.movement = None;
                } else {
                    *unit_direction = UnitDirection::from_vec2(diff);
                    position.position += diff.normalize().extend(0.0)
                        * (time.delta_seconds() * movement_speed).min(dist * 0.99);
                }
            }
        },
    );
}

pub fn unit_collision_system(
    units: Res<Assets<Unit>>,
    entities: Query<Entity, (With<Position>, With<Handle<Unit>>)>,
    mut query: Query<(&mut Position, &UnitInstance, &Handle<Unit>)>,
) {
    for a_entity in entities.iter() {
        let (a_position, a_priority, a_unit) = {
            let (position, unit_instance, unit_handle) = query.get_mut(a_entity).unwrap();
            let unit = if let Some(unit) = units.get(unit_handle) {
                unit
            } else {
                warn!("Invalid handle to unit");
                continue;
            };

            let priority = if unit_instance.movement.is_some() {
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
                let (position, unit_instance, unit_handle) = query.get_mut(b_entity).unwrap();
                let unit = if let Some(unit) = units.get(&*unit_handle) {
                    unit
                } else {
                    warn!("Invalid handle to unit");
                    continue;
                };

                let priority = if unit_instance.movement.is_some() {
                    unit.movement_priority
                } else {
                    0.0
                };

                (position.position.clone(), priority, unit.clone())
            };

            let a = a_position.truncate();
            let b = b_position.truncate();

            let mut diff = a - b;
            let dist = diff.length();

            if dist == 0.0 {
                let mut rng = rand::thread_rng();
                diff = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
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

            let a_mod = b_priority / (a_priority + b_priority) * 0.5;
            let b_mod = a_priority / (a_priority + b_priority) * 0.5;

            // apply
            let (mut a_position, _, _) = query.get_mut(a_entity).unwrap();
            a_position.position += diff.normalize().extend(0.0) * a_mod * overlap;

            // apply
            let (mut b_position, _, _) = query.get_mut(b_entity).unwrap();
            b_position.position -= diff.normalize().extend(0.0) * b_mod * overlap;
        }
    }
}
