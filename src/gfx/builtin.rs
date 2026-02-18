use std::collections::HashMap;

use crate::{geometry::{GBufferVertex, Vertex}, gfx::{Context, render_graph::{RenderPassKind, RenderPassNode}, renderer::Renderable, resource::{ResourceData, ResourceId}, texture}, shader::{BindGroupLayout, BindGroupLayoutBuilder, ShaderBuilder}};

pub struct WriteGBuffersPassFrameData<'a> {
    pub world_buffer: &'a wgpu::Buffer,
    pub camera_buffer: &'a wgpu::Buffer,
}

// A builtin renderpass for writing geometries to the gbuffer textures
pub struct WriteGBuffersPass {
    name: &'static str,
    kind: RenderPassKind,

    pipeline: wgpu::RenderPipeline,

    scene_bind_group_layout: BindGroupLayout,
    scene_bind_group: Option<wgpu::BindGroup>,

    /* Textures */
    normal_texture_handle: ResourceId,
    albedo_texture_handle: ResourceId,
    depth_texture_handle: ResourceId,

    renderables: Vec<Renderable>,
}

impl WriteGBuffersPass {
    pub fn new(
        context: &mut Context,
        normal_texture_handle: ResourceId,
        albedo_texture_handle: ResourceId,
        depth_texture_handle: ResourceId,
        renderables: Vec<Renderable>
    ) -> Self {
        let normal_texture = match context.get_resource(&normal_texture_handle).unwrap() {
            ResourceData::Texture(texture) => texture,
            _ => panic!("Got normal texture as different resource!"),
        };

        let albedo_texture = match context.get_resource(&albedo_texture_handle).unwrap() {
            ResourceData::Texture(texture) => texture,
            _ => panic!("Got albedo texture as different resource!"),
        };
        
        let depth_texture = match context.get_resource(&depth_texture_handle).unwrap() {
            ResourceData::Texture(texture) => texture,
            _ => panic!("Got depth texture as different resource!"),
        };

        let gbuffer_shader = ShaderBuilder::new(&context.device, include_str!("../../shaders/common/gbuffer.wgsl").into())
            .vert_entry("vs_main")
            .frag_entry("fs_main")
            .label("shader")
            .add_vertex_layout(GBufferVertex::layout())
            .build();

        let scene_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("scene"))
            .add_uniform(wgpu::ShaderStages::VERTEX)
            .add_uniform(wgpu::ShaderStages::VERTEX)
            .build_layout();

        let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GBuffer pipeline layout"),
            bind_group_layouts: &[
                scene_bind_group_layout.layout()
            ],
            immediate_size: 0,
        });

        let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GBuffer Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &gbuffer_shader.module(),
                entry_point: gbuffer_shader.vert_entry(),
                buffers: gbuffer_shader.vertex_buffers(),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &gbuffer_shader.module(),
                entry_point: gbuffer_shader.frag_entry(),
                targets: &[
                    // Normal 
                    Some(wgpu::ColorTargetState {
                        format: normal_texture.format(),
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

        Self {
            name: "write_gbuffers_pass",
            kind: RenderPassKind::Graphics,
            pipeline,
            normal_texture_handle,
            albedo_texture_handle,
            depth_texture_handle,
            scene_bind_group_layout,
            scene_bind_group: None,
            renderables
        }
    }

    pub fn set_frame_data(&mut self, context: &Context, frame_data: &WriteGBuffersPassFrameData) {
        self.scene_bind_group = Some(self.scene_bind_group_layout.create_bind_group(&context.device, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: frame_data.world_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: frame_data.camera_buffer.as_entire_binding(),
            },
        ]));
    }
}

