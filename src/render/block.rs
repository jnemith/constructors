use cgmath::Vector3;

use super::chunk::CHUNK_SIZE;
use super::Vertex;

// const BLOCK_SIZE: f32 = 1.0 / 2.0;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct BlockVertex {
    position: [f32; 3],
    color: [f32; 3],
    normal: [f32; 3],
}

unsafe impl bytemuck::Pod for BlockVertex {}
unsafe impl bytemuck::Zeroable for BlockVertex {}

#[derive(Copy, Clone)]
pub struct Block {
    id: usize,
    pub is_active: bool,
}
#[allow(dead_code)]
impl Block {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            is_active: true,
        }
    }

    pub fn quad(
        width: Vector3<f32>,
        height: Vector3<f32>,
        position: Vector3<i32>,
        normal: Vector3<f32>,
    ) -> (Vec<BlockVertex>, Vec<u32>) {
        let color: [f32; 3] = [0.8, 0.0, 0.5];

        let offset = (CHUNK_SIZE / 2) as i32;
        let position = Vector3::new(
            (position.x - offset) as f32,
            position.y as f32,
            (position.z - offset) as f32,
        );

        let normal = [normal.x, normal.y, normal.z];
        let vertices: Vec<BlockVertex> = [
            BlockVertex::new([position.x, position.y, position.z], color, normal),
            BlockVertex::new(
                [
                    position.x + width.x,
                    position.y + width.y,
                    position.z + width.z,
                ],
                color,
                normal,
            ),
            BlockVertex::new(
                [
                    position.x + width.x + height.x,
                    position.y + width.y + height.y,
                    position.z + width.z + height.z,
                ],
                color,
                normal,
            ),
            BlockVertex::new(
                [
                    position.x + height.x,
                    position.y + height.y,
                    position.z + height.z,
                ],
                color,
                normal,
            ),
        ]
        .into();

        let indices: Vec<u32> = [0, 3, 2, 2, 1, 0].into();

        (vertices, indices)
    }
}

impl BlockVertex {
    pub fn new(position: [f32; 3], color: [f32; 3], normal: [f32; 3]) -> Self {
        BlockVertex {
            position,
            color,
            normal,
        }
    }
}

impl Vertex for BlockVertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<BlockVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}
