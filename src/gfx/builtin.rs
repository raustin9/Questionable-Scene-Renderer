use crate::gfx::texture;

pub fn write_gbuffers_pass_record_commands(
    encoder: &mut wgpu::CommandEncoder, 
    pipeline: &wgpu::RenderPipeline,
    normal_texture: &texture::Texture,
    albedo_texture: &texture::Texture,
    depth_texture: &texture::Texture,
    scene_uniform_bind_group: &wgpu::BindGroup,
    vertex_buffer: &wgpu::Buffer,
    num_vertices: u32,
) {
    let mut gbuffer_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("GBuffer pass"),
        color_attachments: &[
            Some(wgpu::RenderPassColorAttachment {
                view: &normal_texture.view(),
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                }
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &albedo_texture.view(),
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                }
            }),
        ],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth_texture.view(),
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });

    gbuffer_pass.set_pipeline(&pipeline);
    gbuffer_pass.set_bind_group(0, scene_uniform_bind_group, &[]);
    gbuffer_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    gbuffer_pass.draw(0..num_vertices, 0..1);
}

pub fn deferred_pass_record_commands(
    encoder: &mut wgpu::CommandEncoder, 
    pipeline: &wgpu::RenderPipeline,
    gbuffer_textures_bind_group: &wgpu::BindGroup,
    camera_bind_group: &wgpu::BindGroup,
    view: &wgpu::TextureView,
) {
    let mut deferred_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("deferred_pass"),
        color_attachments: &[
            Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0
                    }),
                    store: wgpu::StoreOp::Store
                },
            }),
        ],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });
    deferred_pass.set_pipeline(&pipeline);
    deferred_pass.set_bind_group(0, gbuffer_textures_bind_group, &[]);
    deferred_pass.set_bind_group(1, camera_bind_group, &[]);
    deferred_pass.draw(0..6, 0..1); // 6 vertices since this pass in only drawing a quad to the
                                    // screen
}
