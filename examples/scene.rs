use std::collections::HashMap;

use qsr::geometry::{GBufferVertex, Mesh};

fn main() {
    // let mut scene = qsr::Scene::new();

    let mesh = load_obj("resources/meshes/cube.obj");
    println!("{:?}", mesh.vertices);

    qsr::driver::Driver::run();
}

fn load_obj(file_path: &str) -> Mesh {
    let (models, _materials) = tobj::load_obj(file_path, &tobj::LoadOptions {
        triangulate: true,
        single_index: false,
        ..Default::default()
    }).expect("Failed to load obj file");

    println!("Loading {} models", models.len());

    let mut vertices: Vec<GBufferVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut index_map: HashMap<(u32, u32, u32), u32> = HashMap::new();

    for (i, m) in models.iter().enumerate() {
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

    return Mesh::new(&vertices, Some(indices));
}
