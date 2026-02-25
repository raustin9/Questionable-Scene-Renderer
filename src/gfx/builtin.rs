use std::num::{NonZero, NonZeroU64};

use crate::{geometry::{GBufferVertex, Vertex}, gfx::{Context, material::DiffuseResource, render_graph::{RenderPassKind, RenderPassNode}, renderer::Renderable, resource::{BufferHandle, PipelineBuilder, PipelineHandle, PipelineRequestInfo, ResourceData, ResourceId, TextureHandle}, texture}, shader::{BindGroupLayout, BindGroupLayoutBuilder, ShaderBuilder}};

pub struct WriteGBuffersPassFrameData<'a> {
    pub camera_buffer: &'a wgpu::Buffer,
}

// A builtin renderpass for writing geometries to the gbuffer textures
pub struct WriteGBuffersPass {
    name: &'static str,
    kind: RenderPassKind,

    scene_bind_group_layout: BindGroupLayout,
    scene_bind_group: wgpu::BindGroup,
    material_bind_group_layout: BindGroupLayout,
    material_no_texture_bind_group_layout: BindGroupLayout,

    has_texture_shader_module: wgpu::ShaderModule,
    no_texture_shader_module: wgpu::ShaderModule,
    color_targets: Vec<wgpu::ColorTargetState>,
    depth_target: wgpu::DepthStencilState,
    multisample: wgpu::MultisampleState,
    topology: wgpu::PrimitiveState,
    
    geometry_bind_group_layout: BindGroupLayout,

    /* Textures */
    normal_texture_handle: ResourceId,
    albedo_texture_handle: ResourceId,
    depth_texture_handle: ResourceId,

    camera_buffer_handle: BufferHandle,

    renderables: Vec<Renderable>,
}

impl WriteGBuffersPass {
    pub fn new(
        context: &mut Context,
        normal_texture_handle: ResourceId,
        albedo_texture_handle: ResourceId,
        depth_texture_handle: ResourceId,
        camera_buffer_handle: BufferHandle,
        renderables: Vec<Renderable>
    ) -> Self {
        let normal_texture = context.get_texture(normal_texture_handle).expect("Failed to get normal texture from context");
        let albedo_texture = context.get_texture(albedo_texture_handle).expect("Failed to get albedo texture from context");
        let depth_texture = context.get_texture(depth_texture_handle).expect("Failed to get depth texture from context");

        let gbuffer_shader = ShaderBuilder::new(&context.device, include_str!("../../shaders/common/gbuffer.wgsl").into())
            .vert_entry("vs_main")
            .frag_entry("fs_main")
            .label("shader")
            .add_vertex_layout(GBufferVertex::layout())
            .build();
        
        let no_texture_gbuffer_shader = ShaderBuilder::new(&context.device, include_str!("../../shaders/common/no-texture-write-gbuffers.wgsl").into())
            .vert_entry("vs_main")
            .frag_entry("fs_main")
            .label("shader")
            .add_vertex_layout(GBufferVertex::layout())
            .build();

        let scene_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("scene"))
            .add_uniform(wgpu::ShaderStages::VERTEX)
            .build_layout();

