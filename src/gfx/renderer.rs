use std::sync::Arc;

use crate::{Scene, camera::Camera, geometry::{self, Mesh}, gfx::{Context, FrameResource, builtin::{self, LightingPassFrameData, WriteGBuffersPassFrameData}, render_graph::RenderPassNode, resource::{ResourceData, ResourceHandle, ResourceId}}};

pub trait Renderer<'a> {
    fn new(scene: &'a Scene<'a>, context: &mut Context) -> Self;

    fn render(&mut self, context: &Context, frame_resource: &mut FrameResource);
}

pub struct DeferredRenderer<'a> {
    scene: &'a Scene<'a>,
    write_gbuffers_pass: builtin::WriteGBuffersPass,
    lighting_pass: builtin::LightingPass,

    gbuffer_normal_texture_handle: ResourceId,
    gbuffer_albedo_texture_handle: ResourceId,
    gbuffer_depth_texture_handle: ResourceId,
}

impl<'a> Renderer<'a> for DeferredRenderer<'a> {
    fn new(scene: &'a Scene<'a>, context: &mut Context) -> Self {
        let render_data = RenderData::new(scene, context);
        
        let gbuffer_normal_texture_handle = context.create_texture(
            "normal_texture", 
            wgpu::Extent3d {
                width: context.surface_config.width,
                height: context.surface_config.height,
                depth_or_array_layers: 1
            }, 
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
            wgpu::TextureFormat::Rgba16Float
        );
        
        let gbuffer_albedo_texture_handle = context.create_texture(
            "albedo_texture", 
            wgpu::Extent3d {
                width: context.surface_config.width,
                height: context.surface_config.height,
                depth_or_array_layers: 1
            }, 
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
            wgpu::TextureFormat::Bgra8Unorm
        );
        
        let gbuffer_depth_texture_handle = context.create_texture(
            "depth_texture", 
            wgpu::Extent3d {
                width: context.surface_config.width,
                height: context.surface_config.height,
                depth_or_array_layers: 1
            }, 
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
            wgpu::TextureFormat::Depth24Plus
        );

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

        let normal_texture = match context.get_resource(&self.gbuffer_normal_texture_handle).unwrap() {
            ResourceData::Texture(texture) => texture,
            _ => panic!("Got normal texture as different resource!"),
        };

        let albedo_texture = match context.get_resource(&self.gbuffer_albedo_texture_handle).unwrap() {
            ResourceData::Texture(texture) => texture,
            _ => panic!("Got albedo texture as different resource!"),
        };
        
        let depth_texture = match context.get_resource(&self.gbuffer_depth_texture_handle).unwrap() {
            ResourceData::Texture(texture) => texture,
            _ => panic!("Got depth texture as different resource!"),
        };

        self.lighting_pass.update_frame_data(context, &LightingPassFrameData {
            camera_buffer: &frame_resource.camera_buffer,
            view: &frame_resource.output_view,
            normal_texture,
            albedo_texture,
            depth_texture
        });

        self.lighting_pass.execute(&mut encoder, context);
    }
}

pub struct Renderable {
    pub mesh: Option<ResourceId>
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

            render_data.opaque_renderables.push(Renderable { mesh: Some(mesh_handle) });
        }

        render_data
    }
}
