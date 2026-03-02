use std::{collections::HashMap, sync::Arc};

use wgpu::{SurfaceError, util::DeviceExt};
use winit::window::Window;

pub mod texture;
pub mod shader;
pub mod renderer;
pub mod render_graph;
pub mod resource;
pub mod builtin;
pub mod material;

use crate::{geometry::{GBufferVertex, Mesh, Vertex}, gfx::resource::{BufferHandle, BufferRegistry, CameraInfoFeature, DiffuseColorFeature, DiffuseTextureFeature, GBufferTexturesFeature, LightsDataFeature, PipelineHandle, PipelineManager, PipelineRequestInfo, ResourceData, ResourceId, SamplerDescriptor, ShaderFeature, ShaderFeatureId, ShaderFeatureRegistry, ShaderRegistry, ShaderResource, TextureHandle, TextureRegistry, TransformFeature, TransparentMaterialFeatureDC, TransparentMaterialFeatureDT}, shader::UniformBuffer};

pub struct Context {
    device: wgpu::Device,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_configured: bool,
    queue: wgpu::Queue,
    // camera_buffer: wgpu::Buffer,
    resources: HashMap<ResourceId, ResourceData>,
    texture_registry: TextureRegistry,
    buffer_registry: BufferRegistry,
    pub pipeline_manager: PipelineManager,
    pub shader_registry: ShaderRegistry,
}

pub struct FrameResource {
    encoder: wgpu::CommandEncoder,
    output_view: wgpu::TextureView,
    output: wgpu::SurfaceTexture,
    // camera_buffer: wgpu::Buffer,
}

/// Contains the available feature id's for the shaders used by materials.
pub struct MaterialShaderFeatures {
    pub camera: ShaderFeatureId,
    pub transform: ShaderFeatureId,
    pub diffuse_texture: ShaderFeatureId,
    pub diffuse_color: ShaderFeatureId,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

impl<'a> Context {
    pub async fn new(window: Arc<Window>) -> Self {
        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,

            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())
            .expect("Failed to create surface");

        let (device, queue, surface_capabilities) = Self::get_device(&instance, &surface).await;