        let geometry_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("geometry"))
            .add_uniform(wgpu::ShaderStages::VERTEX)
            .build_layout();

        let material_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("material"))
            .add_texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: true }, false)
            .add_sampler(wgpu::ShaderStages::FRAGMENT, wgpu::SamplerBindingType::Filtering)
            .build_layout();

        let material_no_texture_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("material"))
            .add_uniform(wgpu::ShaderStages::FRAGMENT)
            .build_layout();
        
        let color_targets = vec![
            wgpu::ColorTargetState {
                format: normal_texture.format(),
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL
            },
            wgpu::ColorTargetState {
                format: albedo_texture.format(),
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL
            },
        ];
        let depth_target = wgpu::DepthStencilState {
            format: depth_texture.format(),
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };
        let multisample = wgpu::MultisampleState {
            count: 1,
            mask: 0xFFFF_FFFF_FFFF_FFFF_u64, // use all sample mask
            alpha_to_coverage_enabled: false
        };

        let topology = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        };

        let camera_buffer = context.get_buffer(camera_buffer_handle)
            .expect("Failed to get camera buffer when creating write gbuffer pass");
        
        let scene_bind_group = scene_bind_group_layout.create_bind_group(&context.device, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            },
        ]);

        Self {
            name: "write_gbuffers_pass",
            kind: RenderPassKind::Graphics,
            depth_target,
            color_targets,
            multisample,
            topology,
            has_texture_shader_module: gbuffer_shader.module().clone(),
            no_texture_shader_module: no_texture_gbuffer_shader.module().clone(),
            normal_texture_handle,
            albedo_texture_handle,
            depth_texture_handle,
            geometry_bind_group_layout,
            material_bind_group_layout,
            scene_bind_group_layout,
            scene_bind_group,
            renderables,
            camera_buffer_handle,
            material_no_texture_bind_group_layout,
        }
    }

    pub fn set_frame_data(&mut self, _context: &Context, _frame_data: &WriteGBuffersPassFrameData) {
    }
}

impl<'a> RenderPassNode for WriteGBuffersPass {
    fn name(&self) -> &str {
        self.name
    }

    fn kind(&self) -> RenderPassKind {
        self.kind
    }

    fn on_resize(&mut self, _context: &Context, _width: u32, _height: u32) {}

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &mut Context) {
        let normal_texture_view = context.get_texture_view(self.normal_texture_handle).expect("Failed to get normal texture view from context");
        let albedo_texture_view = context.get_texture_view(self.albedo_texture_handle).expect("Failed to get albedo texture view from context");
        let depth_texture_view = context.get_texture_view(self.depth_texture_handle).expect("Failed to get depth texture view from context");

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(self.name),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &normal_texture_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    }
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &albedo_texture_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    }
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        let has_texture_layouts = [
            self.scene_bind_group_layout.layout(),
            self.geometry_bind_group_layout.layout(),
            self.material_bind_group_layout.layout(),
        ];
        let no_texture_layouts = [
            self.scene_bind_group_layout.layout(),
            self.geometry_bind_group_layout.layout(),
            self.material_no_texture_bind_group_layout.layout(),
        ];
        
        render_pass.set_bind_group(0, &self.scene_bind_group, &[]);

        for renderable in &self.renderables {
            render_pass.set_bind_group(1, &renderable.geometry.bind_group, &[]);

            let (pipeline, bind_group) = match renderable.material.diffuse {
                // Has diffuse texture so we use it
                DiffuseResource::Texture(texture_handle) => {
                    let material_texture_view = context.get_texture_view(texture_handle)
                        .expect("Failed to get texture view for material diffuse texture");
                    let material_sampler = context.get_sampler(texture_handle)
                        .expect("Failed to get sampler for material diffuse texture");

                    let material_bind_group = self.material_bind_group_layout.create_bind_group(&context.device, &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(material_texture_view)
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(material_sampler)
                        },
                    ]);
                    let requirements = PipelineRequestInfo {
                        color_targets: self.color_targets.as_slice(),
                        depth_target: Some(self.depth_target.clone()),
                        vertex_module: &self.has_texture_shader_module,
                        fragment_module: Some(&self.has_texture_shader_module),
                        fragment_entry: Some("fs_main"),
                        vertex_entry: "vs_main",
                        multisample: &self.multisample,
                        topology: self.topology,
                        vertex_layouts: &[GBufferVertex::layout()],
                        bind_group_layouts: &has_texture_layouts
                    };

                    let pipeline_handle = context.request_pipeline(&requirements);
                    let pipeline = context.get_pipeline(pipeline_handle)
                        .expect("Failed to get texture pipeline");

                    (pipeline, material_bind_group)
                },

                // No diffuse texture use diffuse color
                DiffuseResource::Color(buffer_handle) => {
                    let diffuse_buffer = context.get_buffer(buffer_handle)
                        .expect("Failed to get renderable diffuse color buffer");
                    
                    let bind_group = self.material_no_texture_bind_group_layout.create_bind_group(&context.device, &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: diffuse_buffer.as_entire_binding(),
                        },
                    ]);
                    let requirements = PipelineRequestInfo {
                        color_targets: self.color_targets.as_slice(),
                        depth_target: Some(self.depth_target.clone()),
                        vertex_module: &self.no_texture_shader_module,
                        fragment_module: Some(&self.no_texture_shader_module),
                        fragment_entry: Some("fs_main"),
                        vertex_entry: "vs_main",
                        multisample: &self.multisample,
                        topology: self.topology,
                        vertex_layouts: &[GBufferVertex::layout()],
                        bind_group_layouts: &no_texture_layouts
                    };

                    let pipeline_handle = context.request_pipeline(&requirements);
                    let pipeline = context.get_pipeline(pipeline_handle)
                        .expect("Failed to get texture pipeline");
                    (pipeline, bind_group)
                }
            };

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(2, &bind_group, &[]);

            match &renderable.geometry.mesh {
                Some(mesh_id) => {
                    let mesh = match context.get_resource(&mesh_id).unwrap() {
                        ResourceData::Mesh(mesh) => mesh,
                        _ => continue,
                    };
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

                    match &mesh.index_buffer {
                        Some(index_buffer) => {
                            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                        },
                        None => {
                            render_pass.draw(0..mesh.vertex_count, 0..1);
                        },
                    }
                },
                None => continue,
            }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightPropertiesUniform {
    pub position: [f32; 4],
    pub color: [f32; 4],
}

