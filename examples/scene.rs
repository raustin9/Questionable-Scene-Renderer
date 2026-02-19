fn main() {
    let mut scene = qsr::Scene::new();
    
    scene.create_node()
        .with_geometry("resources/meshes/cube.obj")
        .with_transform(qsr::Transform::Translate([5.0, 0.0, 0.0]))
        .with_transform(qsr::Transform::Scale([4.0, 2.0, 1.0]));
    
    scene.create_node()
        .with_geometry("resources/meshes/cube.obj")
        .with_transform(qsr::Transform::Translate([-2.0, 3.0, 0.0]))
        ;

    let _ = qsr::driver::Driver::run(&mut scene);
}
