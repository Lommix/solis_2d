use bevy::{
    prelude::*,
    render::{
        render_resource::{ShaderType, UniformBuffer},
        renderer::{RenderDevice, RenderQueue},
        Extract,
    },
    window::{PrimaryWindow, WindowResized},
};

use crate::{constant, prelude::GiConfig, targets::RenderTargets};

#[derive(Resource, Clone, Debug, ShaderType, Default)]
pub struct ComputedSize {
    pub native: Vec2,
    pub scaled: Vec2,
    pub probe: Vec2,
    pub factor: f32,
    pub cascade_count: u32,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ComputedSizeBuffer(UniformBuffer<ComputedSize>);

#[rustfmt::skip]
impl ComputedSize {
    pub fn from_window(
        window: &Window,
        scale: f32,
        cascade_count : u32,
    ) -> Self {
        let width = window.physical_width();
        let height = window.physical_height();
        let size = Vec2::new(width as f32, height as f32);
        let downscaled_size = (size / scale) - size % scale;
        let probe_size = (downscaled_size + Vec2::splat(2.) - (downscaled_size % 2.)) * Vec2::new(4., 1.);
        Self {
            native: size,
            scaled: downscaled_size,
            probe: probe_size,
            factor: constant::SDF_DOWNSCALE_FACTOR,
            cascade_count,
        }
    }
}

pub fn extract_size(mut buffer: ResMut<ComputedSizeBuffer>, size: Extract<Res<ComputedSize>>) {
    let buffer = buffer.get_mut();
    buffer.native = size.native;
    buffer.scaled = size.scaled;
    buffer.factor = size.factor;
    buffer.probe = size.probe;
    buffer.cascade_count = size.cascade_count;
}

pub fn prepare_bindgroup(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut buffer: ResMut<ComputedSizeBuffer>,
) {
    buffer.write_buffer(&render_device, &render_queue);
}

pub fn on_startup(
    mut cmd: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    config: Res<GiConfig>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };

    let computed_size = ComputedSize::from_window(&window, config.scale, config.cascade_count);
    let targets = RenderTargets::from_size(&computed_size, &mut images);
    cmd.insert_resource(targets);
    cmd.insert_resource(computed_size);
}

pub fn on_win_resize(
    mut events: EventReader<WindowResized>,
    mut cmd: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    config: Res<GiConfig>,
) {
    let Some(event) = events.read().next() else {
        return;
    };

    let Ok(window) = window.get(event.window) else {
        return;
    };

    let computed_size = ComputedSize::from_window(&window, config.scale, config.cascade_count);
    let targets = RenderTargets::from_size(&computed_size, &mut images);
    cmd.insert_resource(computed_size);
    cmd.insert_resource(targets);
}
