use std::{sync::{Arc, Mutex}, time::Instant};

use winit::{application::ApplicationHandler, dpi::LogicalSize, error::EventLoopError, event::{KeyEvent, WindowEvent}, event_loop::{ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::Window};

use crate::{Scene, camera::{Camera, CameraController}, gfx::{self, renderer::{DeferredRenderer, Renderer}}};

pub struct Driver<'a> {
    /// Window for driving the application
    window: Option<Arc<Window>>,
    context: Option<gfx::Context>,
    scene: &'a Scene<'a>,
    renderer: Option<DeferredRenderer<'a>>,
    frame_count: u64,
    last_start_time: Instant,
    camera_controller: CameraController,
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
            scene,
            frame_count: 0,
            last_start_time: Instant::now(),
            camera_controller: CameraController::new(0.2)
        };
        
        return event_loop.run_app(&mut driver);
    }
}

impl<'a> ApplicationHandler for Driver<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(self.scene.width, self.scene.height));
        
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

        let window = match &mut self.window {
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

                self.frame_count = self.frame_count + 1;
                match context.begin_frame() {
                    // Likely configuring surface
                    Ok(None) => {},

                    // Got a frame resource. Execute rendering
                    Ok(Some(mut frame_resource)) => {
                        renderer.update_camera(&self.camera_controller, context);
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

                print!("\rFrame Count: {}", self.frame_count);

                if self.last_start_time.elapsed().as_secs() > 2 {
                    let elapsed_time = self.last_start_time.elapsed().as_secs();
                    let fps = self.frame_count as f32 / elapsed_time as f32;
                    
                    window.set_title(format!("FPS: {}", fps).as_str());

                    self.frame_count = 0;
                    self.last_start_time = Instant::now();
                }
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
                _ => {
                    self.camera_controller.handle_key(code, key_state.is_pressed());
                }
            },
            _ => {}
        }
    }
}
