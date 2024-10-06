use crate::{
    common::Light2dCameraTag,
    composite::CompositePipeline,
    config::{ConfigBuffer, GiConfig},
    merge::{MergePipeline, ProbeBuffer},
    mipmap::MipMapPipeline,
    sdf::{SdfBuffers, SdfPipeline},
    size::ComputedSizeBuffer,
    targets::RenderTargets,
};
use bevy::{
    ecs::{query::QueryItem, system::lifetimeless::Read},
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{self, NodeRunError, RenderGraphContext, RenderLabel},
        render_resource::{
            BindGroupEntries, Operations, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor,
        },
        renderer::RenderContext,
        texture::GpuImage,
        view::{ViewTarget, ViewUniformOffset, ViewUniforms},
    },
};

#[derive(Hash, PartialEq, Eq, Clone, Copy, RenderLabel, Debug)]
pub struct LightNodeLabel;

#[derive(Default)]
pub struct LightNode;
impl render_graph::ViewNode for LightNode {
    type ViewQuery = (
        Read<ViewUniformOffset>,
        Read<ViewTarget>,
        Read<Light2dCameraTag>,
    );

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (view_offset, view_target, _): QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        // -------------------------------------------
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();
        let sdf_pipeline = world.resource::<SdfPipeline>();
        let composite_pipeline = world.resource::<CompositePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let sdf_buffers = world.resource::<SdfBuffers>();
        let post_process = view_target.post_process_write();
        let size_buffer = world.resource::<ComputedSizeBuffer>();
        let config_buffer = world.resource::<ConfigBuffer>();
        let merge_pipeline = world.resource::<MergePipeline>();
        let merge_unifrom = world.resource::<ProbeBuffer>();
        let mipmap_pipeline = world.resource::<MipMapPipeline>();
        let config = world.resource::<GiConfig>();

        let Some(render_targets) = world.get_resource::<RenderTargets>() else {
            warn!("no targets");
            return Ok(());
        };

        // ------------------------------------
        // load piplines

        let Some(sdf_render_pipeline) = pipeline_cache.get_render_pipeline(sdf_pipeline.id) else {
            // warn!("sdf pipeline missing");
            return Ok(());
        };

        let Some(composite_render_pipeline) =
            pipeline_cache.get_render_pipeline(composite_pipeline.id)
        else {
            // warn!("composite pipeline missing");
            return Ok(());
        };

        let Some(no_merge_render_pipeline) =
            pipeline_cache.get_render_pipeline(merge_pipeline.no_merge_id)
        else {
            // warn!("merge pipeline missing");
            return Ok(());
        };

        let Some(merge_render_pipeline) =
            pipeline_cache.get_render_pipeline(merge_pipeline.merge_id)
        else {
            // warn!("merge pipeline missing");
            return Ok(());
        };

        let Some(mipmap_render_pipeline) = pipeline_cache.get_render_pipeline(mipmap_pipeline.id)
        else {
            // warn!("mipmap pipeline missing");
            return Ok(());
        };

        // ------------------------------------

        let (
            Some(view_uniform_binding),
            Some(circle_binding),
            Some(rect_binding),
            Some(size_binding),
            Some(config_binding),
            Some(merge_uniform_binding),
        ) = (
            world.resource::<ViewUniforms>().uniforms.binding(),
            sdf_buffers.circle_buffer.binding(),
            sdf_buffers.rect_buffer.binding(),
            size_buffer.binding(),
            config_buffer.binding(),
            merge_unifrom.buffer.binding(),
        )
        else {
            warn!("binding missing");
            return Ok(());
        };

        let Some(sdf_target) = gpu_images.get(&render_targets.sdf_target) else {
            warn!("failed to load targets");
            return Ok(());
        };

        let Some(mipmap_target) = gpu_images.get(&render_targets.light_mipmap_target) else {
            warn!("failed to load mipmap target");
            return Ok(());
        };

        let merge_targets = render_targets
            .sorted_merge_targets(&gpu_images)
            .collect::<Vec<_>>();

        // ---------------------------------------------------------------
        // create sdf texture

        let sdf_bind_group = render_context.render_device().create_bind_group(
            Some("sdf_bind_group".into()),
            &sdf_pipeline.layout,
            &BindGroupEntries::sequential((
                view_uniform_binding.clone(),
                circle_binding,
                rect_binding,
                size_binding.clone(),
            )),
        );
        {
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("sdf_pass".into()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &sdf_target.texture_view,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_render_pipeline(sdf_render_pipeline);
            render_pass.set_bind_group(0, &sdf_bind_group, &[view_offset.offset]);
            render_pass.draw(0..3, 0..1);
        }

        // ---------------------------------------------------------------
        // merge probes
        // small to high resolution ...
        for i in 0..(config.cascade_count as usize) {
            let last_index = i + 1;
            let merge_bind_group = render_context.render_device().create_bind_group(
                Some("merge_bind_group".into()),
                &merge_pipeline.layout,
                &BindGroupEntries::sequential((
                    &sdf_target.texture_view,
                    &composite_pipeline.linear_sampler,
                    &merge_targets[i % 2].texture_view,
                    size_binding.clone(),
                    config_binding.clone(),
                    merge_uniform_binding.clone(),
                )),
            );
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("merge_pass".into()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &merge_targets[(i + 1) % 2].texture_view,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let offset = merge_unifrom.offsets[i];

            // if i == 0 {
            //     render_pass.set_render_pipeline(no_merge_render_pipeline);
            // } else {
            render_pass.set_render_pipeline(merge_render_pipeline);
            // }

            render_pass.set_bind_group(0, &merge_bind_group, &[offset]);
            render_pass.draw(0..3, 0..1);
        }

        // -------------------------------------------
        // create light mimap
        // ---------------------------------------------------------------

        let mipmap_bind_group = render_context.render_device().create_bind_group(
            Some("mipmap_bind_group".into()),
            &mipmap_pipeline.layout,
            &BindGroupEntries::sequential((
                &merge_targets[0].texture_view,
                merge_uniform_binding.clone(),
            )),
        );
        {
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("mipmap_pass".into()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &mipmap_target.texture_view,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let offset = merge_unifrom.offsets.last().unwrap();
            render_pass.set_render_pipeline(mipmap_render_pipeline);
            render_pass.set_bind_group(0, &mipmap_bind_group, &[*offset]);
            render_pass.draw(0..3, 0..1);
        }

        // ---------------------------------------------------------------
        // composite

        let composite_bind_group = render_context.render_device().create_bind_group(
            Some("composite_bind_group".into()),
            &composite_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &mipmap_target.texture_view,
                &sdf_target.texture_view,
                &merge_targets[0].texture_view,
                &merge_targets[1].texture_view,
                &composite_pipeline.point_sampler,
                &composite_pipeline.linear_sampler,
                // merge filter
                config_binding,
                size_binding,
            )),
        );

        {
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("composite_pass".into()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &post_process.destination,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_render_pipeline(composite_render_pipeline);
            render_pass.set_bind_group(0, &composite_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}
