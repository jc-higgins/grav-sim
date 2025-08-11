use anyhow::Result;
use std::sync::Arc;

use wgpu::{naga::valid::WidthError, Device, Instance, Queue, Surface};
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct State<'a> {
    pub window: Arc<Window>,
    pub instance: Instance,
    pub surface: Surface<'a>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub config: wgpu::SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,
    pub bodies: Vec<Body>,
    pub g_constant: f32,
    pub time_step: f32,
}

impl<'a> State<'a> {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        // Create an instance to interact with the GPU
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        };
        let instance = wgpu::Instance::new(&instance_descriptor);

        // Create a surface to render to
        let surface = instance.create_surface(window.clone()).unwrap();

        // Request an adapter + device to
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = match adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("GPU"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
        {
            Ok((device, queue)) => (device, queue),
            Err(e) => {
                eprintln!("Error requesting device: {e:?}");
                std::process::exit(1);
            }
        };

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let present_mode = if caps.present_modes.contains(wgpu::PresentMode::Fifo) {
            wgpu::PresentMode::Fifo
        } else {
            caps.present_modes[0]
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            window,
            instance,
            surface,
            size,
            config,
            device,
            queue,
            bodies: vec![
                Body::new(100.0, (-1.0, 0.0), (0.0, 1.0)).unwrap(),
                Body::new(100.0, (1.0, 0.0), (0.0, -1.0)).unwrap(),
            ],
            g_constant: 1.0,
            time_step: 0.0001, // Small time step for accuracy
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.size.width = width;
        self.size.height = height;
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) {
        self.window.request_redraw();

        // Can add some rendering here later
    }

    pub fn total_kinetic_energy(&self) -> f32 {
        self.bodies
            .iter()
            .fold(0.0, |s, x| s + x.get_kinetic_energy())
    }

    pub fn total_potential_energy(&self) -> f32 {
        let mut potential_energy = 0.0;
        for i in 0..self.bodies.len() {
            for j in (i + 1)..self.bodies.len() {
                let distance = self.bodies[i].distance_to(&self.bodies[j]);
                // U = -G * m1 * m2 / r
                potential_energy -= self.g_constant * self.bodies[i].mass * self.bodies[j].mass / distance;
            }
        }
        potential_energy
    }

    pub fn total_energy(&self) -> f32 {
        self.total_kinetic_energy() + self.total_potential_energy()
    }

    pub fn step(&mut self) {
        // 1. Pairwise forces
        let n = self.bodies.len();
        let mut forces = vec![(0.0, 0.0); n];

        // a(t)
        for i in 0..n {
            for j in (i + 1)..n {
                let f = self.bodies[i].gravitational_force(&self.bodies[j], self.g_constant);

                forces[i].0 += f.0 / self.bodies[i].mass;
                forces[i].1 += f.1 / self.bodies[i].mass;
                forces[j].0 -= f.0/ self.bodies[j].mass;
                forces[j].1 -= f.1/ self.bodies[j].mass;
            }
        }

        // x(t+dt) = x(t) + v(t) dt + 0.5 a(t) dt^2
        for i in 0..n {
            self.bodies[i].position.0 += self.bodies[i].velocity.0 * self.time_step + 0.5 * forces[i].0 * self.time_step * self.time_step;
            self.bodies[i].position.1 += self.bodies[i].velocity.1 * self.time_step + 0.5 * forces[i].1 * self.time_step * self.time_step;
        }

        // a(t+dt)
        let mut forces_new = vec![(0.0, 0.0); n];
        for i in 0..n {
            for j in (i + 1)..n {
                let f = self.bodies[i].gravitational_force(&self.bodies[j], self.g_constant);
                forces_new[i].0 += f.0 / self.bodies[i].mass;
                forces_new[i].1 += f.1 / self.bodies[i].mass;
                forces_new[j].0 -= f.0 / self.bodies[j].mass;
                forces_new[j].1 -= f.1 / self.bodies[j].mass;
            }
        }
        
        // v(t+dt) = v(t) + 0.5 (a(t) + a(t+dt)) dt
        for i in 0..n {
            self.bodies[i].velocity.0 += 0.5 * (forces[i].0 + forces_new[i].0) * self.time_step;
            self.bodies[i].velocity.1 += 0.5 * (forces[i].1 + forces_new[i].1) * self.time_step;
        }
    }
}

#[derive(Copy, Clone)]
pub struct Body {
    pub mass: f32,
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub radius: f32,
}

impl Body {
    pub fn new(mass: f32, position: (f32, f32), velocity: (f32, f32)) -> Result<Self> {
        let radius: f32 = 1.0;
        Ok(Self {
            mass,
            position,
            velocity,
            radius,
        })
    }

