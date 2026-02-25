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
    return output;
}

@group(2) @binding(0) var<uniform> diffuse: vec3<f32>;
@group(2) @binding(1) var<uniform> dissolve: f32;

@fragment
fn fs_main(
    @location(0) fragNormal: vec3f,
    @location(1) fragUV: vec2f,
) -> @location(0) vec4f {
    // var result: vec4f;

    return vec4(diffuse.xyz, dissolve);
    // result = textureSample(t_diffuse, s_diffuse, fragUV);

    // return vec4(result.xyz, dissolve);
}
