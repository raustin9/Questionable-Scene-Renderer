# Questionable Scene Renderer
This is a scene-based rendering API built on top of wgpu.

A basic scene can be created like this:
```rust
fn main() {
    let mut scene = qsr::Scene::new();
    
    scene.create_node()
        .with_geometry("resources/meshes/tree.obj")
        .with_texture("resources/materials/default_grid.png")
        .with_transform(qsr::Transform::Translate([0.0, -6.0, 0.0]))
        .with_transform(qsr::Transform::Scale([1.0, 1.0, 1.0]));

    let _ = qsr::driver::Driver::run(&mut scene);
}
```
This will pull up a window with the scene rendered.
![example 1](https://private-user-images.githubusercontent.com/71673490/553033778-9b6dae31-f493-4813-a208-e445f4ec8b5b.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE2OTUyODYsIm5iZiI6MTc3MTY5NDk4NiwicGF0aCI6Ii83MTY3MzQ5MC81NTMwMzM3NzgtOWI2ZGFlMzEtZjQ5My00ODEzLWEyMDgtZTQ0NWY0ZWM4YjViLnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjElMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjIxVDE3Mjk0NlomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPTY1YzBhNGMxNmJiMmFmNDRmZjcyMjMwNzQxNTM1ZmZhMTQ4OTk2YjM3MjQ4N2IxZTQwYjRjYjZhZmJjYzdiMTkmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.IY2I7j64hbdq2-IAbR1LYX-GvJ1ygjcilcIWy5fGiAw)

You can expand a scene with more nodes as well:
```rust
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
```
Which results in this:
![example 2](https://private-user-images.githubusercontent.com/71673490/553033793-3299ff36-7a05-4b28-89a9-08e11591204a.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE2OTUyODYsIm5iZiI6MTc3MTY5NDk4NiwicGF0aCI6Ii83MTY3MzQ5MC81NTMwMzM3OTMtMzI5OWZmMzYtN2EwNS00YjI4LTg5YTktMDhlMTE1OTEyMDRhLnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjElMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjIxVDE3Mjk0NlomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPTAwMGE1YWQ4NGE2ODgzNWVhZGY1NDNiYjZjZjQ5ZGEwZGQ4NGVlNjM5MzhhM2ViOGJhMzEwNmFiZTBkN2FlNGQmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.tSiGBfuGYsuq9WftUwD0vD7sSYnM2nfTCD5z_yD7ZX0)
