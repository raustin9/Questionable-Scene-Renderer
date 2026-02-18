use std::{collections::HashMap, sync::Arc};

use cgmath::{Matrix, SquareMatrix};
use wgpu::{SurfaceError, util::DeviceExt};
use winit::window::Window;

pub mod texture;
pub mod shader;
pub mod renderer;
pub mod render_graph;
pub mod resource;
pub mod builtin;

use crate::{geometry::{self, GBufferVertex, Mesh, Vertex}, gfx::{builtin::{deferred_pass_record_commands, write_gbuffers_pass_record_commands}, resource::{ResourceData, ResourceHandle, ResourceId, ResourceKind}}, shader::{BindGroupLayoutBuilder, ShaderBuilder, UniformBuffer}};

pub struct Context {
    device: wgpu::Device,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_configured: bool,
    queue: wgpu::Queue,
    // gbuffer_pipeline: wgpu::RenderPipeline,
    // deferred_pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    world_buffer: wgpu::Buffer,

//    normal_texture: texture::Texture,
//    albedo_texture: texture::Texture,
//    depth_texture: texture::Texture,
//
//    scene_uniform_bind_group: wgpu::BindGroup,
//    gbuffer_textures_bind_group: wgpu::BindGroup,
//    deferred_camera_bind_group: wgpu::BindGroup,
//
//    vertex_buffer: wgpu::Buffer,
//
    resources: HashMap<ResourceId, ResourceData>
}

pub struct FrameResource {
    encoder: wgpu::CommandEncoder,
    output_view: wgpu::TextureView,
    output: wgpu::SurfaceTexture,
    world_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
}

// TODO: abstract this
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection: [[f32; 4]; 4],
    inv_view_projection: [[f32; 4]; 4],
}

struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self { view_projection: cgmath::Matrix4::identity().into(), inv_view_projection: cgmath::Matrix4::identity().into() }
    }

    pub fn update_projections(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix().into();
        self.inv_view_projection = camera.build_inv_view_projection_matrix().into();
    }
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
    pub fn build_inv_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        return self.build_view_projection_matrix().invert().unwrap();
    }
}

// TODO: abstract this
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DataUniform {
    pub model_matrix: [[f32; 4]; 4],
    pub normal_model_matrix: [[f32; 4]; 4],
}

const VERTICES: &[geometry::GBufferVertex] = &[
    geometry::GBufferVertex {
        position: [0.0, 0.5, 0.0],
        normal: [0.0, 0.0, -1.0],
        texel: [0.0, 0.5]
    },
    geometry::GBufferVertex {
        position: [-0.5, -0.5, 0.0],
        normal: [0.0, 0.0, -1.0],
        texel: [-0.5, -0.5]
    },
    geometry::GBufferVertex {
        position: [0.5, -0.5, 0.0],
        normal: [0.0, 0.0, -1.0],
        texel: [0.5, -0.5]
    },
];


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
        