    pub fn distance_to(&self, other: &Body) -> f32 {
        let dx = self.position.0 - other.position.0;
        let dy = self.position.1 - other.position.1;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn gravitational_force(&self, other: &Body, gravity: f32) -> (f32, f32) {
        let distance = self.distance_to(other);
        let direct_force = gravity * self.mass * other.mass / (distance * distance);
        let dx = other.position.0 - self.position.0;
        let dy = other.position.1 - self.position.1;
        let fx = direct_force * dx / distance;
        let fy = direct_force * dy / distance;
        (fx, fy)
    }

    pub fn update(&mut self, acceleration: (f32, f32), dt: f32) {
        self.velocity.0 += acceleration.0 * dt;
        self.velocity.1 += acceleration.1 * dt;
        self.position.0 += self.velocity.0 * dt;
        self.position.1 += self.velocity.1 * dt;
    }

    pub fn get_kinetic_energy(&self) -> f32 {
        0.5 * self.mass * (self.velocity.0 * self.velocity.0 + self.velocity.1 * self.velocity.1)
    }

    pub fn get_linear_momentum(&self) -> (f32, f32) {
        (self.mass * self.velocity.0, self.mass * self.velocity.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_creation() {
        let body = Body::new(100.0, (0.0, 0.0), (1.0, 0.0)).unwrap();

        assert_eq!(body.mass, 100.0);
        assert_eq!(body.position, (0.0, 0.0));
        assert_eq!(body.velocity, (1.0, 0.0));
        // Test that radius was calculated (adjust based on your formula)
        assert!(body.radius > 0.0);
    }

    #[test]
    fn test_distance_calculation() {
        let body1 = Body::new(1.0, (0.0, 0.0), (0.0, 0.0)).unwrap();
        let body2 = Body::new(1.0, (3.0, 4.0), (0.0, 0.0)).unwrap();

        let distance = body1.distance_to(&body2);
        assert!((distance - 5.0).abs() < 1e-6); // 3-4-5 triangle (using f32 precision)
    }

    #[test]
    fn test_gravitational_force_symmetry() {
        let body1 = Body::new(100.0, (0.0, 0.0), (0.0, 0.0)).unwrap();
        let body2 = Body::new(200.0, (1.0, 0.0), (0.0, 0.0)).unwrap();
        let g = 1.0; // Simplified constant

        let force_12 = body1.gravitational_force(&body2, g);
        let force_21 = body2.gravitational_force(&body1, g);

        // Forces should be equal and opposite (Newton's third law)
        assert!((force_12.0 + force_21.0).abs() < 1e-10);
        assert!((force_12.1 + force_21.1).abs() < 1e-10);
    }

    #[test]
    fn test_gravitational_force_magnitude() {
        let body1 = Body::new(100.0, (0.0, 0.0), (0.0, 0.0)).unwrap();
        let body2 = Body::new(200.0, (2.0, 0.0), (0.0, 0.0)).unwrap();
        let g = 1.0;

        let force = body1.gravitational_force(&body2, g);

        // F = G * m1 * m2 / r^2 = 1 * 100 * 200 / 4 = 5000
        let expected_magnitude = 5000.0;
        let actual_magnitude = (force.0 * force.0 + force.1 * force.1).sqrt();

        assert!((actual_magnitude - expected_magnitude).abs() < 1e-10);

        // Force should point in positive x direction (toward body2)
        assert!(force.0 > 0.0);
        assert!(force.1.abs() < 1e-10);
    }

    #[test]
    fn test_body_update() {
        let mut body = Body::new(1.0, (0.0, 0.0), (0.0, 0.0)).unwrap();
        let acceleration = (2.0, 1.0);
        let dt = 1.0;

        body.update(acceleration, dt);

        // After 1 second with acceleration (2, 1):
        // velocity = (0, 0) + (2, 1) * 1 = (2, 1)
        // position = (0, 0) + (2, 1) * 1 = (2, 1)
        assert_eq!(body.velocity, (2.0, 1.0));
        assert_eq!(body.position, (2.0, 1.0));
    }

    #[test]
    fn test_get_momentum() {
        let body = Body::new(1.0, (0.0, 0.0), (1.0, 0.0)).unwrap();
        assert_eq!(body.get_linear_momentum(), (1.0, 0.0));
    }

    #[test]
    fn test_energy_conservation_two_body() {
        // Create a minimal state for testing (no GPU resources needed)
        let mut bodies = vec![
            Body::new(100.0, (-1.0, 0.0), (0.0, 1.0)).unwrap(),
            Body::new(100.0, (1.0, 0.0), (0.0, -1.0)).unwrap(),
        ];
        let g_constant = 1.0;
        let time_step = 0.0001;

        // Calculate total energy (kinetic + potential)
        let calculate_total_energy = |bodies: &[Body]| -> f32 {
            let kinetic: f32 = bodies.iter().map(|b| b.get_kinetic_energy()).sum();
            let mut potential = 0.0;
            for i in 0..bodies.len() {
                for j in (i + 1)..bodies.len() {
                    let distance = bodies[i].distance_to(&bodies[j]);
                    potential -= g_constant * bodies[i].mass * bodies[j].mass / distance;
                }
            }
            kinetic + potential
        };

        let initial_energy = calculate_total_energy(&bodies);

        // Run a few simulation steps
        for _ in 0..10 {
            // 1. Pairwise forces
            let n = bodies.len();
            let mut forces = vec![(0.0, 0.0); n];

            // a(t)
            for i in 0..n {
                for j in (i + 1)..n {
                    let f = bodies[i].gravitational_force(&bodies[j], g_constant);

                    forces[i].0 += f.0 / bodies[i].mass;
                    forces[i].1 += f.1 / bodies[i].mass;
                    forces[j].0 -= f.0/ bodies[j].mass;
                    forces[j].1 -= f.1/ bodies[j].mass;
                }
            }

            // x(t+dt) = x(t) + v(t) dt + 0.5 a(t) dt^2
            for i in 0..n {
                bodies[i].position.0 += bodies[i].velocity.0 * time_step + 0.5 * forces[i].0 * time_step * time_step;
                bodies[i].position.1 += bodies[i].velocity.1 * time_step + 0.5 * forces[i].1 * time_step * time_step;
            }

            // a(t+dt)
            let mut forces_new = vec![(0.0, 0.0); n];
            for i in 0..n {
                for j in (i + 1)..n {
                    let f = bodies[i].gravitational_force(&bodies[j], g_constant);
                    forces_new[i].0 += f.0 / bodies[i].mass;
                    forces_new[i].1 += f.1 / bodies[i].mass;
                    forces_new[j].0 -= f.0 / bodies[j].mass;
                    forces_new[j].1 -= f.1 / bodies[j].mass;
                }
            }
        
            // v(t+dt) = v(t) + 0.5 (a(t) + a(t+dt)) dt
            for i in 0..n {
                bodies[i].velocity.0 += 0.5 * (forces[i].0 + forces_new[i].0) * time_step;
                bodies[i].velocity.1 += 0.5 * (forces[i].1 + forces_new[i].1) * time_step;
            }
        }

        let final_energy = calculate_total_energy(&bodies);
        let final_kinetic: f32 = bodies.iter().map(|b| b.get_kinetic_energy()).sum();

        // Total energy should be conserved (within numerical error)
        println!("Initial total energy: {initial_energy}, Final total energy: {final_energy}");
        println!("Final kinetic energy: {final_kinetic}");
        println!("Total energy difference: {}", (final_energy - initial_energy).abs());
        assert!((final_energy - initial_energy).abs() < 0.01);
    }

    #[test]
    fn test_momentum_conservation_two_body() {
        // Create a minimal state for testing (no GPU resources needed)
        let mut bodies = vec![
            Body::new(100.0, (-1.0, 0.0), (0.0, 1.0)).unwrap(),
            Body::new(100.0, (1.0, 0.0), (0.0, -1.0)).unwrap(),
        ];
        let g_constant = 1.0;
        let time_step = 0.0001;

        let (px0, py0) = bodies.iter().fold((0.0, 0.0), |(sx, sy), b| {
            let (px, py) = b.get_linear_momentum();
            (sx + px, sy + py)
        });

        let p0 = (px0 * px0 + py0 * py0).sqrt();

        // Run a few simulation steps
        for _ in 0..10 {
            // 1. Pairwise forces
            let n = bodies.len();
            let mut forces = vec![(0.0, 0.0); n];

            // a(t)
            for i in 0..n {
                for j in (i + 1)..n {
                    let f = bodies[i].gravitational_force(&bodies[j], g_constant);

                    forces[i].0 += f.0 / bodies[i].mass;
                    forces[i].1 += f.1 / bodies[i].mass;
                    forces[j].0 -= f.0/ bodies[j].mass;
                    forces[j].1 -= f.1/ bodies[j].mass;
                }
            }

            // x(t+dt) = x(t) + v(t) dt + 0.5 a(t) dt^2
            for i in 0..n {
                bodies[i].position.0 += bodies[i].velocity.0 * time_step + 0.5 * forces[i].0 * time_step * time_step;
                bodies[i].position.1 += bodies[i].velocity.1 * time_step + 0.5 * forces[i].1 * time_step * time_step;
            }

            // a(t+dt)
            let mut forces_new = vec![(0.0, 0.0); n];
            for i in 0..n {
                for j in (i + 1)..n {
                    let f = bodies[i].gravitational_force(&bodies[j], g_constant);
                    forces_new[i].0 += f.0 / bodies[i].mass;
                    forces_new[i].1 += f.1 / bodies[i].mass;
                    forces_new[j].0 -= f.0 / bodies[j].mass;
                    forces_new[j].1 -= f.1 / bodies[j].mass;
                }
            }
        
            // v(t+dt) = v(t) + 0.5 (a(t) + a(t+dt)) dt
            for i in 0..n {
                bodies[i].velocity.0 += 0.5 * (forces[i].0 + forces_new[i].0) * time_step;
                bodies[i].velocity.1 += 0.5 * (forces[i].1 + forces_new[i].1) * time_step;
            }
        }

        let (px1, py1) = bodies.iter().fold((0.0, 0.0), |(sx, sy), b| {
            let (px, py) = b.get_linear_momentum();
            (sx + px, sy + py)
        });


        let p1: f32 = (px1 * px1 + py1 * py1).sqrt();

        // Total energy should be conserved (within numerical error)
        println!("Initial momentum: {p0}, Final momentum: {p1}");
        println!("Total energy difference: {}", (p1 - p0).abs());
        assert!((p1 - p0).abs() < 0.01);
    }
}
