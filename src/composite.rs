use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, FilterMode, FragmentState, MultisampleState, PipelineCache,
            PrimitiveState, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, TextureFormat, TextureSampleType,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
    },
};

use crate::{
    config::GpuConfig,
    prelude::{ComputedSize, GiConfig},
};

#[derive(Resource)]
pub struct CompositePipeline {
    pub layout: BindGroupLayout,
    pub id: CachedRenderPipelineId,
    pub point_sampler: Sampler,
    pub linear_sampler: Sampler,
}

impl FromWorld for CompositePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "composite_pipeline_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    //main tex
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    //light mipmap tex
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    //sdf tex
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    //merge tex 0
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    //merge tex 1
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    //point sample
                    sampler(SamplerBindingType::Filtering),
                    //linear sample
                    sampler(SamplerBindingType::Filtering),
                    //config
                    uniform_buffer::<GpuConfig>(false),
                    uniform_buffer::<ComputedSize>(false),
                ),
            ),
        );

        let server = world.resource::<AssetServer>();
        // let shader = server.load("composite.wgsl");
        let shader = server.load("embedded://lommix_light/shaders/composite.wgsl");

        let point_sampler = render_device.create_sampler(&SamplerDescriptor::default());
        let linear_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("linear_sampler"),
            mipmap_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            ..default()
        });

        let id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("composite_pipeline".into()),
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
                            format: TextureFormat::Rgba16Float,
                            // format: TextureFormat::bevy_default(),
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                });

        Self {
            layout,
            id,
            point_sampler,
            linear_sampler,
        }
    }
}