pub struct LightingPassFrameData<'a> {
    pub view: &'a wgpu::TextureView,
}

pub struct LightingPass {
    name: &'static str,
    kind: RenderPassKind,

    pipeline: wgpu::RenderPipeline,
    gbuffer_textures_bind_group_layout: BindGroupLayout,

    #[allow(unused)]
    camera_bind_group_layout: BindGroupLayout, // Unused for now, but could be useful to have later
    camera_bind_group: Option<wgpu::BindGroup>,

    lights_bind_group_layout: BindGroupLayout,
    lights_storage_buffer_handle: BufferHandle,
    lights_uniform_buffer_handle: BufferHandle,

    gbuffer_textures_bind_group: Option<wgpu::BindGroup>,
    view: Option<wgpu::TextureView>,
    gbuffer_normal_texture_handle: TextureHandle,
    gbuffer_albedo_texture_handle: TextureHandle,
    gbuffer_depth_texture_handle: TextureHandle,

    #[allow(unused)] 
    camera_buffer_handle: BufferHandle, // unused for now, but this might be useful later
}

impl LightingPass {
    pub fn new(
        context: &mut Context,
        gbuffer_normal_texture_handle: TextureHandle,
        gbuffer_albedo_texture_handle: TextureHandle,
        gbuffer_depth_texture_handle: TextureHandle,
        camera_buffer_handle: BufferHandle,
        lights_storage_buffer_handle: BufferHandle,
        lights_uniform_buffer_handle: BufferHandle,
    ) -> Self {
        let gbuffer_textures_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("read_gbuffers_layout"))
            .add_texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false }, false)
            .add_texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false }, false)
            .add_texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false }, false)
            .build_layout();

        let camera_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("lighting_camera"))
            .add_uniform(wgpu::ShaderStages::FRAGMENT)
            .build_layout();

        let lights_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("lighting_lights"))
            .add_uniform(wgpu::ShaderStages::FRAGMENT)
            .add_storage_buffer(wgpu::ShaderStages::FRAGMENT, Some(NonZero::new(32_u64).unwrap()))
            .build_layout();

        let deferred_shader = ShaderBuilder::new(&context.device, include_str!("../../shaders/common/deferred.wgsl").into())
            .vert_entry("vs_main")
            .frag_entry("fs_main")
            .label("deferred_shader")
            .build();
        
        let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                gbuffer_textures_bind_group_layout.layout(),
                camera_bind_group_layout.layout(),
                lights_bind_group_layout.layout(),
            ],
            immediate_size: 0
        });

        let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Lighting Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: deferred_shader.module(),
                entry_point: deferred_shader.vert_entry(),
                buffers: deferred_shader.vertex_buffers(),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: deferred_shader.module(),
                entry_point: deferred_shader.frag_entry(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: context.surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: None,
            /*
            depth_stencil: Some(wgpu::DepthStencilState {
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                format: wgpu::TextureFormat::Depth24Plus,
                bias: wgpu::DepthBiasState::default(),
                stencil: wgpu::StencilState::default(),
            }),
            */
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
        
        let normal_texture_view = context.get_texture_view(gbuffer_normal_texture_handle).expect("Failed to get gbuffer normal texture view");
        let albedo_texture_view = context.get_texture_view(gbuffer_albedo_texture_handle).expect("Failed to get gbuffer albedo texture view");
        let depth_texture_view = context.get_texture_view(gbuffer_depth_texture_handle).expect("Failed to get gbuffer depth texture view");

        let gbuffer_textures_bind_group = gbuffer_textures_bind_group_layout.create_bind_group(&context.device, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(normal_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(albedo_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(depth_texture_view),
            },
        ]);
        
        let camera_buffer = context.get_buffer(camera_buffer_handle)
            .expect("Failed to get camera buffer in lighting pass");

        let camera_bind_group = camera_bind_group_layout.create_bind_group(&context.device, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding()
            }
        ]);

        Self {
            name: "lighting_pass",
            kind: RenderPassKind::Graphics,
            pipeline,
            gbuffer_textures_bind_group_layout,
            camera_bind_group_layout,
            camera_bind_group: Some(camera_bind_group),
            gbuffer_textures_bind_group: Some(gbuffer_textures_bind_group),
            view: None,
            lights_bind_group_layout,
            lights_uniform_buffer_handle,
            lights_storage_buffer_handle,
            gbuffer_normal_texture_handle,
            gbuffer_albedo_texture_handle,
            gbuffer_depth_texture_handle,
            camera_buffer_handle,
        }
    }

    pub fn update_frame_data(&mut self, _context: &Context, frame_data: &LightingPassFrameData) {
        self.view = Some(frame_data.view.clone());
    }
}

