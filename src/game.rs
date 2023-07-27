use std::time::Duration;

use crate::{player::Player, Error};

pub const WORLD_SIZE: usize = 24;

/// Stores all the information about the game state
pub struct GameContext {
    pub player: Player,
    pub world: [[i32; WORLD_SIZE]; WORLD_SIZE],
    move_state: [bool; 4]
}

impl GameContext {
    pub fn new() -> Result<Self, Error> {

        // The world object is a 2D array of cells ranging from 0-9.
        // Current limitations of the engine require world to have
        // square dimensions, will likely change in future.
        let world: [[i32; WORLD_SIZE]; WORLD_SIZE] = [
            [1,1,2,1,1,1,2,1,1,1,2,1,1,1,2,1,1,1,2,1,1,3,3,1],
            [1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,1],
            [3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3,0,0,0,0,0,0,1],
            [3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3,0,0,0,0,0,0,1],
            [1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,1],
            [1,1,0,1,1,1,2,1,1,1,0,1,1,1,0,1,1,1,0,1,1,0,0,1],
            [1,0,0,0,1,0,0,0,0,0,0,0,1,1,0,1,1,0,0,0,1,0,0,1],
            [2,0,0,0,2,0,0,0,0,0,0,0,1,2,0,2,1,0,0,0,1,0,0,1],
            [1,0,0,8,1,0,0,0,0,0,0,0,1,1,0,1,1,0,0,0,1,0,0,1],
            [1,1,2,1,1,1,2,1,1,1,2,1,1,1,0,1,1,1,0,1,1,0,0,1],
            [7,7,7,7,7,7,7,7,7,7,7,7,1,0,0,0,0,0,0,0,0,0,0,1],
            [7,0,0,0,0,0,0,0,0,0,0,7,1,0,0,0,0,0,0,0,0,0,0,1],
            [7,0,0,0,0,0,0,0,0,0,0,7,1,0,0,0,0,0,0,0,1,0,0,1],
            [7,0,0,0,0,0,0,0,0,0,0,7,1,0,0,0,0,0,0,0,1,3,3,1],
            [7,0,0,0,0,0,0,0,0,0,0,7,1,0,0,0,0,0,0,0,1,1,1,1],
            [7,0,0,0,0,0,0,0,0,0,0,7,1,0,0,0,0,0,0,0,1,1,1,1],
            [7,0,0,0,0,0,0,0,0,0,0,7,1,0,0,0,0,0,0,0,1,3,3,1],
            [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1],
            [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
            [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
            [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
            [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
            [1,2,3,4,5,6,7,8,9,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
            [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]
        ];

        // Check for error in the world
        for (y, row) in world.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                // If the current cell is on the boundrary of the world
                if x == 0 || y == 0 || x == WORLD_SIZE-1 || y == WORLD_SIZE-1 {
                    if cell == &0 {
                        return Err(format!("WORLD - World contains exposed wall at ({x}, {y})").into())
                    }
                }
                // If the current is above or below the set of valid cells
                if cell > &9 || cell < &0 {
                    return Err(format!("WORLD - World contains invalid wall at ({x}, {y})").into())
                }
            }
        }


        Ok(Self{
            player: Player::new(),
            world,
            move_state: [false; 4]
        })

    }

    /// Increment the game state by one tick.
    /// This function is called periodically to advance the game state by one unit of time, known as a "tick." 
    /// 
    /// At the moment it only updates the player position,
    /// as that is the only dynamic state.
    pub fn tick(&mut self, dt: Duration) {
        self.handle_player_movement(dt);
    }

    /// Enables 'forward' in the move state
    pub fn move_forward(&mut self) {
        self.move_state[0] = true;
    }

    /// Enables 'backwards' in the move state
    pub fn move_backwards(&mut self) {
        self.move_state[1] = true;
    }

    /// Enables 'left' in the move state
    pub fn turn_left(&mut self) {
        self.move_state[2] = true;
    }

    /// Enables 'left' in the move state
    pub fn turn_right(&mut self) {
        self.move_state[3] = true;
    }

    /// Modifies the player position based on the current move
    /// state and the current frame time.
    fn handle_player_movement(&mut self, dt: Duration) {

        // Calculate a new move and rotation speed based of the current frame time.
        // Ensures movement speed is consistent with varying framerate.
        let actual_move_speed = self.player.move_speed * (dt.as_millis() as f32) / 1000.0;
        let actual_rot_speed = self.player.rot_speed * (dt.as_millis() as f32) / 1000.0;

        // Create shorthand variables for better readability
        let pos_x = &mut self.player.position.x;
        let pos_y = &mut self.player.position.y;

        let dir_x = &mut self.player.camera_direction.x;
        let dir_y = &mut self.player.camera_direction.y;

        let plane_x = &mut self.player.camera_plane.x;
        let plane_y = &mut self.player.camera_plane.y;

        // Credit to [Lode's Computer Graphics Tutorial](https://lodev.org/cgtutor/raycasting.html)
        // for movement code

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

        // if Left==true
        if self.move_state[2] {
            prev_dir_x = *dir_x;
            *dir_x = *dir_x * actual_rot_speed.cos() - *dir_y * actual_rot_speed.sin();
            *dir_y = prev_dir_x * actual_rot_speed.sin() + *dir_y * actual_rot_speed.cos();
            prev_plane_x = *plane_x;
            *plane_x = *plane_x * actual_rot_speed.cos() - *plane_y * actual_rot_speed.sin();
            *plane_y = prev_plane_x * actual_rot_speed.sin() + *plane_y * actual_rot_speed.cos();
        }

        // if Right==true
        if self.move_state[3] {
            prev_dir_x = *dir_x;
            *dir_x = *dir_x * (-actual_rot_speed).cos() - *dir_y * (-actual_rot_speed).sin();
            *dir_y = prev_dir_x * (-actual_rot_speed).sin() + *dir_y * (-actual_rot_speed).cos();
            prev_plane_x = *plane_x;
            *plane_x = *plane_x * (-actual_rot_speed).cos() - *plane_y * (-actual_rot_speed).sin();
            *plane_y = prev_plane_x * (-actual_rot_speed).sin() + *plane_y * (-actual_rot_speed).cos();
        }

        // Once we have processed all the movements, reset the move state
        self.move_state = [false; 4]

    }
}