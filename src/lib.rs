#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod app;
pub mod state;

pub use app::App;
pub use state::State;

use winit::event_loop::{ControlFlow, EventLoop};


pub fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::<State>::with_user_event().build().unwrap();

    // Poll controlflow for games etc
    // event_loop.set_control_flow(ControlFLow::Poll);
    event_loop.set_control_flow(ControlFlow::Wait);

    // let window_attributes = Window::default_attributes().with_title("A fantastic window.");
    // let window = Some(event_loop.create_window(window_attributes).unwrap());

    let mut app = App::new(&event_loop);
    let _ = event_loop.run_app(&mut app);

    Ok(())
}