impl RenderPassNode for LightingPass {
    fn name(&self) -> &str {
        self.name
    }

    fn kind(&self) -> RenderPassKind {
        self.kind
    }

    fn on_resize(&mut self, context: &Context, _width: u32, _height: u32) {
        let normal_texture_view = context.get_texture_view(self.gbuffer_normal_texture_handle).expect("Failed to get gbuffer normal texture view");
        let albedo_texture_view = context.get_texture_view(self.gbuffer_albedo_texture_handle).expect("Failed to get gbuffer albedo texture view");
        let depth_texture_view = context.get_texture_view(self.gbuffer_depth_texture_handle).expect("Failed to get gbuffer depth texture view");
        
        self.gbuffer_textures_bind_group = Some(self.gbuffer_textures_bind_group_layout.create_bind_group(&context.device, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&normal_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&albedo_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&depth_texture_view),
            },
        ]));
    }

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &mut Context) {
        let view = match &self.view {
            Some(view) => view,
            None => {
                println!("Failed to get view for lighting pass");
                return
            }
        };

        let depth_texture_view = context.get_texture_view(self.gbuffer_depth_texture_handle)
            .expect("Failed to get depth texture view in lighting pass");
        
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(format!("{}_render_pass", self.name).as_str()),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store
                    },
                }),
            ],
            depth_stencil_attachment: None,
            /*
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            */
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        let gbuffer_textures_bind_group = match &self.gbuffer_textures_bind_group {
            Some(group) => group,
            None => panic!("Cannot call execute without udpating frame data")
        };

        let camera_bind_group = match &self.camera_bind_group {
            Some(group) => group,
            None => panic!("Cannot call execute without udpating frame data")
        };

        // TODO: do not do this every frame :/
        let lights_uniform = context.get_buffer(self.lights_uniform_buffer_handle)
            .expect("Failed to get lights uniform buffer");
        let lights_storage = context.get_buffer(self.lights_storage_buffer_handle)
            .expect("Failed to get lights storage buffer");
        let lights_bind_group = self.lights_bind_group_layout.create_bind_group(&context.device, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: lights_uniform.as_entire_binding()
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: lights_storage.as_entire_binding()
            },
        ]);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, gbuffer_textures_bind_group, &[]);
        render_pass.set_bind_group(1, camera_bind_group, &[]);
        render_pass.set_bind_group(2, &lights_bind_group, &[]);
        render_pass.draw(0..6, 0..1); // 6 vertices since this pass in only drawing a quad to the
    }
}

