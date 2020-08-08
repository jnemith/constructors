use cgmath::Vector3;

use super::block::Block;

const CHUNK_SIZE: usize = 16;
const CHUNK_3D_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub struct Chunk {
    id: usize,
    pub position: Vector3<i32>,
    pub is_active: bool,
    pub blocks: [Option<Block>; CHUNK_3D_SIZE],
    pub mesh: Option<ChunkMesh>,
}

pub struct ChunkMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
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

    pub fn build_mesh(&self, device: &wgpu::Device) -> Option<ChunkMesh> {
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

pub trait DrawBlock<'a, 'b>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, chunk_mesh: &'b ChunkMesh, uniforms: &'b wgpu::BindGroup);
    fn draw_chunks(&mut self, chunks: &'b Vec<Chunk>, uniforms: &'b wgpu::BindGroup);
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

    fn draw_chunks(&mut self, chunks: &'b Vec<Chunk>, uniforms: &'b wgpu::BindGroup) {
        for chunk in chunks {
            if !chunk.is_active {
                continue;
            }
            if let Some(mesh) = &chunk.mesh {
                self.draw_mesh(&mesh, uniforms);
            }
        }
    }
}
