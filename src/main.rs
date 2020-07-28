mod context;
mod player;
mod render;

use futures::executor::block_on;
use winit::{
    dpi::{LogicalPosition, PhysicalPosition},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::context::Context;

fn main() {
    env_logger::init();

    log::info!("Building window");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Constructor")
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    log::info!("Initializing Context");
    let mut context = block_on(Context::new(&window));

    let mut last_time = std::time::Instant::now();
    log::info!("Begin loop");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !context.input(event) {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            context.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            context.resize(**new_inner_size);
                        }

                        WindowEvent::CursorMoved { .. } => {
                            window
                                .set_cursor_position(PhysicalPosition::new(
                                    context.size.width as f32 / 2.0,
                                    context.size.height as f32 / 2.0,
                                ))
                                .unwrap();
                        }

                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => {
                                log::info!("Escape pressed - exiting");
                                *control_flow = ControlFlow::Exit
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = now - last_time;
                last_time = now;
                context.graphics.update(dt);
                context.graphics.render(&mut context.swap_chain);
            }
            _ => {}
        }
    });
}
