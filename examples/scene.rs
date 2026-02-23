use qsr::MaterialDesc;

fn main() {
    let mut scene = qsr::Scene::new();
    
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
