use bevy::prelude::*;

#[derive(Clone)]
pub struct Position {
    pub position: Vec3,
}

pub fn position_system(mut query: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in query.iter_mut() {
        let mut translation = Vec3::zero();
        translation.x = position.position.x;
        translation.y = position.position.y / 2.0;
        translation.z -= position.position.y / 1024.0;
        translation.y += position.position.z / 2.0;

        transform.translation = translation;
    }
}
