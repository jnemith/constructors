use cgmath::{InnerSpace, Rad, Vector3};
use std::time::Duration;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, VirtualKeyCode},
};

use crate::render::camera::Camera;

#[derive(PartialEq)]
pub enum Action {
    // WASD movement
    Forward,
    Backward,
    Left,
    Right,

    // Flight
    Up,
    Down,
}

pub struct Player {
    pub speed: f32,
    pub sensitivity: f32,
    pub camera: Camera,
    pub actions: Vec<Action>,

    pub mouse_d: (f32, f32),
}

impl Player {
    pub fn new(camera: Camera) -> Self {
        Self {
            speed: 8.0,
            sensitivity: 0.05,
            camera,
            actions: Vec::new(),
            mouse_d: (0.0, 0.0),
        }
    }

    pub fn new_action(&mut self, action: Action) {
        if !self.actions.contains(&action) {
            self.actions.push(action);
        }
    }

    pub fn remove_action(&mut self, action: Action) {
        let pos = self.actions.iter().position(|a| *a == action);

        if let Some(i) = pos {
            self.actions.swap_remove(i);
        }
    }

    pub fn update_player(&mut self, dt: Duration) {
        use std::f32::consts::FRAC_PI_2;
        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = self.camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        for action in self.actions.iter() {
            match action {
                Action::Forward => self.camera.position += forward * self.speed * dt,
                Action::Backward => self.camera.position -= forward * self.speed * dt,
                Action::Left => self.camera.position -= right * self.speed * dt,
                Action::Right => self.camera.position += right * self.speed * dt,
                Action::Up => self.camera.position.y += self.speed * dt,
                Action::Down => self.camera.position.y -= self.speed * dt,
            }
        }

        self.camera.yaw -= Rad(self.mouse_d.0) * self.sensitivity * dt;
        self.camera.pitch += Rad(self.mouse_d.1) * self.sensitivity * dt;

        self.mouse_d = (0.0, 0.0);

        if self.camera.pitch < -Rad(FRAC_PI_2) {
            self.camera.pitch = -Rad(FRAC_PI_2);
        } else if self.camera.pitch > Rad(FRAC_PI_2) {
            self.camera.pitch = Rad(FRAC_PI_2);
        }
    }

    pub fn process_keys(&mut self, key: &VirtualKeyCode, state: &ElementState) -> bool {
        match key {
            VirtualKeyCode::W => {
                if *state == ElementState::Pressed {
                    self.new_action(Action::Forward);
                } else {
                    self.remove_action(Action::Forward);
                }
                true
            }
            VirtualKeyCode::A => {
                if *state == ElementState::Pressed {
                    self.new_action(Action::Left);
                } else {
                    self.remove_action(Action::Left);
                }
                true
            }
            VirtualKeyCode::S => {
                if *state == ElementState::Pressed {
                    self.new_action(Action::Backward);
                } else {
                    self.remove_action(Action::Backward);
                }
                true
            }
            VirtualKeyCode::D => {
                if *state == ElementState::Pressed {
                    self.new_action(Action::Right);
                } else {
                    self.remove_action(Action::Right);
                }
                true
            }
            VirtualKeyCode::Space => {
                if *state == ElementState::Pressed {
                    self.new_action(Action::Up);
                } else {
                    self.remove_action(Action::Up);
                }
                true
            }
            VirtualKeyCode::LShift => {
                if *state == ElementState::Pressed {
                    self.new_action(Action::Down);
                } else {
                    self.remove_action(Action::Down);
                }
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, pos: &PhysicalPosition<f64>, width: u32, height: u32) {
        self.mouse_d.0 += (width as f32 / 2.0) - pos.x as f32;
        self.mouse_d.1 += (height as f32 / 2.0) - pos.y as f32;
    }
}