//        let normal_texture = texture::Texture::new(
//            &device,
//            "normal_texture", 
//            texture_size.width, 
//            texture_size.height, 
//            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
//            wgpu::TextureFormat::Rgba16Float,
//        );
//        
//        let albedo_texture = texture::Texture::new(
//            &device,
//            "albedo_texture", 
//            texture_size.width, 
//            texture_size.height, 
//            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
//            wgpu::TextureFormat::Bgra8Unorm,
//        );
//
//        let depth_texture = texture::Texture::new(
//            &device,
//            "depth_texture", 
//            texture_size.width, 
//            texture_size.height, 
//            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
//            wgpu::TextureFormat::Depth24Plus,
//        );
//
//        let gbuffer_shader = ShaderBuilder::new(&device, include_str!("../../shaders/common/gbuffer.wgsl").into())
//            .vert_entry("vs_main")
//            .frag_entry("fs_main")
//            .label("shader")
//            .add_vertex_layout(GBufferVertex::layout())
//            .build();
//
//        let write_gbuffers_bind_group_layout = BindGroupLayoutBuilder::new(&device, Some("scene"))
//            .add_uniform(wgpu::ShaderStages::VERTEX)
//            .add_uniform(wgpu::ShaderStages::VERTEX)
//            .build_layout();
//
//        let gbuffer_textures_bind_group_layout = BindGroupLayoutBuilder::new(&device, Some("gbuffer_write"))
//            .add_texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false }, false)
//            .add_texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false }, false)
//            .add_texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false }, false)
//            .build_layout();
//
//        let deferred_bind_group_layout = BindGroupLayoutBuilder::new(&device, Some("deferred"))
//            .add_uniform(wgpu::ShaderStages::FRAGMENT)
//            .build_layout();
//
//        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//            label: Some("GBuffer pipeline layout"),
//            bind_group_layouts: &[
//                write_gbuffers_bind_group_layout.layout()
//            ],
//            immediate_size: 0,
//        });
//        
//        let gbuffer_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//            label: Some("GBuffer Pipeline"),
//            layout: Some(&pipeline_layout),
//            vertex: wgpu::VertexState {
//                module: &gbuffer_shader.module(),
//                entry_point: gbuffer_shader.vert_entry(),
//                buffers: gbuffer_shader.vertex_buffers(),
//                compilation_options: wgpu::PipelineCompilationOptions::default(),
//            },
//            fragment: Some(wgpu::FragmentState {
//                module: &gbuffer_shader.module(),
//                entry_point: gbuffer_shader.frag_entry(),
//                targets: &[
//                    // Normal 
//                    Some(wgpu::ColorTargetState {
//                        format: normal_texture.format(),
//                        blend: Some(wgpu::BlendState::REPLACE),
//                        write_mask: wgpu::ColorWrites::ALL
//                    }),
//                    
//                    // Normal 
//                    Some(wgpu::ColorTargetState {
//                        format: albedo_texture.format(),
//                        blend: Some(wgpu::BlendState::REPLACE),
//                        write_mask: wgpu::ColorWrites::ALL
//                    }),
//                ],
//                compilation_options: wgpu::PipelineCompilationOptions::default(),
//            }),
//            depth_stencil: Some(wgpu::DepthStencilState {
//                format: depth_texture.format(),
//                depth_write_enabled: true,
//                depth_compare: wgpu::CompareFunction::Less,
//                stencil: wgpu::StencilState::default(),
//                bias: wgpu::DepthBiasState::default(),
//            }),
//            primitive: wgpu::PrimitiveState {
//                topology: wgpu::PrimitiveTopology::TriangleList,
//                strip_index_format: None,
//                front_face: wgpu::FrontFace::Ccw,
//                cull_mode: Some(wgpu::Face::Back),
//                polygon_mode: wgpu::PolygonMode::Fill,
//                unclipped_depth: false,
//                conservative: false,
//            },
//            multisample: wgpu::MultisampleState {
//                count: 1,
//                mask: 0xFFFF_FFFF_FFFF_FFFF_u64, // use all sample mask
//                alpha_to_coverage_enabled: false
//            },
//            multiview_mask: None,
//            cache: None,
//        });
//
//        let gbuffer_textures_bind_group = gbuffer_textures_bind_group_layout.create_bind_group(&device, &[
//            wgpu::BindGroupEntry {
//                binding: 0,
//                resource: wgpu::BindingResource::TextureView(&normal_texture.view()),
//            },
//            wgpu::BindGroupEntry {
//                binding: 1,
//                resource: wgpu::BindingResource::TextureView(&albedo_texture.view()),
//            },
//            wgpu::BindGroupEntry {
//                binding: 2,
//                resource: wgpu::BindingResource::TextureView(&depth_texture.view()),
//            },
//        ]);
//
//        let deferred_shader = ShaderBuilder::new(&device, include_str!("../../shaders/common/deferred.wgsl").into())
//            .vert_entry("vs_main")
//            .frag_entry("fs_main")
//            .label("deferred_shader")
//            .build();
//                
//        let deferred_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//            label: Some("Deferred pipeline layout"),
//            bind_group_layouts: &[
//                &gbuffer_textures_bind_group_layout.layout(),
//                &deferred_bind_group_layout.layout(),
//            ],
//            immediate_size: 0,
//        });
//        let deferred_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//            label: Some("Deferred Pipeline"),
//            layout: Some(&deferred_pipeline_layout),
//            vertex: wgpu::VertexState {
//                module: deferred_shader.module(),
//                entry_point: deferred_shader.vert_entry(),
//                buffers: deferred_shader.vertex_buffers(),
//                compilation_options: wgpu::PipelineCompilationOptions::default(),
//            },
//            fragment: Some(wgpu::FragmentState {
//                module: deferred_shader.module(),
//                entry_point: deferred_shader.frag_entry(),
//                targets: &[
//                    Some(wgpu::ColorTargetState {
//                        format: surface_config.format,
//                        blend: Some(wgpu::BlendState::REPLACE),
//                        write_mask: wgpu::ColorWrites::ALL,
//                    }),
//                ],
//                compilation_options: wgpu::PipelineCompilationOptions::default(),
//            }),
//            depth_stencil: None,
//            primitive: wgpu::PrimitiveState {
//                topology: wgpu::PrimitiveTopology::TriangleList,
//                strip_index_format: None,
//                front_face: wgpu::FrontFace::Ccw,
//                cull_mode: Some(wgpu::Face::Back),
//                polygon_mode: wgpu::PolygonMode::Fill,
//                unclipped_depth: false,
//                conservative: false,
//            },
//            multisample: wgpu::MultisampleState {
//                count: 1,
//                mask: 0xFFFF_FFFF_FFFF_FFFF_u64, // use all sample mask
//                alpha_to_coverage_enabled: false
//            },
//            multiview_mask: None,
//            cache: None
//        });.

        let camera = Camera {
            eye: (0.0, 2.0, 4.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: surface_config.width as f32 / surface_config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 2000.0,
        };
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_projections(&camera);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_uniform_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let model_matrix = cgmath::Matrix4::from_translation(cgmath::Vector3::<f32> { 
            x: 0.0, y: 0.0, z: 0.0
        });
        let inverse_model = model_matrix.invert().unwrap();
        let inverse_transpose_model = inverse_model.transpose();
        let data_uniform = DataUniform {
            model_matrix: model_matrix.into(),
            normal_model_matrix: inverse_transpose_model.into()
        };
        let world_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world_uniform_buffer"),
            contents: bytemuck::cast_slice(&[data_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

//        let scene_uniform_bind_group = write_gbuffers_bind_group_layout.create_bind_group(&device, &[
//            wgpu::BindGroupEntry {
//                binding: 0,
//                resource: world_buffer.as_entire_binding(),
//            },
//            wgpu::BindGroupEntry {
//                binding: 1,
//                resource: camera_buffer.as_entire_binding(),
//            },
//        ]);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX
        });
        
//        let deferred_camera_bind_group = deferred_bind_group_layout.create_bind_group(&device, &[
//            wgpu::BindGroupEntry {
//                binding: 0,
//                resource: camera_buffer.as_entire_binding()
//            }
//        ]);

        Self {
            device,
            queue,
            surface,
            surface_config,
            surface_configured: false,
//            gbuffer_pipeline,
//            deferred_pipeline,
            camera_buffer,
            world_buffer,

//            normal_texture,
//            albedo_texture,
//            depth_texture,
//
//            scene_uniform_bind_group,
//            gbuffer_textures_bind_group,
//            vertex_buffer,
//            deferred_camera_bind_group,

            resources: HashMap::new()
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
    }

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
            camera_buffer: self.camera_buffer.clone(), 
            world_buffer: self.world_buffer.clone() 
        }))
    }

    pub fn end_frame(&self, frame_resource: FrameResource) {
//        let mut encoder = frame_resource.encoder;
//        let output_view = frame_resource.output_view;
//        let output = frame_resource.output;
//        
//        write_gbuffers_pass_record_commands(
//            &mut encoder, 
//            &self.gbuffer_pipeline, 
//            &self.normal_texture, 
//            &self.albedo_texture, 
//            &self.depth_texture, 
//            &self.scene_uniform_bind_group, 
//            &self.vertex_buffer, 
//            VERTICES.len() as u32
//        );
//
//        deferred_pass_record_commands(
//            &mut encoder, 
//            &self.deferred_pipeline, 
//            &self.gbuffer_textures_bind_group, 
//            &self.deferred_camera_bind_group, 
//            &output_view
//        );

        // let encoder = frame_resource.encoder;
        // let output = &frame_resource.output;


        // let mut encoder = frame_resource.encoder;
        
        let encoder = frame_resource.encoder;
        let output = frame_resource.output;
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
    }

    // Create a texture resource to be used by consumers like the render graph
    // TODO: update the lifetime params of the return value here
    pub fn create_texture(&mut self, name: &str, size: wgpu::Extent3d, usage: wgpu::TextureUsages, format: wgpu::TextureFormat) -> ResourceId {
        let texture = texture::Texture::new(&self.device, name, size.width, size.height, usage, format);

        // let id = ResourceId(self.resources.len());
        let id = ResourceId::new();
        self.resources.insert(id, ResourceData::Texture(texture));

        id
    }

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

    pub fn get_resource(&self, id: &ResourceId) -> Option<&ResourceData> {
        self.resources.get(id)
    }
}
