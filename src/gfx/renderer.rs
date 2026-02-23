use std::convert::identity;

use cgmath::{Rad, prelude::*};
use wgpu::util::DeviceExt;
use crate::{RotationUnit, Scene, Transform, camera::Camera, gfx::{Context, FrameResource, builtin::{self, LightingPassFrameData, WriteGBuffersPassFrameData}, material::Material, render_graph::RenderPassNode, resource::{self, ResourceData, ResourceId, TextureHandle}}};

pub trait Renderer<'a> {
    fn new(scene: &'a Scene<'a>, context: &mut Context) -> Self;

    fn render(&mut self, context: &Context, frame_resource: &mut FrameResource);

    fn resize(&mut self, context: &Context, width: u32, height: u32);

    fn update(&mut self);
}

pub struct DeferredRenderer<'a> {
    scene: &'a Scene<'a>,
    write_gbuffers_pass: builtin::WriteGBuffersPass,
    lighting_pass: builtin::LightingPass,
    debug_grid_pass: builtin::DebugGridPass,

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
            },
            None
        );
        
        let gbuffer_albedo_texture_handle = context.create_texture(resource::TextureDescriptor {
                label: String::from("albedo_texture"),
                size: resource::TextureSize::Full,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                format: wgpu::TextureFormat::Bgra8Unorm
            },
            None
        );
        
        let gbuffer_depth_texture_handle = context.create_texture(resource::TextureDescriptor {
                label: String::from("depth_texture"), 
                size: resource::TextureSize::Full,
                usage: 
                    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                format: wgpu::TextureFormat::Depth24Plus
            },
            None
        );

        let write_gbuffers_pass = builtin::WriteGBuffersPass::new(
            context, 
            gbuffer_normal_texture_handle,
            gbuffer_albedo_texture_handle,
            gbuffer_depth_texture_handle,
            render_data.opaque_renderables 
        );

        let lighting_pass = builtin::LightingPass::new(
            context,
            gbuffer_normal_texture_handle,
            gbuffer_albedo_texture_handle,
            gbuffer_depth_texture_handle
        );

        let debug_grid_pass = builtin::DebugGridPass::new(context, gbuffer_depth_texture_handle);

        Self {
            scene,
            write_gbuffers_pass,
            lighting_pass,
            debug_grid_pass,
            gbuffer_depth_texture_handle,
            gbuffer_albedo_texture_handle,
            gbuffer_normal_texture_handle,
        }
    }

    fn update(&mut self) {
        
    }

    fn resize(&mut self, context: &Context, width: u32, height: u32) {
        self.write_gbuffers_pass.on_resize(context, width, height);
        self.lighting_pass.on_resize(context, width, height);
        self.debug_grid_pass.on_resize(context, width, height);
    }

    fn render(&mut self, context: &Context, frame_resource: &mut FrameResource) {
        let mut encoder = &mut frame_resource.encoder;
        self.write_gbuffers_pass.set_frame_data(context, &WriteGBuffersPassFrameData {
            camera_buffer: &frame_resource.camera_buffer,
        });
        self.write_gbuffers_pass.execute(&mut encoder, context);

        self.debug_grid_pass.update_frame_data(frame_resource.output_view.clone());
        self.debug_grid_pass.execute(&mut encoder, context);

        self.lighting_pass.update_frame_data(context, &LightingPassFrameData {
            view: &frame_resource.output_view,
        });
        self.lighting_pass.execute(&mut encoder, context);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct RenderableGeometryUniform {
    model_matrix: [[f32; 4]; 4],
    normal_model_matrix: [[f32; 4]; 4],
}

pub struct Renderable {
    pub mesh: Option<ResourceId>,
    pub model_matrix: cgmath::Matrix4<f32>,
    pub normal_model_matrix: cgmath::Matrix4<f32>, 
    pub uniform: wgpu::Buffer,
    pub material: Material
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
            // Get the opaque renderables from the Node.model field
//            match &node.model {
//                Some(model) => {
//                    let mesh_handle = context.create_mesh(
//                        model.name.as_str(), 
//                        model.vertices.len() as u32, 
//                        bytemuck::cast_slice(model.vertices.as_slice()), 
//                        match &model.indices {
//                            None => None,
//                            Some(indices) => Some(indices.as_slice())
//                        }
//                    );
//
//                    // Form the model matrix
//                    let mut model_matrix = cgmath::Matrix4::<f32>::identity();
//                    for transform in &node.transforms {
//                        match transform {
//                            Transform::Scale(scale) => {
//                                let scale_matrix = cgmath::Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
//                                model_matrix = model_matrix * scale_matrix;
//                            },
//                            Transform::Rotate(axis, unit) => {
//                                let _rotation = cgmath::Matrix4::from_axis_angle(
//                                    cgmath::Vector3 { x: axis[0], y: axis[1], z: axis[2] }, 
//                                    match unit {
//                                        RotationUnit::Rad(scalar) => cgmath::Rad(scalar.clone()),
//                                        RotationUnit::Deg(scalar) => cgmath::Rad(scalar * 0.0174533),
//                                    }
//                                );
//                            },
//                            Transform::Translate(translate) => {
//                                let translation_matrix = cgmath::Matrix4::<f32>::from_translation(cgmath::Vector3 { x: translate[0], y: translate[1], z: translate[2] });
//                                model_matrix = model_matrix * translation_matrix;
//                            }
//                        }
//                    }
//
//                    let inverse_model = model_matrix.invert().expect("Failed to invert model matrix");
//                    let inverse_transpose_model = inverse_model.transpose();
//
//                    let uniform_data = RenderableGeometryUniform {
//                        model_matrix: model_matrix.into(),
//                        normal_model_matrix: inverse_transpose_model.into()
//                    };
//
//                    let uniform = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                        label: Some("geometry_uniform"),
//                        contents: bytemuck::cast_slice(&[uniform_data]),
//                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
//                    });
//
//                    let material = Material::from_path(node.material_path, context);
//
//                    render_data.opaque_renderables.push(Renderable { 
//                        mesh: Some(mesh_handle), 
//                        model_matrix, 
//                        normal_model_matrix: inverse_transpose_model,
//                        uniform,
//                        material,
//                    });
//                },
//                None => {},
//            }

            // Get the models from the Node.objs
            match &node.objs {
                Some(models) => {
                    for model in models {
                        let mesh_handle = context.create_mesh(
                            model.mesh.name.as_str(), 
                            model.mesh.vertices.len() as u32, 
                            bytemuck::cast_slice(model.mesh.vertices.as_slice()), 
                            match &model.mesh.indices {
                                None => None,
                                Some(indices) => Some(indices.as_slice())
                            }
                        );

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
                                    let translation_matrix = cgmath::Matrix4::<f32>::from_translation(cgmath::Vector3 { x: translate[0], y: translate[1], z: translate[2] });
                                    model_matrix = model_matrix * translation_matrix;
                                }
                            }
                        }

                        let inverse_model = model_matrix.invert().expect("Failed to invert model matrix");
                        let inverse_transpose_model = inverse_model.transpose();

                        let uniform_data = RenderableGeometryUniform {
                            model_matrix: model_matrix.into(),
                            normal_model_matrix: inverse_transpose_model.into()
                        };

                        let uniform = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("geometry_uniform"),
                            contents: bytemuck::cast_slice(&[uniform_data]),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
                        });

                        let default_material = Material::from_path(&node.material_path, context);
                        let material = match &model.material {
                            Some(material) => {
                                match &material.diffuse_texture {
                                    Some(diffuse_data) => {
                                        Material::from_image_data(&diffuse_data, context)
                                    },
                                    None => default_material,
                                }
                            },
                            None => default_material,
                        };

                        // let material = Material::from_path(node.material_path, context);

                        render_data.opaque_renderables.push(Renderable { 
                            mesh: Some(mesh_handle), 
                            model_matrix, 
                            normal_model_matrix: inverse_transpose_model,
                            uniform,
                            material,
                        });
                    }
                },
                None => {},
            }
        }

        render_data
    }
}
