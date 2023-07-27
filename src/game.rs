use std::time::Duration;

use crate::player::Player;

pub struct GameContext {
    pub player: Player,
    pub world: Vec<Vec<i32>>,
    move_state: [bool; 4]
}

impl GameContext {
    pub fn new() -> Self {

        let world: Vec<Vec<i32>> = vec![
            vec![1,1,2,1,1,1,2,1,1,1,2,1,1,1,2,1,1,1,2,1,1,3,3,1],
            vec![1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,1],
            vec![3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3,0,0,0,0,0,0,1],
            vec![3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3,0,0,0,0,0,0,1],
            vec![1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,1],
            vec![1,1,0,1,1,1,2,1,1,1,0,1,1,1,0,1,1,1,0,1,1,0,0,1],
            vec![1,0,0,0,1,0,0,0,0,0,0,0,1,1,0,1,1,0,0,0,1,0,0,1],
            vec![2,0,0,0,2,0,0,0,0,0,0,0,1,2,0,2,1,0,0,0,1,0,0,1],
            vec![1,0,0,8,1,0,0,0,0,0,0,0,1,1,0,1,1,0,0,0,1,0,0,1],
            vec![1,1,2,1,1,1,2,1,1,1,2,1,1,1,0,1,1,1,0,1,1,0,0,1],
            vec![7,7,7,7,7,7,7,7,7,7,7,7,1,0,0,0,0,0,0,0,0,0,0,1],
            vec![7,0,0,0,0,0,0,0,0,0,0,7,1,0,0,0,0,0,0,0,0,0,0,1],
            vec![7,0,0,0,0,0,0,0,0,0,0,7,1,0,0,0,0,0,0,0,1,0,0,1],
            vec![7,0,0,0,0,0,0,0,0,0,0,7,1,0,0,0,0,0,0,0,1,0,3,1],
            vec![7,0,0,0,0,0,0,0,0,0,0,7,1,0,0,0,0,0,0,0,1,0,0,0],
            vec![7,0,0,0,0,0,0,0,0,0,0,7,1,0,0,0,0,0,0,0,1,0,0,0],
            vec![7,0,0,0,0,0,0,0,0,0,0,7,1,0,0,0,0,0,0,0,1,3,3,1],
            vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1],
            vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
            vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
            vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
            vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
            vec![1,2,3,4,5,6,7,8,9,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
            vec![1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]
        ];


        Self {
            player: Player::new(),
            world,
            move_state: [false; 4]
        }

    }

    pub fn tick(&mut self, dt: Duration) {
        self.handle_player_movement(dt);
    }

    pub fn move_forward(&mut self) {
        self.move_state[0] = true;
    }

    pub fn move_backwards(&mut self) {
        self.move_state[1] = true;
    }

    pub fn turn_left(&mut self) {
        self.move_state[2] = true;
    }

    pub fn turn_right(&mut self) {
        self.move_state[3] = true;
    }

    fn handle_player_movement(&mut self, dt: Duration) {

        let actual_move_speed = self.player.move_speed * (dt.as_millis() as f32) / 1000.0;
        let actual_rot_speed = self.player.rot_speed * (dt.as_millis() as f32) / 1000.0;

        let pos_x = &mut self.player.position.x;
        let pos_y = &mut self.player.position.y;

        let dir_x = &mut self.player.camera_direction.x;
        let dir_y = &mut self.player.camera_direction.y;

        let plane_x = &mut self.player.camera_plane.x;
        let plane_y = &mut self.player.camera_plane.y;

        // if Forward==true
        if self.move_state[0] {
            if self.world[(*pos_x + *dir_x * actual_move_speed) as usize][*pos_y as usize] == 0 {
                *pos_x += *dir_x * actual_move_speed
            }
            if self.world[*pos_x as usize][(*pos_y + *dir_y * actual_move_speed) as usize] == 0 {
                *pos_y += *dir_y * actual_move_speed
            }
        }

        // if Backwards==true
        if self.move_state[1] {
            if self.world[(*pos_x - *dir_x * actual_move_speed) as usize][*pos_y as usize] == 0 {
                *pos_x -= *dir_x * actual_move_speed
            }
            if self.world[*pos_x as usize][(*pos_y - *dir_y * actual_move_speed) as usize] == 0 {
                *pos_y -= *dir_y * actual_move_speed
            }
        }

        let mut prev_dir_x: f32;
        let mut prev_plane_x: f32;

        if self.move_state[2] {
            prev_dir_x = *dir_x;
            *dir_x = *dir_x * actual_rot_speed.cos() - *dir_y * actual_rot_speed.sin();
            *dir_y = prev_dir_x * actual_rot_speed.sin() + *dir_y * actual_rot_speed.cos();
            prev_plane_x = *plane_x;
            *plane_x = *plane_x * actual_rot_speed.cos() - *plane_y * actual_rot_speed.sin();
            *plane_y = prev_plane_x * actual_rot_speed.sin() + *plane_y * actual_rot_speed.cos();
        }

        if self.move_state[3] {
            prev_dir_x = *dir_x;
            *dir_x = *dir_x * (-actual_rot_speed).cos() - *dir_y * (-actual_rot_speed).sin();
            *dir_y = prev_dir_x * (-actual_rot_speed).sin() + *dir_y * (-actual_rot_speed).cos();
            prev_plane_x = *plane_x;
            *plane_x = *plane_x * (-actual_rot_speed).cos() - *plane_y * (-actual_rot_speed).sin();
            *plane_y = prev_plane_x * (-actual_rot_speed).sin() + *plane_y * (-actual_rot_speed).cos();
        }

        self.move_state = [false; 4]

    }
}