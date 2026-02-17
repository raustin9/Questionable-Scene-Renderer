use crate::Texture;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct ResourceId(pub usize);

pub enum ResourceKind {
    Texture,
    Buffer,
}

pub enum ResourceAccess {
    Read,
    ReadWrite,
    Write
}

pub enum ResourceData {
    Texture(Texture)
}

pub struct ResourceHandle<'a> {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub resource: &'a ResourceData,
}
