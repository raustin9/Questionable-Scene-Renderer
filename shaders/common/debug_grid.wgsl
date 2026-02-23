struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) near_point: vec3<f32>,
    @location(1) far_point: vec3<f32>,
}

struct Camera {
  viewProjectionMatrix : mat4x4f,
  invViewProjectionMatrix : mat4x4f,
}
@group(0) @binding(0) var<uniform> camera: Camera;

fn unproject(clip_xy: vec2<f32>, z: f32) -> vec3<f32> {
    let inv = camera.invViewProjectionMatrix;
    let h = inv * vec4<f32>(clip_xy, z, 1.0);
    return h.xyz / h.w;
}


@vertex
fn vs_main(
  @builtin(vertex_index) VertexIndex : u32
) -> VertexOutput {
    const pos = array( vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0),
        vec2(-1.0, 1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
    );

    var out: VertexOutput;

    let p = pos[VertexIndex];
    out.clip_pos = vec4<f32>(p, 0.0, 1.0);
    out.near_point = unproject(p, 0.0);
    out.far_point = unproject(p, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = -in.near_point.y / (in.far_point.y - in.near_point.y);

    if (t < 0.0) {
        discard;
    }

    let world_pos = in.near_point + t * (in.far_point - in.near_point);

    let grid_uv = fract(world_pos.xz * 0.25);

    let line_width = 0.01; // desired line width as a fraction of a cell
    let aa_width = fwidth(grid_uv) * 1.5; // adjust anti-aliasing spread
    let lines = smoothstep(vec2<f32>(line_width), vec2<f32>(line_width) + aa_width, grid_uv);

    let grid_intensity = 1.0 - min(lines.x, lines.y);

    let grid_color = vec3<f32>(0.3, 0.3, 0.3); // Grey color
    let final_color = grid_color * grid_intensity;

    let camera_pos = vec3(0.0, 16.0, 32.0);
    let dist = length(world_pos.xz - camera_pos.xz);
    let fade_start = 50.0;
    let fade_end = 100.0;
    let fade = smoothstep(fade_start, fade_end, dist);

    let background_color = vec3f(0.01, 0.01, 0.02);
    return vec4<f32>(final_color + background_color, 1.0 - fade);
}
