use crate::*;

pub struct MousePosition {
    position: Vec2,
    normalized_screen_position: Vec2,
    aspect_ratio: f32,
    camera: Entity,
}

impl MousePosition {
    pub fn new(camera: Entity) -> Self {
        Self {
            position: Vec2::zero(),
            normalized_screen_position: Vec2::zero(),
            aspect_ratio: 1.0,
            camera,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn normalized_screen_position(&self) -> Vec2 {
        self.normalized_screen_position
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }
}

pub fn mouse_position_system(
    mut event_reader: Local<EventReader<CursorMoved>>,
    events: Res<Events<CursorMoved>>,
    windows: Res<Windows>,
    mut mouse_position: ResMut<MousePosition>,
    query: Query<&Transform>,
) {
    let camera_transform = query.get(mouse_position.camera).unwrap();

    for event in event_reader.iter(&events) {
        let window = windows.get(event.id).unwrap();
        let size = Vec2::new(window.width() as f32, window.height() as f32);

        let position = event.position - size / 2.0;

        let world_position = camera_transform.compute_matrix() * position.extend(0.0).extend(1.0);

        mouse_position.position = *isometric::SCREEN_TO_ISO * world_position.truncate().truncate();
        mouse_position.normalized_screen_position =
            Vec2::new(position.x / (size.y / 2.0), position.y / (size.y / 2.0));
        mouse_position.aspect_ratio = size.x / size.y;
    }
}
