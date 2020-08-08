use std::time::Duration;
use wgpu_glyph::{Section, Text};
use winit::event::{KeyboardInput, WindowEvent};

use crate::player::Player;
use crate::render::{
    block::{Block, BlockVertex},
    camera::Projection,
    chunk::{Chunk, DrawBlock},
    graphics::{Graphics, Render},
    texture::Texture,
    txt::Txt,
    Uniforms, Vertex,
};

pub struct World {
    player: Player,
    chunks: Vec<Chunk>,
    text: Txt,
    projection: Projection,

    depth_texture: Texture,

    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
}

impl World {
    pub fn new(player: Player, projection: Projection, graphics: &Graphics) -> Self {
        let mut uniforms = Uniforms::new();
        uniforms.update_camera(&player.camera, &projection);

        let uniform_buffer = graphics.device.create_buffer_with_data(
            bytemuck::cast_slice(&[uniforms]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let uniform_bind_group_layout =
            graphics
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    bindings: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    }],
                    label: Some("uniform_bind_group_layout"),
                });

        let uniform_bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
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

        let block = Block::new(0);
        let mut chunks = Vec::new();
        let mut chunk = Chunk::new(0, (0, 0, 0).into());
        let mut chunk2 = Chunk::new(255, (0, -2, 2).into());
        let mut chunk3 = Chunk::new(128, (0, -1, 1).into());
        let mut small_chunk = Chunk::new(56, (1, 0, 0).into());
        for i in 0..16 {
            for j in 0..16 {
                for k in 0..16 {
                    chunk.insert_block(block, i, j, k);
                    chunk2.insert_block(block, i, j, k);
                    chunk3.insert_block(block, i, j, k);
                }
            }
        }
        small_chunk.insert_block(block, 15, 5, 0);
        chunk.mesh = chunk.build_mesh(&graphics.device);
        chunk2.mesh = chunk2.build_mesh(&graphics.device);
        chunk3.mesh = chunk3.build_mesh(&graphics.device);
        small_chunk.mesh = small_chunk.build_mesh(&graphics.device);
        chunks.push(chunk);
        chunks.push(chunk2);
        chunks.push(chunk3);
        chunks.push(small_chunk);

        let vs_src = include_str!("../shaders/shader.vert");
        let fs_src = include_str!("../shaders/shader.frag");
        let pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[&uniform_bind_group_layout],
                });

        let pipeline = graphics.create_render_pipeline(
            &pipeline_layout,
            Some(Texture::DEPTH_FORMAT),
            &[BlockVertex::desc()],
            vs_src,
            fs_src,
        );

        let depth_texture =
            Texture::create_depth_texture(&graphics.device, &graphics.sc_desc, "depth_texture");

        let text = Txt::new(String::from("x: y: z: "), &graphics.device);

        Self {
            player,
            chunks,
            text,
            projection,
            depth_texture,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            pipeline,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, graphics: &Graphics) {
        self.projection.resize(new_size.width, new_size.height);

        self.depth_texture =
            Texture::create_depth_texture(&graphics.device, &graphics.sc_desc, "depth_texture");
    }

    pub fn handle_input(&mut self, event: &WindowEvent, width: u32, height: u32) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
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
}

impl Render for World {
    fn update(&mut self, dt: Duration, graphics: &Graphics) {
        self.player.update_player(dt);
        self.uniforms
            .update_camera(&self.player.camera, &self.projection);

        let mut encoder = graphics
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("update encoder"),
            });

        let staging_buffer = graphics.device.create_buffer_with_data(
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

        graphics.queue.submit(&[encoder.finish()]);
        self.text.update_debug(&self.player);
    }

    fn render(&mut self, graphics: &mut Graphics) {
        let frame = graphics
            .swap_chain
            .get_next_texture()
            .expect("Failed to acquire next swap chain texture");

        let mut encoder = graphics
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Redraw"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.4,
                        g: 0.4,
                        b: 0.5,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_stencil: 0,
                }),
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw_chunks(&self.chunks, &self.uniform_bind_group);
        }

        // Text rendering
        self.text.glyph_brush.queue(Section {
            screen_position: (5.0, 5.0),
            bounds: (graphics.size.width as f32, graphics.size.height as f32),
            text: vec![Text::new(&self.text.debug_text[..])],
            ..Section::default()
        });

        self.text
            .glyph_brush
            .draw_queued(
                &graphics.device,
                &mut encoder,
                &frame.view,
                graphics.size.width,
                graphics.size.height,
            )
            .expect("Draw queued");

        graphics.queue.submit(&[encoder.finish()]);
    }
}
