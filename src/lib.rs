#[cfg(target_arch = "wasm32")]
use tracing_subscriber::fmt::format::Pretty;
#[cfg(target_arch = "wasm32")]
use tracing_subscriber::fmt::time::UtcTime;
#[cfg(target_arch = "wasm32")]
use tracing_subscriber::prelude::*;
#[cfg(target_arch = "wasm32")]
use tracing_web::{performance_layer, MakeConsoleWriter};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod app;
pub mod state;

pub use app::App;
pub use state::State;

use winit::event_loop::{ControlFlow, EventLoop};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
 