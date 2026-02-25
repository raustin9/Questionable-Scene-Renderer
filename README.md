# Questionable Scene Renderer
This is a scene-based rendering API built on top of wgpu.

A basic scene can be created like this:

<table>
    <tr>
        <td>
            <img 
                width="550px"
                src="https://drive.google.com/uc?export=view&id=11_tDVKy1SbXfcvLb1oa2V8-MNE_-wAMK" 
            />
        </td>
        <td>
            <img 
                width="550px"
                src="https://drive.google.com/uc?export=view&id=13ZRWShUHi5HR6xSA_HUld6wXrEXTLdgE" 
            />
        </td>
    </tr>
    <tr>
        <td>
            <img 
                width="550px"
                src="https://drive.google.com/uc?export=view&id=17-U_w4-jWSRN-6ryDX1wfh4NrslqhLKY" 
            />
        </td>
        <td>
            <img 
                width="550px"
                src="https://drive.google.com/uc?export=view&id=1pNaBPgn3SaNZYOqEg5OvVWsNCJLBCNXR" 
            />
        </td>
    </tr>
</table>

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

    scene.add_light(qsr::LightNode { 
        location: [15.0, 10.0, 0.0],
        color: [1.0, 1.0, 1.0], 
    });
    
    scene.create_node()
        .with_model(qsr::ModelSpec::ObjFile { path: "resources/aircraft/aircraft.obj", texture_path: None })
        .with_transform(qsr::Transform::Scale([3.5, 3.5, 3.5]));

    let _ = qsr::driver::Driver::run(&mut scene);
}
```
This will pull up a window with the scene rendered.
![example 1](https://private-user-images.githubusercontent.com/71673490/554530195-8152a6ee-dc80-456e-a536-0683ae22bf3d.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE5OTQwODQsIm5iZiI6MTc3MTk5Mzc4NCwicGF0aCI6Ii83MTY3MzQ5MC81NTQ1MzAxOTUtODE1MmE2ZWUtZGM4MC00NTZlLWE1MzYtMDY4M2FlMjJiZjNkLnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjUlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjI1VDA0Mjk0NFomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPWI0MmY3ZjRlNmY4NzNkZDIwMzNkNmFhMDY1OWJmMGM5MDllMWZmMzlhNTVlZjk2MDQzODE0MjMyMTEwZjcxYjEmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.AHK6KS7GIrv7VWeuslP6G7tFa0S47fDdxbdkd7OT1gw)

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

    scene.add_light(qsr::LightNode { 
        location: [15.0, 10.0, 0.0],
        color: [1.0, 1.0, 1.0], 
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
![example 1](https://private-user-images.githubusercontent.com/71673490/554530736-9dbf4564-7fa4-4db4-aa88-738cf3a78887.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzE5OTQyMTcsIm5iZiI6MTc3MTk5MzkxNywicGF0aCI6Ii83MTY3MzQ5MC81NTQ1MzA3MzYtOWRiZjQ1NjQtN2ZhNC00ZGI0LWFhODgtNzM4Y2YzYTc4ODg3LnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNjAyMjUlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjYwMjI1VDA0MzE1N1omWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPWVhOTNmZDczNWM5MDMwZmUwYzY1YWM0NTkwMGRiZTU0MzZlYmZmOTY2OThkODFjNDAyNGIwNGMzMGZmZTE1MDQmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.0SxABYXDl4S0yu3XxsGutS2NRRPy1tdjRr_0aJUnaH0)

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
