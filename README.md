# Questionable Scene Renderer
This is a scene-based rendering API built on top of wgpu.

A basic scene can be created like this:
```rust
fn main() {
    let mut scene = qsr::Scene::new();
    
    scene.create_node()
        .with_model("resources/aircraft/aircraft.obj")
        .with_transform(qsr::Transform::Scale([3.5, 3.5, 3.5]));

    let _ = qsr::driver::Driver::run(&mut scene);
}
```
This will pull up a window with the scene rendered.
![example 1](https://private-user-images.githubusercontent.com/71673490/553661037-8aa77057-c6d9-4b27-b20a-25230eda2024.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE4NzQwNzUsIm5iZiI6MTc3MTg3Mzc3NSwicGF0aCI6Ii83MTY3MzQ5MC81NTM2NjEwMzctOGFhNzcwNTctYzZkOS00YjI3LWIyMGEtMjUyMzBlZGEyMDI0LnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjMlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjIzVDE5MDkzNVomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPTVlYjk2NGVjNmU2MzA0ZDJhMDY1NjM5MTBjNGQyNTdlZGRlZWExODZhZTYxYjRmNTU2ODU4NzliZTg4YWM0YTgmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.z-KqC3rvmSU7AemUJCnzghLk-jgnQS7EG736owGIeVo)

This `.obj` file is rendered per-model and attaches the correct textures to each model based on what the file specifies.

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
