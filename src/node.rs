use crate::{
    radiance::RadiancePipeline,
    sdf::{SdfBuffers, SdfPipeline},
    view::{NormalTarget, RadianceBuffers, RadianceConfig, RadianceTargets},
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
        texture::{FallbackImage, GpuImage},
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
        Read<RadianceBuffers>,
        Read<RadianceTargets>,
        Read<RadianceConfig>,
        Option<Read<NormalTarget>>,
    );

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (view_offset, view_target, radiance_buffers, radiance_targets, config, normal): QueryItem<
            'w,
            Self::ViewQuery,
        >,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let sdf_pipeline = world.resource::<SdfPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let sdf_buffers = world.resource::<SdfBuffers>();
        let radiance_pipline = world.resource::<RadiancePipeline>();
        let post_process = view_target.post_process_write();
        let gpu_imges = world.resource::<RenderAssets<GpuImage>>();
        let fallback_image = world.resource::<FallbackImage>();

        let normal_target = normal
            .map(|n| gpu_imges.get(&n.0))
            .flatten()
            .unwrap_or(&fallback_image.d2);

        // ------------------------------------
        // load piplines

        let Some(sdf_render_pipeline) = pipeline_cache.get_render_pipeline(sdf_pipeline.id) else {
            // warn!("sdf pipeline missing");
            return Ok(());
        };

        let Some(composite_render_pipeline) =
            pipeline_cache.get_render_pipeline(radiance_pipline.composite_id)
        else {
            // warn!("composite pipeline missing");
            return Ok(());
        };

        let Some(cascade_render_pipeline) =
            pipeline_cache.get_render_pipeline(radiance_pipline.cascade_id)
        else {
            // warn!("merge pipeline missing")
            return Ok(());
        };

        let Some(mipmap_render_pipeline) =
            pipeline_cache.get_render_pipeline(radiance_pipline.mipmap_id)
        else {
            // warn!("merge pipeline missing")
            return Ok(());
        };

        // ------------------------------------

        let Some(gi_config_binding) = radiance_buffers.config_buffer.binding() else {
            warn!("missing config");
            return Ok(());
        };

        let Some(probe_binding) = radiance_buffers.probe_buffer.binding() else {
            warn!("missing config");
            return Ok(());
        };

        let (Some(view_uniform_binding), Some(sdf_circle_binding), Some(sdf_rect_binding)) = (
            world.resource::<ViewUniforms>().uniforms.binding(),
            sdf_buffers.circle_buffer.binding(),
            sdf_buffers.rect_buffer.binding(),
        ) else {
            warn!("binding missing");
            return Ok(());
        };

        // ---------------------------------------------------------------
        // create sdf texture

        let sdf_bind_group = render_context.render_device().create_bind_group(
            Some("sdf_bind_group".into()),
            &sdf_pipeline.layout,
            &BindGroupEntries::sequential((
                view_uniform_binding.clone(),
                sdf_circle_binding,
                sdf_rect_binding,
                gi_config_binding.clone(),
            )),
        );
        {
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("sdf_pass".into()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &radiance_targets.sdf.default_view,
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
        // ping pong cascades
        for i in 0..(config.cascade_count as usize) {
            let (current_target, last_target) = if i % 2 == 0 {
                (
                    &radiance_targets.merge1.default_view,
                    &radiance_targets.merge0.default_view,
                )
            } else {
                (
                    &radiance_targets.merge0.default_view,
                    &radiance_targets.merge1.default_view,
                )
            };

            let cascade_bind_group = render_context.render_device().create_bind_group(
                Some("cascade_bind_group".into()),
                &radiance_pipline.cascade_layout,
                &BindGroupEntries::sequential((
                    &radiance_targets.sdf.default_view,
                    last_target,
                    &normal_target.texture_view,
                    &radiance_pipline.radiance_sampler,
                    gi_config_binding.clone(),
                    probe_binding.clone(),
                )),
            );

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("cascade_pass".into()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: current_target,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let offset = radiance_buffers.probe_offsets[i];
            render_pass.set_render_pipeline(cascade_render_pipeline);
            render_pass.set_bind_group(0, &cascade_bind_group, &[offset]);
            render_pass.draw(0..3, 0..1);
        }

        // ---------------------------------------------------------------
        // mipmap
        let mipmap_bind_group = render_context.render_device().create_bind_group(
            Some("mipmap_bind_group".into()),
            &radiance_pipline.mipmap_layout,
            &BindGroupEntries::sequential((
                if config.cascade_count % 2 == 0 {
                    &radiance_targets.merge0.default_view
                } else {
                    &radiance_targets.merge1.default_view
                },
                gi_config_binding.clone(),
            )),
        );
        {
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("mipmap_pass".into()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &radiance_targets.mipmap.default_view,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_render_pipeline(mipmap_render_pipeline);
            render_pass.set_bind_group(0, &mipmap_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // ---------------------------------------------------------------
        // composite

        let composite_bind_group = render_context.render_device().create_bind_group(
            Some("composite_bind_group".into()),
            &radiance_pipline.composite_layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &radiance_targets.sdf.default_view,
                &radiance_targets.merge0.default_view,
                &radiance_targets.merge1.default_view,
                &radiance_targets.mipmap.default_view,
                &normal_target.texture_view,
                &radiance_pipline.radiance_sampler,
                &radiance_pipline.point_sampler,
                gi_config_binding,
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
