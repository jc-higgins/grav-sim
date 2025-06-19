use std::sync::Arc;
use anyhow::Result;

use winit::{
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct State {
    pub window: Arc<Window>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        Ok(Self {
            window,
        })
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {
        //
    }

    pub fn render(&mut self) {
        self.window.request_redraw();

        // Can add some rendering here later
    }
}