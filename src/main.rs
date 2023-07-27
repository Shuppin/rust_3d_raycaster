mod texture;
mod player;
mod engine;
mod render;
mod game;
mod util;

type Error = Box<dyn std::error::Error>;

use engine::Engine;

fn main() -> Result<(), Error> {

    // TODO:
    // Make code more robust in general
    // (how do i cast numeric types in a non horrendously ugly way)
    // Collision is frame-rate dependant, completely breaks at fps < 5
    // Add mouse movement/strafing (DOOM music starts to play)
    // Add immediate mode GUI

    let mut system = Engine::new("3D Raycaster", 900, 600)?;

    system.set_target_fps(0);
    system.main_loop()?;

    Ok(())
}