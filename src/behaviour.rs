use crate::*;
use std::collections::HashMap;

/*
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Behaviour {
    Move {
        target: Vec2,
    },
    Attack {
        target_position: Vec2,
        target: Entity,
        damage: HashMap<u32, f32>,
    },
    Idle,
}

pub fn unit_move_behaviour_system(
    time: Res<Time>,
    units: Res<Assets<Unit>>,
    mut query: Query<(
        &Behaviour,
        &mut Position,
        &Animator,
        &mut UnitAnimator,
        &Handle<Unit>,
        &mut UnitDirection,
    )>,
) {
    for (behaviour, mut position, animator, mut unit_animator, unit_handle, mut direction) in
        query.iter_mut()
    {
        match behaviour {
            Behaviour::Move { target } => {
                let unit = units.get(&*unit_handle).unwrap();
                let movement_speed = match &unit.movement_speed {
                    MovementSpeed::Constant(s) => *s,
                    MovementSpeed::FrameWise { speed, frame_mods } => {
                        frame_mods[animator.current_frame() as usize] * *speed
                    }
                };

                let diff = *target - position.position.truncate();
                let dist = diff.length();

                if dist == 0.0 {
                    continue;
                }

                unit_animator.set_playing("walk");

                let move_dist = dist.min(movement_speed * time.delta_seconds());

                let step = diff.normalize();

                if dist > 0.0 {
                    *direction = UnitDirection::from_vec2(step);
                }

                position.position += step.extend(0.0) * move_dist;
            }
            _ => (),
        }
    }
}

pub fn unit_attack_behaviour_system(
    mut net: ResMut<NetworkResource>,
    mut unit_instance_query: Query<&mut UnitInstance>,
    mut query: Query<(
        &Behaviour,
        &Position,
        &Animator,
        &mut UnitAnimator,
        &mut UnitDirection,
    )>,
) {
    for (behaviour, position, animator, mut unit_animator, mut direction) in query.iter_mut() {
        match behaviour {
            Behaviour::Attack {
                target_position,
                target,
                damage,
            } => {
                let diff = *target_position - position.position.truncate();

                let unit_direction = UnitDirection::from_vec2(diff);

                *direction = unit_direction;

                if unit_animator.playing().as_str() != "attack" {
                    unit_animator.play("attack");
                }

                if animator.frame_just_changed() {
                    if let Some(damage) = damage.get(&animator.current_frame()) {
                        if let Ok(mut unit_instance) = unit_instance_query.get_mut(*target) {
                            unit_instance.subtract_health(*damage);
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

pub fn unit_idle_behaviour_system(
    units: Res<Assets<Unit>>,
    unit_query: Query<(&Position, &Owner, &NetworkEntity), With<Handle<Unit>>>,
    mut query: Query<(
        &Behaviour,
        &Position,
        &Handle<Unit>,
        &mut CommandQueue,
        &mut UnitAnimator,
        &Owner,
    )>,
) {
    for (behaviour, position, unit_handle, mut command_queue, mut unit_animator, owner) in
        query.iter_mut()
    {
        match behaviour {
            Behaviour::Idle => {
                if unit_animator.playing().as_str() != "idle" {
                    unit_animator.play("idle");
                }

                let unit = units.get(&*unit_handle).unwrap();

                for (unit_position, unit_owner, network_entity) in unit_query.iter() {
                    let dist =
                        (unit_position.position.truncate() - position.position.truncate()).length();

                    if dist <= unit.engage_range && owner.0 != unit_owner.0 {
                        command_queue
                            .set_command(
                                Command::Attack {
                                    target: CommandTarget::Enemy(*network_entity)
                                }
                            );
                    }
                }
            }
            _ => (),
        }
    }
}
*/
