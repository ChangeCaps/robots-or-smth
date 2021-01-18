use crate::*;
use bevy::tasks::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    Attack {
        target_position: Option<Vec2>,
        target_unit: Option<NetworkEntity>,
        persue: bool,
    },
    Move {
        target: CommandTarget,
        precise: bool,
    },
}

pub fn idle_command_system(
    task_pool: Res<ComputeTaskPool>,
    units: Res<Assets<Unit>>,
    enemy_query: Query<(&Position, &NetworkEntity, &Owner)>,
    mut query: Query<(
        &Position,
        &Handle<Unit>,
        &mut UnitAnimator,
        &mut CommandQueue,
        &NetworkEntity,
        &Owner,
    )>,
) {
    query.par_iter_mut(16).for_each(
        &task_pool,
        |(position, unit_handle, mut unit_animator, mut command_queue, network_entity, owner)| {
            // if the command queue is empty, we must be idle
            if command_queue.is_empty() {
                let unit = units.get(unit_handle).unwrap();

                // idle animation
                if unit_animator.playing() != "idle" {
                    unit_animator.set_playing("idle");
                }

                if let Some(attack) = &unit.attack {
                    // find closest enemy
                    let target = enemy_query
                        .iter()
                        .filter(|(_, enemy_network_entity, enemy_owner)| {
                            enemy_owner.0 != owner.0 && *enemy_network_entity != network_entity
                        })
                        .map(|(enemy_position, enemy_network_entity, _enemy_owner)| {
                            (
                                (enemy_position.position.truncate() - position.position.truncate())
                                    .length(),
                                enemy_network_entity,
                            )
                        })
                        .min_by(|(a_dist, _), (b_dist, _)| a_dist.partial_cmp(b_dist).unwrap());

                    // if closest enemy is within engage range, do so
                    if let Some((enemy_dist, enemy_network_entity)) = target {
                        if enemy_dist <= attack.engage_range {
                            command_queue.add_command(Command::Attack {
                                target_position: None,
                                target_unit: Some(*enemy_network_entity),
                                persue: false,
                            });
                        }
                    }
                }
            }
        },
    );
}

pub fn move_command_system(
    task_pool: Res<ComputeTaskPool>,
    units: Res<Assets<Unit>>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    position_query: Query<&Position, With<Handle<Unit>>>,
    mut query: Query<(
        &Position,
        &mut UnitInstance,
        &mut CommandQueue,
        &Handle<Unit>,
    )>,
) {
    query.par_iter_mut(16).for_each(
        &task_pool,
        |(position, mut unit_instance, mut command_queue, unit_handle)| {
            // no reason to do anything if there is no Command
            if command_queue.is_empty() {
                return;
            }

            match command_queue.current().unwrap() {
                Command::Move {
                    target, precise, ..
                } => {
                    let unit = units.get(&*unit_handle).unwrap();

                    let target_position = match target {
                        CommandTarget::Position(position) => *position,
                        CommandTarget::Ally(entity) | CommandTarget::Enemy(entity) => {
                            let entity = if let Some(entity) = network_entity_registry.get(entity) {
                                entity
                            } else {
                                return;
                            };

                            let position = position_query.get(*entity).unwrap();
                            position.position.truncate()
                        }
                    };

                    let diff = target_position - position.position.truncate();
                    let dist = diff.length();

                    let done = (*precise && dist < 0.1) || (!*precise && dist < unit.size);

                    if done {
                        unit_instance.movement = None;
                        command_queue.complete();
                    } else {
                        unit_instance.movement = Some(target_position);
                    }
                }
                _ => return,
            }
        },
    );
}

#[derive(Serialize, Deserialize)]
pub enum AttackEvent {
    /// Deals flat damage to the target unit.
    Damage(f32),
}

#[derive(Serialize, Deserialize)]
pub struct Attack {
    /// Attack range.
    pub range: f32,
    /// Unit tries to attack target enemy unit when idle and within engage_range.
    pub engage_range: f32,
    /// Disallow attack canceling when the animation is within this.
    pub animation_lock: std::ops::Range<u32>,
    /// Handle events on the specified frames.
    pub events: std::collections::HashMap<u32, AttackEvent>,
}

