# grav-sim

A gravitational interaction simulation built in Rust using WGPU for cross-platform graphics rendering. This project can run both as a native desktop application and in web browsers via WebAssembly.

## Current State

This project is in early development stages. The foundational graphics and windowing infrastructure is in place using WGPU and winit, but the actual gravitational physics simulation is not yet implemented. Currently, the application creates a window with basic event handling and a black canvas ready for rendering.

**Implemented:**
- Cross-platform window creation (desktop + web)
- WGPU graphics initialization
- WebAssembly compilation support
- Basic application lifecycle management

**TODO:**
- Gravitational physics simulation
- Particle/body rendering
- Interactive controls
- Performance optimizations

## Dependencies

- **Rust**: Modern Rust toolchain (edition 2021)
- **Graphics**: WGPU for cross-platform rendering
- **Windowing**: winit for event handling and window management
- **Web Target**: WebAssembly with wasm-bindgen

## Building and Running

### Prerequisites

Install Rust from [rustup.rs](https://rustup.rs/)

For web builds, install wasm-pack:
```bash
# Option 1: Via conda/mamba (using provided environment.yml)
conda env create -f environment.yml
conda activate grav-sim

# Option 2: Direct installation
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Desktop Application

```bash
# Build and run in debug mode
cargo run

# Build optimized release
cargo build --release
```

### Web Application

```bash
# Build for web target
wasm-pack build --target web

# Serve locally (requires a local server)
python -m http.server 8000
# Then open http://localhost:8000 in your browser
```

## Project Structure

```
src/
├── main.rs      # Native desktop entry point
├── lib.rs       # Library root and web entry point
├── app.rs       # Application logic and event handling
└── state.rs     # Simulation state and rendering (skeleton)
```

## License

MIT