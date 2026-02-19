use std::convert::identity;

use cgmath::{Rad, prelude::*};
use crate::{RotationUnit, Scene, Transform, camera::Camera, gfx::{Context, FrameResource, builtin::{self, LightingPassFrameData, WriteGBuffersPassFrameData}, render_graph::RenderPassNode, resource::{self, ResourceData, ResourceId, TextureHandle}}};

pub trait Renderer<'a> {
    fn new(scene: &'a Scene<'a>, context: &mut Context) -> Self;

    fn render(&mut self, context: &Context, frame_resource: &mut FrameResource);
}

pub struct DeferredRenderer<'a> {
    scene: &'a Scene<'a>,
    write_gbuffers_pass: builtin::WriteGBuffersPass,
    lighting_pass: builtin::LightingPass,

    gbuffer_normal_texture_handle: TextureHandle,
    gbuffer_albedo_texture_handle: TextureHandle,
    gbuffer_depth_texture_handle: TextureHandle,
}

impl<'a> Renderer<'a> for DeferredRenderer<'a> {
    fn new(scene: &'a Scene<'a>, context: &mut Context) -> Self {
        let render_data = RenderData::new(scene, context);
        
        let gbuffer_normal_texture_handle = context.create_texture(resource::TextureDescriptor {
                label: String::from("normal_texture"),
                size: resource::TextureSize::Full,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                format: wgpu::TextureFormat::Rgba16Float
            }
        );
        
        let gbuffer_albedo_texture_handle = context.create_texture(resource::TextureDescriptor {
                label: String::from("albedo_texture"),
                size: resource::TextureSize::Full,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                format: wgpu::TextureFormat::Bgra8Unorm
            }
        );
        
        let gbuffer_depth_texture_handle = context.create_texture(resource::TextureDescriptor {
            label: String::from("depth_texture"), 
            size: resource::TextureSize::Full,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
            format: wgpu::TextureFormat::Depth24Plus
        });

        let write_gbuffers_pass = builtin::WriteGBuffersPass::new(
            context, 
            gbuffer_normal_texture_handle,
            gbuffer_albedo_texture_handle,
            gbuffer_depth_texture_handle,
            render_data.opaque_renderables 
        );

        let lighting_pass = builtin::LightingPass::new(context);

        Self {
            scene,
            write_gbuffers_pass,
            lighting_pass,
            gbuffer_depth_texture_handle,
            gbuffer_albedo_texture_handle,
            gbuffer_normal_texture_handle,
        }
    }

    fn render(&mut self, context: &Context, frame_resource: &mut FrameResource) {
        let mut encoder = &mut frame_resource.encoder;
        self.write_gbuffers_pass.set_frame_data(context, &WriteGBuffersPassFrameData {
            world_buffer: &frame_resource.world_buffer,
            camera_buffer: &frame_resource.camera_buffer,
        });
        self.write_gbuffers_pass.execute(&mut encoder, context);

        let normal_texture_view = context.get_texture_view(self.gbuffer_normal_texture_handle).expect("Failed to get gbuffer normal texture view");
        let albedo_texture_view = context.get_texture_view(self.gbuffer_albedo_texture_handle).expect("Failed to get gbuffer albedo texture view");
        let depth_texture_view = context.get_texture_view(self.gbuffer_depth_texture_handle).expect("Failed to get gbuffer depth texture view");

        self.lighting_pass.update_frame_data(context, &LightingPassFrameData {
            camera_buffer: &frame_resource.camera_buffer,
            view: &frame_resource.output_view,
            normal_texture_view,
            albedo_texture_view,
            depth_texture_view
        });

        self.lighting_pass.execute(&mut encoder, context);
    }
}

pub struct Renderable {
    pub mesh: Option<ResourceId>,
    pub model_matrix: cgmath::Matrix4<f32>,
}

pub struct RenderData {
    pub camera: Camera,
    pub opaque_renderables: Vec<Renderable>
}

impl<'a> RenderData {
    pub fn new(scene: &Scene<'a>, context: &mut Context) -> Self {
        let mut render_data = RenderData {
            camera: Camera::default(),
            opaque_renderables: vec![]
        };

        // TODO: find camera here

        for node in &scene.nodes {
            let mesh_handle = match &node.geometry {
                Some(geometry) => {
                    context.create_mesh(
                        geometry.name, 
                        geometry.vertices.len() as u32, 
                        bytemuck::cast_slice(geometry.vertices.as_slice()), 
                        match &geometry.indices {
                            None => None,
                            Some(indices) => Some(indices.as_slice())
                        }
                    )
                },
                None => continue,
            };

            // Form the model matrix
            let mut model_matrix = cgmath::Matrix4::<f32>::identity();
            for transform in &node.transforms {
                match transform {
                    Transform::Scale(scale) => {
                        let scale_matrix = cgmath::Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
                        model_matrix = model_matrix * scale_matrix;
                    },
                    Transform::Rotate(axis, unit) => {
                        let _rotation = cgmath::Matrix4::from_axis_angle(
                            cgmath::Vector3 { x: axis[0], y: axis[1], z: axis[2] }, 
                            match unit {
                                RotationUnit::Rad(scalar) => cgmath::Rad(scalar.clone()),
                                RotationUnit::Deg(scalar) => cgmath::Rad(scalar * 0.0174533),
                            }
                        );
                    },
                    Transform::Translate(translate) => {
                        let translation_matirx = cgmath::Matrix4::<f32>::from_translation(cgmath::Vector3 { x: translate[0], y: translate[1], z: translate[2] });
                        model_matrix = model_matrix * translation_matirx;
                    }
                }
            }

            render_data.opaque_renderables.push(Renderable { mesh: Some(mesh_handle), model_matrix });
        }

        render_data
    }
}
