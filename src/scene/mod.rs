use crate::{camera::Camera, geometry::{InputGeometry, Mesh, ObjModel}};

pub struct Scene<'a> {
    pub width: u32,
    pub height: u32,
    pub nodes: Vec<Node<'a>>,
    pub camera: Camera,
    pub lights: Vec<LightNode>,
}

impl<'a> Scene<'a> {
    pub fn new(width: u32, height: u32) -> Self {
        env_logger::init();
        Self {
            width,
            height,
            nodes: vec![],
            camera: Camera::default(),
            lights: vec![],
        }
    }

    pub fn add_light(&mut self, light: LightNode) -> &mut Self {
        self.lights.push(light);

        self
    }

    pub fn create_node(&mut self) -> &mut Node<'a> {
        self.nodes.push(Node::new());

        self.nodes.last_mut().unwrap()
    }

    pub fn set_camera(&mut self, camera: Camera) -> &mut Self {
        self.camera = camera;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LightNode {
    pub color: [f32; 3],
    pub location: [f32; 3],
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
    pub model: Option<InputGeometry>,

    pub objs: Option<Vec<ObjModel>>,

    /// The diffuse_texture of this scene node.
    pub material_path: &'a str,

    // Transforms to manipulate the object.
    // These will be applied in the order 
    // that they are added.
    pub transforms: Vec<Transform>,
}

pub type MaterialDesc = tobj::Material;

pub enum ModelSpec<'a> {
    ObjFile {
        /// File path to read object from
        path: &'a str,

        /// Optional directory path to lookup textures in.
        /// If `None` then the directory used is the same
        /// one as the `path` parent directory.
        texture_path: Option<std::path::PathBuf>,
    },
    Custom {
        /// The name of the model
        name: &'a str,

        /// File path to the geometry to read
        geometry_path: &'a str,

        /// File path to the texture
        material_info: MaterialDesc,
    }
}

impl<'a> Node<'a> {
    pub fn new() -> Self {
        Self {
            model: None,
            objs: None,
            material_path: "resources/materials/default_material.png",
            transforms: vec![],
        }
    }

    /// Set the geometry of the node in the scene
    pub fn with_geometry(&mut self, file_path: &'a str) -> &mut Self {
        self.model = Some(InputGeometry::from_obj(file_path));
        self
    }

    pub fn with_model(&mut self, spec: ModelSpec) -> &mut Self {
        match spec {
            ModelSpec::ObjFile { path, texture_path } => {
                self.objs = Some(ObjModel::get_models(path, texture_path));
            },
            ModelSpec::Custom { name, geometry_path, material_info } => {
                self.objs = Some(vec![ObjModel::from_custom(name, geometry_path, &material_info)]);
            },
        };
        
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
