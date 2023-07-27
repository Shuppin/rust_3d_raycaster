
use crate::{
    Error,
    render::Renderer,
    game::GameContext
};

use std::time::{Instant, Duration};
use sdl2::{event::Event, keyboard::{Keycode, Scancode}};

pub struct Engine {
    sdl_context: sdl2::Sdl,
    game_context: GameContext,
    renderer: Renderer,
    frame_time: Duration,
    frame_time_with_wait: Duration,
}

impl Engine {
    pub fn new(window_title: &'static str, window_width: u32, window_height: u32) -> Result<Self, Error> {
        let sdl_context = sdl2::init()?;
        let game_context = GameContext::new();
        let renderer = Renderer::new(&sdl_context, window_title, window_width, window_height)?;
        let frame_time = Duration::ZERO;
        let frame_time_with_wait = Duration::ZERO;

        Ok(Self {
            sdl_context, game_context, renderer,
            frame_time, frame_time_with_wait
        })
    }

    pub fn set_target_fps(&mut self, fps: i32) {
        self.renderer.render_context.desired_frame_time =
            if fps <= 0 { Duration::ZERO } 
            else { Duration::from_millis((1000/fps) as u64) };
    }

    pub fn main_loop(&mut self) -> Result<(), Error> {

        let mut frame_start: Instant;

        loop {
            frame_start = Instant::now();

            if self.handle_events()? { break }

            self.game_context.tick(self.frame_time_with_wait);

            self.renderer.draw(&self.game_context)?;

            self.wait(frame_start);
        }

        Ok(())
    }

    fn wait(&mut self, frame_start: Instant) {
        self.frame_time = Instant::now().duration_since(frame_start);
        if self.frame_time <= self.renderer.render_context.desired_frame_time {
            std::thread::sleep(self.renderer.render_context.desired_frame_time-self.frame_time);
            self.frame_time_with_wait = self.renderer.render_context.desired_frame_time
        } else {
            self.frame_time_with_wait = self.frame_time;
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