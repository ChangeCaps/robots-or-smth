use crate::*;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UnitDirection {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

impl UnitDirection {
    pub fn from_vec2(vec2: Vec2) -> Self {
        let angle = vec2.y.atan2(vec2.x) / std::f32::consts::PI * 4.0 - 1.0;

        if angle < -3.5 {
            UnitDirection::Left
        } else if angle < -2.5 {
            UnitDirection::DownLeft
        } else if angle < -1.5 {
            UnitDirection::Down
        } else if angle < -0.5 {
            UnitDirection::DownRight
        } else if angle < 0.5 {
            UnitDirection::Right
        } else if angle < 1.5 {
            UnitDirection::UpRight
        } else if angle < 2.5 {
            UnitDirection::Up
        } else if angle < 3.5 {
            UnitDirection::UpLeft
        } else {
            UnitDirection::Left
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnitAnimation {
    up: String,
    up_right: String,
    right: String,
    down_right: String,
    down: String,
    down_left: String,
    left: String,
    up_left: String,
}

impl UnitAnimation {
    pub fn get_animation(&self, direction: &UnitDirection) -> String {
        match direction {
            UnitDirection::Up => self.up.clone(),
            UnitDirection::UpRight => self.up_right.clone(),
            UnitDirection::Right => self.right.clone(),
            UnitDirection::DownRight => self.down_right.clone(),
            UnitDirection::Down => self.down.clone(),
            UnitDirection::DownLeft => self.down_left.clone(),
            UnitDirection::Left => self.left.clone(),
            UnitDirection::UpLeft => self.up_left.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, bevy::reflect::TypeUuid)]
#[uuid = "6a7b1ce9-c18a-437a-9bb1-3bffa3e3d55c"]
pub struct UnitAnimationSet {
    animations: HashMap<String, UnitAnimation>,
}

impl UnitAnimationSet {
    pub fn get(&self, name: impl Into<String>, direction: &UnitDirection) -> Option<String> {
        let name = name.into();

        if let Some(unit_animation) = self.animations.get(&name) {
            Some(unit_animation.get_animation(direction))
        } else {
            None
        }
    }
}

pub struct UnitAnimator {
    current_animation: String,
    reset_animation: bool,
}

impl UnitAnimator {
    pub fn new(current_animation: String) -> Self {
        Self {
            current_animation,
            reset_animation: false,
        }
    }

    pub fn play(&mut self, name: impl Into<String>) {
        let name = name.into();

        self.current_animation = name;
        self.reset_animation = false;
    }

    pub fn playing(&self) -> &String {
        &self.current_animation
    }

    pub fn set_playing(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.current_animation = name;
    }
}

pub fn unit_animation_system(
    unit_animation_sets: Res<Assets<UnitAnimationSet>>,
    mut query: Query<
        (
            &mut Animator,
            &UnitDirection,
            &mut UnitAnimator,
            &Handle<UnitAnimationSet>,
        ),
        Or<(Changed<UnitDirection>, Changed<UnitAnimator>)>,
    >,
) {
    for (mut animator, direction, mut unit_animator, unit_animation_set_handle) in query.iter_mut()
    {
        let unit_animation_set = unit_animation_sets
            .get(&*unit_animation_set_handle)
            .unwrap();

        let animation = unit_animation_set
            .get(unit_animator.current_animation.clone(), &direction)
            .unwrap();

        if unit_animator.reset_animation {
            animator.play(animation);
            unit_animator.reset_animation = false;
        } else {
            animator.set_playing(animation);
        }
    }
}

pub struct UnitAnimationSetLoader;

ron_loader!(UnitAnimationSetLoader, "unit_anim" => UnitAnimationSet);

pub struct UnitAnimationPlugin;

impl Plugin for UnitAnimationPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.add_asset::<UnitAnimationSet>();
        app_builder.add_asset_loader(UnitAnimationSetLoader);
        app_builder.add_system(unit_animation_system.system());
    }
}
