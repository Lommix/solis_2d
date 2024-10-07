use crate::{camera::GiGpuConfig, constant::CASCADE_FORMAT};
use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, FilterMode, FragmentState, MultisampleState, PipelineCache,
            PrimitiveState, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, ShaderType, TextureSampleType,
        },
        renderer::RenderDevice,
    },
};

/// folding the cascades back into one
#[derive(Resource)]
pub struct CascadePipeline {
    pub cascade_layout: BindGroupLayout,
    pub cascade_id: CachedRenderPipelineId,
    pub composite_id: CachedRenderPipelineId,
    pub composite_layout: BindGroupLayout,
    pub radiance_sampler: Sampler,
    pub point_sampler: Sampler,
}

impl FromWorld for CascadePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let cascade_layout = create_cascade_layout(&render_device);
        let composite_layout = create_composite_layout(&render_device);
        let server = world.resource_ref::<AssetServer>();
        let cascade_shader = server.load("embedded://lommix_light/shaders/cascade.wgsl");
        let composite_shader = server.load("embedded://lommix_light/shaders/composite.wgsl");
        let cache = world.resource::<PipelineCache>();

        let cascade_id = cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("merge_pipline".into()),
            layout: vec![cascade_layout.clone()],
            push_constant_ranges: vec![],
            vertex: fullscreen_shader_vertex_state(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                shader: cascade_shader,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: CASCADE_FORMAT,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
        });

        let composite_id = cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("composite_pipeline".into()),
            layout: vec![composite_layout.clone()],
            push_constant_ranges: vec![],
            vertex: fullscreen_shader_vertex_state(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                shader: composite_shader,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: CASCADE_FORMAT,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
        });

        let radiance_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("radiance sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.,
            lod_max_clamp: 32.,
            ..default()
        });

        Self {
            cascade_id,
            cascade_layout,
            composite_id,
            composite_layout,
            radiance_sampler,
            point_sampler: render_device.create_sampler(&SamplerDescriptor::default()),
        }
    }
}

fn create_composite_layout(render_device: &RenderDevice) -> BindGroupLayout {
    return render_device.create_bind_group_layout(
        "composite_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                //main tex
                texture_2d(TextureSampleType::Float { filterable: true }),
                //sdf tex
                texture_2d(TextureSampleType::Float { filterable: true }),
                //merge tex 0
                texture_2d(TextureSampleType::Float { filterable: true }),
                //merge tex 1
                texture_2d(TextureSampleType::Float { filterable: true }),
                //linear sample
                sampler(SamplerBindingType::Filtering),
                //point sample
                sampler(SamplerBindingType::Filtering),
                //config
                uniform_buffer::<GiGpuConfig>(false),
            ),
        ),
    );
}
fn create_cascade_layout(render_device: &RenderDevice) -> BindGroupLayout {
    return render_device.create_bind_group_layout(
        "cascade_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                // sdf
                texture_2d(TextureSampleType::Float { filterable: true }),
                // last cascade
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<GiGpuConfig>(false),
                uniform_buffer::<Probe>(true),
            ),
        ),
    );
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
