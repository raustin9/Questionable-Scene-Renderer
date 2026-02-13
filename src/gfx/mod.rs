use std::sync::Arc;

use wgpu::SurfaceError;
use winit::window::Window;

use crate::geometry::{self, Vertex};

pub struct Context {
    device: wgpu::Device,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_configured: bool,
    queue: wgpu::Queue,
    gbuffer_pipeline: wgpu::RenderPipeline,
}

// TODO: abstract this
struct Camera {
    view_projection: [[f32; 4]; 4],
    inv_view_projection: [[f32; 4]; 4],
}

// TODO: abstract this
struct Uniform {
    model_matrix: [[f32; 4]; 4],
    normal_model_matrix: [[f32; 4]; 4],
}

impl Context {
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
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let texture_size = wgpu::Extent3d {
            width: window_size.width,
            height: window_size.height,
            depth_or_array_layers: 1,
        };
        
        let gbuffer_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GBuffer Texture"),
            size: texture_size,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            format: wgpu::TextureFormat::Rgba16Float,
            dimension: wgpu::TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        });
        let gbuffer_view = gbuffer_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let albedo_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Albedo Texture"),
            size: texture_size,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            format: wgpu::TextureFormat::Bgra8Unorm,
            dimension: wgpu::TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        });
        let albedo_view = albedo_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: texture_size,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            format: wgpu::TextureFormat::Depth24Plus,
            dimension: wgpu::TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let gbuffer_shader_bytes = include_str!("../../shaders/common/gbuffer.wgsl");
        let gbuffer_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GBuffer Shader"),
            source: wgpu::ShaderSource::Wgsl(String::from(gbuffer_shader_bytes).into())
        });

        let gbuffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // View projection
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ],
            label: Some("gbuffer_bind_group_layout"),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GBuffer pipeline layout"),
            bind_group_layouts: &[
                &gbuffer_bind_group_layout
            ],
            immediate_size: 0,
        });
        
        let gbuffer_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GBuffer Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &gbuffer_shader,
                entry_point: Some("vs_main"),
                buffers: &[geometry::GBufferVertex::layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &gbuffer_shader,
                entry_point: Some("fs_main"),
                targets: &[
                    // Normal 
                    Some(wgpu::ColorTargetState {
                        format: gbuffer_texture.format(),
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL
                    }),
                    
                    // Normal 
                    Some(wgpu::ColorTargetState {
                        format: albedo_texture.format(),
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL
                    }),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_texture.format(),
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: 0xFFFF_FFFF_FFFF_FFFF_u64, // use all sample mask
                alpha_to_coverage_enabled: false
            },
            multiview_mask: None,
            cache: None,
        });

        let gbuffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GBuffer bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ]
        });
        let deferred_shader_bytes = include_str!("../../shaders/common/deferred.wgsl");
        let deferred_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Deferred shader module"),
            source: wgpu::ShaderSource::Wgsl(String::from(deferred_shader_bytes).into())
        });
        let deferred_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("deferred_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });
        let deferred_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Deferred pipeline layout"),
            bind_group_layouts: &[
                &gbuffer_bind_group_layout,
                &deferred_bind_group_layout,
            ],
            immediate_size: 0,
        });
        let deferred_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Deferred Pipeline"),
            layout: Some(&deferred_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &deferred_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &deferred_shader,
                entry_point: Some("fs_main"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: 0xFFFF_FFFF_FFFF_FFFF_u64, // use all sample mask
                alpha_to_coverage_enabled: false
            },
            multiview_mask: None,
            cache: None
        });

        Self {
            device,
            queue,
            surface,
            surface_config,
            surface_configured: false,
            gbuffer_pipeline,
        }
    }

    async fn get_device(instance: &wgpu::Instance, surface: &wgpu::Surface<'static>) -> (wgpu::Device, wgpu::Queue, wgpu::SurfaceCapabilities) {
        let adapter = 
            instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
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

    pub fn update_dimensions(&mut self, width: u32, height: u32) {
        if width != 0 && height != 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            self.surface_configured = true;
        }

        println!("Updating dimensions: {}, {}", width, height);
    }

    pub fn begin_frame(&mut self) -> Result<(), SurfaceError> {
        Ok(())
    }

    pub fn end_frame(&mut self) {}
}
