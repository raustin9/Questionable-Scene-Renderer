pub struct Texture {
    size: wgpu::Extent3d,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Texture {
    pub fn new(device: &wgpu::Device, name: &str, width: u32, height: u32, usage: wgpu::TextureUsages, format: wgpu::TextureFormat) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size,
            usage,
            dimension: wgpu::TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
            format,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            size,
            texture,
            view,
        }
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.size
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.texture.format()
    }
}