pub struct DebugGridPass {
    name: &'static str,
    kind: RenderPassKind,

    pipeline: wgpu::RenderPipeline,
    camera_bind_group: wgpu::BindGroup,

    depth_texture_handle: TextureHandle,
    view: Option<wgpu::TextureView>,

    #[allow(unused)]
    camera_buffer_handle: BufferHandle, // This is unused for now but might useful later
}

impl DebugGridPass {
    pub fn new(
        context: &mut Context,
        depth_texture_handle: TextureHandle,
        camera_buffer_handle: BufferHandle,
    ) -> Self {
        let camera_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("lighting_camera"))
            .add_uniform(wgpu::ShaderStages::VERTEX)
            .build_layout();

        let shader = ShaderBuilder::new(&context.device, include_str!("../../shaders/common/debug_grid.wgsl").into())
            .vert_entry("vs_main")
            .frag_entry("fs_main")
            .label("debug_grid_shader")
            .build();

        let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                camera_bind_group_layout.layout(),
            ],
            immediate_size: 0
        });
        
        let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Debug Grid Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader.module(),
                entry_point: shader.vert_entry(),
                buffers: shader.vertex_buffers(),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader.module(),
                entry_point: shader.frag_entry(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: context.surface_config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: Some(wgpu::DepthStencilState {
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                format: wgpu::TextureFormat::Depth24Plus,
                bias: wgpu::DepthBiasState::default(),
                stencil: wgpu::StencilState::default(),
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
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None
        });
        
        let camera_buffer = context.get_buffer(camera_buffer_handle)
            .expect("Failed to get camera buffer when creating grid pass");
        let camera_bind_group = camera_bind_group_layout.create_bind_group(&context.device, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding()
            }
        ]);

        Self {
            name: "debug_grid_render_pass",
            kind: RenderPassKind::Graphics,
            pipeline,
            camera_bind_group,
            depth_texture_handle,
            camera_buffer_handle,
            view: None
        }
    }

    pub fn update_frame_data(&mut self, output: wgpu::TextureView) {
        self.view = Some(output);
    }
}

impl RenderPassNode for DebugGridPass {
    fn name(&self) -> &str {
        self.name
    }

    fn kind(&self) -> RenderPassKind {
        self.kind
    }

    fn on_resize(&mut self, _context: &Context, _width: u32, _height: u32) {
        
    }

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &mut Context) {
        let view = match &self.view {
            Some(view) => view,
            None => return,
        };
        
        let depth_texture_view = context.get_texture_view(self.depth_texture_handle)
            .expect("Failed to get depth texture view in debug grid pass");

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(format!("{}_render_pass", self.name).as_str()),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        // load: wgpu::LoadOp::Load,
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0
                        }),
                        store: wgpu::StoreOp::Store
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.draw(0..6, 0..1); // 6 vertices since this pass in only drawing a quad to the
    }
}

pub struct AlphaRenderPass {
    name: &'static str,
    kind: RenderPassKind,

    has_texture_pipeline: wgpu::RenderPipeline,
    no_texture_pipeline: wgpu::RenderPipeline,
    camera_bind_group: wgpu::BindGroup,

    geometry_bind_group_layout: BindGroupLayout,
    material_bind_group_layout: BindGroupLayout,
    no_texture_material_bind_group_layout: BindGroupLayout,

    lights_bind_group_layout: BindGroupLayout,
    lights_storage_buffer_handle: BufferHandle,
    lights_uniform_buffer_handle: BufferHandle,

    depth_texture_handle: TextureHandle,
    

    view: Option<wgpu::TextureView>,

    renderables: Vec<Renderable>,
}

