use std::sync::Arc;
use anyhow::Result;

use winit::{
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct State {
    pub window: Arc<Window>,
    pub bodies: Vec<Body>,
    pub g_constant: f32,
    pub time_step: f32,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        Ok(Self {
            window,
            bodies: vec![
                Body::new(100.0, (-1.0, 0.0), (0.0, 1.0)).unwrap(),
                Body::new(100.0, (1.0, 0.0), (0.0, -1.0)).unwrap(),
            ],
            g_constant: 1.0,
            time_step: 0.001, // Small time step for accuracy
        })
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {
        //
    }

    pub fn render(&mut self) {
        self.window.request_redraw();

        // Can add some rendering here later
    }

    pub fn total_kinetic_energy(&self) -> f32{
        self.bodies.iter().fold(0.0, |s, x| s + x.get_kinetic_energy())
    }

    pub fn step(&mut self) {
        // 1. Pairwise forces
        let mut forces = vec![(0.0, 0.0); self.bodies.len()];

        for i in 0..self.bodies.len() {
            for j in (i+1)..self.bodies.len() {
                let force_ij = self.bodies[i].gravitational_force(&self.bodies[j], self.g_constant);
                let force_ji = (-force_ij.0, -force_ij.1);

                forces[i].0 += force_ij.0;
                forces[i].1 += force_ij.1;
                forces[j].0 += force_ji.0;
                forces[j].1 += force_ji.1;
            }
        }

        for (i, body) in self.bodies.iter_mut().enumerate() {
            let acceleration = (forces[i].0 / body.mass, forces[i].1 / body.mass);
            body.update(acceleration, self.time_step);
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
        Ok(Self { mass, position, velocity, radius })
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
        ((self.velocity.0 * self.velocity.0) + (self.velocity.1 * self.velocity.1)).sqrt() * self.mass
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
    #[ignore = "Requires window system - run with 'cargo test -- --ignored'"]
    fn test_energy_conservation_two_body() {
        // Create a real window for testing (may fail in headless environments)
        use winit::event_loop::EventLoop;
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(event_loop.create_window(winit::window::Window::default_attributes()).unwrap());
        
        let mut state = State {
            window,
            bodies: vec![
                Body::new(100.0, (-1.0, 0.0), (0.0, 1.0)).unwrap(),
                Body::new(100.0, (1.0, 0.0), (0.0, -1.0)).unwrap(),
            ],
            g_constant: 1.0,
            time_step: 0.001, // Small time step for accuracy
        };
        
        let initial_energy = state.total_kinetic_energy();
        
        // Run a few simulation steps
        for _ in 0..10 {
            state.step();
        }
        
        let final_energy = state.total_kinetic_energy();
        
        // Energy should be approximately conserved (within numerical error)
        // Note: This is a simple test - real energy conservation would include potential energy
        assert!((final_energy - initial_energy).abs() < 0.1);
    }
}