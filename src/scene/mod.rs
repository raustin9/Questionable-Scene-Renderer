use crate::geometry::{InputGeometry, Mesh};

pub struct Scene<'a> {
    pub nodes: Vec<Node<'a>>,
}

impl<'a> Scene<'a> {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
        }
    }

    pub fn create_node(&mut self) -> &mut Node<'a> {
        self.nodes.push(Node::new());

        self.nodes.last_mut().unwrap()
    }
}

pub enum RotationUnit {
    Rad(f32),
    Deg(f32),
}

pub enum Transform {
    Translate([f32; 3]),
    Rotate([f32; 3], RotationUnit),
    Scale([f32; 3]),
}

pub struct Node<'a> {
    /// The input geometry set for this node in the scene.
    /// This will later be turned into a `gfx::geometry::Mesh`
    /// for use in the renderer.
    pub geometry: Option<InputGeometry<'a>>,

    /// The diffuse_texture of this scene node.
    pub material_path: &'a str,

    // Transforms to manipulate the object.
    // These will be applied in the order 
    // that they are added.
    pub transforms: Vec<Transform>,
}

impl<'a> Node<'a> {
    pub fn new() -> Self {
        Self {
            geometry: None,
            material_path: "resources/materials/default_material.png",
            transforms: vec![]
        }
    }

    /// Set the geometry of the node in the scene
    pub fn with_geometry(&mut self, file_path: &'a str) -> &mut Self {
        self.geometry = Some(InputGeometry::from_obj(file_path));
        self
    }

    /// Set the diffuse texture of the node in the scene
    pub fn with_texture(&mut self, texture: &'a str) -> &mut Self {
        self.material_path = texture;
        self
    }

    pub fn with_transform(&mut self, transform: Transform) -> &mut Self {
        self.transforms.push(transform);
        self
    }
}
