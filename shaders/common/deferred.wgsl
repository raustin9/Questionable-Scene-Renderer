@vertex
fn vs_main(
  @builtin(vertex_index) VertexIndex : u32
) -> @builtin(position) vec4f {
  const pos = array( vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0),
    vec2(-1.0, 1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
  );

  return vec4f(pos[VertexIndex], 0.0, 1.0);
}

@group(0) @binding(0) var gBufferNormal: texture_2d<f32>;
@group(0) @binding(1) var gBufferAlbedo: texture_2d<f32>;
@group(0) @binding(2) var gBufferDepth: texture_2d<f32>;
struct Camera {
  viewProjectionMatrix : mat4x4f,
  invViewProjectionMatrix : mat4x4f,
}

@group(1) @binding(0) var<uniform> camera: Camera;

struct LightProperty {
    position: vec4<f32>,
    color: vec4<f32>,
}
@group(2) @binding(0) var<uniform> u_num_lights: u32;
@group(2) @binding(1) var<storage, read> s_lights: array<LightProperty>;

fn world_from_screen_coord(coord : vec2f, depth_sample: f32) -> vec3f {
  // reconstruct world-space position from the screen coordinate.
  let posClip = vec4(coord.x * 2.0 - 1.0, (1.0 - coord.y) * 2.0 - 1.0, depth_sample, 1.0);
  let posWorldW = camera.invViewProjectionMatrix * posClip;
  let posWorld = posWorldW.xyz / posWorldW.www;
  return posWorld;
}

@fragment
fn fs_main(
  @builtin(position) coord : vec4f
) -> @location(0) vec4f {
    var result : vec3f;

    let depth = textureLoad(
        gBufferDepth,
        vec2i(floor(coord.xy)),
        0
    ).x;

    // Don't light the sky.
    if (depth >= 1.0) {
        discard;
    }

    let bufferSize = textureDimensions(gBufferDepth);
    let coordUV = coord.xy / vec2f(bufferSize);
    let position = world_from_screen_coord(coordUV, depth);

    let normal = textureLoad(
        gBufferNormal,
        vec2i(floor(coord.xy)),
        0
    ).xyz;

    let albedo = textureLoad(
        gBufferAlbedo,
        vec2i(floor(coord.xy)),
        0
    ).rgb;

    let light_radius = 100.0;
    for (var i = 0u; i < u_num_lights; i++) {
        let L = s_lights[i].position.xyz - position;
        let distance = length(L);
        if (distance > light_radius) {
            continue;
        }

        let lambert = max(dot(normal, normalize(L)), 0.0);
        result += vec3f(
            lambert * pow(1.0 - distance / light_radius, 2.0) * s_lights[i].color.xyz * albedo
        );
    }

    result += vec3f(0.02) * albedo;

    return vec4(result, 1.0);
}

