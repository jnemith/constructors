use cgmath::Vector3;

use super::Vertex;

const CHUNK_SIZE: usize = 16;
const CHUNK_3D_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

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

#[derive(Copy, Clone)]
pub struct Block {
    id: usize,
    is_active: bool,
    // pub vertices: Vec<BlockVertex>,
    // pub indices: Vec<u32>,
}

pub struct ChunkMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

pub struct Chunk {
    id: usize,
    pub position: Vector3<i32>,
    pub is_active: bool,
    pub blocks: [Option<Block>; CHUNK_3D_SIZE],
    pub mesh: Option<ChunkMesh>,
}

impl Chunk {
    pub fn new(id: usize, position: Vector3<i32>) -> Self {
        let blocks: [Option<Block>; CHUNK_3D_SIZE] = [None; CHUNK_3D_SIZE];
        Self {
            id,
            position,
            is_active: true,
            blocks,
            mesh: None,
        }
    }

    pub fn build_mesh(
        &self,
        // blocks: &[Option<Block>; CHUNK_3D_SIZE],
        device: &wgpu::Device,
    ) -> Option<ChunkMesh> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let mut offset: u32 = 0;
        for (i, block) in self.blocks.iter().enumerate() {
            if let Some(b) = block {
                if !b.is_active {
                    continue;
                }

                let z = (i % CHUNK_SIZE) as i32;
                let y = ((i / CHUNK_SIZE) % CHUNK_SIZE) as i32;
                let x = (i / (CHUNK_SIZE * CHUNK_SIZE)) as i32;

                let chunk_pos = self.position * CHUNK_SIZE as i32;
                let (mut v_data, mut i_data) = Block::build(
                    (0.4, 0.0, 0.0).into(),
                    (x + chunk_pos.x, y + chunk_pos.y, z + chunk_pos.z).into(),
                );

                for val in &mut i_data {
                    *val += offset as u32;
                }

                vertices.append(&mut v_data);
                indices.append(&mut i_data);

                offset += 24;
            }
        }

        if vertices.is_empty() || indices.is_empty() {
            return None;
        }

        let vertex_buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(&vertices), wgpu::BufferUsage::VERTEX);
        let index_buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(&indices), wgpu::BufferUsage::INDEX);

        Some(ChunkMesh {
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
        })
    }

    pub fn insert_block(&mut self, block: Block, x: usize, y: usize, z: usize) {
        let limit = CHUNK_SIZE - 1;
        if x <= limit && y <= limit && z <= limit {
            let index = ((x * CHUNK_SIZE + y) * CHUNK_SIZE) + z;
            if self.blocks[index].is_none() {
                self.blocks[index] = Some(block);
            }
        }
    }
}

impl Block {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            is_active: true,
        }
    }

    pub fn build(color: Vector3<f32>, position: Vector3<i32>) -> (Vec<BlockVertex>, Vec<u32>) {
        let s = 1.0 / 2.0;
        let color: [f32; 3] = color.into();
        let v_data = [
            // Top
            BlockVertex::new([-s, -s, s], color, [0.0, 0.0, 1.0], position),
            BlockVertex::new([s, -s, s], color, [0.0, 0.0, 1.0], position),
            BlockVertex::new([s, s, s], color, [0.0, 0.0, 1.0], position),
            BlockVertex::new([-s, s, s], color, [0.0, 0.0, 1.0], position),
            // Bottom
            BlockVertex::new([-s, s, -s], color, [0.0, 0.0, -1.0], position),
            BlockVertex::new([s, s, -s], color, [0.0, 0.0, -1.0], position),
            BlockVertex::new([s, -s, -s], color, [0.0, 0.0, -1.0], position),
            BlockVertex::new([-s, -s, -s], color, [0.0, 0.0, -1.0], position),
            // Rigst
            BlockVertex::new([s, -s, -s], color, [1.0, 0.0, 0.0], position),
            BlockVertex::new([s, s, -s], color, [1.0, 0.0, 0.0], position),
            BlockVertex::new([s, s, s], color, [1.0, 0.0, 0.0], position),
            BlockVertex::new([s, -s, s], color, [1.0, 0.0, 0.0], position),
            // Left
            BlockVertex::new([-s, -s, s], color, [-1.0, 0.0, 0.0], position),
            BlockVertex::new([-s, s, s], color, [-1.0, 0.0, 0.0], position),
            BlockVertex::new([-s, s, -s], color, [-1.0, 0.0, 0.0], position),
            BlockVertex::new([-s, -s, -s], color, [-1.0, 0.0, 0.0], position),
            // Front
            BlockVertex::new([s, s, -s], color, [0.0, 1.0, 0.0], position),
            BlockVertex::new([-s, s, -s], color, [0.0, 1.0, 0.0], position),
            BlockVertex::new([-s, s, s], color, [0.0, 1.0, 0.0], position),
            BlockVertex::new([s, s, s], color, [0.0, 1.0, 0.0], position),
            // Back
            BlockVertex::new([s, -s, s], color, [0.0, -1.0, 0.0], position),
            BlockVertex::new([-s, -s, s], color, [0.0, -1.0, 0.0], position),
            BlockVertex::new([-s, -s, -s], color, [0.0, -1.0, 0.0], position),
            BlockVertex::new([s, -s, -s], color, [0.0, -1.0, 0.0], position),
        ];

        (v_data.to_vec(), INDEX_DATA.to_vec())
    }
}

impl BlockVertex {
    pub fn new(position: [f32; 3], color: [f32; 3], normal: [f32; 3], p: Vector3<i32>) -> Self {
        let position = [
            position[0] + p.x as f32,
            position[1] + p.y as f32,
            position[2] + p.z as f32,
        ];
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

pub trait DrawBlock<'a, 'b>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, chunk_mesh: &'b ChunkMesh, uniforms: &'b wgpu::BindGroup);
    fn draw_chunk(&mut self, chunk: &'b Chunk, uniforms: &'b wgpu::BindGroup);
}

impl<'a, 'b> DrawBlock<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, chunk_mesh: &'b ChunkMesh, uniforms: &'b wgpu::BindGroup) {
        self.set_vertex_buffer(0, &chunk_mesh.vertex_buffer, 0, 0);
        self.set_index_buffer(&chunk_mesh.index_buffer, 0, 0);
        self.set_bind_group(0, &uniforms, &[]);
        self.draw_indexed(0..chunk_mesh.num_elements, 0, 0..1);
    }

    fn draw_chunk(&mut self, chunk: &'b Chunk, uniforms: &'b wgpu::BindGroup) {
        if !chunk.is_active {
            return;
        }
        if let Some(mesh) = &chunk.mesh {
            self.draw_mesh(&mesh, uniforms);
        }
    }
}

// fn clamp<N>(num: N, min: N, max: N) -> N
// where
//     N: std::cmp::Ord,
// {
//     num.min(max).max(min)
// }
