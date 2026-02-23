# Questionable Scene Renderer
This is a scene-based rendering API built on top of wgpu.

A basic scene can be created like this:
```rust
fn main() {
    let mut scene = qsr::Scene::new();
    
    scene.create_node()
        .with_model(qsr::ModelSpec::ObjFile { path: "resources/aircraft/aircraft.obj", texture_path: None })
        .with_transform(qsr::Transform::Scale([3.5, 3.5, 3.5]));

    let _ = qsr::driver::Driver::run(&mut scene);
}
```
This will pull up a window with the scene rendered.
![example 1](https://private-user-images.githubusercontent.com/71673490/553661037-8aa77057-c6d9-4b27-b20a-25230eda2024.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE4NzQwNzUsIm5iZiI6MTc3MTg3Mzc3NSwicGF0aCI6Ii83MTY3MzQ5MC81NTM2NjEwMzctOGFhNzcwNTctYzZkOS00YjI3LWIyMGEtMjUyMzBlZGEyMDI0LnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjMlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjIzVDE5MDkzNVomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPTVlYjk2NGVjNmU2MzA0ZDJhMDY1NjM5MTBjNGQyNTdlZGRlZWExODZhZTYxYjRmNTU2ODU4NzliZTg4YWM0YTgmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.z-KqC3rvmSU7AemUJCnzghLk-jgnQS7EG736owGIeVo)

This `.obj` file is rendered per-model and attaches the correct textures to each model based on what the file specifies.

You can add more nodes to the scene graph like this:
```rust
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
```
Resulting in an image that would look like this:
![example 1](https://private-user-images.githubusercontent.com/71673490/553684542-6aea2cd7-82b2-43c7-9c9e-f1d8dec2d6c1.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE4Nzc0NjQsIm5iZiI6MTc3MTg3NzE2NCwicGF0aCI6Ii83MTY3MzQ5MC81NTM2ODQ1NDItNmFlYTJjZDctODJiMi00M2M3LTljOWUtZjFkOGRlYzJkNmMxLnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjMlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjIzVDIwMDYwNFomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPWJiNzhiODQxZTZjNmI4Nzk5MWYxNmFlZTZlYjFhMGY3ZWFjZGQ3YjU2ZmNhZTVhY2U4N2U1YmM3ZDIyM2QxMzUmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.fvVfBFl4VIn_aUNHea8d-SW5l5P6NEaMeWFJgxXIPxc)

Note that the new object is loaded from an obj file like before, but it is specified using a `qsr::ModelSpec::Custom` which allows for splitting the material and geometry specs rather than having them be in the same file.

## Renderer Design
### Render Graph and Deferred Rendering
This renderer is a personal experiment to play around with different architectures and rendering techniques.

Right now, this is using a deferred rendering architecture where each renderable first writes all the geometry data (albedo, depth, normals) to gbuffers, and later render passes load those textures to perform their operations.

### Material and Texture handling
The renderer is based around a central `qsr::gfx::Context` object which interfaces the renderer with GPU operations.

Textures are created through the context, and are managed by a `TextureRegistry` that stores the textures and distributes handles to them.
By storing, the TextureRegistry keeps the `wgpu::Texture` and `wgpu::TextureView` objects. 

When a texture is created that is the same as a previous one, it is deduplicated to prevent excessive resource use, and a handle to the 
existing texture is given.

### Current and Future Work
This is still in very early development.
The goal is to put more and more builtin renderpasses to the render graph. Here are the immediate ones:
* Normal Mapping [STATUS: partial]
* Phong lighting [STATUS: partial]
* Bloom [STATUS: not started]
