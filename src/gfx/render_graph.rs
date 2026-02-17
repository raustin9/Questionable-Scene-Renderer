use crate::gfx::resource::{ResourceAccess, ResourceHandle};

struct RenderGraph<'a> {
    nodes: Vec<RenderPassNode<'a>>,
}

impl<'a> RenderGraph<'a> {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
        }
    }
}

pub enum RenderPassKind {
    Compute,
    Graphics,
}

type RenderPassResource<'a> = (ResourceHandle<'a>, ResourceAccess);

struct RenderPassNode<'a> {
    name: &'static str,
    kind: RenderPassKind,
    color_attachments: Vec<RenderPassResource<'a>>,
    depth_attachments: Vec<RenderPassResource<'a>>,
    command_recorder: Box<dyn RenderPassCommandRecorder>,
}

impl<'a> RenderPassNode<'a> {
    pub fn new(name: &'static str, kind: RenderPassKind, command_recorder: Box<dyn RenderPassCommandRecorder>) -> Self {
        Self {
            name,
            kind,
            color_attachments: vec![],
            depth_attachments: vec![],
            command_recorder,
        }
    }

    pub fn add_color_attachment(&mut self, resource: ResourceHandle<'a>, access: ResourceAccess) -> &mut Self {
        self.color_attachments.push((resource, access));
        self
    }

    pub fn add_depth_attachment(&mut self, resource: ResourceHandle<'a>, access: ResourceAccess) -> &mut Self {
        self.depth_attachments.push((resource, access));
        self
    }
}

pub trait RenderPassCommandRecorder: Send + Sync {
    fn record_commands(&self, encoder: &mut wgpu::CommandEncoder);
}

impl<T> RenderPassCommandRecorder for T
where 
    T: Fn(&mut wgpu::CommandEncoder) + Send + Sync,
{
    fn record_commands(&self, encoder: &mut wgpu::CommandEncoder) {
        self(encoder)
    }
}