impl AlphaRenderPass {
    pub fn new(
        context: &Context,
        camera_buffer_handle: BufferHandle,
        depth_texture_handle: TextureHandle,
        renderables: Vec<Renderable>,
        lights_storage_buffer_handle: BufferHandle,
        lights_uniform_buffer_handle: BufferHandle,
    ) -> Self {
        let shader = ShaderBuilder::new(&context.device, include_str!("../../shaders/common/alpha.wgsl").into())
            .vert_entry("vs_main")
            .frag_entry("fs_main")
            .label("alpha_shader")
            .add_vertex_layout(GBufferVertex::layout())
            .build();

        let no_texture_shader = ShaderBuilder::new(&context.device, include_str!("../../shaders/common/no-texture-alpha.wgsl").into())
            .vert_entry("vs_main")
            .frag_entry("fs_main")
            .label("alpha_shader")
            .add_vertex_layout(GBufferVertex::layout())
            .build();

        let depth_texture = context.get_texture(depth_texture_handle)
            .expect("Failed to get depth texture in alpha pass");
        
        let scene_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("scene"))
            .add_uniform(wgpu::ShaderStages::VERTEX)
            .build_layout();

        let geometry_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("geometry"))
            .add_uniform(wgpu::ShaderStages::VERTEX)
            .build_layout();

        let material_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("material"))
            .add_texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: true }, false)
            .add_sampler(wgpu::ShaderStages::FRAGMENT, wgpu::SamplerBindingType::Filtering)
            .add_uniform(wgpu::ShaderStages::FRAGMENT)
            .build_layout();
        
        let no_texture_material_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("material"))
            .add_uniform(wgpu::ShaderStages::FRAGMENT)
            .add_uniform(wgpu::ShaderStages::FRAGMENT)
            .build_layout();

        let lights_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("lighting_lights"))
            .add_uniform(wgpu::ShaderStages::FRAGMENT)
            .add_storage_buffer(wgpu::ShaderStages::FRAGMENT, Some(NonZero::new(32_u64).unwrap()))
            .build_layout();
        
        let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GBuffer pipeline layout"),
            bind_group_layouts: &[
                scene_bind_group_layout.layout(),
                geometry_bind_group_layout.layout(),
                material_bind_group_layout.layout(),
                lights_bind_group_layout.layout(),
            ],
            immediate_size: 0,
        });

        // Create the base pipeline builder
        let mut pipeline_builder = PipelineBuilder::new(shader.module(), Some(shader.module()));
        pipeline_builder
            .vert_entry("vs_main")
            .frag_entry("fs_main")
            .set_vertex_layouts(shader.vertex_buffers())
            .add_color_target(wgpu::ColorTargetState {
                format: context.surface_config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL
            })
            .depth_stencil(wgpu::DepthStencilState {
                format: depth_texture.format(),
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
            .topology(wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            })
            .multisample(wgpu::MultisampleState {
                count: 1,
                mask: 0xFFFF_FFFF_FFFF_FFFF_u64, // use all sample mask
                alpha_to_coverage_enabled: false
            });

        let has_texture_layouts = [
            scene_bind_group_layout.layout(),
            geometry_bind_group_layout.layout(),
            material_bind_group_layout.layout(),
            lights_bind_group_layout.layout(),
        ];
        let no_texture_layouts = [
            scene_bind_group_layout.layout(),
            geometry_bind_group_layout.layout(),
            no_texture_material_bind_group_layout.layout(),
            lights_bind_group_layout.layout(),
        ];

        pipeline_builder.set_bind_group_layouts(&has_texture_layouts);
        println!("Build alpha has_texture_pipeline");
        let has_texture_pipeline = pipeline_builder.build(&context.device);

        pipeline_builder
            .vert_module(no_texture_shader.module())
            .frag_module(no_texture_shader.module())
            .set_bind_group_layouts(&no_texture_layouts);
        let no_texture_pipeline = pipeline_builder.build(&context.device);
        println!("Build alpha no_texture_pipeline");

        let camera_buffer = context.get_buffer(camera_buffer_handle)
            .expect("Failed to get camera buffer in lighting pass");

        let camera_bind_group = scene_bind_group_layout.create_bind_group(&context.device, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding()
            }
        ]);

        Self {
            name: "alpha_forward_pass",
            kind: RenderPassKind::Graphics,
            has_texture_pipeline,
            no_texture_pipeline,
            renderables,
            camera_bind_group,
            geometry_bind_group_layout,
            material_bind_group_layout,
            no_texture_material_bind_group_layout,
            depth_texture_handle,
            lights_bind_group_layout,
            lights_storage_buffer_handle,
            lights_uniform_buffer_handle,
            view: None,
        }
    }

    pub fn update_frame_data(&mut self, output: wgpu::TextureView) {
        self.view = Some(output);
    }
}

