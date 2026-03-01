use crate::gfx::{Context, resource::{BufferRegistry, PipelineManager, ResourceAccess, ResourceHandle, ShaderRegistry, TextureRegistry}};

struct RenderGraph {
    nodes: Vec<Box<dyn RenderPassNode>>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RenderPassKind {
    Compute,
    Graphics,
}

type RenderPassResource<'a> = (ResourceHandle<'a>, ResourceAccess);

pub struct RenderPassContext<'a> {
    pub device: &'a wgpu::Device,
    pub shader_registry: &'a ShaderRegistry,
    pub texture_registry: &'a TextureRegistry,
    pub pipeline_manager: &'a mut PipelineManager,
    pub buffer_registry: &'a BufferRegistry,
}

pub trait RenderPassNode {
    fn name(&self) -> &str;
    fn kind(&self) -> RenderPassKind;
    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &mut RenderPassContext<'_>);
    // fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &mut Context);
    fn on_resize(&mut self, context: &Context, width: u32, height: u32);
}
