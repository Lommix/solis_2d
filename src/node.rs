use crate::{
    common::Light2dCameraTag,
    composite::CompositePipeline,
    config::ConfigBuffer,
    merge::{MergePipeline, MergeUniforms},
    mipmap::MipMapPipeline,
    probe::ProbePipeline,
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
        let start_time = std::time::Instant::now();

        // -------------------------------------------
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();
        let sdf_pipeline = world.resource::<SdfPipeline>();
        let composite_pipeline = world.resource::<CompositePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let sdf_buffers = world.resource::<SdfBuffers>();
        let post_process = view_target.post_process_write();
        let size_buffer = world.resource::<ComputedSizeBuffer>();
        let config_buffer = world.resource::<ConfigBuffer>();
        let probe_pipeline = world.resource::<ProbePipeline>();
        let merge_pipeline = world.resource::<MergePipeline>();
        let merge_unifrom = world.resource::<MergeUniforms>();
        let mipmap_pipeline = world.resource::<MipMapPipeline>();

        let Some(render_targets) = world.get_resource::<RenderTargets>() else {
            warn!("no targets");
            return Ok(());
        };

        // ------------------------------------
        // load piplines

        let Some(probe_render_pipeline) = pipeline_cache.get_render_pipeline(probe_pipeline.id)
        else {
            // warn!("probe pipeline missing");
            return Ok(());
        };

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

        let Some(merge_render_pipeline) = pipeline_cache.get_render_pipeline(merge_pipeline.id)
        else {
            // warn!("merge pipeline missing");
            return Ok(());
        };

        let Some(mipmap_render_pipeline) = pipeline_cache.get_render_pipeline(mipmap_pipeline.id)
        else {
            warn!("mipmap pipeline missing");
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

        let (Some(sdf_target), Some(probe_target), Some(light_target), Some(bounce_target)) = (
            gpu_images.get(&render_targets.sdf_target),
            gpu_images.get(&render_targets.probe_target),
            gpu_images.get(&render_targets.light_target),
            gpu_images.get(&render_targets.bounce_target),
        ) else {
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
        // probe

        let probe_bind_group = render_context.render_device().create_bind_group(
            Some("probe_bind_group".into()),
            &probe_pipeline.layout,
            &BindGroupEntries::sequential((
                view_uniform_binding.clone(),
                &sdf_target.texture_view,
                &sdf_target.sampler,
                size_binding.clone(),
                config_binding.clone(),
            )),
        );
        {
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("probe_pass".into()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &probe_target.texture_view,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_render_pipeline(probe_render_pipeline);
            render_pass.set_bind_group(0, &probe_bind_group, &[view_offset.offset]);
            render_pass.draw(0..3, 0..1);
        }

        // ---------------------------------------------------------------
        // merge probes
        // small to high resolution ...
        for i in 0..merge_targets.len() {
            let last_index = i.checked_sub(1).unwrap_or(merge_targets.len() - 1);

            let merge_bind_group = render_context.render_device().create_bind_group(
                Some("merge_bind_group".into()),
                &merge_pipeline.layout,
                &BindGroupEntries::sequential((
                    &probe_target.texture_view,
                    &probe_target.sampler,
                    &merge_targets[last_index].texture_view,
                    &merge_targets[last_index].sampler,
                    size_binding.clone(),
                    config_binding.clone(),
                    merge_uniform_binding.clone(),
                )),
            );

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("merge_pass".into()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &merge_targets[i].texture_view,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let offset = merge_unifrom.offsets[i];

            render_pass.set_render_pipeline(merge_render_pipeline);
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
                &merge_targets.last().unwrap().texture_view,
                &merge_targets.last().unwrap().sampler,
                size_binding.clone(),
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

            render_pass.set_render_pipeline(mipmap_render_pipeline);
            render_pass.set_bind_group(0, &mipmap_bind_group, &[0]);
            render_pass.draw(0..3, 0..1);
        }

        // ---------------------------------------------------------------
        // composite

        let composite_bind_group = render_context.render_device().create_bind_group(
            Some("composite_bind_group".into()),
            &composite_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &sdf_target.sampler, //@todo: fix this
                &mipmap_target.texture_view,
                &mipmap_target.sampler,
                &sdf_target.texture_view,
                &sdf_target.sampler,
                &bounce_target.texture_view,
                &bounce_target.sampler,
                &probe_target.texture_view,
                &probe_target.sampler,
                &merge_targets.last().unwrap().texture_view,
                &merge_targets.last().unwrap().sampler,
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

        let elsaped = start_time.elapsed().as_secs_f64();
        // info!("pass time: {}", elsaped * 1000.);

        Ok(())
    }
}