impl RenderPassNode for AlphaRenderPass {
    fn name(&self) -> &str {
        self.name
    }

    fn kind(&self) -> RenderPassKind {
        self.kind
    }

    fn on_resize(&mut self, context: &Context, width: u32, height: u32) {
        
    }

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &mut Context) {
        let view = match &self.view {
            Some(view) => view,
            None => return,
        };
        
        let depth_texture_view = context.get_texture_view(self.depth_texture_handle)
            .expect("Failed to get depth texture view in debug grid pass");

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(format!("{}_render_pass", self.name).as_str()),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,


            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        // render_pass.set_pipeline(&self.pipeline);
        
        let lights_uniform = context.get_buffer(self.lights_uniform_buffer_handle)
            .expect("Failed to get lights uniform buffer");
        let lights_storage = context.get_buffer(self.lights_storage_buffer_handle)
            .expect("Failed to get lights storage buffer");
        let lights_bind_group = self.lights_bind_group_layout.create_bind_group(&context.device, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: lights_uniform.as_entire_binding()
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: lights_storage.as_entire_binding()
            },
        ]);
        render_pass.set_bind_group(3, &lights_bind_group, &[]);

        for renderable in &self.renderables {
            render_pass.set_bind_group(1, &renderable.geometry.bind_group, &[]);

            let (pipeline, bind_group) = match renderable.material.diffuse {
                DiffuseResource::Texture(texture_handle) => {
                    let material_texture_view = context.get_texture_view(texture_handle)
                        .expect("Failed to get texture view for material diffuse texture");
                    let material_sampler = context.get_sampler(texture_handle)
                        .expect("Failed to get sampler for material diffuse texture");
                    let dissolve_buffer = context.get_buffer(renderable.material.dissolve.unwrap())
                        .expect("Failed to get dissolve buffer for alpha-based renderable");

                    let material_bind_group = self.material_bind_group_layout.create_bind_group(&context.device, &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(material_texture_view)
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(material_sampler)
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Buffer(dissolve_buffer.as_entire_buffer_binding())
                        }
                    ]);

                    (&self.has_texture_pipeline, material_bind_group)
                },
                DiffuseResource::Color(buffer_handle) => {
                    let diffuse_buffer = context.get_buffer(buffer_handle)
                        .expect("Failed to get diffuse buffer for renderable in alpha renderpass");
                    let dissolve_buffer = context.get_buffer(renderable.material.dissolve.unwrap())
                        .expect("Failed to get dissolve buffer for alpha-based renderable");
                    let material_bind_group = self.no_texture_material_bind_group_layout.create_bind_group(&context.device, &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(diffuse_buffer.as_entire_buffer_binding())
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(dissolve_buffer.as_entire_buffer_binding())
                        }
                    ]);

                    (&self.no_texture_pipeline, material_bind_group)
                },
            };
            
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(2, &bind_group, &[]);
            
            match &renderable.geometry.mesh {
                Some(mesh_id) => {
                    let mesh = match context.get_resource(&mesh_id).unwrap() {
                        ResourceData::Mesh(mesh) => mesh,
                        _ => continue,
                    };
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

                    match &mesh.index_buffer {
                        Some(index_buffer) => {
                            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                        },
                        None => {
                            render_pass.draw(0..mesh.vertex_count, 0..1);
                        },
                    }
                },
                None => continue,
            }
        }
    }
}
