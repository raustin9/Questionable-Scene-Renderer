use std::sync::{Arc, Mutex};

use winit::{application::ApplicationHandler, error::EventLoopError, event::{KeyEvent, WindowEvent}, event_loop::{ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::Window};

use crate::{Scene, gfx::{self, renderer::{DeferredRenderer, Renderer}}};

pub struct Driver<'a> {
    /// Window for driving the application
    window: Option<Arc<Window>>,
    context: Option<gfx::Context>,
    scene: &'a Scene<'a>,
    renderer: Option<DeferredRenderer<'a>>,
}

impl<'a> Driver<'a> {
    pub fn run(scene: &Scene) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new()
            .expect("Failed to create event loop");
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut driver = Driver { 
            window: None,
            context: None,
            renderer: None,
            scene
        };
        
        return event_loop.run_app(&mut driver);
    }
}

impl<'a> ApplicationHandler for Driver<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes)
            .expect("Failed to create window"));

        self.window = Some(window.clone());
        self.context = Some(
            pollster::block_on(gfx::Context::new(window.clone()))
        );
        
        self.renderer = Some(
            DeferredRenderer::new(&self.scene, self.context.as_mut().unwrap())
        );
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

        let renderer = match &mut self.renderer {
            Some(renderer) => renderer,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                window.request_redraw();

                match context.begin_frame() {
                    // Likely configuring surface
                    Ok(None) => {},

                    // Got a frame resource. Execute rendering
                    Ok(Some(mut frame_resource)) => {
                        renderer.render(context, &mut frame_resource);
                        context.end_frame(frame_resource);
                    },

                    // Reconfigure the surface
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = window.inner_size();
                        context.update_dimensions(size.width, size.height);
                        renderer.resize(context, size.width, size.height);
                    },

                    // Misc error
                    Err(e) => log::error!("Error when rendering: {}", e),
                };
            },
            WindowEvent::Resized(size) => {
                context.update_dimensions(size.width, size.height);
                renderer.resize(context, size.width, size.height);
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
