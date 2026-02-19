use std::collections::HashMap;

use cgmath::SquareMatrix;

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
    pub model_matrix: cgmath::Matrix4<f32>
}

impl Mesh {
    pub fn new(vertex_buffer: wgpu::Buffer, vertex_count: u32, index_buffer: Option<wgpu::Buffer>, index_count: u32) -> Self {
        Self {
            vertex_buffer,
            vertex_count,
            index_buffer,
            index_count,
            model_matrix: cgmath::Matrix4::identity(),
        }
    }
}

pub struct InputGeometry<'a> {
    pub name: &'a str,
    pub vertices: Vec<GBufferVertex>,
    pub indices: Option<Vec<u32>>,
}

impl<'a> InputGeometry<'a> {
    pub fn from_obj(file_path: &'a str) -> Self {
        let (models, _materials) = tobj::load_obj(file_path, &tobj::LoadOptions {
            triangulate: true,
            single_index: false,
            ..Default::default()
        }).expect("Failed to load obj file");
        println!("Loading {} models", models.len());

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
            name: file_path,
            vertices,
            indices: Some(indices),
        }
    }
}
