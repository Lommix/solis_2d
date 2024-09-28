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
            PrimitiveState, RenderPipelineDescriptor, SamplerBindingType, ShaderStages, ShaderType,
            TextureSampleType,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};

/// folding the cascades back into one
#[derive(Resource)]
pub struct MergePipeline {
    pub layout: BindGroupLayout,
    pub id: CachedRenderPipelineId,
}

impl FromWorld for MergePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            "merge_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // probes
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::NonFiltering),
                    // merge
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::NonFiltering),
                    uniform_buffer::<ComputedSize>(false),
                    uniform_buffer::<GpuConfig>(false),
                    uniform_buffer::<MergeUniform>(true),
                ),
            ),
        );

        let server = world.resource::<AssetServer>();
        let shader = server.load("embedded://lommix_light/shaders/merge.wgsl");

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
                        shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: MERGE_FORMAT,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                });

        Self { id, layout }
    }
}

#[derive(ShaderType)]
pub struct MergeUniform {
    pub iteration: u32,
    pub target_size: Vec2,
}

#[derive(Resource, Default)]
pub struct MergeUniforms {
    pub buffer: DynamicUniformBuffer<MergeUniform>,
    pub offsets: Vec<u32>,
}

//todo: should be loaded once + on change
// impl FromWorld for MergeUniforms {
//     fn from_world(world: &mut World) -> Self {
//         let mut buffer = DynamicUniformBuffer::<MergeUniform>::default();
//         let mut offsets = Vec::new();
//
//         let render_device = world.resource::<RenderDevice>();
//         let render_queue = world.resource::<RenderQueue>();
//         let gi_config = world.resource::<GiConfig>();
//
//         if let Some(mut writer) = buffer.get_writer(4, &render_device, &render_queue) {
//             for i in 0..gi_config.cascade_count {
//                 offsets.push(writer.write(&MergeUniform {
//                     iteration: i,
//                     target_size: Vec2::ZERO,
//                 }));
//             }
//         }
//
//         MergeUniforms { buffer, offsets }
//     }
// }

pub(crate) fn prepare_uniform(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut uniforms: ResMut<MergeUniforms>,
    targets: Res<RenderTargets>,
) {
    let mut offsets = Vec::new();
    if let Some(mut writer) = uniforms.buffer.get_writer(4, &render_device, &render_queue) {
        targets
            .merge_targets
            .iter()
            .enumerate()
            .for_each(|(i, target)| {
                offsets.push(writer.write(&MergeUniform {
                    iteration: i as u32,
                    target_size: target.size,
                }));
            });
    }
    uniforms.offsets = offsets;
}

#[derive(Default, ShaderType)]
pub struct MergeConfig {
    index: u32,
}
