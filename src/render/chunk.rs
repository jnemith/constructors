use cgmath::Vector3;
use std::collections::{HashMap, HashSet};

use super::{block::Block, camera::Camera};

pub const CHUNK_SIZE: usize = 16;
const CHUNK_3D_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

const MAX_REBUILD_FRAME: usize = 2;

type ChunkPosition = Vector3<i32>;

pub struct ChunkManager {
    // Main list:
    pub chunks: HashMap<ChunkPosition, Chunk>,

    pub rebuild: HashSet<ChunkPosition>,

    // The list of chunks to be rendered
    render: HashSet<ChunkPosition>,

    render_dist: u16,
    old_chunk_pos: Option<Vector3<i32>>,
}

pub struct Chunk {
    id: usize,
    pub position: ChunkPosition,
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
    pub fn new(chunks: HashMap<ChunkPosition, Chunk>) -> Self {
        Self {
            chunks,
            rebuild: HashSet::new(),
            render: HashSet::new(),
            render_dist: 2,
            old_chunk_pos: None,
        }
    }

    pub fn default(width: i32) -> Self {
        let mut chunks = HashMap::new();
        for x in (-width / 2)..((width / 2) + 1) {
            for z in (-width / 2)..((width / 2) + 1) {
                let pos = Vector3::new(x, 0, z);
                chunks.insert(pos, Chunk::full(0, pos));
            }
        }

        Self::new(chunks)
    }

    pub fn update(&mut self, camera: &Camera, device: &wgpu::Device) {
        let camera_chunk_pos: Vector3<i32> = (
            if camera.position.x.is_sign_positive() {
                ((camera.position.x + 8.0) / 16.0).floor() as i32
            } else {
                ((camera.position.x - 8.0) / 16.0).ceil() as i32
            },
            if camera.position.y.is_sign_positive() {
                ((camera.position.y) / 16.0).floor() as i32
            } else {
                ((camera.position.y) / 16.0).ceil() as i32
            },
            if camera.position.z.is_sign_positive() {
                ((camera.position.z + 8.0) / 16.0).floor() as i32
            } else {
                ((camera.position.z - 8.0) / 16.0).ceil() as i32
            },
        )
            .into();

        let old_chunk_pos = if let Some(op) = self.old_chunk_pos {
            op
        } else {
            Vector3::new(
                camera_chunk_pos.x,
                camera_chunk_pos.y - 1,
                camera_chunk_pos.z,
            )
        };

        // Add chunks that are within the current render distance
        let mut new_render = HashSet::new();
        let render_dist = self.render_dist as i32;
        if old_chunk_pos != camera_chunk_pos {
            for x in (camera_chunk_pos.x - render_dist)..(camera_chunk_pos.x + render_dist + 1) {
                for y in
                    (camera_chunk_pos.y - render_dist).max(0)..(camera_chunk_pos.y + render_dist)
                {
                    for z in
                        (camera_chunk_pos.z - render_dist)..(camera_chunk_pos.z + render_dist + 1)
                    {
                        let position = Vector3::new(x, y, z);

                        if let Some(chunk) = self.get_chunk(&position.into()) {
                            if let None = chunk.mesh {
                                self.rebuild.insert(position);
                            }
                        }
                        if self.chunks.contains_key(&position) {
                            new_render.insert(position);
                        }
                    }
                }
            }
            self.render = new_render;
        }

        self.old_chunk_pos = Some(camera_chunk_pos);

        self.rebuild_chunks(device);
    }

    pub fn rebuild_chunks(&mut self, device: &wgpu::Device) {
        // Rebuild the mesh of chunks that were modified
        let positions = self.rebuild.clone();

        let mut rebuilt = 0;
        for position in positions {
            if rebuilt >= MAX_REBUILD_FRAME {
                break;
            }
            if let Some(chunk) = self.get_chunk_mut(&position) {
                chunk.greedy_mesh(device);
                rebuilt += 1;
            }
            self.rebuild.remove(&position);
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        // Prevent overwriting
        if self.chunks.contains_key(&chunk.position) {
            return;
        }

        self.chunks.insert(chunk.position, chunk);
    }

    pub fn get_chunk(&self, position: &ChunkPosition) -> Option<&Chunk> {
        self.chunks.get(&position)
    }
    pub fn get_chunk_mut(&mut self, position: &ChunkPosition) -> Option<&mut Chunk> {
        self.chunks.get_mut(&position)
    }
}

impl Chunk {
    pub fn new(id: usize, position: ChunkPosition) -> Self {
        let blocks: [Option<Block>; CHUNK_3D_SIZE] = [None; CHUNK_3D_SIZE];
        Self {
            id,
            position,
            is_active: false,
            blocks,
            mesh: None,
        }
    }

