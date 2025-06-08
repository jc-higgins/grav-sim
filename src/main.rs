use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod state;
use crate::app::App;
use crate::state::State;

fn main() {
    let event_loop = EventLoop::<State>::with_user_event().build().unwrap();

    // Poll controlflow for games etc
    // event_loop.set_control_flow(ControlFLow::Poll);
    event_loop.set_control_flow(ControlFlow::Wait);

    // let window_attributes = Window::default_attributes().with_title("A fantastic window.");
    // let window = Some(event_loop.create_window(window_attributes).unwrap());

    let mut app = App::new(&event_loop);
    let _ = event_loop.run_app(&mut app);
}