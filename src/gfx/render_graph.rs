use crate::gfx::{Context, resource::{ResourceAccess, ResourceHandle}};

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

pub trait RenderPassNode {
    fn name(&self) -> &str;
    fn kind(&self) -> RenderPassKind;
    fn execute(&self, encoder: &mut wgpu::CommandEncoder, context: &Context);
}
