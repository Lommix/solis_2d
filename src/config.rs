use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_resource::{ShaderType, UniformBuffer},
        renderer::{RenderDevice, RenderQueue},
    },
};

#[derive(Resource, ExtractResource, Clone, Copy, PartialEq)]
pub struct GiConfig {
    /// base ray range
    pub interval: f32,
    /// downscale scale
    pub scale_factor: f32,
    /// debug flags
    pub flags: GiFlags,
    /// the amount of cascades, defaults: 4
    pub cascade_count: u32,
    /// the starting base, defaults 8 (8x8)
    pub probe_stride: u32,
}

#[derive(Clone, Copy, ShaderType, Default)]
pub struct GpuConfig {
    pub probe_size: f32,
    pub scale_factor: f32,
    pub flags: u32,
    pub cascade_count: u32,
    pub probe_stride: u32,
}

impl Default for GiConfig {
    fn default() -> Self {
        Self {
            interval: 15.,
            scale_factor: 1.,
            flags: GiFlags::DEFAULT,
            cascade_count: 4,
            probe_stride: 8,
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ConfigBuffer(UniformBuffer<GpuConfig>);

pub fn prepare(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut buffer: ResMut<ConfigBuffer>,
    config: Res<GiConfig>,
) {
    let cfg = buffer.get_mut();
    cfg.probe_size = config.interval;
    cfg.scale_factor = config.scale_factor;
    cfg.flags = config.flags.bits();
    cfg.cascade_count = config.cascade_count;
    cfg.probe_stride = config.probe_stride;
    buffer.write_buffer(&render_device, &render_queue);
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct GiFlags: u32 {
        const DEFAULT       = 0;
        const DEBUG_VORONOI = 0x1 << 0;
        const DEBUG_SDF     = 0x1 << 1;
        const DEBUG_LIGHT   = 0x1 << 2;
        const DEBUG_BOUNCE  = 0x1 << 3;
        const DEBUG_MERGE0  = 0x1 << 4;
        const DEBUG_MERGE1  = 0x1 << 5;
    }
}
