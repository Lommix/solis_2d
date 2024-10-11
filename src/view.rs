use crate::{constant::CASCADE_FORMAT, radiance::Probe};
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

/// screen space radiance cascade
/// 2D camera bundle
#[derive(Bundle, Default)]
pub struct RadianceCameraBundle {
    pub camera_bundle: Camera2dBundle,
    pub radiance_cfg: RadianceConfig,
    pub radiance_debug: RadianceDebug,
}

/// radiance cascade configuration
#[derive(Component, ExtractComponent, Clone)]
pub struct RadianceConfig {
    /// ray base range
    pub interval: f32,
    /// screen space scale factor
    pub scale_factor: f32,
    /// max cascade count
    pub cascade_count: u32,
    /// probe base, base*base = angular resolution
    pub probe_base: u32,
    /// highlighting edges
    pub edge_hightlight: f32,
    /// light z pos
    pub light_z: f32,
}

impl Default for RadianceConfig {
    fn default() -> Self {
        Self {
            interval: 6.,
            scale_factor: 1.,
            cascade_count: 6,
            probe_base: 1,
            edge_hightlight: 1.,
            light_z: 2.,
        }
    }
}

/// debug component, enable debug flags
#[derive(Component, ExtractComponent, Clone, Default, Deref, DerefMut)]
pub struct RadianceDebug(pub GiFlags);

impl RadianceDebug {
    pub const NORMALS: RadianceDebug = RadianceDebug(GiFlags::APPLY_NORMALS);
}

// ------------------------------
// render world

#[derive(ShaderType, Clone, Default)]
pub struct GiGpuConfig {
    native: UVec2,
    scaled: UVec2,
    probe_base: u32,
    interval: f32,
    scale: f32,
    cascade_count: u32,
    flags: u32,
    edge_hightlight: f32,
    light_z: f32,
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
    pub mipmap: CachedTexture,
}

#[derive(Component, ExtractComponent, Clone, Default, Deref, DerefMut)]
pub struct NormalTarget(pub Handle<Image>);

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
        let config = config_buffer.get_mut();
        config.native = native.as_uvec2();
        config.scaled = scaled.as_uvec2();
        config.cascade_count = cfg.cascade_count;
        config.scale = cfg.scale_factor;
        config.flags = flags.map(|f| f.0.bits()).unwrap_or_default();
        config.probe_base = cfg.probe_base;
        config.interval = cfg.interval;
        config.edge_hightlight = cfg.edge_hightlight;
        config.light_z = cfg.light_z;
        config_buffer.write_buffer(&render_device, &render_queue);

        let mut probe_buffer = DynamicUniformBuffer::default();
        let mut probe_offsets = vec![];
        for c in 0..cfg.cascade_count {
            let index = cfg.cascade_count - 1 - c;
            let probe = Probe {
                cascade_index: index,
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
        let mut scaled_size = view_target.main_texture().size();
        scaled_size.depth_or_array_layers = 1;
        scaled_size.width = (scaled_size.width as f32 / cfg.scale_factor) as u32;
        scaled_size.height = (scaled_size.height as f32 / cfg.scale_factor) as u32;
        scaled_size.width += scaled_size.width % 2;
        scaled_size.height += scaled_size.height % 2;

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

        let merge0 = new_texture(scaled_size);
        let merge1 = new_texture(scaled_size);
        let sdf = new_texture(scaled_size);
        let mipmap_size = Extent3d {
            width: (scaled_size.width + cfg.probe_base + 1) / cfg.probe_base,
            height: (scaled_size.height + cfg.probe_base + 1) / cfg.probe_base,
            depth_or_array_layers: 1,
        };

        let mipmap = new_texture(mipmap_size);

        cmd.entity(entity).insert(RadianceTargets {
            merge0,
            merge1,
            sdf,
            mipmap,
        });
    });
}

bitflags::bitflags! {
    #[derive(Clone, Default, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct GiFlags: u32 {
        const DEFAULT       = 0;
        const DEBUG_SDF     = 0x1 << 0;
        const DEBUG_VORONOI = 0x1 << 1;
        const DEBUG_MERGE0  = 0x1 << 3;
        const DEBUG_MERGE1  = 0x1 << 4;
        const APPLY_NORMALS = 0x1 << 5;
    }
}
