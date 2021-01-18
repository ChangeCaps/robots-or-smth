use crate::*;

pub struct MousePosition {
    position: Vec2,
    world_position: Vec2,
    screen_position: Vec2,
    screen_world_position: Vec2,
    normalized_screen_position: Vec2,
    window_size: Vec2,
    aspect_ratio: f32,
}

impl Default for MousePosition {
    fn default() -> Self {
        Self {
            position: Vec2::zero(),
            world_position: Vec2::zero(),
            screen_position: Vec2::zero(),
            screen_world_position: Vec2::zero(),
            normalized_screen_position: Vec2::zero(),
            window_size: Vec2::zero(),
            aspect_ratio: 1.0,
        }
    }
}

impl MousePosition {
    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn world_position(&self) -> Vec2 {
        self.world_position
    }

    pub fn screen_position(&self) -> Vec2 {
        self.screen_position
    }

    pub fn screen_world_position(&self) -> Vec2 {
        self.screen_world_position
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
    main_camera: Res<MainCamera>,
    mut mouse_position: ResMut<MousePosition>,
    query: Query<&Transform>,
) {
    let camera_transform = query.get(main_camera.0).unwrap();

    for event in event_reader.iter(&events) {
        let window = windows.get(event.id).unwrap();
        let size = Vec2::new(window.width() as f32, window.height() as f32);

        mouse_position.window_size = size;

        mouse_position.aspect_ratio = size.x / size.y;

        mouse_position.screen_position = event.position;
    }

    let position = mouse_position.screen_position - mouse_position.window_size / 2.0;

    let world_position = camera_transform.compute_matrix() * position.extend(0.0).extend(1.0);
    let screen_world_position =
        camera_transform.compute_matrix() * mouse_position.position.extend(0.0).extend(0.5);

    mouse_position.position = *isometric::SCREEN_TO_ISO * world_position.truncate().truncate();

    mouse_position.normalized_screen_position = Vec2::new(
        position.x / (mouse_position.window_size.y / 2.0),
        position.y / (mouse_position.window_size.y / 2.0),
    );

    mouse_position.world_position = world_position.truncate().truncate();

    mouse_position.screen_world_position = screen_world_position.truncate().truncate();
}
