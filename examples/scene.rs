fn main() {
    let mut scene = qsr::Scene::new();
    
    scene.create_node()
        .with_geometry("resources/meshes/cube.obj")
        .with_transform(qsr::Transform::Translate([0.0, 0.0, 0.0]));

    let _ = qsr::driver::Driver::run(&mut scene);
}
