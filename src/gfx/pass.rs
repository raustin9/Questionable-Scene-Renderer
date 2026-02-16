use wgpu::RenderPipelineDescriptor;

use crate::gfx::{shader::Shader, texture::*};

pub struct Node<'a> {
    inputs: &'a [Texture],
    outputs: &'a [Texture],
}

impl<'a> Node<'a> {
    pub fn new(device: &wgpu::Device, inputs: &'a [Texture], outputs: &'a [Texture], label: &str, fragment_shader: &'a Shader, vertex_shader: &'a Shader) -> Self {
        let input_bind_group_layout_entries = inputs
            .iter()
            .enumerate()
            .map(|(index, texture)| {
                wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, // TODO: better API for this
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count: None
                }
            })
            .collect::<Vec<_>>();

        let input_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(format!("{}_bind_group_layout", label).as_ref()),
            entries: &input_bind_group_layout_entries.as_ref()
        });

        let input_bind_group_entries = inputs
            .iter()
            .enumerate()
            .map(|(index, texture)| {
                wgpu::BindGroupEntry {
                    binding: index as u32,
                    resource: wgpu::BindingResource::TextureView(&texture.view())
                }
            })
            .collect::<Vec<_>>();

        let input_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(format!("{}_bind_group", label).as_ref()),
            layout: &input_bind_group_layout,
            entries: &input_bind_group_entries
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{}_pipeline_layout", label).as_ref()),
            bind_group_layouts: &[
                &input_bind_group_layout
            ],
            immediate_size: 0
        });

    
        Self {
            inputs,
            outputs
        }
    }
}

pub struct NodeBuilder<'a> {
    inputs: &'a [Texture],
    outputs: &'a [Texture],
}

impl<'a> NodeBuilder<'a> {
    pub fn new(inputs: &'a [Texture], outputs: &'a [Texture]) -> Self {
        Self {
            inputs,
            outputs
        }
    }
}

pub struct PassManager<'a> {
    nodes: Vec<Node<'a>>
}

impl<'a> PassManager<'a> {
    pub fn new() -> Self {
        Self {
            nodes: vec![]
        }
    }

    pub fn add_node(&mut self, node: Node<'a>) {
        self.nodes.push(node);
    }
}
