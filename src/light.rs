use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        render_resource::{
            binding_types::{sampler, storage_buffer_read_only, texture_2d, uniform_buffer},
            BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, FilterMode, FragmentState, MultisampleState, PipelineCache,
            PrimitiveState, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, ShaderType, StorageBuffer, TextureSampleType,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{ImageSampler, ImageSamplerDescriptor},
        view::ViewUniform,
        Extract,
    },
};

use crate::{config::GpuConfig, constant::LIGHT_FORMAT, prelude::ComputedSize};

#[derive(Resource)]
pub struct LightPipeline {
    pub layout: BindGroupLayout,
    pub id: CachedRenderPipelineId,
    pub sampler: Sampler,
    pub rad_sampler: Sampler,
}

impl FromWorld for LightPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "light_pipelin_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    uniform_buffer::<ViewUniform>(true),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    storage_buffer_read_only::<GpuLights>(false),
                    uniform_buffer::<ComputedSize>(false),
                    uniform_buffer::<GpuConfig>(false),
                ),
            ),
        );

        let main_sampler = render_device.create_sampler(&SamplerDescriptor::default());
        let radiance_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("radiance_sampler"),
            mipmap_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            ..default()
        });

        let server = world.resource::<AssetServer>();
        // let shader = server.load("light.wgsl");
        let shader = server.load("embedded://lommix_light/shaders/light.wgsl");
        // let shader = super::constant::LIGHT_SHADER;

        let id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("sdf_pipeline".into()),
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
                            format: LIGHT_FORMAT,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                });

        Self {
            layout,
            id,
            sampler: main_sampler,
            rad_sampler: radiance_sampler,
        }
    }
}

// --------------------------------------
// Light Sources

#[derive(Component)]
pub struct PointLight2d {
    pub intensity: f32,
    pub range: f32,
}

#[derive(ShaderType, Default, Clone)]
pub struct ExtractedPointLight2d {
    pub position: Vec2,
    pub intensity: f32,
    pub range: f32,
}

#[derive(ShaderType, Default, Clone)]
pub struct GpuLights {
    pub count: u32,
    #[size(runtime)]
    pub data: Vec<ExtractedPointLight2d>,
}

#[derive(Resource, Default)]
pub struct LightBuffers {
    pub point_light_buffer: StorageBuffer<GpuLights>,
}

pub fn extract_lights(
    point_lights: Extract<Query<(&PointLight2d, &GlobalTransform, &ViewVisibility)>>,
    mut buffers: ResMut<LightBuffers>,
) {
    let extracted_lights: Vec<_> = point_lights
        .iter()
        .flat_map(|(light, global, visibilty)| {
            if !visibilty.get() {
                return None;
            }

            Some(ExtractedPointLight2d {
                position: global.translation().truncate(),
                intensity: light.intensity,
                range: light.range,
            })
        })
        .collect();

    let point_light_buffer = buffers.point_light_buffer.get_mut();
    point_light_buffer.count = extracted_lights.len() as u32;
    point_light_buffer.data.extend(extracted_lights);
}
pub fn prepare_light_buffers(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut prepared: ResMut<LightBuffers>,
) {
    prepared
        .point_light_buffer
        .write_buffer(&render_device, &render_queue);
}
