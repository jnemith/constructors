use cgmath::Vector3;
use std::collections::{HashMap, HashSet};

use super::block::{Block, BlockFace};

const CHUNK_SIZE: usize = 16;
const CHUNK_3D_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub struct ChunkManager {
    chunks: HashMap<Vector3<i32>, Chunk>,
}

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

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn default(device: &wgpu::Device) -> Self {
        let mut chunks = HashMap::new();
        for x in -1..2 {
            for z in -1..2 {
                let pos = Vector3::new(x, -1, z);
                chunks.insert(pos, Chunk::new(0, pos));
            }
        }
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    for (_, chunk) in chunks.iter_mut() {
                        chunk.insert_block(Block::new(0), Vector3::new(x, y, z));
                    }
                }
            }
        }
        for (_, chunk) in chunks.iter_mut() {
            chunk.build_mesh(device);
        }
        Self { chunks }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position, chunk);
    }

    pub fn get_chunk(&self, position: &Vector3<i32>) -> Option<&Chunk> {
        self.chunks.get(&position)
    }
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

    pub fn build_mesh(&mut self, device: &wgpu::Device) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let mut offset: u32 = 0;
        for (i, block) in self.blocks.iter().enumerate() {
            if let Some(b) = block {
                if !b.is_active {
                    continue;
                }

                let z = (i % CHUNK_SIZE) as usize;
                let y = ((i / CHUNK_SIZE) % CHUNK_SIZE) as usize;
                let x = (i / (CHUNK_SIZE * CHUNK_SIZE)) as usize;

                let mut faces = HashSet::new();
                faces.insert(BlockFace::North);
                faces.insert(BlockFace::South);
                faces.insert(BlockFace::East);
                faces.insert(BlockFace::West);
                faces.insert(BlockFace::Top);
                faces.insert(BlockFace::Bottom);

                if x > 0 {
                    if self.block_active((x + 1, y, z).into()) {
                        faces.remove(&BlockFace::North);
                    }
                    if x < CHUNK_SIZE - 1 && self.block_active((x - 1, y, z).into()) {
                        faces.remove(&BlockFace::South);
                    }
                }
                if y > 0 {
                    if self.block_active((x, y + 1, z).into()) {
                        faces.remove(&BlockFace::Top);
                    }
                    if y < CHUNK_SIZE - 1 && self.block_active((x, y - 1, z).into()) {
                        faces.remove(&BlockFace::Bottom);
                    }
                }
                if z > 0 {
                    if self.block_active((x, y, z + 1).into()) {
                        faces.remove(&BlockFace::East);
                    }
                    if z < CHUNK_SIZE - 1 && self.block_active((x, y, z - 1).into()) {
                        faces.remove(&BlockFace::West);
                    }
                }

                let chunk_pos = self.position * CHUNK_SIZE as i32;
                let (mut v_data, mut i_data) = Block::build(
                    (0.8, 0.0, 0.5).into(),
                    (
                        x as i32 + chunk_pos.x,
                        y as i32 + chunk_pos.y,
                        z as i32 + chunk_pos.z,
                    )
                        .into(),
                    faces,
                );

                for val in &mut i_data {
                    *val += offset as u32;
                }

                vertices.append(&mut v_data);
                indices.append(&mut i_data);

                offset += 24;
            }
        }

        let vertex_buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(&vertices), wgpu::BufferUsage::VERTEX);
        let index_buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(&indices), wgpu::BufferUsage::INDEX);

        if !vertices.is_empty() && !indices.is_empty() {
            self.mesh = Some(ChunkMesh {
                vertex_buffer,
                index_buffer,
                num_elements: indices.len() as u32,
            });
        } else {
            self.mesh = None;
        }
    }

    pub fn insert_block(&mut self, block: Block, position: Vector3<usize>) {
        let x = position.x;
        let y = position.y;
        let z = position.z;

        let limit = CHUNK_SIZE - 1;
        if x <= limit && y <= limit && z <= limit {
            let index = ((x * CHUNK_SIZE + y) * CHUNK_SIZE) + z;
            if self.blocks[index].is_none() {
                self.blocks[index] = Some(block);
            }
        }
    }

    pub fn block_active(&self, position: Vector3<usize>) -> bool {
        let x = position.x;
        let y = position.y;
        let z = position.z;

        let limit = CHUNK_SIZE - 1;
        if x <= limit && y <= limit && z <= limit {
            let index = ((x * CHUNK_SIZE + y) * CHUNK_SIZE) + z;
            if let Some(block) = self.blocks[index] {
                return block.is_active;
            }
        }
        false
    }
}

pub trait DrawBlock<'a, 'b>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, chunk_mesh: &'b ChunkMesh, uniforms: &'b wgpu::BindGroup);
    fn draw_chunks(&mut self, chunk_manager: &'b ChunkManager, uniforms: &'b wgpu::BindGroup);
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

    fn draw_chunks(&mut self, chunk_manager: &'b ChunkManager, uniforms: &'b wgpu::BindGroup) {
        for (_, chunk) in &chunk_manager.chunks {
            if !chunk.is_active {
                continue;
            }
            if let Some(mesh) = &chunk.mesh {
                self.draw_mesh(&mesh, uniforms);
            }
        }
    }
}
