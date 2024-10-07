use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            DynamicUniformBuffer, Extent3d, ShaderType, TextureDescriptor, TextureDimension,
            TextureUsages, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{CachedTexture, TextureCache},
        view::ViewTarget,
    },
};

use crate::{cascade::Probe, constant::CASCADE_FORMAT};

//@dep
#[derive(Component, Default, ExtractComponent, Clone, Copy)]
pub struct Light2dCameraTag;

#[derive(Bundle, Default)]
pub struct RadianceCameraBundle {
    pub camera_bundle: Camera2dBundle,
    pub radiance_cfg: RadianceConfig,
    pub radiance_debug: RadianceDebug,
}

#[derive(Component, ExtractComponent, Clone)]
pub struct RadianceConfig {
    pub interval: f32,
    pub scale_factor: f32,
    pub cascade_count: u32,
    pub probe_base: u32,
}

impl Default for RadianceConfig {
    fn default() -> Self {
        Self {
            interval: 1.,
            scale_factor: 1.,
            cascade_count: 6,
            probe_base: 2,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, Default, Deref, DerefMut)]
pub struct RadianceDebug(pub GiFlags);

// ------------------------------
// render world

#[derive(ShaderType, Clone, Default)]
pub struct GiGpuConfig {
    native: UVec2,
    scaled: UVec2,
    scale: f32,
    cascade_count: u32,
    flags: u32,
}

#[derive(Component, Default)]
pub struct RadianceBuffers {
    pub config_buffer: UniformBuffer<GiGpuConfig>,
    pub probe_buffer: DynamicUniformBuffer<Probe>,
    pub probe_offsets: Vec<u32>,
}

#[derive(Component)]
pub struct RadianceTargets {
    pub sdf: CachedTexture,
    pub merge0: CachedTexture,
    pub merge1: CachedTexture,
}

pub(crate) fn prepare_config(
    views: Query<(Entity, &ViewTarget, &RadianceConfig, Option<&RadianceDebug>)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut cmd: Commands,
) {
    views.iter().for_each(|(entity, view_target, cfg, flags)| {
        let target_size = view_target.main_texture().size();
        let native = Vec2::new(target_size.width as f32, target_size.height as f32);
        let scaled = native / cfg.scale_factor;


        let mut config_buffer = UniformBuffer::<GiGpuConfig>::default();
        config_buffer.get_mut().native = native.as_uvec2();
        config_buffer.get_mut().scaled = scaled.as_uvec2();
        config_buffer.get_mut().cascade_count = cfg.cascade_count;
        config_buffer.get_mut().scale = cfg.scale_factor;
        config_buffer.get_mut().flags = flags.map(|f| f.0.bits()).unwrap_or_default();
        config_buffer.write_buffer(&render_device, &render_queue);

        let mut probe_buffer = DynamicUniformBuffer::default();
        let mut probe_offsets = vec![];
        for c in 0..cfg.cascade_count {
            let index = cfg.cascade_count - 1 - c;
            let probe = Probe {
                cascade_count: cfg.cascade_count,
                cascade_index: index,
                cascade_interval: cfg.interval,
                probe_base: cfg.probe_base,
            };
            probe_offsets.push(probe_buffer.push(&probe));
        }

        probe_buffer.write_buffer(&render_device, &render_queue);
        cmd.entity(entity).insert(RadianceBuffers {
            config_buffer,
            probe_buffer,
            probe_offsets,
        });
    });
}

pub(crate) fn prepare_textures(
    views: Query<(Entity, &ViewTarget, &RadianceConfig)>,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    mut cmd: Commands,
) {
    views.iter().for_each(|(entity, view_target, cfg)| {
        let mut size = view_target.main_texture().size();
        size.depth_or_array_layers = 1;
        size.width = (size.width as f32 / cfg.scale_factor) as u32;
        size.height = (size.height as f32 / cfg.scale_factor) as u32;

        let mut new_texture = |extent: Extent3d| {
            texture_cache.get(
                &render_device,
                TextureDescriptor {
                    label: Some("radiance_mipmap_texture"),
                    size: extent,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: CASCADE_FORMAT,
                    usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                },
            )
        };

        let merge0 = new_texture(size);
        let merge1 = new_texture(size);
        let sdf = new_texture(size);

        cmd.entity(entity).insert(RadianceTargets {
            merge0,
            merge1,
            sdf,
        });
    });
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct GiFlags: u32 {
        const DEFAULT       = 0;
        const DEBUG_SDF     = 0x1 << 0;
        const DEBUG_VORONOI = 0x1 << 1;
        const DEBUG_MERGE0  = 0x1 << 3;
        const DEBUG_MERGE1  = 0x1 << 4;
    }
}
