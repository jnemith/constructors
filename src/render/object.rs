#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

pub struct Object {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Object {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self { vertices, indices }
    }

    pub fn build_vertices() -> Object {
        let v_data = [
            Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0, 0.0]),
            Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0, 0.0]),
            Vertex::new([1.0, 1.0, 1.0], [1.0, 0.0, 0.0]),
            Vertex::new([-1.0, 1.0, 1.0], [1.0, 0.0, 0.0]),
            Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 0.0]),
            Vertex::new([1.0, 1.0, -1.0], [0.0, 1.0, 0.0]),
            Vertex::new([1.0, -1.0, -1.0], [0.0, 1.0, 0.0]),
            Vertex::new([-1.0, -1.0, -1.0], [0.0, 1.0, 0.0]),
            Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 1.0]),
            Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0, 1.0]),
            Vertex::new([1.0, 1.0, 1.0], [1.0, 0.0, 1.0]),
            Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0, 1.0]),
            Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0]),
            Vertex::new([-1.0, 1.0, 1.0], [0.0, 0.0, 1.0]),
            Vertex::new([-1.0, 1.0, -1.0], [0.0, 0.0, 1.0]),
            Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0, 1.0]),
            Vertex::new([1.0, 1.0, -1.0], [0.0, 1.0, 1.0]),
            Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 1.0]),
            Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0, 1.0]),
            Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 1.0]),
            Vertex::new([1.0, -1.0, 1.0], [0.5, 0.5, 0.5]),
            Vertex::new([-1.0, -1.0, 1.0], [0.5, 0.5, 0.5]),
            Vertex::new([-1.0, -1.0, -1.0], [0.5, 0.5, 0.5]),
            Vertex::new([1.0, -1.0, -1.0], [0.5, 0.5, 0.5]),
        ];

        let i_data = [
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        Object::new(v_data.to_vec(), i_data.to_vec())
    }
}

impl Vertex {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Vertex { position, color }
    }
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
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
            ],
        }
    }
}
