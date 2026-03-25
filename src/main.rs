use winit::event_loop::{ControlFlow, EventLoop};

pub mod ecs;
pub mod engine;
pub mod gpu;
pub mod systems;

fn main() {
    // NOTE: tracing calls (info!, error!) are silent until a subscriber is set up.
    // Add tracing-subscriber when you want to see log output.

    // Create the OS event loop
    let event_loop = EventLoop::new().unwrap();

    // Tell it to constantly poll so game loop runs forever
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create App
    let mut app = engine::app::App::new();

    // Hand complete control over to app.rs

    if let Err(e) = event_loop.run_app(&mut app) {
        tracing::error!("Event loop error: {e}");
        std::process::exit(1);
    }
}
