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

use crate::{
    bounce::BouncePipeline,
    common::Light2dCameraTag,
    composite::CompositePipeline,
    config::ConfigBuffer,
    light::{LightBuffers, LightPipeline},
    merge::{MergePipeline, MergeTargets},
    probe::ProbePipeline,
    sdf::{SdfBuffers, SdfPipeline},
    size::ComputedSizeBuffer,
    targets::RenderTargets,
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
        let light_pipeline = world.resource::<LightPipeline>();
        let light_buffers = world.resource::<LightBuffers>();
        let composite_pipeline = world.resource::<CompositePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let sdf_buffers = world.resource::<SdfBuffers>();
        let post_process = view_target.post_process_write();
        let size_buffer = world.resource::<ComputedSizeBuffer>();
        let config_buffer = world.resource::<ConfigBuffer>();
        let bounce_pipeline = world.resource::<BouncePipeline>();
        let probe_pipeline = world.resource::<ProbePipeline>();
        let merge_pipeline = world.resource::<MergePipeline>();

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

        // ------------------------------------

        let (
            Some(view_uniform_binding),
            Some(circle_binding),
            Some(rect_binding),
            // Some(light_binding),
            Some(size_binding),
            Some(config_binding),
        ) = (
            world.resource::<ViewUniforms>().uniforms.binding(),
            sdf_buffers.circle_buffer.binding(),
            sdf_buffers.rect_buffer.binding(),
            // light_buffers.point_light_buffer.binding(),
            size_buffer.binding(),
            config_buffer.binding(),
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
        //
        // info!("---------------------------------------");
        for i in 0..merge_targets.len() {
            let last_index = (merge_targets.len() - i) % merge_targets.len();
            let render_index = merge_targets.len() - 1 - i;

            // info!("last {last_index}, this {render_index}");

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
                )),
            );

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("merge_pass".into()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &merge_targets[render_index].texture_view,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_render_pipeline(merge_render_pipeline);
            render_pass.set_bind_group(0, &merge_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // ---------------------------------------------------------------
        // calculate light

        // let light_bind_group = render_context.render_device().create_bind_group(
        //     Some("light_bind_group".into()),
        //     &light_pipeline.layout,
        //     &BindGroupEntries::sequential((
        //         view_uniform_binding.clone(),
        //         &sdf_target.texture_view,
        //         &sdf_target.sampler,
        //         light_binding,
        //         size_binding.clone(),
        //         config_binding.clone(),
        //     )),
        // );
        //
        // {
        //     let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        //         label: Some("light_pass".into()),
        //         color_attachments: &[Some(RenderPassColorAttachment {
        //             view: &light_target.texture_view,
        //             resolve_target: None,
        //             ops: Operations::default(),
        //         })],
        //         depth_stencil_attachment: None,
        //         timestamp_writes: None,
        //         occlusion_query_set: None,
        //     });
        //
        //     render_pass.set_render_pipeline(light_render_pipeline);
        //     render_pass.set_bind_group(0, &light_bind_group, &[view_offset.offset]);
        //     render_pass.draw(0..3, 0..1);
        // }

        // ---------------------------------------------------------------
        // bounce light

        // let bounce_bind_group = render_context.render_device().create_bind_group(
        //     Some("bounce_bind_group".into()),
        //     &bounce_pipeline.layout,
        //     &BindGroupEntries::sequential((
        //         view_uniform_binding,
        //         &sdf_target.texture_view,
        //         &sdf_target.sampler,
        //         &light_target.texture_view,
        //         &light_target.sampler,
        //         size_binding.clone(),
        //         config_binding.clone(),
        //     )),
        // );
        // {
        //     let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        //         label: Some("bounce_pass".into()),
        //         color_attachments: &[Some(RenderPassColorAttachment {
        //             view: &bounce_target.texture_view,
        //             resolve_target: None,
        //             ops: Operations::default(),
        //         })],
        //         depth_stencil_attachment: None,
        //         timestamp_writes: None,
        //         occlusion_query_set: None,
        //     });
        //
        //     render_pass.set_render_pipeline(bounce_render_pipeline);
        //     render_pass.set_bind_group(0, &bounce_bind_group, &[view_offset.offset]);
        //     render_pass.draw(0..3, 0..1);
        // }

        // ---------------------------------------------------------------
        // composite

        let composite_bind_group = render_context.render_device().create_bind_group(
            Some("composite_bind_group".into()),
            &composite_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &sdf_target.sampler, //@todo: fix this
                &light_target.texture_view,
                &light_target.sampler,
                &sdf_target.texture_view,
                &sdf_target.sampler,
                &bounce_target.texture_view,
                &bounce_target.sampler,
                &probe_target.texture_view,
                &probe_target.sampler,
                &merge_targets[0].texture_view,
                &merge_targets[0].sampler,
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
