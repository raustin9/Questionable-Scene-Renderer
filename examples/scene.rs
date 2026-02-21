fn main() {
    let mut scene = qsr::Scene::new();
    
    scene.create_node()
        .with_geometry("resources/meshes/agera.obj")
        .with_texture("resources/materials/default_grid.png")
        .with_transform(qsr::Transform::Translate([5.0, 0.0, 0.0]))
        .with_transform(qsr::Transform::Scale([0.5, 0.5, 0.5]));
    
    scene.create_node()
        .with_geometry("resources/meshes/tower.obj")
        .with_texture("resources/materials/wood.jpg")
        .with_transform(qsr::Transform::Translate([-5.0, 0.0, 10.0]))
        .with_transform(qsr::Transform::Scale([1.0, 1.0, 1.0]));

    let _ = qsr::driver::Driver::run(&mut scene);
}
