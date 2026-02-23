use std::{collections::HashMap, fs, usize};

use image::DynamicImage;

use crate::gfx::material::{Material, MaterialInfo};

pub trait Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}


/// Vertex layout for a .obj file
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GBufferVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texel: [f32; 2],
}

impl Vertex for GBufferVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ]
        }
    }
}

pub(crate) struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub index_buffer: Option<wgpu::Buffer>,
    pub index_count: u32,
}

impl Mesh {
    pub fn new(vertex_buffer: wgpu::Buffer, vertex_count: u32, index_buffer: Option<wgpu::Buffer>, index_count: u32) -> Self {
        Self {
            vertex_buffer,
            vertex_count,
            index_buffer,
            index_count,
        }
    }
}

pub struct InputGeometry {
    pub name: String,
    pub vertices: Vec<GBufferVertex>,
    pub indices: Option<Vec<u32>>,

    // pub material: Material
}

pub struct ObjModel {
    // The mesh data for an Obj model
    pub mesh: InputGeometry,

    // The material data for a mesh object
    pub material: Option<MaterialInfo>,
}

impl ObjModel {
    pub fn get_models(file_path: &str) -> Vec<Self> {
        println!("------------------ GETTING MODELS ------------------");
        let mut objs: Vec<Self> = Vec::new();
        let parent = std::path::Path::new(file_path).parent().expect("Failed to get parent directory of obj file");
        println!("PARENT: {:?}", parent);

        let (models, materials) = tobj::load_obj(file_path, &tobj::LoadOptions {
            triangulate: true,
            single_index: false,
            ..Default::default()
        }).expect("Failed to read obj file");

        for (index, model) in models.iter().enumerate() {
            println!("Model {}: {}", index, model.name);
            let material = match &model.mesh.material_id {
                Some(id) => {
                    match &materials {
                        Err(_e) => None,
                        Ok(materials) => {
                            let processed_material = Self::get_material_info(&materials[*id], &std::path::PathBuf::from(parent));
                            Some(processed_material)
                        }
                    }
                },
                None => None
            };
            let mesh: InputGeometry = model.into();

            objs.push(Self {
                material,
                mesh
            });
        }

        println!("------------------  READ  MODELS  ------------------");

        objs
    }

    fn load_material_texture(file_path: &std::path::PathBuf) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        let image_buffer = fs::read(file_path)
            .expect(format!("Error reading texture file: {:?}", file_path).as_str());

