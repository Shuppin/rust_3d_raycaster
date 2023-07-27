//! Contains the Engine implementation.

use crate::{
    Error,
    render::Renderer,
    game::GameContext
};

use std::time::{Instant, Duration};
use sdl2::{event::Event, keyboard::{Keycode, Scancode}};

/// Serves as the core of the 3D rendering application.
/// 
/// It initialises all the necessary components and systems, configures the program settings,
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

    /// Initiates the main game loop, stopping when a quit event
    /// is recieved or an error occurs.
    pub fn main_loop(&mut self) -> Result<(), Error> {

        let mut frame_start: Instant;

        loop {
            // Capture the time at the frame start
            frame_start = Instant::now();

            // Break the loop if handle_events returns true
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

    /// Handles all SDL Events.
    /// 
    /// Return type `Ok(true)` signifies that a quit event was recieved.
    fn handle_events(&mut self) -> Result<bool, Error> {

        // Recieve new events this frame
        let mut event_pump = self.sdl_context.event_pump()?;

        // Iterate over events
        for event in event_pump.poll_iter() {
            match event {
                // Quit event
                    Event::Quit{..} |
                    Event::KeyDown{keycode: Some(Keycode::Escape), ..}
                    => return Ok(true),  // `true` represents that we want to quit
                // Default
                    _ => {}
            }
        }

        // The only key related events are KeyUp and KeyDown,
        // but we want to know if a key is held.
        // We could store this state internally, in fact we kind of already do that.
        // However such information is already stored in the keyboard state.
        let ks = event_pump.keyboard_state();

        if ks.is_scancode_pressed(Scancode::W) { self.game_context.move_forward()   }
        if ks.is_scancode_pressed(Scancode::S) { self.game_context.move_backwards() }
        if ks.is_scancode_pressed(Scancode::A) { self.game_context.turn_left()      }
        if ks.is_scancode_pressed(Scancode::D) { self.game_context.turn_right()     }

        Ok(false)
    }
}