        let surface_format = surface_capabilities.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);
        
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: Self::find_present_mode(&surface_capabilities.present_modes),
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let mut shader_features = ShaderFeatureRegistry::new();
        let camera_feature = shader_features.register::<CameraInfoFeature>(&device);
        let transform_feature = shader_features.register::<TransformFeature>(&device);
        let diffuse_texture_feature = shader_features.register::<DiffuseTextureFeature>(&device);
        let diffuse_color_feature = shader_features.register::<DiffuseColorFeature>(&device);
        let gbuffer_textures_feature = shader_features.register::<GBufferTexturesFeature>(&device);
        let lights_data_feature = shader_features.register::<LightsDataFeature>(&device);
        let transparent_material_dt_feature = shader_features.register::<TransparentMaterialFeatureDT>(&device);
        let transparent_material_dc_feature = shader_features.register::<TransparentMaterialFeatureDC>(&device);

        let mut shader_registry = ShaderRegistry::new(shader_features);
        shader_registry.add_material(
            &device, 
            "write_gbuffers_dt", 
            wgpu::ShaderSource::Wgsl(include_str!("../../shaders/common/gbuffer.wgsl").into()), 
            vec![camera_feature, transform_feature, diffuse_texture_feature], 
            &[GBufferVertex::layout()]
        );
        shader_registry.add_material(
            &device, 
            "write_gbuffers_dc", 
            wgpu::ShaderSource::Wgsl(include_str!("../../shaders/common/no-texture-write-gbuffers.wgsl").into()), 
            vec![camera_feature, transform_feature, diffuse_color_feature], 
            &[GBufferVertex::layout()]
        );

        shader_registry.add_global(
            &device, 
            "lighting", 
            wgpu::ShaderSource::Wgsl(include_str!("../../shaders/common/deferred.wgsl").into()), 
            vec![gbuffer_textures_feature, camera_feature, lights_data_feature]
        );

        shader_registry.add_material(
            &device, 
            "alpha_dt", 
            wgpu::ShaderSource::Wgsl(include_str!("../../shaders/common/alpha.wgsl").into()), 
            vec![camera_feature, transform_feature, transparent_material_dt_feature, lights_data_feature], 
            &[GBufferVertex::layout()]
        );
        shader_registry.add_material(
            &device, 
            "alpha_dc", 
            wgpu::ShaderSource::Wgsl(include_str!("../../shaders/common/no-texture-alpha.wgsl").into()), 
            vec![camera_feature, transform_feature, transparent_material_dc_feature, lights_data_feature], 
            &[GBufferVertex::layout()]
        );

        let texture_registry = TextureRegistry::new(window_size.width, window_size.height);
        let buffer_registry = BufferRegistry::new();
        let pipeline_manager = PipelineManager::new();

        Self {
            device,
            queue,
            surface,
            surface_config,
            surface_configured: false,
            // camera_buffer,

            resources: HashMap::new(),
            texture_registry,
            buffer_registry,
            pipeline_manager,
            shader_registry,
        }
    }

    /// Create and get the device
    async fn get_device(instance: &wgpu::Instance, surface: &wgpu::Surface<'static>) -> (wgpu::Device, wgpu::Queue, wgpu::SurfaceCapabilities) {
        let adapter = 
            instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false
            })
            .await
            .expect("Failed to create device");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off
            })
            .await
            .unwrap();

        let capabilities = surface.get_capabilities(&adapter);
        return (device, queue, capabilities); 
    }

    /// Find the preferable present mode if present
    pub fn find_present_mode(present_modes: &[wgpu::PresentMode]) -> wgpu::PresentMode {
        for present_mode in present_modes {
            if *present_mode == wgpu::PresentMode::Immediate {
                log::info!("Using immediate mode");
                return wgpu::PresentMode::Immediate;
            }
            if *present_mode == wgpu::PresentMode::Mailbox {
                log::info!("Using mailbox mode");
                return wgpu::PresentMode::Mailbox;
            }
        }

        log::info!("Using FIFO mode");
        wgpu::PresentMode::Fifo
    }

    /// Update the dimensions for systems dependent on them.
    pub fn update_dimensions(&mut self, width: u32, height: u32) {
        if width != 0 && height != 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            self.surface_configured = true;

            self.texture_registry.resize_textures(&self.device, width, height);
        }
    }

    /// Begin frame behavior for graphics context.
    /// Get the command encoders and other frame resources
    pub fn begin_frame(&self) -> Result<Option<FrameResource>, SurfaceError> {
        if !self.surface_configured {
            return Ok(None);
        }

        let output = self.surface.get_current_texture()?;

        let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("command_encoder")
        });

        Ok(Some(FrameResource { 
            encoder, 
            output, 
            output_view, 
            // camera_buffer: self.camera_buffer.clone(), 
        }))
    }

    /// End frame behavior for the context.
    /// Submits resources and presents to output
    pub fn end_frame(&self, frame_resource: FrameResource) {
        let encoder = frame_resource.encoder;
        let output = frame_resource.output;
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    // Create a texture resource to be used by consumers like the render graph
    // TODO: update the lifetime params of the return value here
    pub fn create_texture(&mut self, descriptor: resource::TextureDescriptor, sampler_options: Option<SamplerDescriptor>) -> resource::TextureHandle {
        self.texture_registry.create_texture(&self.device, descriptor, sampler_options)
    }

    /// Write data to an existing texture
    pub fn write_texture(&self, handle: TextureHandle, data: &[u8], size: wgpu::Extent3d, bytes_per_pixel: u32) {
        let texture = self.texture_registry.get_texture(handle);
        match texture {
            None => {},
            Some(found_texture) => 
                self.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: found_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All
                    }, 
                    data, 
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(size.width * bytes_per_pixel),
                        rows_per_image: Some(size.height)
                    }, 
                    size
                ),
        }
    }

    /// Create a buffer on the gpu and get its corresonding handle
    pub fn create_buffer(&mut self, usages: wgpu::BufferUsages, data: &[u8]) -> BufferHandle {
        self.buffer_registry.create_buffer(&self.device, usages, data)
    }

    /// Get a reference to an existing buffer
    pub fn get_buffer(&self, handle: BufferHandle) -> Option<&wgpu::Buffer> {
        self.buffer_registry.get_buffer(handle)
    }

    /// Get the usages for an exsiting buffer
    pub fn get_buffer_usages(&self, handle: BufferHandle) -> Option<&wgpu::BufferUsages> {
        self.buffer_registry.get_usages(handle)
    }

    /// Write new data to an existing buffer
    pub fn write_buffer(&self, handle: BufferHandle, offset: u64, data: &[u8]) {
        self.buffer_registry.write_buffer(handle, &self.queue, offset, data);
    }

    // TODO: remove?
    pub fn create_uniform_buffer(&mut self, usages: wgpu::BufferUsages, data: &[u8]) -> ResourceId {
        let uniform_buffer = UniformBuffer::new(&self.device, usages, data);

        let id = ResourceId::new();
        self.resources.insert(id, ResourceData::UniformBuffer(uniform_buffer));

        id
    }

    pub fn create_mesh(&mut self, name: &str, vertex_count: u32, vertex_data: &[u8], index_data: Option<&[u32]>) -> ResourceId {
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(name),
            contents: vertex_data,
            usage: wgpu::BufferUsages::VERTEX
        });

        let (index_buffer, index_count) = match index_data {
            None => (
                None, 
                0
            ),
            Some(data) => (
                Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(format!("{}_index_buffer", name).as_ref()),
                    contents: bytemuck::cast_slice(data),
                    usage: wgpu::BufferUsages::INDEX
                })), 
                data.len() as u32
            ),
        };

        let id = ResourceId::new();
        self.resources.insert(id, ResourceData::Mesh(
            Mesh::new(vertex_buffer, vertex_count, index_buffer, index_count))
        );

        id
    }

    pub fn get_texture(&self, handle: TextureHandle) -> Option<&wgpu::Texture> {
        self.texture_registry.get_texture(handle)
    }

    pub fn get_texture_view(&self, handle: TextureHandle) -> Option<&wgpu::TextureView> {
        self.texture_registry.get_view(handle)
    }

    pub fn get_sampler(&self, handle: TextureHandle) -> Option<&wgpu::Sampler> {
        self.texture_registry.get_sampler(handle)
    }

    pub fn get_resource(&self, id: &ResourceId) -> Option<&ResourceData> {
        self.resources.get(id)
    }

    pub fn request_pipeline(&mut self, requirements: &PipelineRequestInfo) -> PipelineHandle {
        self.pipeline_manager.request_pipeline(&self.device, requirements)
    }

    pub fn get_pipeline(&self, handle: PipelineHandle) -> Option<&wgpu::RenderPipeline> {
        self.pipeline_manager.get_pipeline(handle)
    }


    pub fn get_shader_feature<F: ShaderFeature>(&self) -> Option<&resource::ShaderFeatureEntry> {
        self.shader_registry.get_feature::<F>()
    }

    pub fn get_shader_feature_id<F: ShaderFeature>(&self) -> Option<resource::ShaderFeatureId> {
        self.shader_registry.get_feature_id::<F>()
    }

    pub fn get_material(&self, features: &[ShaderFeatureId], vertex_layouts: &[wgpu::VertexBufferLayout<'static>]) -> Option<&ShaderResource> {
        self.shader_registry.get_material(features, vertex_layouts)
    }

    pub fn create_material_bind_group<F: ShaderFeature>(&self, entries: &[wgpu::BindGroupEntry]) -> wgpu::BindGroup {
        let layout = self.device.create_bind_group_layout(&F::layout_descriptor());

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries
        })
    }
}
