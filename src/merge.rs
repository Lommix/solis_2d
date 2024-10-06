use crate::{
    config::GpuConfig,
    constant::MERGE_FORMAT,
    prelude::{ComputedSize, GiConfig},
    targets::RenderTargets,
};
use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, DynamicUniformBuffer, FragmentState, MultisampleState, PipelineCache,
            PrimitiveState, RenderPipelineDescriptor, SamplerBindingType, ShaderDefVal,
            ShaderStages, ShaderType, TextureSampleType,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};

/// folding the cascades back into one
#[derive(Resource)]
pub struct MergePipeline {
    pub layout: BindGroupLayout,
    pub no_merge_id: CachedRenderPipelineId,
    pub merge_id: CachedRenderPipelineId,
}

impl FromWorld for MergePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            "merge_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // sdf
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // lerp sampler
                    sampler(SamplerBindingType::Filtering),
                    // prev cascade
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    uniform_buffer::<ComputedSize>(false),
                    uniform_buffer::<GpuConfig>(false),
                    uniform_buffer::<Probe>(true),
                ),
            ),
        );

        let server = world.resource::<AssetServer>();
        let shader = server.load("embedded://lommix_light/shaders/cascade.wgsl");

        let id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("merge_pipline".into()),
                    layout: vec![layout.clone()],
                    push_constant_ranges: vec![],
                    vertex: fullscreen_shader_vertex_state(),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    fragment: Some(FragmentState {
                        shader: shader.clone(),
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: MERGE_FORMAT,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                });

        let merge_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("merge_pipline".into()),
                    layout: vec![layout.clone()],
                    push_constant_ranges: vec![],
                    vertex: fullscreen_shader_vertex_state(),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![ShaderDefVal::Bool("MERGE".into(), true)],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: MERGE_FORMAT,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                });

        Self {
            no_merge_id: id,
            layout,
            merge_id,
        }
    }
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub struct Probe {
    /// num of cascades
    pub cascade_count: u32,
    /// index of current
    pub cascade_index: u32,
    /// interval
    pub cascade_interval: f32,
    /// min probe size
    pub probe_base: u32,
}

#[derive(Resource, Default)]
pub struct ProbeBuffer {
    pub buffer: DynamicUniformBuffer<Probe>,
    pub offsets: Vec<u32>,
}

pub(crate) fn prepare_uniform(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    gi_cfg: Res<GiConfig>,
    mut uniforms: ResMut<ProbeBuffer>,
) {
    let mut offsets = Vec::new();
    if let Some(mut writer) =
        uniforms
            .buffer
            .get_writer(gi_cfg.cascade_count as usize, &render_device, &render_queue)
    {
        for c in 0..gi_cfg.cascade_count {
            let i = gi_cfg.cascade_count - 1 - c;
            let probe = Probe {
                cascade_count: gi_cfg.cascade_count,
                cascade_index: i,
                cascade_interval: gi_cfg.interval,
                probe_base: gi_cfg.probe_stride,
            };

            offsets.push(writer.write(&probe));
        }
    }
    uniforms.offsets = offsets;
}

#[derive(Default, ShaderType)]
pub struct MergeConfig {
    index: u32,
}
