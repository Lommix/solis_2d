use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_resource::{ShaderType, UniformBuffer},
        renderer::{RenderDevice, RenderQueue},
    },
};

#[derive(Resource, ExtractResource, Clone, Copy)]
pub struct GiConfig {
    pub sample_count: u32,
    pub probe_size: f32,
    pub scale: f32,
    pub flags: GiFlags,
    pub cascade_count: u32,
}

#[derive(Clone, Copy, ShaderType, Default)]
pub struct GpuConfig {
    pub sample_count: u32,
    pub probe_size: f32,
    pub sdf_scale: f32,
    pub flags: u32,
    pub cascade_count: u32,
}

impl Default for GiConfig {
    fn default() -> Self {
        Self {
            sample_count: 32,
            probe_size: 0.1,
            scale: 2.,
            flags: GiFlags::DEFAULT,
            cascade_count: 4,
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
    cfg.sample_count = config.sample_count;
    cfg.probe_size = config.probe_size;
    cfg.sdf_scale = config.scale;
    cfg.flags = config.flags.bits();
    cfg.cascade_count = config.cascade_count;
    buffer.write_buffer(&render_device, &render_queue);
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct GiFlags: u32 {
        const DEFAULT       = 0;
        const DEBUG_VORONOI = 0x1;
        const DEBUG_SDF     = 0x2;
        const DEBUG_LIGHT   = 0x4;
        const DEBUG_BOUNCE  = 0x8;
        const DEBUG_PROBE   = 0x10;
        const DEBUG_MERGE   = 0x20;
    }
}
