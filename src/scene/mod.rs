use crate::geometry::Mesh;

pub struct Scene<'a> {
    nodes: Vec<Node<'a>>,
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

pub struct Node<'a> {
    mesh: Option<&'a Mesh>,
}

impl<'a> Node<'a> {
    pub fn new() -> Self {
        Self {
            mesh: None
        }
    }

    pub fn with_mesh(&mut self, mesh: &'a Mesh) -> &mut Self {
        self.mesh = Some(mesh);
        
        self
    }
}
