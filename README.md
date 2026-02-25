# Questionable Scene Renderer
This is a scene-based rendering API built on top of wgpu.

A basic scene can be created like this:

<table>
    <tr>
        <td>
            <img 
                width="550px"
                src="https://private-user-images.githubusercontent.com/71673490/554527199-9079948f-0fc6-49cf-aaa8-9c66a500c4c4.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE5OTM0MzcsIm5iZiI6MTc3MTk5MzEzNywicGF0aCI6Ii83MTY3MzQ5MC81NTQ1MjcxOTktOTA3OTk0OGYtMGZjNi00OWNmLWFhYTgtOWM2NmE1MDBjNGM0LnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjUlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjI1VDA0MTg1N1omWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPTY1M2U1OWZiODg4NzQwYTVjOTFhNjM5NTA1ZGI4Mjg1MjE0NDRlZGY1ZDFlZDQwNTY0ZGQzOGIwMTNiYWRjNzImWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.I_dYSF2kRS2ahrL6qwZNRV0cDsMO-GP09qEJq7RiPPQ" 
            />
        </td>
        <td>
            <img 
                width="550px"
                src="https://private-user-images.githubusercontent.com/71673490/554524207-ad261f1d-032a-4bfd-bc9d-4ff1c3f6cc1e.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE5OTM0MzcsIm5iZiI6MTc3MTk5MzEzNywicGF0aCI6Ii83MTY3MzQ5MC81NTQ1MjQyMDctYWQyNjFmMWQtMDMyYS00YmZkLWJjOWQtNGZmMWMzZjZjYzFlLnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjUlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjI1VDA0MTg1N1omWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPWZlNGFlOGM2YTQ1MDkyMWMzYjE0NmRhMjU4NzNiMmRhNmNjZWNhMmVmZDNiYmZjZmVkZTgzYzM2NGVlOGJkODkmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.Qa0-t3trWEPx2xpyJZ8nMCfKC6R5dV24QCJTn86dGew" 
            />
        </td>
    </tr>
    <tr>
        <td>
            <img 
                width="550px"
                src="https://private-user-images.githubusercontent.com/71673490/554528096-8c1d7848-8b79-43f6-a08c-3617a9fd2488.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE5OTM2NDIsIm5iZiI6MTc3MTk5MzM0MiwicGF0aCI6Ii83MTY3MzQ5MC81NTQ1MjgwOTYtOGMxZDc4NDgtOGI3OS00M2Y2LWEwOGMtMzYxN2E5ZmQyNDg4LnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjUlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjI1VDA0MjIyMlomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPWUxNmNlMWNkNmE3MDgzYzgyZmY4ZjA1YTkwNWNlZTY2ZjZjZjEwMjViMDU2ZWRkNTI3ZWUxNTM0ZDRmOTIyYzEmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.HUGE0q_f-r3wp7dIFdgaTqtpUjunIbO8e1iQYzfwH_s" 
            />
        </td>
        <td>
            <img 
                width="550px"
                src="https://private-user-images.githubusercontent.com/71673490/554529119-f2686c70-60cb-4d32-acfc-1522926e53da.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE5OTM4NjIsIm5iZiI6MTc3MTk5MzU2MiwicGF0aCI6Ii83MTY3MzQ5MC81NTQ1MjkxMTktZjI2ODZjNzAtNjBjYi00ZDMyLWFjZmMtMTUyMjkyNmU1M2RhLnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjUlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjI1VDA0MjYwMlomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPTdlNjE0ZWMwNmJkNTQ4ZTRlNjU5ZGRmMzg1NTExYWVhNTJlZjE5MGIwMjZkZWYwZTIyYjc5NzkwMzY2ZGIyZjMmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.2zR-6j9AqcE0SXiTa1vW87GBUeL_tzvsC_vjJNuZ-pQ" 
            />
        </td>
    </tr>
</table>

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
![example 1](https://private-user-images.githubusercontent.com/71673490/553867558-00e800f1-426c-4e4a-a0d9-65c9f1402702.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE5MDQ3NjQsIm5iZiI6MTc3MTkwNDQ2NCwicGF0aCI6Ii83MTY3MzQ5MC81NTM4Njc1NTgtMDBlODAwZjEtNDI2Yy00ZTRhLWEwZDktNjVjOWYxNDAyNzAyLnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjQlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjI0VDAzNDEwNFomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPWIzNmY0ODRkNzg0NWI1ZGYxNzNjODVhY2ZiMWU0ZjRmNWE1ZWMxNTJiMTM0YjFhMTBmZjM2ODI2OWM3ODIyZmYmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.6wRMnwQU__hKntAbf8fA_GV7loOrboKMe8ehqOqlaBQ)

This `.obj` file is rendered per-model and attaches the correct textures to each model based on what the file specifies.

You can add more nodes to the scene graph like this:
```rust
fn main() {
    let scene_width = 1000;
    let scene_height = 800;
    let mut scene = qsr::Scene::new(1000, 800);

    scene.set_camera(qsr::camera::Camera { 
        eye: cgmath::Point3 { x: 0.0, y: 16.0, z: 32.0 }, 
        target: cgmath::Point3 { x: 0.0, y: 0.0, z: 0.0 }, 
        up: cgmath::Vector3::unit_y(), 
        aspect: scene_width as f32 / scene_height as f32, 
        fovy: 45.0, 
        znear: 0.1, 
        zfar: 1000.0
    });
    
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
![example 1](https://private-user-images.githubusercontent.com/71673490/553868687-09d5ce12-ed38-4391-ae7a-8cd299da2d46.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE5MDQ4ODUsIm5iZiI6MTc3MTkwNDU4NSwicGF0aCI6Ii83MTY3MzQ5MC81NTM4Njg2ODctMDlkNWNlMTItZWQzOC00MzkxLWFlN2EtOGNkMjk5ZGEyZDQ2LnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjQlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjI0VDAzNDMwNVomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPTFhMTNlNWZhMTFlNjAxZmFmYjZkMTBlYTdiZmY1N2QzNDc2MTUxMTQwNGRmMzlkMWE3ODNhYjRjY2FhMTE3OTUmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.sbzHNRGyM03hVPPf3Y6dlZ2hx6JD-73NBIryewUaNAU)

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
* Alpha Forward Pass [STATUS: done]
* Normal Mapping [STATUS: partial]
* Phong lighting [STATUS: partial]
* Bloom [STATUS: not started]
