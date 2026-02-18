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
