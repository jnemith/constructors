use cgmath::Deg;
use std::time::Duration;
use winit::{event::WindowEvent, window::Window};

use crate::render::{
    camera::{Camera, Projection},
    graphics::{Graphics, Render},
};
use crate::{player::Player, world::World};

pub struct Context {
    pub size: winit::dpi::PhysicalSize<u32>,

    surface: wgpu::Surface,
    graphics: Graphics,

    world: World,
}

impl Context {
    pub async fn new(window: &Window) -> Self {
        log::info!("Initializing Context");
        let size = window.inner_size();
        let surface = wgpu::Surface::create(window);

        // Get GPU
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

        // Request access to GPU
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        log::info!("Initializing Graphics");
        let graphics = Graphics::new(size, adapter, sc_desc, swap_chain, device, queue);

        let camera = Camera::new((-13.0, 16.0, -12.0), Deg(-90.0), Deg(-10.0));
        let projection = Projection::new(
            graphics.sc_desc.width,
            graphics.sc_desc.height,
            Deg(70.0),
            0.1,
            100.0,
        );
        let player = Player::new(camera);

        log::info!("Initializing world");
        let world = World::new(player, projection, &graphics);

        Self {
            size,
            surface,
            graphics,
            world,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.graphics.resize(new_size, &self.surface);
        self.world.resize(new_size, &self.graphics);
    }

    pub fn input(&mut self, event: &WindowEvent, focused: bool) -> bool {
        if !focused {
            return false;
        }
        match event {
            _ => self
                .world
                .handle_input(event, self.size.width, self.size.height),
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.world.update(dt, &self.graphics);
    }

    pub fn render(&mut self) {
        self.world.render(&mut self.graphics);
    }
}
