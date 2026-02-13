use std::sync::Arc;

use winit::{application::ApplicationHandler, error::EventLoopError, event::{KeyEvent, WindowEvent}, event_loop::{ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::Window};

use crate::gfx;

#[derive(Default)]
pub struct Driver {
    /// Window for driving the application
    window: Option<Arc<Window>>,
    context: Option<gfx::Context>,
}

impl Driver {
    pub fn run() -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new()
            .expect("Failed to create event loop");
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut driver = Driver::default();
        return event_loop.run_app(&mut driver);
    }
}

impl ApplicationHandler for Driver {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes)
            .expect("Failed to create window"));

        self.window = Some(window.clone());
        self.context = Some(pollster::block_on(gfx::Context::new(window.clone())));
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            _window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {

        let window = match &self.window {
            Some(window) => window,
            None => return,
        };

        let context = match &mut self.context {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                window.request_redraw();

                match context.begin_frame() {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = window.inner_size();
                        context.update_dimensions(size.width, size.height);
                    },
                    Err(e) => log::error!("Error when rendering: {}", e),
                };
            },
            WindowEvent::Resized(size) => {
                context.update_dimensions(size.width, size.height);
            },
            WindowEvent::KeyboardInput { 
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    ..
                }, 
                .. 
            } => match (code, key_state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                _ => {}
            },
            _ => {}
        }
    }
}
