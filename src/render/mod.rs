pub mod camera;
pub mod graphics;
pub mod object;
pub mod txt;

use cgmath::prelude::Zero;
use cgmath::{Matrix4, SquareMatrix, Vector4};

use camera::{Camera, Projection};

#[derive(Copy, Clone)]
pub struct Uniforms {
    view_position: Vector4<f32>,
    view_proj: Matrix4<f32>,
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_position: Zero::zero(),
            view_proj: Matrix4::identity(),
        }
    }

    fn update_camera(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.to_homogeneous();
        self.view_proj = projection.build_matrix() * camera.build_matrix();
    }
}