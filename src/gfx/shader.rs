use std::{borrow::Cow, error::Error, fs};

use wgpu::util::DeviceExt;

// TODO: use the builder pattern for this
pub struct Shader<'a> {
    module: wgpu::ShaderModule,
    vert_entry: Option<&'a str>,
    frag_entry: Option<&'a str>,
    vertex_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    
}

impl<'a> Shader<'a> {
    pub fn from_source(
        device: &wgpu::Device, 
        source: Cow<'a, str>, 
        vert_entry: Option<&'a str>, 
        frag_entry: Option<&'a str>, 
        label: Option<&'a str>,
        vertex_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    ) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(source.into())
        });

        Self {
            module,
            vert_entry,
            frag_entry,
            vertex_layouts,
        }
    }

    pub fn from_path(
        device: &wgpu::Device, 
        file_path: &'a str, 
        vert_entry: Option<&'a str>, 
        frag_entry: Option<&'a str>, 
        label: Option<&'a str>,
        vertex_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    ) -> Result<Self, Box<dyn Error>> {
        let source = fs::read_to_string(file_path)?;
        
        Ok(
            Self::from_source(
                device, 
                source.into(), 
                vert_entry, 
                frag_entry, 
                label,
                vertex_layouts
            )
        )
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    pub fn vert_entry(&self) -> Option<&'a str> {
        self.vert_entry
    }

    pub fn frag_entry(&self) -> Option<&'a str> {
        self.frag_entry
    }

    pub fn vertex_buffers(&self) -> &[wgpu::VertexBufferLayout<'static>] {
        &self.vertex_layouts
    }
}

pub struct ShaderBuilder<'a> {
    device: &'a wgpu::Device,
    source: Cow<'a, str>,
    vert_entry: Option<&'a str>,
    frag_entry: Option<&'a str>,
    label: Option<&'a str>,
    bind_group_layouts: Vec<GroupBuilder<'a>>,
    vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
}

impl<'a> ShaderBuilder<'a> {
    pub fn new(device: &'a wgpu::Device, source: Cow<'a, str>) -> Self {
        Self {
            device,
            source,
            vert_entry: None,
            frag_entry: None,
            label: None,
            bind_group_layouts: vec![],
            vertex_buffer_layouts: vec![],
        }
    }

    pub fn vert_entry(&mut self, entry: &'a str) -> &mut Self {
        self.vert_entry = Some(entry);
        self
    }

    pub fn frag_entry(&mut self, entry: &'a str) -> &mut Self {
        self.frag_entry = Some(entry);
        self
    }

    pub fn label(&mut self, label: &'a str) -> &mut Self {
        self.label = Some(label);
        self
    }

    pub fn add_vertex_layout(&mut self, layout: wgpu::VertexBufferLayout<'static>) -> &mut Self {
        self.vertex_buffer_layouts.push(layout);
        self
    }

    pub fn build(&self) -> Shader<'a> {
        Shader::from_source(
            &self.device, 
            self.source.clone(), 
            self.vert_entry, 
            self.frag_entry, 
            self.label,
            self.vertex_buffer_layouts.clone(),
        )
    }
}

struct Uniform {
    buffer: wgpu::Buffer,
    usages: wgpu::BufferUsages
}

impl Uniform {
    pub fn new(device: &wgpu::Device, usages: wgpu::BufferUsages, data: &[u8]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform_buffer"),
            contents: data,
            usage: wgpu::BufferUsages::UNIFORM | usages
        });

        Self {
            buffer,
            usages: wgpu::BufferUsages::UNIFORM | usages
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn usages(&self) -> wgpu::BufferUsages {
        self.usages
    }
}

struct GroupBuilder<'a> {
    device: &'a wgpu::Device,
    entries: Vec<wgpu::BindGroupLayoutEntry>
}

impl<'a> GroupBuilder<'a> {
    pub fn new(device: &'a wgpu::Device, label: Option<&str>) -> Self {
        Self {
            device,
            entries: vec![]
        }
    }

    pub fn add_uniform(&mut self, uniform: &Uniform, visibility: wgpu::ShaderStages) -> &GroupBuilder<'a> {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None
            },
            count: None
        });

        self
    }

    pub fn entries(&self) -> &[wgpu::BindGroupLayoutEntry] {
        self.entries.as_ref()
    }
}