impl<'a> RenderPassNode for WriteGBuffersPass {
    fn name(&self) -> &str {
        self.name
    }

    fn kind(&self) -> RenderPassKind {
        self.kind
    }

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &Context) {
        let normal_texture = match context.get_resource(&self.normal_texture_handle).unwrap() {
            ResourceData::Texture(texture) => texture,
            _ => panic!("Got normal texture as different resource!"),
        };

        let albedo_texture = match context.get_resource(&self.albedo_texture_handle).unwrap() {
            ResourceData::Texture(texture) => texture,
            _ => panic!("Got albedo texture as different resource!"),
        };
        
        let depth_texture = match context.get_resource(&self.depth_texture_handle).unwrap() {
            ResourceData::Texture(texture) => texture,
            _ => panic!("Got depth texture as different resource!"),
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(self.name),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &normal_texture.view(),
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
                    view: &albedo_texture.view(),
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
                view: &depth_texture.view(),
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

        let scene_bind_group = match &self.scene_bind_group {
            Some(group) => group,
            None => return
        };
        render_pass.set_bind_group(0, scene_bind_group, &[]);
        render_pass.set_pipeline(&self.pipeline);

        for renderable in &self.renderables {
            match &renderable.mesh {
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

pub struct LightingPassFrameData<'a> {
    pub view: &'a wgpu::TextureView,
    pub camera_buffer: &'a wgpu::Buffer,
    pub normal_texture: &'a texture::Texture,
    pub albedo_texture: &'a texture::Texture,
    pub depth_texture: &'a texture::Texture,
}

pub struct LightingPass {
    name: &'static str,
    kind: RenderPassKind,

    pipeline: wgpu::RenderPipeline,
    gbuffer_textures_bind_group_layout: BindGroupLayout,
    camera_bind_group_layout: BindGroupLayout,
    gbuffer_textures_bind_group: Option<wgpu::BindGroup>,
    camera_bind_group: Option<wgpu::BindGroup>,
    view: Option<wgpu::TextureView>,
}

impl LightingPass {
    pub fn new(
        context: &mut Context,
    ) -> Self {
        let gbuffer_textures_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("read_gbuffers_layout"))
            .add_texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false }, false)
            .add_texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false }, false)
            .add_texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false }, false)
            .build_layout();

        let camera_bind_group_layout = BindGroupLayoutBuilder::new(&context.device, Some("lighting_camera"))
            .add_uniform(wgpu::ShaderStages::FRAGMENT)
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
            name: "lighting_pass",
            kind: RenderPassKind::Graphics,
            pipeline,
            gbuffer_textures_bind_group_layout,
            camera_bind_group_layout,
            camera_bind_group: None,
            gbuffer_textures_bind_group: None,
            view: None,
        }
    }

    pub fn update_frame_data(&mut self, context: &Context, frame_data: &LightingPassFrameData) {
        self.view = Some(frame_data.view.clone());
        
        self.gbuffer_textures_bind_group = Some(self.gbuffer_textures_bind_group_layout.create_bind_group(&context.device, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&frame_data.normal_texture.view()),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&frame_data.albedo_texture.view()),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&frame_data.depth_texture.view()),
            },
        ]));

        self.camera_bind_group = Some(self.camera_bind_group_layout.create_bind_group(&context.device, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: frame_data.camera_buffer.as_entire_binding()
            }
        ]));
    }
}

impl RenderPassNode for LightingPass {
    fn name(&self) -> &str {
        self.name
    }

    fn kind(&self) -> RenderPassKind {
        self.kind
    }

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, _context: &Context) {
        let view = match &self.view {
            Some(view) => view,
            None => return
        };
        
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(format!("{}_render_pass", self.name).as_str()),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
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

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, gbuffer_textures_bind_group, &[]);
        render_pass.set_bind_group(1, camera_bind_group, &[]);
        render_pass.draw(0..6, 0..1); // 6 vertices since this pass in only drawing a quad to the
    }
}

pub fn write_gbuffers_pass_record_commands(
    encoder: &mut wgpu::CommandEncoder, 
    pipeline: &wgpu::RenderPipeline,
    normal_texture: &texture::Texture,
    albedo_texture: &texture::Texture,
    depth_texture: &texture::Texture,
    scene_uniform_bind_group: &wgpu::BindGroup,
    vertex_buffer: &wgpu::Buffer,
    num_vertices: u32,
) {
    let mut gbuffer_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("GBuffer pass"),
        color_attachments: &[
            Some(wgpu::RenderPassColorAttachment {
                view: &normal_texture.view(),
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
                view: &albedo_texture.view(),
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
            view: &depth_texture.view(),
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

    gbuffer_pass.set_pipeline(&pipeline);
    gbuffer_pass.set_bind_group(0, scene_uniform_bind_group, &[]);
    gbuffer_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    gbuffer_pass.draw(0..num_vertices, 0..1);
}

pub fn deferred_pass_record_commands(
    encoder: &mut wgpu::CommandEncoder, 
    pipeline: &wgpu::RenderPipeline,
    gbuffer_textures_bind_group: &wgpu::BindGroup,
    camera_bind_group: &wgpu::BindGroup,
    view: &wgpu::TextureView,
) {
    let mut deferred_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("deferred_pass"),
        color_attachments: &[
            Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
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
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });
    deferred_pass.set_pipeline(&pipeline);
    deferred_pass.set_bind_group(0, gbuffer_textures_bind_group, &[]);
    deferred_pass.set_bind_group(1, camera_bind_group, &[]);
    deferred_pass.draw(0..6, 0..1); // 6 vertices since this pass in only drawing a quad to the
                                    // screen
}
