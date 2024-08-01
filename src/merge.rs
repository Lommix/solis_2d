use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, FragmentState, MultisampleState, PipelineCache, PrimitiveState,
            RenderPipelineDescriptor, SamplerBindingType, ShaderStages, ShaderType,
            TextureSampleType,
        },
        renderer::RenderDevice,
        texture::GpuImage,
        view::ViewUniform,
    },
};

use crate::{
    config::GpuConfig,
    constant::{LIGHT_FORMAT, MERGE_FORMAT},
    prelude::ComputedSize,
};

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

#[derive(Default, ShaderType)]
pub struct MergeConfig {
    index: u32,
}

pub struct MergeTargets<'a> {
    swap: bool,
    source: &'a GpuImage,
    dest: &'a GpuImage,
}

impl<'a> From<(&'a GpuImage, &'a GpuImage)> for MergeTargets<'a> {
    fn from(value: (&'a GpuImage, &'a GpuImage)) -> Self {
        Self {
            swap: false,
            source: value.0,
            dest: value.1,
        }
    }
}

impl<'a> MergeTargets<'a> {
    pub fn destination(&self, index: u32) -> &GpuImage {
        if index % 2 == 0 {
            self.source
        } else {
            self.dest
        }
    }
    pub fn source(&self, index: u32) -> &GpuImage {
        if index % 2 == 0 {
            self.dest
        } else {
            self.source
        }
    }
}
