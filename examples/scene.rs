fn main() {
    let mut scene = qsr::Scene::new();
    
    scene.create_node()
        .with_geometry("resources/meshes/cube.obj");

    let _ = qsr::driver::Driver::run(&mut scene);
}
