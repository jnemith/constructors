use std::time::Duration;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub trait Render {
    fn update(&mut self, dt: Duration, graphics: &Graphics);
    fn render(&mut self, graphics: &mut Graphics);
}

#[allow(dead_code)]
pub struct Graphics {
    pub size: winit::dpi::PhysicalSize<u32>,
    adapter: wgpu::Adapter,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Graphics {
    pub fn new(
        size: winit::dpi::PhysicalSize<u32>,
        adapter: wgpu::Adapter,
        sc_desc: wgpu::SwapChainDescriptor,
        swap_chain: wgpu::SwapChain,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Self {
        Self {
            size,
            adapter,
            sc_desc,
            swap_chain,
            device,
            queue,
        }
    }

    pub fn create_render_pipeline(
        &self,
        layout: &wgpu::PipelineLayout,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_descs: &[wgpu::VertexBufferDescriptor],
        vs_src: &str,
        fs_src: &str,
    ) -> wgpu::RenderPipeline {
        let mut compiler = shaderc::Compiler::new().unwrap();
        let vs_spirv = compiler
            .compile_into_spirv(
                vs_src,
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None,
            )
            .unwrap();
        let fs_spirv = compiler
            .compile_into_spirv(
                fs_src,
                shaderc::ShaderKind::Fragment,
                "shader.frag",
                "main",
                None,
            )
            .unwrap();

        let vs_data = wgpu::read_spirv(std::io::Cursor::new(vs_spirv.as_binary_u8())).unwrap();
        let fs_data = wgpu::read_spirv(std::io::Cursor::new(fs_spirv.as_binary_u8())).unwrap();

        let vs_module = self.device.create_shader_module(&vs_data);
        let fs_module = self.device.create_shader_module(&fs_data);

        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: &layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vs_module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &fs_module,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::None,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    format: self.sc_desc.format,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                depth_stencil_state: depth_format.map(|format| wgpu::DepthStencilStateDescriptor {
                    format,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                    stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                    stencil_read_mask: 0,
                    stencil_write_mask: 0,
                }),
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint32,
                    vertex_buffers: vertex_descs,
                },
            })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, surface: &wgpu::Surface) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&surface, &self.sc_desc);
    }
}
