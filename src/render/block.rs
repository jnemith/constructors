use cgmath::Vector3;

use super::{BlockMeshUniforms, Vertex};

const SIZE_MAX: u8 = 10;
const SIZE_MIN: u8 = 1;

const INDEX_DATA: &[u16] = &[
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

pub struct BlockComponent {
    pub vertices: Vec<BlockVertex>,
    pub indices: Vec<u16>,
    pub local_pos: Vector3<f32>,
}

pub struct BlockMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub position_buffer: wgpu::Buffer,
    pub position_bind_group: wgpu::BindGroup,
    pub num_elements: u32,
}
pub struct Block {
    // model: Matrix4<f32>,
    pub meshes: Vec<BlockMesh>,
}

impl Block {
    pub fn new(
        components: &[BlockComponent],
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let mut meshes = Vec::new();

        for bc in components.iter() {
            let vertex_buffer = device.create_buffer_with_data(
                bytemuck::cast_slice(&bc.vertices),
                wgpu::BufferUsage::VERTEX,
            );
            let index_buffer = device.create_buffer_with_data(
                bytemuck::cast_slice(&bc.indices),
                wgpu::BufferUsage::INDEX,
            );

            let position_data = BlockMeshUniforms::new(&bc.local_pos);
            let position_buffer = device.create_buffer_with_data(
                bytemuck::cast_slice(&[position_data]),
                wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            );

            let position_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                bindings: &[wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &position_buffer,
                        range: 0..std::mem::size_of_val(&position_data) as wgpu::BufferAddress,
                    },
                }],
                label: Some("position_bind_group"),
            });

            meshes.push(BlockMesh {
                vertex_buffer,
                index_buffer,
                position_buffer,
                position_bind_group,
                num_elements: bc.indices.len() as u32,
            });
        }

        Self { meshes }
    }
}

#[allow(dead_code)]
impl BlockComponent {
    pub fn new(vertices: Vec<BlockVertex>, indices: Vec<u16>, local_pos: Vector3<f32>) -> Self {
        Self {
            vertices,
            indices,
            local_pos,
        }
    }

    pub fn build(l: f32, h: f32, w: f32, color: Vector3<f32>, local_pos: Vector3<i8>) -> Self {
        let local_pos = Vector3::new(
            clamp(local_pos.x, SIZE_MIN as i8 - 1, SIZE_MAX as i8 - 1) as f32 / SIZE_MAX as f32,
            clamp(local_pos.y, SIZE_MIN as i8 - 1, SIZE_MAX as i8 - 1) as f32 / SIZE_MAX as f32,
            clamp(local_pos.z, SIZE_MIN as i8 - 1, SIZE_MAX as i8 - 1) as f32 / SIZE_MAX as f32,
        );
        let color: [f32; 3] = color.into();
        let v_data = [
            // Top
            BlockVertex::new([-l, -h, w], color, [0.0, 0.0, 1.0]),
            BlockVertex::new([l, -h, w], color, [0.0, 0.0, 1.0]),
            BlockVertex::new([l, h, w], color, [0.0, 0.0, 1.0]),
            BlockVertex::new([-l, h, w], color, [0.0, 0.0, 1.0]),
            // Bottom
            BlockVertex::new([-l, h, -w], color, [0.0, 0.0, -1.0]),
            BlockVertex::new([l, h, -w], color, [0.0, 0.0, -1.0]),
            BlockVertex::new([l, -h, -w], color, [0.0, 0.0, -1.0]),
            BlockVertex::new([-l, -h, -w], color, [0.0, 0.0, -1.0]),
            // Right
            BlockVertex::new([l, -h, -w], color, [1.0, 0.0, 0.0]),
            BlockVertex::new([l, h, -w], color, [1.0, 0.0, 0.0]),
            BlockVertex::new([l, h, w], color, [1.0, 0.0, 0.0]),
            BlockVertex::new([l, -h, w], color, [1.0, 0.0, 0.0]),
            // Left
            BlockVertex::new([-l, -h, w], color, [-1.0, 0.0, 0.0]),
            BlockVertex::new([-l, h, w], color, [-1.0, 0.0, 0.0]),
            BlockVertex::new([-l, h, -w], color, [-1.0, 0.0, 0.0]),
            BlockVertex::new([-l, -h, -w], color, [-1.0, 0.0, 0.0]),
            // Front
            BlockVertex::new([l, h, -w], color, [0.0, 1.0, 0.0]),
            BlockVertex::new([-l, h, -w], color, [0.0, 1.0, 0.0]),
            BlockVertex::new([-l, h, w], color, [0.0, 1.0, 0.0]),
            BlockVertex::new([l, h, w], color, [0.0, 1.0, 0.0]),
            // Back
            BlockVertex::new([l, -h, w], color, [0.0, -1.0, 0.0]),
            BlockVertex::new([-l, -h, w], color, [0.0, -1.0, 0.0]),
            BlockVertex::new([-l, -h, -w], color, [0.0, -1.0, 0.0]),
            BlockVertex::new([l, -h, -w], color, [0.0, -1.0, 0.0]),
        ];

        Self::new(v_data.to_vec(), INDEX_DATA.to_vec(), local_pos)
    }

    pub fn with_scale(scale: u8, color: Vector3<f32>, local_pos: Vector3<i8>) -> Self {
        let scale = clamp(scale, SIZE_MIN, SIZE_MAX) as f32;
        Self::build(scale, scale, scale, color, local_pos)
    }

    pub fn max(color: Vector3<f32>, local_pos: Vector3<i8>) -> Self {
        Self::with_scale(SIZE_MAX, color, local_pos)
    }

    pub fn min(color: Vector3<f32>, local_pos: Vector3<i8>) -> Self {
        Self::with_scale(SIZE_MIN, color, local_pos)
    }

    pub fn with_dimensions(l: u8, h: u8, w: u8, color: Vector3<f32>, local_pos: Vector3<i8>) {
        let l = clamp(l, SIZE_MIN, SIZE_MAX) as f32;
        let h = clamp(h, SIZE_MIN, SIZE_MAX) as f32;
        let w = clamp(w, SIZE_MIN, SIZE_MAX) as f32;
        Self::build(l, w, h, color, local_pos);
    }
}

impl BlockVertex {
    pub fn new(position: [f32; 3], color: [f32; 3], normal: [f32; 3]) -> Self {
        let denom = SIZE_MAX as f32;
        let position = [
            position[0] / denom,
            position[1] / denom,
            position[2] / denom,
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
    fn draw_mesh(&mut self, block_mesh: &'b BlockMesh, uniforms: &'b wgpu::BindGroup);
    fn draw_block(&mut self, block: &'b Block, uniforms: &'b wgpu::BindGroup);
}

impl<'a, 'b> DrawBlock<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, block_mesh: &'b BlockMesh, uniforms: &'b wgpu::BindGroup) {
        self.set_vertex_buffer(0, &block_mesh.vertex_buffer, 0, 0);
        self.set_index_buffer(&block_mesh.index_buffer, 0, 0);
        self.set_bind_group(0, &uniforms, &[]);
        self.set_bind_group(1, &block_mesh.position_bind_group, &[]);
        self.draw_indexed(0..block_mesh.num_elements, 0, 0..1);
    }

    fn draw_block(&mut self, block: &'b Block, uniforms: &'b wgpu::BindGroup) {
        for mesh in &block.meshes {
            self.draw_mesh(mesh, uniforms);
        }
    }
}

fn clamp<N>(num: N, min: N, max: N) -> N
where
    N: std::cmp::Ord,
{
    num.min(max).max(min)
}
