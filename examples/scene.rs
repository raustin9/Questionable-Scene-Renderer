use qsr::MaterialDesc;

fn main() {
    let scene_width = 1000;
    let scene_height = 800;
    let mut scene = qsr::Scene::new(1000, 800);

    scene.set_camera(qsr::camera::Camera { 
        eye: cgmath::Point3 { x: 0.0, y: 16.0, z: 32.0 }, 
        target: cgmath::Point3 { x: 0.0, y: 0.0, z: 0.0 }, 
        up: cgmath::Vector3::unit_y(), 
        aspect: scene_width as f32 / scene_height as f32, 
        fovy: 45.0, 
        znear: 0.1, 
        zfar: 1000.0
    });
    
    scene.create_node()
        .with_model(qsr::ModelSpec::ObjFile { path: "resources/aircraft/aircraft.obj", texture_path: None })
        .with_transform(qsr::Transform::Translate([-8.0, 0.0, 0.0]))
        .with_transform(qsr::Transform::Scale([3.5, 3.5, 3.5]));

    scene.create_node()
        .with_model(qsr::ModelSpec::Custom { 
            name: "tree", 
            geometry_path: "resources/meshes/tree.obj", 
            material_info: MaterialDesc {
                diffuse_texture: Some("resources/materials/default_grid.png".into()),
                ..Default::default()
            }
        })
        .with_transform(qsr::Transform::Translate([8.0, 0.0, 10.0]))
        .with_transform(qsr::Transform::Scale([0.5, 0.5, 0.5]));

    let _ = qsr::driver::Driver::run(&mut scene);
}
