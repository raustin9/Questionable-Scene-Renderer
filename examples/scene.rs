fn main() {
    let mut scene = qsr::Scene::new();
    
    scene.create_node()
        .with_model("resources/aircraft/aircraft.obj")
        .with_transform(qsr::Transform::Scale([3.5, 3.5, 3.5]));

    let _ = qsr::driver::Driver::run(&mut scene);
}
