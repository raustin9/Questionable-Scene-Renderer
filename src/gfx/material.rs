
use std::fs;

use crate::gfx::{Context, resource::{self, BufferHandle, SamplerRepeat, TextureHandle}};

#[derive(Debug)]
pub struct MaterialInfo {
    pub diffuse_texture: Option<image::DynamicImage>,
    pub ambient_texture: Option<image::DynamicImage>,
    pub specular_texture: Option<image::DynamicImage>,
    pub normal_texture: Option<image::DynamicImage>,
    pub shininess_texture: Option<image::DynamicImage>,
    pub dissolve_texture: Option<image::DynamicImage>,
    
    pub illumination_model: Option<u8>,

    pub optical_density: Option<f32>,

    pub ambient_color: Option<[f32; 3]>,
    pub diffuse_color: Option<[f32; 3]>,
    pub specular_color: Option<[f32; 3]>,
    pub shininess_coef: Option<f32>,
    pub dissolve_coef: Option<f32>,
}

pub struct Material {
    /// The base diffuse texture for a material
    pub diffuse_texture_handle: TextureHandle,

    pub diffuse_color: [f32; 3],

    pub dissolve: Option<BufferHandle>,
}

impl Material {
    pub fn from_path(file_path: &str, context: &mut Context) -> Self {
        let bytes = fs::read(file_path).expect("Failed to read material file path");

        Self::from_bytes(&bytes, context)
    }

    pub fn from_bytes(bytes: &[u8], context: &mut Context) -> Self {
        let diffuse_image = image::load_from_memory(bytes).expect("Failed to read image from bytes");
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();
        
        let diffuse_texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1
        };

        let texture_handle = context.create_texture(
            resource::TextureDescriptor {
                label: String::from("material_texture"),
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                size: resource::TextureSize::Fixed(diffuse_texture_size.width, diffuse_texture_size.height) 
            }, 
            Some(resource::SamplerDescriptor {
                address_mode: SamplerRepeat::Repeat
            })
        );

        context.write_texture(texture_handle, &diffuse_rgba, diffuse_texture_size, 4);

        Self { 
            diffuse_texture_handle: texture_handle,
            diffuse_color: [1.0, 1.0, 1.0],
            dissolve: None
        }
    }

    pub fn get_handle(image: &image::DynamicImage, context: &mut Context) -> TextureHandle {
        use image::GenericImageView;
        let rgba = image.to_rgba8();
        let dimensions = image.dimensions();
        
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1
        };

        let texture_handle = context.create_texture(
            resource::TextureDescriptor {
                label: String::from("material_texture"),
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                size: resource::TextureSize::Fixed(texture_size.width, texture_size.height) 
            }, 
            Some(resource::SamplerDescriptor {
                address_mode: SamplerRepeat::Repeat
            })
        );

        context.write_texture(texture_handle, &rgba, texture_size, 4);

        texture_handle
    }

    pub fn from_info(info: &MaterialInfo, context: &mut Context) -> Self {
        let diffuse_texture_handle = match &info.diffuse_texture {
            Some(data) => Self::get_handle(&data, context),
            None => panic!("Cannot create a material with no diffuse texture yet"),
        };
        let diffuse_color = match info.diffuse_color {
            Some(color) => color,
            None => [1.0, 1.0, 1.0]
        };

        let dissolve = match info.dissolve_coef {
            Some(coef) => {
                match coef {
                    1.0 => None,
                    _ => Some(context.create_buffer(
                        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, 
                        bytemuck::cast_slice(&[coef])
                    ))
                }
            },
            None => None,
        };

        Self {
            diffuse_texture_handle,
            diffuse_color,
            dissolve,
        }
    }
}
