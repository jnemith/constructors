use cgmath::Vector3;
use std::collections::HashSet;

use super::Vertex;

const INDEX_DATA: &[u32] = &[
    0, 1, 2, 2, 3, 0, // top
    4, 5, 6, 6, 7, 4, // bottom
    8, 9, 10, 10, 11, 8, // right
    12, 13, 14, 14, 15, 12, // left
    16, 17, 18, 18, 19, 16, // front
    20, 21, 22, 22, 23, 20, // back
];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct BlockVertex {
    position: [f32; 3],
    color: [f32; 3],
    normal: [f32; 3],
}

unsafe impl bytemuck::Pod for BlockVertex {}
unsafe impl bytemuck::Zeroable for BlockVertex {}

#[derive(PartialEq, Eq, Hash)]
pub enum BlockFace {
    North,  // Positive-X
    South,  // Negative-X
    Top,    // Positive-Y
    Bottom, // Negative-Y
    East,   // Positive-Z
    West,   // Negative-Z
}

#[derive(Copy, Clone)]
pub struct Block {
    id: usize,
    pub is_active: bool,
}

impl Block {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            is_active: true,
        }
    }

    pub fn build(
        color: Vector3<f32>,
        position: Vector3<i32>,
        faces: HashSet<BlockFace>,
    ) -> (Vec<BlockVertex>, Vec<u32>) {
        let s = 1.0 / 2.0;
        let color: [f32; 3] = color.into();

        use super::chunk::CHUNK_SIZE;
        let offset = (CHUNK_SIZE / 2) as i32;
        let position = Vector3::new(
            (position.x - offset) as f32,
            position.y as f32,
            (position.z - offset) as f32,
        );
        let pos1 = [position.x - s, position.y - s, position.z + s];
        let pos2 = [position.x + s, position.y - s, position.z + s];
        let pos3 = [position.x - s, position.y + s, position.z + s];
        let pos4 = [position.x + s, position.y + s, position.z + s];
        let pos5 = [position.x - s, position.y - s, position.z - s];
        let pos6 = [position.x + s, position.y - s, position.z - s];
        let pos7 = [position.x - s, position.y + s, position.z - s];
        let pos8 = [position.x + s, position.y + s, position.z - s];

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mut v_data = Vec::new();
        for face in &faces {
            match face {
                BlockFace::North => v_data.append(
                    &mut [
                        BlockVertex::new(pos6, color, [1.0, 0.0, 0.0]),
                        BlockVertex::new(pos8, color, [1.0, 0.0, 0.0]),
                        BlockVertex::new(pos4, color, [1.0, 0.0, 0.0]),
                        BlockVertex::new(pos2, color, [1.0, 0.0, 0.0]),
                    ]
                    .to_vec(),
                ),
                BlockFace::South => v_data.append(
                    &mut [
                        BlockVertex::new(pos1, color, [-1.0, 0.0, 0.0]),
                        BlockVertex::new(pos3, color, [-1.0, 0.0, 0.0]),
                        BlockVertex::new(pos7, color, [-1.0, 0.0, 0.0]),
                        BlockVertex::new(pos5, color, [-1.0, 0.0, 0.0]),
                    ]
                    .to_vec(),
                ),
                BlockFace::East => v_data.append(
                    &mut [
                        BlockVertex::new(pos1, color, [0.0, 0.0, 1.0]),
                        BlockVertex::new(pos2, color, [0.0, 0.0, 1.0]),
                        BlockVertex::new(pos4, color, [0.0, 0.0, 1.0]),
                        BlockVertex::new(pos3, color, [0.0, 0.0, 1.0]),
                    ]
                    .to_vec(),
                ),
                BlockFace::West => v_data.append(
                    &mut [
                        BlockVertex::new(pos7, color, [0.0, 0.0, -1.0]),
                        BlockVertex::new(pos8, color, [0.0, 0.0, -1.0]),
                        BlockVertex::new(pos6, color, [0.0, 0.0, -1.0]),
                        BlockVertex::new(pos5, color, [0.0, 0.0, -1.0]),
                    ]
                    .to_vec(),
                ),
                BlockFace::Top => v_data.append(
                    &mut [
                        BlockVertex::new(pos8, color, [0.0, 1.0, 0.0]),
                        BlockVertex::new(pos7, color, [0.0, 1.0, 0.0]),
                        BlockVertex::new(pos3, color, [0.0, 1.0, 0.0]),
                        BlockVertex::new(pos4, color, [0.0, 1.0, 0.0]),
                    ]
                    .to_vec(),
                ),
                BlockFace::Bottom => v_data.append(
                    &mut [
                        BlockVertex::new(pos2, color, [0.0, -1.0, 0.0]),
                        BlockVertex::new(pos1, color, [0.0, -1.0, 0.0]),
                        BlockVertex::new(pos5, color, [0.0, -1.0, 0.0]),
                        BlockVertex::new(pos6, color, [0.0, -1.0, 0.0]),
                    ]
                    .to_vec(),
                ),
            }
        }

        (v_data, INDEX_DATA.to_vec())
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

// fn clamp<N>(num: N, min: N, max: N) -> N
// where
//     N: std::cmp::Ord,
// {
//     num.min(max).max(min)
// }
