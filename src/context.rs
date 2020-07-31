use winit::{event::WindowEvent, window::Window};

use crate::render::graphics::Graphics;

pub struct Context {
    pub size: winit::dpi::PhysicalSize<u32>,

    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,

    pub graphics: Graphics,
}

impl Context {
    pub async fn new(window: &Window) -> Self {
        // Initialize wgpu
        log::info!("Initializing Wgpu");
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
        let graphics = Graphics::new(size, device, queue, &sc_desc);

        Self {
            size,
            surface,
            adapter,
            sc_desc,
            swap_chain,
            graphics,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.graphics
            .projection
            .resize(new_size.width, new_size.height);

        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self
            .graphics
            .device
            .create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn input(&mut self, event: &WindowEvent, focused: bool) -> bool {
        if !focused {
            return false;
        }
        match event {
            _ => self
                .graphics
                .handle_input(event, self.size.width, self.size.height),
        }
    }
}