pub fn attack_command_system(
    network_entity_registry: Res<NetworkEntityRegistry>,
    units: Res<Assets<Unit>>,
    mut unit_instance_query: Query<&mut UnitInstance>,
    position_query: Query<(&NetworkEntity, &Position, &Owner), With<UnitInstance>>,
    mut query: Query<
        (
            Entity,
            &mut UnitDirection,
            &mut CommandQueue,
            &mut UnitAnimator,
            &Animator,
            &Position,
            &Handle<Unit>,
            &NetworkEntity,
            &Owner,
        ),
        With<UnitInstance>,
    >,
) {
    for (
        entity,
        mut unit_direction,
        mut command_queue,
        mut unit_animator,
        animator,
        position,
        unit_handle,
        network_entity,
        owner,
    ) in query.iter_mut()
    {
        // if there is no command in the queue, then what are we even doing?
        if command_queue.is_empty() {
            continue;
        }

        // retard logic, but like, please understand that it sucked to
        // write and i don't want to do it again
        // NOTE: this sucks
        match command_queue.current_mut().unwrap() {
            Command::Attack {
                target_position,
                target_unit,
                persue,
                ..
            } => {
                // might aswell retrive them now
                let unit = units.get(&*unit_handle).unwrap();
                let mut unit_instance = unit_instance_query.get_mut(entity).unwrap();

                // retrive attack and throw error if it unit doesn't have one
                let attack = if let Some(attack) = &unit.attack {
                    attack
                } else {
                    error!("Unit without an attack ended up with an attack command, this should NEVER happen. Blame the client.");

                    // might aswell complete the command so the unit
                    // wont just stand around like an idiot for eternity
                    command_queue.complete();
                    continue;
                };

                // check target position and unit, update them accordingly
                let engaging = if let Some(target_entity) = target_unit {
                    // ensure our target actually exists
                    if let Some(target_entity) = network_entity_registry.get(target_entity) {
                        let target_position = position_query.get(*target_entity).unwrap().1;

                        let diff =
                            target_position.position.truncate() - position.position.truncate();
                        let dist = diff.length();

                        // check if target is within attack range
                        if dist >= attack.range {
                            // if persue is enabled, keep, well, persuing
                            if *persue {
                                unit_instance.movement = Some(target_position.position.truncate());
                                continue;
                            } else if dist <= attack.engage_range {
                                true
                            } else {
                                // if not, unset target_unit
                                *target_unit = None;

                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        // if not, then well, unset
                        *target_unit = None;
                        
                        false
                    }
                } else {
                    false
                };

                // if we don't have a target_unit, but do have a target_position
                // look for target in range
                if target_unit.is_none() && target_position.is_some() || engaging {
                    // look, im not even going to try to explain it,
                    // i just went into a trance, and this happened
                    let target = position_query
                        .iter()
                        .filter(|(_, _, target_owner)| {
                            owner.0 != target_owner.0
                        })
                        .map(|(entity, target_position, _)| {
                            (
                                entity,
                                target_position
                                    .position
                                    .truncate()
                                    .distance(position.position.truncate()),
                            )
                        })
                        .min_by(|(_, dist_a), (_, dist_b)| {
                            dist_a.partial_cmp(dist_b).unwrap()
                        });

                    if let Some((entity, dist)) = target {
                        if dist < attack.range {
                            *target_unit = Some(*entity);
                        }
                    }
                }

                // check to see if we have reached out target_position
                if let Some(t_position) = target_position {
                    let diff = *t_position - position.position.truncate();
                    let dist = diff.length();

                    if dist < 0.1 {
                        *target_position = None;
                    }
                }

                // execute the command
                //
                // first check if we have a target unit
                match target_unit {
                    // if we do, carry out the attack
                    Some(target_entity) => {
                        
                        let target_entity = network_entity_registry.get(target_entity).unwrap();
                        
                        let target_position = position_query.get(*target_entity).unwrap().1;
                        
                        let diff =
                            target_position.position.truncate() - position.position.truncate();
                        let dist = diff.length();

                        if dist > attack.range {
                            unit_instance.movement = Some(target_position.position.truncate());
                            continue;
                        }

                        *unit_direction = UnitDirection::from_vec2(diff);
                        
                        // play attack animation
                        if unit_animator.playing() != "attack" {
                            unit_animator.play("attack");
                        }

                        // stop movement
                        unit_instance.movement = None;

                        // handle attack events
                        if animator.frame_just_changed() {
                            if let Some(event) = attack.events.get(&animator.current_frame()) {
                                match event {
                                    // get UnitInstance and deal damage.
                                    AttackEvent::Damage(damage) => {
                                        let mut target_unit_instance =
                                            unit_instance_query.get_mut(*target_entity).unwrap();

                                        target_unit_instance.subtract_health(*damage);
                                    }
                                }
                            }
                        }
                    }
                    // in not check if we have a target position
                    None => {
                        match target_position {
                            // if we do, move there
                            Some(target_position) => {
                                unit_instance.movement = Some(*target_position);
                            }
                            // if not, we must be done!
                            None => {
                                command_queue.complete();
                                continue;
                            }
                        }
                    }
                }

                // if the animation is playing and within the lock-range, lock the queue
                if unit_animator.playing() == "attack"
                    && animator.current_frame() >= attack.animation_lock.start
                    && animator.current_frame() <= attack.animation_lock.end
                {
                    command_queue.lock();
                } else {
                    command_queue.unlock();
                }
            }
            _ => continue,
        };
    }
}