        let image = image::load_from_memory(image_buffer.as_slice())
            .expect("Failed to load texture image from memory");
        Ok(image)
    }


    fn get_material_info(material: &tobj::Material, directory: &std::path::PathBuf) -> MaterialInfo {
        assert!(directory.is_dir(), "directory parameter must be a valid directory");

        println!("Reading material file: {}", material.name);
        print!("Diffuse texture: ");
        let diffuse_texture = match &material.diffuse_texture {
            Some(path) => {
                println!("PATH: {:?}", path);
                let diffuse_path = std::path::PathBuf::from(directory).join(util::get_cleaned_path(path));
                match Self::load_material_texture(&diffuse_path) {
                    Err(e) => {
                        log::error!("Failed reading material file \"{:?}\": {}", util::get_cleaned_path(path), e);
                        None
                    },
                    Ok(img) => {
                        println!("Read successfully.");
                        Some(img)
                    }
                }
            },
            None => {
                println!("No diffuse texture for this material");
                None
            },
        };

        print!("Ambient texture: ");
        let ambient_texture = match &material.ambient_texture {
            Some(path) => {
                let ambient_path = std::path::PathBuf::from(directory).join(util::get_cleaned_path(path));
                println!("Diffuse path: {:?}", ambient_path);
                match Self::load_material_texture(&ambient_path) {
                    Err(e) => {
                        log::error!("Failed reading material file \"{:?}\": {}", util::get_cleaned_path(path), e);
                        None
                    },
                    Ok(img) => {
                        println!("Read successfully.");
                        Some(img)
                    }
                }
            },
            None => {
                println!("No ambient texture for this material");
                None
            },
        };

        print!("Specular texture: ");
        let specular_texture = match &material.specular_texture {
            Some(path) => {
                let specular_path = std::path::PathBuf::from(directory).join(util::get_cleaned_path(path));
                match Self::load_material_texture(&specular_path) {
                    Err(e) => {
                        log::error!("Failed reading material file \"{:?}\": {}", util::get_cleaned_path(path), e);
                        None
                    },
                    Ok(img) => {
                        println!("Read successfully.");
                        Some(img)
                    }
                }
            },
            None => {
                println!("No specular texture for this material");
                None
            },
        };

        print!("Normal texture: ");
        let normal_texture = match &material.normal_texture {
            Some(path) => {
                let normal_path = std::path::PathBuf::from(directory).join(util::get_cleaned_path(path));
                match Self::load_material_texture(&normal_path) {
                    Err(e) => {
                        log::error!("Failed reading material file \"{:?}\": {}", util::get_cleaned_path(path), e);
                        None
                    },
                    Ok(img) => {
                        println!("Read successfully.");
                        Some(img)
                    }
                }
            },
            None => {
                println!("No normal texture for this material");
                None
            },
        };

        print!("Dissolve texture: ");
        let dissolve_texture = match &material.dissolve_texture {
            Some(path) => {
                let dissolve_path = std::path::PathBuf::from(directory).join(util::get_cleaned_path(path));
                match Self::load_material_texture(&dissolve_path) {
                    Err(e) => {
                        log::error!("Failed reading material file \"{:?}\": {}", util::get_cleaned_path(path), e);
                        None
                    },
                    Ok(img) => {
                        println!("Read successfully.");
                        Some(img)
                    }
                }
            },
            None => {
                println!("No dissolve texture for this material");
                None
            },
        };

        print!("Shininess texture: ");
        let shininess_texture = match &material.shininess_texture {
            Some(path) => {
                let shininess_path = std::path::PathBuf::from(directory).join(util::get_cleaned_path(path));
                match Self::load_material_texture(&shininess_path) {
                    Err(e) => {
                        println!("Error reading material texture file: {}", e);
                        None
                    },
                    Ok(img) => {
                        println!("Read successfully.");
                        Some(img)
                    }
                }
            },
            None => {
                println!("No shininess texture for this material");
                None
            },
        };

        println!("diffuse: {:?}", material.diffuse);
        println!("ambient: {:?}", material.ambient);
        println!("specular: {:?}", material.diffuse);
        println!("dissolve: {:?}", material.dissolve);
        println!("shininess: {:?}", material.shininess);
        println!("illumination_model: {:?}", material.illumination_model);
        println!("optical_density: {:?}", material.optical_density);

        MaterialInfo {
            diffuse_texture,
            diffuse_color: material.diffuse,

            ambient_texture,
            ambient_color: material.ambient,

            shininess_texture,
            shininess_coef: material.shininess,

            specular_texture,
            specular_color: material.specular,

            dissolve_texture,
            dissolve_coef: material.dissolve,

            normal_texture,

            illumination_model: material.illumination_model,

            optical_density: material.optical_density,
        }
    }
}

impl InputGeometry {
    // TODO: make this return different meshes from each model found in a .obj file
    pub fn from_obj_2(file_path: &str) -> Vec<Self> {
        let mut constructed_models: Vec<Self> = Vec::new();

        let (models, materials) = tobj::load_obj(file_path, &tobj::LoadOptions {
            triangulate: true,
            single_index: false,
            ..Default::default()
        }).expect("Failed to load obj file");
        println!("Loading {} models", models.len());

        match materials {
            Err(e) => println!("{:?}", e),
            Ok(materials) => {
                for material in materials {
                    println!("Found Material: {}", material.name);
                    println!("{:?}", material);
                }
            }
        };

        for (idx, m) in models.iter().enumerate() {
            println!("Reading model {}: {}", idx, m.name);
            constructed_models.push(m.into());
        }

        constructed_models
    }

    pub fn from_obj(file_path: & str) -> Self {
        let (models, materials) = tobj::load_obj(file_path, &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        }).expect("Failed to load obj file");
        println!("Loading {} models", models.len());

        match materials {
            Err(e) => println!("{:?}", e),
            Ok(materials) => {
                for material in materials {
                    println!("Found Material: {}", material.name);
                    println!("{:?}", material);
                }
            }
        };

        let mut vertices: Vec<GBufferVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut index_map: HashMap<(u32, u32, u32), u32> = HashMap::new();

