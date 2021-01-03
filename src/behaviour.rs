use crate::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Behaviour {
    Move { target: Vec2 },
    Attack,
    Idle,
}

pub fn unit_command_behaviour_system(
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

                *direction = UnitDirection::from_vec2(step);

                position.position += step.extend(0.0) * move_dist;
            }
            Behaviour::Attack => {}
            Behaviour::Idle => {}
        }
    }
}
