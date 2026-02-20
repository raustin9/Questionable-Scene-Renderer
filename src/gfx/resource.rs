use std::collections::HashMap;

use crate::{Texture, shader::UniformBuffer, geometry::Mesh};

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct ResourceId(u64);

impl ResourceId {
    pub fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);

        ResourceId(
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
    }
}

pub enum ResourceKind {
    Texture,
    UniformBuffer,
    Mesh,
}

pub enum ResourceAccess {
    Read,
    ReadWrite,
    Write
}

pub enum ResourceData {
    Texture(Texture),
    UniformBuffer(UniformBuffer),
    Mesh(Mesh),
}

pub struct ResourceHandle<'a> {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub resource: &'a ResourceData,
}

pub enum TextureSize {
    /// Set the texture to a fixed size that will not be updated 
    /// corresponding to screen resizes
    Fixed(u32, u32),

    /// Set the texture size to fill the screen.
    /// When the `TextureRegistry` is resized to 
    /// a new size, textures with this size will
    /// resize accordingly.
    Full,
}

impl TextureSize {
    /// Return whether or not a texture's size is relative to the screen.
    /// `Fixed` textures do not.
    pub fn screen_relative(&self) -> bool {
        match self {
            Self::Fixed(_, _) => false,
            Self::Full => true,
        }
    }
}

pub enum SamplerRepeat {
    Repeat,
    Clamp,
    MirrorRepeat,
    Border(wgpu::Color)
}

impl From<SamplerRepeat> for wgpu::AddressMode {
    fn from(value: SamplerRepeat) -> Self {
        match value {
            SamplerRepeat::Clamp => wgpu::AddressMode::ClampToEdge,
            SamplerRepeat::Repeat => wgpu::AddressMode::Repeat,
            SamplerRepeat::Border(_color) => wgpu::AddressMode::ClampToBorder,
            SamplerRepeat::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
        }
    }
}

pub struct SamplerDescriptor {
    pub address_mode: SamplerRepeat,
}

pub struct TextureDescriptor {
    pub label: String,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
    pub size: TextureSize,
}

/// Resource for a texture to be used in the renderer
pub struct TextureResource {
    /// The wgpu texture
    pub texture: wgpu::Texture,

    /// The wpgu texture view
    pub view: wgpu::TextureView,

    pub sampler: Option<wgpu::Sampler>,

    /// Description of the texture used to create it.
    /// Can be used to find other relevant information 
    /// about the texture like size, format, etc.
    pub descriptor: TextureDescriptor
}

/// Handle to a `TextureResource` held in the `TextureRegistry`
pub type TextureHandle = ResourceId;

/// Registry and manager of texture resources.
/// Resources can be retrieved with a `TextureHandle`
pub struct TextureRegistry {
    /// Storage for the textures
    textures: HashMap<TextureHandle, TextureResource>,

    /// The current width to keep the textures
    current_width: u32,

    /// The current height to keep the textures
    current_height: u32,
}

impl TextureRegistry {
    pub fn new(current_width: u32, current_height: u32) -> Self {
        Self {
            textures: HashMap::new(),
            current_width,
            current_height,
        }
    }

    /// Create and store a texture resource.
    /// Returns a handle to the resource to be retrieved
    pub fn create_texture(&mut self, device: &wgpu::Device, descriptor: TextureDescriptor, sampler_options: Option<SamplerDescriptor>) -> TextureHandle {
        let handle = TextureHandle::new();

        let (width, height) = self.resolve_size(&descriptor.size);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&descriptor.label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1
            },
            usage: descriptor.usage,
            dimension: wgpu::TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
            format: descriptor.format
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            ..Default::default()
        });

        let sampler = match sampler_options {
            None => None,
            Some(options) => {
                let address_mode = options.address_mode.into();
                
                Some(device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("sampler"),
                    address_mode_u: address_mode,
                    address_mode_v: address_mode,
                    address_mode_w: address_mode,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                    ..Default::default()
                }))
            }
        };

        let resource = TextureResource {
            texture,
            view,
            sampler,
            descriptor
        };

        self.textures.insert(handle, resource);

        handle
    }

    /// The `wgpu::TexureView` for the texture resource corresponding to the input handle
    pub fn get_view(&self, handle: TextureHandle) -> Option<&wgpu::TextureView> {
        self.textures.get(&handle).map(|t| &t.view)
    }

    /// The `wgpu::Texure` for the texture resource corresponding to the input handle
    pub fn get_texture(&self, handle: TextureHandle) -> Option<&wgpu::Texture> {
        self.textures.get(&handle).map(|t| &t.texture)
    }

    pub fn get_sampler(&self, handle: TextureHandle) -> Option<&wgpu::Sampler> {
        match self.textures.get(&handle) {
            Some(texture) => {
                match &texture.sampler {
                    Some(sampler) => Some(sampler),
                    None => None,
                }
            },
            None => None
        }
    }

    /// Get the resolved width and height from a `TextureSize` enum value
    pub fn resolve_size(&self, size: &TextureSize) -> (u32, u32) {
        match size {
            TextureSize::Fixed(width, height) => (*width, *height),
            TextureSize::Full => (self.current_width, self.current_height)
        }
    }

    /// Set the texture registry to a new size (normally to adjust to updated screen size)
    /// and update all the screen-relative textures to fit this new size
    pub fn resize_textures(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.current_width = width;
        self.current_height = height;

        let to_update = self.textures
            .iter()
            .filter(|(_, resource)| resource.descriptor.size.screen_relative())
            .map(|(handle, resource)| {
                let (w, h) = self.resolve_size(&resource.descriptor.size);
                (*handle, w, h)
            })
        .collect::<Vec<_>>();

        for (handle, width, height) in to_update {
            if let Some(resource) = self.textures.get_mut(&handle) {
                let updated_texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(&resource.descriptor.label),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    format: resource.descriptor.format,
                    usage: resource.descriptor.usage,
                    dimension: wgpu::TextureDimension::D2,
                    view_formats: &[],
                });

                let updated_view = updated_texture.create_view(&wgpu::TextureViewDescriptor::default());

                resource.texture = updated_texture;
                resource.view = updated_view;
            }
        }
    }
}
