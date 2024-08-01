use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        render_resource::{
            binding_types::{sampler, storage_buffer_read_only, texture_2d, uniform_buffer},
            BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, FragmentState, MultisampleState, PipelineCache, PrimitiveState,
            RenderPipelineDescriptor, SamplerBindingType, ShaderStages, TextureSampleType,
        },
        renderer::RenderDevice,
        view::ViewUniform,
    },
};

use crate::{config::GpuConfig, constant::PROBE_FORMAT, prelude::ComputedSize};

#[derive(Resource)]
pub struct ProbePipeline {
    pub layout: BindGroupLayout,
    pub id: CachedRenderPipelineId,
}

impl FromWorld for ProbePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            "probe_pipelin_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    uniform_buffer::<ViewUniform>(true),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<ComputedSize>(false),
                    uniform_buffer::<GpuConfig>(false),
                ),
            ),
        );

        let server = world.resource::<AssetServer>();
        // let shader = server.load("light.wgsl");
        let shader = server.load("embedded://lommix_light/shaders/probes.wgsl");
        // let shader = super::constant::LIGHT_SHADER;

        let id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("probe_pipeline".into()),
                    layout: vec![layout.clone()],
                    push_constant_ranges: vec![],
                    vertex: fullscreen_shader_vertex_state(),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: PROBE_FORMAT,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                });

        Self { layout, id }
    }
}
