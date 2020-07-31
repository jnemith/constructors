pub mod block;
pub mod camera;
pub mod graphics;
pub mod txt;

use cgmath::prelude::Zero;
use cgmath::{Matrix4, SquareMatrix, Vector3, Vector4};

use camera::{Camera, Projection};

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}

#[derive(Copy, Clone)]
pub struct Uniforms {
    view_position: Vector4<f32>,
    view_proj: Matrix4<f32>,
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

#[derive(Copy, Clone)]
pub struct BlockMeshUniforms {
    position: Vector3<f32>,
}

unsafe impl bytemuck::Pod for BlockMeshUniforms {}
unsafe impl bytemuck::Zeroable for BlockMeshUniforms {}

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

impl BlockMeshUniforms {
    fn new(position: &Vector3<f32>) -> Self {
        let fposition = Vector3::new(position.x as f32, position.y as f32, position.z as f32);
        Self {
            position: fposition,
        }
    }
}
