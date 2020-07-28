use cgmath::Deg;
use shaderc::{Compiler, ShaderKind};
use std::io::Cursor;
use std::time::Duration;
use winit::event::*;

use super::camera::{Camera, Projection};
use super::object::{Object, Vertex};
use super::Uniforms;
use crate::player::Player;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Graphics {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,

    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    pub player: Player,
    pub projection: Projection,

    pub render_pipeline: wgpu::RenderPipeline,
}

impl Graphics {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        // Compile shaders to SPIRV
        let vs_src = include_str!("../../shaders/shader.vert");
        let fs_src = include_str!("../../shaders/shader.frag");

        let mut compiler = Compiler::new().unwrap();
        let vs_spirv = compiler
            .compile_into_spirv(vs_src, ShaderKind::Vertex, "shader.vert", "main", None)
            .unwrap();
        let fs_spirv = compiler
            .compile_into_spirv(fs_src, ShaderKind::Fragment, "shader.frag", "main", None)
            .unwrap();

        let vs_data = wgpu::read_spirv(Cursor::new(vs_spirv.as_binary_u8())).unwrap();
        let fs_data = wgpu::read_spirv(Cursor::new(fs_spirv.as_binary_u8())).unwrap();

        let vs_module = device.create_shader_module(&vs_data);
        let fs_module = device.create_shader_module(&fs_data);

        // Initialize player data
        let camera = Camera::new((0.0, 3.0, 5.0), Deg(-90.0), Deg(-10.0));
        let player = Player::new(camera);
        let projection = Projection::new(sc_desc.width, sc_desc.height, Deg(45.0), 0.1, 100.0);

        // Initialize object data
        let obj = Object::build_vertices();
        let vertex_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&obj.vertices),
            wgpu::BufferUsage::VERTEX,
        );
        let index_buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(&obj.indices), wgpu::BufferUsage::INDEX);

        let mut uniforms = Uniforms::new();
        uniforms.update_camera(&player.camera, &projection);

        let uniform_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[uniforms]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    range: 0..std::mem::size_of_val(&uniforms) as wgpu::BufferAddress,
                },
            }],
            label: Some("uniform_bind_group_layout"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&uniform_bind_group_layout],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
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
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: 10,
            alpha_to_coverage_enabled: false,
        });

        let num_indices = obj.indices.len() as u32;

        Self {
            device,
            queue,
            vertex_buffer,
            index_buffer,
            num_indices,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            player,
            projection,
            render_pipeline,
        }
    }

    pub fn handle_input(&mut self, event: &WindowEvent, width: u32, height: u32) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                device_id,
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(key),
                        ..
                    },
                ..
            } => self.player.process_keys(key, state),
            WindowEvent::CursorMoved { position, .. } => {
                self.player.process_mouse(position, width, height);
                false
            }
            _ => false,
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.player.update_player(dt);
        self.uniforms
            .update_camera(&self.player.camera, &self.projection);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("update encoder"),
            });

        let staging_buffer = self.device.create_buffer_with_data(
            bytemuck::cast_slice(&[self.uniforms]),
            wgpu::BufferUsage::COPY_SRC,
        );

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
        );

        self.queue.submit(&[encoder.finish()]);
    }
    pub fn render(&self, swap_chain: &mut wgpu::SwapChain) {
        let frame = swap_chain
            .get_next_texture()
            .expect("Failed to acquire next swap chain texture");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.2,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
            render_pass.set_index_buffer(&self.index_buffer, 0, 0);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(&[encoder.finish()]);
    }
}
