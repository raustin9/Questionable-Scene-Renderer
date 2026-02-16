use std::{borrow::Cow, error::Error, fs};

// TODO: use the builder pattern for this
pub struct Shader<'a> {
    module: wgpu::ShaderModule,
    vert_entry: Option<&'a str>,
    frag_entry: Option<&'a str>,
}

impl<'a> Shader<'a> {
    pub fn from_source(device: &wgpu::Device, source: Cow<'a, str>, vert_entry: Option<&'a str>, frag_entry: Option<&'a str>, label: Option<&'a str>) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(source.into())
        });

        Self {
            module,
            vert_entry,
            frag_entry,
        }
    }

    pub fn from_path(device: &wgpu::Device, file_path: &'a str, vert_entry: Option<&'a str>, frag_entry: Option<&'a str>, label: Option<&'a str>) -> Result<Self, Box<dyn Error>> {
        let source = fs::read_to_string(file_path)?;
        
        Ok(
            Self::from_source(
                device, 
                source.into(), 
                vert_entry, 
                frag_entry, 
                label
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


    // TODO: add functions like "add_uniform" for safer construction of shader module
}
