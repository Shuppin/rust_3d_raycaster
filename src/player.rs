use crate::util::Vector2;

pub struct Player {
    pub move_speed: f32,
    pub rot_speed: f32,
    pub position: Vector2<f32>,
    pub camera_direction: Vector2<f32>,
    pub camera_plane: Vector2<f32>
}

impl Player {
    pub fn new() -> Player {
        Player {
            move_speed: 10.0,
            rot_speed: 5.0,
            position: Vector2::new(22.0, 12.0),
            camera_direction: Vector2::new(-1.0, 0.0),
            camera_plane: Vector2::new(0.0, 0.66)
        }
    }
    
}