    pub fn full(id: usize, position: ChunkPosition) -> Self {
        let block = Block::new(0);
        let blocks: [Option<Block>; CHUNK_3D_SIZE] = [Some(block); CHUNK_3D_SIZE];

        Self {
            id,
            position,
            is_active: false,
            blocks,
            mesh: None,
        }
    }

    pub fn greedy_mesh(&mut self, device: &wgpu::Device) {
        // Adapted from https://github.com/roboleary/GreedyMesh
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let mut offset = 0;
        for d in 0..3 {
            let u = (d + 1) % 3;
            let v = (d + 2) % 3;
            let mut x: [f32; 3] = [0.0; 3];

            // Determines the direction
            let mut q: [f32; 3] = [0.0; 3];
            q[d] = 1.0;

            let size = CHUNK_SIZE as f32;
            let mut mask: [bool; CHUNK_SIZE * CHUNK_SIZE] = [false; CHUNK_SIZE * CHUNK_SIZE];
            x[d] = -1.0;
            while x[d] < size {
                // Compute the mask.
                x[v] = 0.0;
                x[u] = 0.0;
                let mut n = 0;
                while x[v] < size {
                    while x[u] < size {
                        let block_current = if 0.0 <= x[d] {
                            self.block_active((x[0] as usize, x[1] as usize, x[2] as usize).into())
                        } else {
                            false
                        };
                        let block_compare = if x[d] < CHUNK_SIZE as f32 - 1.0 {
                            self.block_active(
                                (
                                    (x[0] + q[0]) as usize,
                                    (x[1] + q[1]) as usize,
                                    (x[2] + q[2]) as usize,
                                )
                                    .into(),
                            )
                        } else {
                            false
                        };
                        mask[n] = block_current != block_compare;
                        n += 1;
                        x[u] += 1.0;
                    }
                    x[v] += 1.0;
                    x[u] = 0.0;
                }
                x[d] += 1.0;
                n = 0;

                let mut i;
                for j in 0..CHUNK_SIZE {
                    i = 0;
                    while i < CHUNK_SIZE {
                        if mask[n] {
                            // Calculate width and height.
                            let mut w = 1;
                            while (i + w) < CHUNK_SIZE && mask[n + w] {
                                w += 1;
                            }

                            let mut h = 1;
                            'outer: while (j + h) < CHUNK_SIZE {
                                for k in 0..w {
                                    if !mask[n + k + h * CHUNK_SIZE] {
                                        break 'outer;
                                    }
                                }
                                h += 1;
                            }

                            x[u] = i as f32;
                            x[v] = j as f32;

                            let mut du: [f32; 3] = [0.0; 3];
                            du[u] = w as f32;
                            let mut dv: [f32; 3] = [0.0; 3];
                            dv[v] = h as f32;

                            let chunk_pos = self.position * CHUNK_SIZE as i32;
                            let mut quad = Block::quad(
                                Vector3::new(du[0], du[1], du[2]),
                                Vector3::new(dv[0], dv[1], dv[2]),
                                Vector3::new(
                                    x[0] as i32 + chunk_pos.x,
                                    x[1] as i32 + chunk_pos.y,
                                    x[2] as i32 + chunk_pos.z,
                                ),
                                (q[0], q[1], q[2]).into(),
                            );

                            vertices.append(&mut quad.0);
                            for val in &mut quad.1 {
                                *val += offset;
                            }
                            offset += 4;
                            indices.append(&mut quad.1);

                            for l in 0..h {
                                for k in 0..w {
                                    mask[n + k + l * CHUNK_SIZE] = false;
                                }
                            }

                            i += w;
                            n += w;
                        } else {
                            i += 1;
                            n += 1;
                        }
                    }
                }
            }
        }
        self.is_active = true;

        if !vertices.is_empty() && !indices.is_empty() {
            let vertex_buffer = device.create_buffer_with_data(
                bytemuck::cast_slice(&vertices),
                wgpu::BufferUsage::VERTEX,
            );
            let index_buffer = device
                .create_buffer_with_data(bytemuck::cast_slice(&indices), wgpu::BufferUsage::INDEX);

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

    pub fn remove_block(&mut self, position: Vector3<usize>) {
        let x = position.x;
        let y = position.y;
        let z = position.z;

        let limit = CHUNK_SIZE - 1;
        if x <= limit && y <= limit && z <= limit {
            let index = ((x * CHUNK_SIZE + y) * CHUNK_SIZE) + z;
            if self.blocks[index].is_some() {
                self.blocks[index] = None;
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
        for chunk_position in &chunk_manager.render {
            let chunk: Option<&'b Chunk> = chunk_manager.get_chunk(chunk_position);

            if chunk.is_none() {
                continue;
            }

            if let Some(mesh) = &chunk.unwrap().mesh {
                self.draw_mesh(mesh, uniforms);
            }
        }
    }
}