        for (_, m) in models.iter().enumerate() {
            let mesh = &m.mesh;

            println!("Model name: {}", m.name);

            let has_tc = !mesh.texcoord_indices.is_empty();
            let has_n = !mesh.normal_indices.is_empty();

            for i in 0..mesh.indices.len() {
                let pos_idx = mesh.indices[i];
                let texel_idx = if has_tc { mesh.texcoord_indices[i] } else { 0 };
                let normal_idx = if has_n { mesh.normal_indices[i] } else { 0 };

                let key = (pos_idx, texel_idx, normal_idx);

                if let Some(&existing) = index_map.get(&key) {
                    indices.push(existing);
                } else {
                    let p = (pos_idx * 3) as usize;
                    let position = [
                        mesh.positions[p],
                        mesh.positions[p + 1],
                        mesh.positions[p + 2],
                    ];

                    let tex_coord = if has_tc {
                        let t = (texel_idx * 2) as usize;
                        [mesh.texcoords[t], mesh.texcoords[t + 1]]
                    } else {
                        [0.0, 0.0]
                    };

                    let normal = if has_n {
                        let n = (normal_idx * 3) as usize;
                        [mesh.normals[n], mesh.normals[n+1], mesh.normals[n+2]]
                    } else {
                        [0.0, 0.0, 0.0]
                    };

                    let new_idx = vertices.len() as u32;
                    vertices.push(GBufferVertex { position, normal, texel: tex_coord });
                    index_map.insert(key, new_idx);
                    indices.push(new_idx);
                }
            }
        }
        Self {
            name: String::from(file_path),
            vertices,
            indices: Some(indices),
        }
    }
}

impl From<&tobj::Model> for InputGeometry {
    fn from(value: &tobj::Model) -> Self {
        let mesh = &value.mesh;
        let name = value.name.clone();
        let has_texel_coords = !mesh.texcoord_indices.is_empty();
        let has_normals = !mesh.normal_indices.is_empty();
        
        println!("Model: {}", value.name);
        println!("  indices: {}", mesh.indices.len());
        println!("  texcoord_indices: {}", mesh.texcoord_indices.len());
        println!("  normal_indices: {}", mesh.normal_indices.len());
        println!("  positions: {}", mesh.positions.len() / 3);
        println!("  texcoords: {}", mesh.texcoords.len() / 2);
        println!("  normals: {}", mesh.normals.len() / 3);

        let mut vertices: Vec<GBufferVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
            
        let mut index_map: HashMap<(u32, u32, u32), u32> = HashMap::new();

        for i in 0..mesh.indices.len() { 
            let position_index = mesh.indices[i];
            let texel_index = if has_texel_coords { mesh.texcoord_indices[i] } else { 0 };
            let normal_index = if has_normals { mesh.normal_indices[i] } else { 0 };
            
            let index_key = (position_index, texel_index, normal_index);
            if let Some(&existing_index) = index_map.get(&index_key) {
                indices.push(existing_index);
            } else {
                let p = (position_index * 3) as usize;
                let position = [
                    mesh.positions[p],
                    mesh.positions[p + 1],
                    mesh.positions[p + 2],
                ];

                let texel = if has_texel_coords {
                    let t = (texel_index * 2) as usize;
                    [mesh.texcoords[t], 1.0 - mesh.texcoords[t + 1]]
                } else {
                    [0.0, 0.0]
                };

                let normal = if has_normals {
                    let n = (normal_index * 3) as usize;
                    [
                        mesh.normals[n], 
                        mesh.normals[n + 1], 
                        mesh.normals[n + 2], 
                    ]
                } else {
                    [0.0, 0.0, 0.0]
                };

                let new_index = vertices.len() as u32;
                vertices.push(GBufferVertex { position, normal, texel });
                index_map.insert(index_key, new_index);
                indices.push(new_index);
            }
        }

        Self {
            name,
            vertices,
            indices: Some(indices)
        }
    }
}

mod util {
    pub fn get_cleaned_path(path: &str) -> std::path::PathBuf {
        let first = path.replace('\\', "/");
        let mut path = std::path::PathBuf::new();
        for part in first.split('/') {
            path = path.join(part);
        }

        path
    }
}
