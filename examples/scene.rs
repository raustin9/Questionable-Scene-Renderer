fn main() {
    let mut scene = qsr::Scene::new();
    
    scene.create_node()
        .with_geometry("resources/meshes/tree.obj")
        .with_texture("resources/materials/default_grid.png")
        .with_transform(qsr::Transform::Translate([0.0, -6.0, 0.0]))
        .with_transform(qsr::Transform::Scale([1.0, 1.0, 1.0]));

    let _ = qsr::driver::Driver::run(&mut scene);
}
