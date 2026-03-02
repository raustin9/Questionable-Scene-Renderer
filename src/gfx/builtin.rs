use std::num::{NonZero, NonZeroU64};

use crate::{geometry::{GBufferVertex, Vertex}, gfx::{Context, material::{DiffuseResource, MaterialShaderFeatures}, render_graph::{RenderPassContext, RenderPassKind, RenderPassNode}, renderer::Renderable, resource::{BufferHandle, DiffuseColorFeature, DiffuseTextureFeature, PipelineBuilder, PipelineHandle, PipelineRequestInfo, ResourceData, ResourceId, ShaderFeature, TextureHandle, TransparentMaterialFeatureDC, TransparentMaterialFeatureDT}, texture}, shader::{BindGroupLayout, BindGroupLayoutBuilder, ShaderBuilder}};

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
            .add_uniform(wgpu::ShaderStages::VERTEX_FRAGMENT)
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

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &mut RenderPassContext) {
        let normal_texture_view = context.texture_registry.get_view(self.normal_texture_handle).expect("Failed to get normal texture view from context");
        let albedo_texture_view = context.texture_registry.get_view(self.albedo_texture_handle).expect("Failed to get albedo texture view from context");
        let depth_texture_view = context.texture_registry.get_view(self.depth_texture_handle).expect("Failed to get depth texture view from context");

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
        
        render_pass.set_bind_group(0, &self.scene_bind_group, &[]);

        let mut bind_group_layouts: Vec<&wgpu::BindGroupLayout> = vec![];
        for renderable in &self.renderables {
            render_pass.set_bind_group(1, &renderable.geometry.bind_group, &[]);

            let features = renderable.material.diffuse.features(context.shader_registry);
            let shader = context.shader_registry.get_material(&features, &[GBufferVertex::layout()])
                .expect("Failed to get shader for material");
            bind_group_layouts = shader.bind_group_layouts
                .iter()
                .map(|l| l)
                .collect::<Vec<_>>();

            let requirements = PipelineRequestInfo {
                color_targets: self.color_targets.as_slice(),
                depth_target: Some(self.depth_target.clone()),
                vertex_module: &shader.vert_module,
                fragment_module: Some(&shader.frag_module),
                fragment_entry: Some("fs_main"),
                vertex_entry: "vs_main",
                multisample: &self.multisample,
                topology: self.topology,
                vertex_layouts: &[GBufferVertex::layout()],
                bind_group_layouts: &bind_group_layouts.as_slice(),
            };

            let pipeline = context.pipeline_manager.request_pipeline(context.device, &requirements);
            let pipeline = context.pipeline_manager.get_pipeline(pipeline)
                .expect("Failed to get pipeline");

            let bind_group = match &renderable.material.diffuse {
                DiffuseResource::Texture(texture_handle) => {
                    let material_texture_view = context.texture_registry.get_view(texture_handle.clone())
                        .expect("Failed to get texture view for material diffuse texture");
                    let material_sampler = context.texture_registry.get_sampler(texture_handle.clone())
                        .expect("Failed to get sampler for material diffuse texture");

                    let layout = context.device.create_bind_group_layout(&DiffuseTextureFeature::layout_descriptor());
                    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(material_texture_view)
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(material_sampler)
                            },
                        ]
                    })

                }
                DiffuseResource::Color(buffer_handle) => {
                    let buffer = context.buffer_registry.get_buffer(buffer_handle.clone())
                        .expect("Failed to get diffuse color buffer");

                    let layout = context.device.create_bind_group_layout(&DiffuseColorFeature::layout_descriptor());
                    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &layout,
                        entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: buffer.as_entire_binding(),
                        },
                        ]
                    })
                }
            };

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(2, &bind_group, &[]);

            render_pass.set_vertex_buffer(0, renderable.geometry.mesh.vertex_buffer.slice(..));
            match &renderable.geometry.mesh.index_buffer {
                Some(index_buffer) => {
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..renderable.geometry.mesh.index_count, 0, 0..1);
                },
                None => {
                    render_pass.draw(0..renderable.geometry.mesh.vertex_count, 0..1);
                }
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

    color_targets: Vec<wgpu::ColorTargetState>,
    depth_target: Option<wgpu::DepthStencilState>,
    multisample: wgpu::MultisampleState,
    topology: wgpu::PrimitiveState,

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

        let color_targets = vec![
            wgpu::ColorTargetState {
                format: context.surface_config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }
        ];
        let depth_target = None;
        let topology = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        };
        let multisample = wgpu::MultisampleState {
            count: 1,
            mask: 0xFFFF_FFFF_FFFF_FFFF_u64, // use all sample mask
            alpha_to_coverage_enabled: false
        };

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

            color_targets,
            depth_target,
            topology,
            multisample,
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

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &mut RenderPassContext) {
        let view = match &self.view {
            Some(view) => view,
            None => {
                println!("Failed to get view for lighting pass");
                return
            }
        };

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
        let lights_uniform = context.buffer_registry.get_buffer(self.lights_uniform_buffer_handle)
            .expect("Failed to get lights uniform buffer");
        let lights_storage = context.buffer_registry.get_buffer(self.lights_storage_buffer_handle)
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

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &mut RenderPassContext) {
        let view = match &self.view {
            Some(view) => view,
            None => return,
        };
        let depth_texture_view = context.texture_registry.get_view(self.depth_texture_handle)
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

    camera_bind_group: wgpu::BindGroup,

    lights_bind_group_layout: BindGroupLayout,
    lights_storage_buffer_handle: BufferHandle,
    lights_uniform_buffer_handle: BufferHandle,

    depth_texture_handle: TextureHandle,
    
    color_targets: Vec<wgpu::ColorTargetState>,
    depth_target: wgpu::DepthStencilState,
    multisample: wgpu::MultisampleState,
    topology: wgpu::PrimitiveState,

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
        let depth_texture = context.get_texture(depth_texture_handle)
            .expect("Failed to get depth texture in alpha pass");
        
        let scene_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("scene"))
            .add_uniform(wgpu::ShaderStages::VERTEX_FRAGMENT)
            .build_layout();

        let lights_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("lighting_lights"))
            .add_uniform(wgpu::ShaderStages::FRAGMENT)
            .add_storage_buffer(wgpu::ShaderStages::FRAGMENT, None)
            .build_layout();

        let color_targets = vec![
            wgpu::ColorTargetState {
                format: context.surface_config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL
            }
        ];
        let depth_target = wgpu::DepthStencilState {
            format: depth_texture.format(),
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
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
        let multisample = wgpu::MultisampleState {
            count: 1,
            mask: 0xFFFF_FFFF_FFFF_FFFF_u64, // use all sample mask
            alpha_to_coverage_enabled: false
        };

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
            renderables,
            camera_bind_group,
            depth_texture_handle,
            lights_bind_group_layout,
            lights_storage_buffer_handle,
            lights_uniform_buffer_handle,
            view: None,
            color_targets,
            depth_target,
            topology,
            multisample,
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

    fn on_resize(&mut self, _context: &Context, _width: u32, _height: u32) {
        
    }

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &mut RenderPassContext) {
        let view = match &self.view {
            Some(view) => view,
            None => return,
        };
        let depth_texture_view = context.texture_registry.get_view(self.depth_texture_handle)
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
        
        let lights_uniform = context.buffer_registry.get_buffer(self.lights_uniform_buffer_handle)
            .expect("Failed to get lights uniform buffer");
        let lights_storage = context.buffer_registry.get_buffer(self.lights_storage_buffer_handle)
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

        let mut bind_group_layouts: Vec<&wgpu::BindGroupLayout> = vec![];
        for renderable in &self.renderables {
            render_pass.set_bind_group(1, &renderable.geometry.bind_group, &[]);

            let features = renderable.material.features(&context.shader_registry);
            let shader = context.shader_registry.get_material(
                features.as_slice(), 
                &[GBufferVertex::layout()]
            ).expect("Failed to get alpha shader");
            bind_group_layouts = shader.bind_group_layouts
                .iter()
                .map(|l| l)
                .collect::<Vec<_>>();
            
            let requirements = PipelineRequestInfo {
                color_targets: self.color_targets.as_slice(),
                depth_target: Some(self.depth_target.clone()),
                vertex_module: &shader.vert_module,
                fragment_module: Some(&shader.frag_module),
                fragment_entry: Some("fs_main"),
                vertex_entry: "vs_main",
                multisample: &self.multisample,
                topology: self.topology,
                vertex_layouts: &[GBufferVertex::layout()],
                bind_group_layouts: &bind_group_layouts.as_slice(),
            };

            let pipeline = context.pipeline_manager.request_pipeline(context.device, &requirements);
            let pipeline = context.pipeline_manager.get_pipeline(pipeline)
                .expect("Failed to get pipeline");

            let bind_group = match &renderable.material.diffuse {
                DiffuseResource::Texture(texture_handle) => {
                    let material_texture_view = context.texture_registry.get_view(texture_handle.clone())
                        .expect("Failed to get texture view for material diffuse texture");
                    let material_sampler = context.texture_registry.get_sampler(texture_handle.clone())
                        .expect("Failed to get sampler for material diffuse texture");
                    let dissolve_buffer = context.buffer_registry.get_buffer(renderable.material.dissolve.unwrap())
                        .expect("Failed to get dissolve buffer for alpha-based renderable");

                    let layout = context.device.create_bind_group_layout(&TransparentMaterialFeatureDT::layout_descriptor());
                    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &layout,
                        entries: &[
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
                            },
                        ]
                    })

                }
                DiffuseResource::Color(buffer_handle) => {
                    let buffer = context.buffer_registry.get_buffer(buffer_handle.clone())
                        .expect("Failed to get diffuse color buffer");
                    let dissolve_buffer = context.buffer_registry.get_buffer(renderable.material.dissolve.unwrap())
                        .expect("Failed to get dissolve buffer for alpha-based renderable");
                    let layout = context.device.create_bind_group_layout(&TransparentMaterialFeatureDC::layout_descriptor());
                    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Buffer(dissolve_buffer.as_entire_buffer_binding())
                            },
                        ]
                    })
                }
            };
            
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(2, &bind_group, &[]);

            render_pass.set_vertex_buffer(0, renderable.geometry.mesh.vertex_buffer.slice(..));
            match &renderable.geometry.mesh.index_buffer {
                Some(index_buffer) => {
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..renderable.geometry.mesh.index_count, 0, 0..1);
                },
                None => {
                    render_pass.draw(0..renderable.geometry.mesh.vertex_count, 0..1);
                }
            }
        }
    }
}
