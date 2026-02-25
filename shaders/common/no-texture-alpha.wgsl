struct Uniforms {
    modelMatrix: mat4x4f,
    normalModelMatrix: mat4x4f,
}

struct Camera {
    viewProjection: mat4x4f,
    invViewProjection: mat4x4f,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) Position: vec4f,
    @location(0) fragNormal: vec3f,
    @location(1) fragUV: vec2f,
    @location(2) worldPos: vec3f,
}

@vertex
fn vs_main(
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) uv: vec2f
) -> VertexOutput {
    var output : VertexOutput;
    let worldPosition = (uniforms.modelMatrix * vec4(position, 1.0)).xyz;
    output.Position = camera.viewProjection * vec4(worldPosition, 1.0);
    output.fragNormal = normalize((uniforms.normalModelMatrix * vec4(normal, 1.0)).xyz);
    output.fragUV = uv;
    output.worldPos = worldPosition;
    return output;
}

@group(2) @binding(0) var<uniform> diffuse: vec3<f32>;
@group(2) @binding(1) var<uniform> dissolve: f32;

struct LightProperty {
    position: vec4<f32>,
    color: vec4<f32>,
}
@group(3) @binding(0) var<uniform> u_num_lights: u32;
@group(3) @binding(1) var<storage, read> s_lights: array<LightProperty>;

@fragment
fn fs_main(
    @location(0) fragNormal: vec3f,
    @location(1) fragUV: vec2f,
    @location(2) worldPosition: vec3f,
) -> @location(0) vec4f {
    var result: vec3f;

    let albedo = diffuse.xyz;

    let light_radius = 100.0;
    
    for (var i = 0u; i < u_num_lights; i++) {
        let L = s_lights[i].position.xyz - worldPosition.xyz;
        let distance = length(L);
        if (distance > light_radius) {
            continue;
        }

        let lambert = max(dot(fragNormal, normalize(L)), 0.0);
        result += vec3f(
            lambert * pow(1.0 - distance / light_radius, 2.0) * s_lights[i].color.xyz * albedo
        );
    }

    result += vec3f(0.08) * albedo;

    return vec4(result.xyz, dissolve);
}
