//! Contains the Engine implementation.

use crate::{
    Error,
    render::Renderer,
    game::GameContext
};

use std::time::{Instant, Duration};
use sdl2::{event::Event, keyboard::{Keycode, Scancode}};

/// Serves as the core of our 3D rendering application, acting as the Game Engine.
/// 
/// It initializes all the necessary components and systems, configures the program settings,
/// and manages the main game loop.
pub struct Engine {
    sdl_context: sdl2::Sdl,
    game_context: GameContext,
    renderer: Renderer,
    delta_time: Duration,
}

impl Engine {
    pub fn new(window_title: &'static str, window_width: u32, window_height: u32) -> Result<Self, Error> {
        let sdl_context = sdl2::init()?;
        let game_context = GameContext::new();
        let renderer = Renderer::new(&sdl_context, window_title, window_width, window_height)?;
        let delta_time = Duration::ZERO;

        Ok(Self {
            sdl_context, game_context, renderer, delta_time
        })
    }

    /// Sets the target frames per second (FPS) for rendering engine.
    /// 
    /// If 'fps' is <= 0, the rendering engine will achieve as high
    /// fps as possible.
    pub fn set_target_fps(&mut self, fps: i32) {
        self.renderer.render_context.desired_frame_time =
            if fps <= 0 { Duration::ZERO } 
            else { Duration::from_millis((1000/fps) as u64) };
    }
    pub fn main_loop(&mut self) -> Result<(), Error> {

        let mut frame_start: Instant;

        loop {
            frame_start = Instant::now();  // Capture the time at the frame start

            if self.handle_events()? { break }

            self.game_context.tick(self.delta_time);

            self.renderer.draw(&self.game_context)?;

            self.wait(frame_start);
        }

        Ok(())
    }

    /// Compares the desired frame time with the current frame time,
    /// subtracting the values and sleeping for that duration.
    /// 
    /// Calclates current frame time based of `frame_start`.
    fn wait(&mut self, frame_start: Instant) {

        let frame_time = Instant::now().duration_since(frame_start);
        
        if frame_time <= self.renderer.render_context.desired_frame_time {
            std::thread::sleep(self.renderer.render_context.desired_frame_time-frame_time);
            self.delta_time = self.renderer.render_context.desired_frame_time
        } else {
            self.delta_time = frame_time;
        }
    }

    fn handle_events(&mut self) -> Result<bool, Error> {
        let mut event_pump = self.sdl_context.event_pump()?;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown{keycode: Some(Keycode::Escape), ..}
                => return Ok(true),
                _ => {}
            }
        }

        let ks = event_pump.keyboard_state();

        if ks.is_scancode_pressed(Scancode::W) { self.game_context.move_forward()   }
        if ks.is_scancode_pressed(Scancode::S) { self.game_context.move_backwards() }
        if ks.is_scancode_pressed(Scancode::A) { self.game_context.turn_left()      }
        if ks.is_scancode_pressed(Scancode::D) { self.game_context.turn_right()     }

        Ok(false)
    }
}