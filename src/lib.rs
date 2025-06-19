use log::info;

pub mod app;
pub mod state;

pub use app::App;
pub use state::State;

use winit::event_loop::{ControlFlow, ActiveEventLoop};
use winit::window::Window;

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

    let event_loop = ActiveEventLoop::<State>::with_user_event().build().unwrap();
    // Poll controlflow for games etc
    // event_loop.set_control_flow(ControlFLow::Poll);
    event_loop.set_control_flow(ControlFlow::Wait);

    let window_attributes = Window::default_attributes().with_title("A fantastic window.");
    let window: Option<winit::window::Window> = Some(event_loop.create_window(window_attributes).unwrap());

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.body()?; //.get_element_by_id("wasm-example")?;
                let canvas =
                    web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect(
                "Couldn't append canvas to document body.",
            );
    }

    let mut app = App::new(&event_loop);
    let _ = event_loop.run_app(&mut app);
}
 