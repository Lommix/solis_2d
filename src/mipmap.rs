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
            TextureSampleType, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        Extract,
    },
};

/// creates the final mipmap from cascade 0
#[derive(Resource)]
pub struct MipMapPipeline {
    pub layout: BindGroupLayout,
    pub id: CachedRenderPipelineId,
}

impl FromWorld for MipMapPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            "mipmap_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::NonFiltering),
                    uniform_buffer::<ComputedSize>(true),
                ),
            ),
        );

        let server = world.resource::<AssetServer>();
        let shader: Handle<Shader> = server.load("embedded://lommix_light/shaders/mipmap.wgsl");

        let id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("mipmap_pipeline".into()),
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

        dbg!(&id);

        Self { id, layout }
    }
}
