use log::info;

pub mod app;
pub mod state;

pub use app::App;
pub use state::{State, Body};

use winit::event_loop::{EventLoop};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    info!("Starting application");
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            info!("WASM logger initialized");
        } else {
            tracing_subscriber::fmt::init();
            info!("Desktop logger initialized");
        }
    }

    let event_loop = EventLoop::<State>::with_user_event().build().unwrap();
    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}
 