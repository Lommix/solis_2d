#![allow(unused)]

use bevy::{
    asset::{embedded_asset, load_internal_asset},
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        extract_resource::ExtractResourcePlugin,
        render_graph::{RenderGraphApp, ViewNodeRunner},
        render_resource::Source,
        Render, RenderApp, RenderSet,
    },
    time::common_conditions::on_timer,
    window::WindowResized,
};
use size::ComputedSizeBuffer;
use std::{path::PathBuf, time::Duration};

mod bounce;
mod common;
mod composite;
mod config;
mod constant;
mod light;
mod merge;
mod mipmap;
mod node;
mod probe;
mod sdf;
mod size;
mod targets;

pub mod prelude {
    pub use super::common::Light2dCameraTag;
    pub use super::config::{GiConfig, GiFlags};
    pub use super::light::PointLight2d;
    pub use super::sdf::{Emitter, Occluder, SdfShape};
    pub use super::size::{ComputedSize, ResizeEvent};
    pub use super::targets::RenderTargets;
    pub use super::LightPlugin;
}

#[derive(Default)]
pub struct LightPlugin {
    settings: config::GiConfig,
}

#[derive(Resource)]
struct AssetHolder(Vec<Handle<Shader>>);

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        #[rustfmt::skip]
        app
            .insert_resource(self.settings.clone())
            .add_plugins((
                ExtractResourcePlugin::<targets::RenderTargets>::default(),
                ExtractResourcePlugin::<config::GiConfig>::default(),
                ExtractComponentPlugin::<common::Light2dCameraTag>::default(),
            ))
            .add_event::<size::ResizeEvent>()
            .observe(size::resize)
            .add_systems(PreStartup, size::on_startup)
            .add_systems(Update, size::on_win_resize.run_if(on_event::<WindowResized>()),
        );

        // adds some hot reloading for dev
        #[cfg(debug_assertions)]
        app.add_systems(Last, watch.run_if(on_timer(Duration::from_millis(50))));

        // ---------------
        // fix later
        // bevy's wgsl definition do not work with embedded assets
        load_internal_asset!(
            app,
            constant::COMMON_SHADER,
            "shaders/common.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            constant::SHAPES_SHADER,
            "shaders/shapes.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            constant::RAYMARCH_SHADER,
            "shaders/raymarch.wgsl",
            Shader::from_wgsl
        );

        //embedd
        embedded_asset!(app, "shaders/merge.wgsl");
        embedded_asset!(app, "shaders/probes.wgsl");
        embedded_asset!(app, "shaders/light.wgsl");
        embedded_asset!(app, "shaders/bounce.wgsl");
        embedded_asset!(app, "shaders/sdf.wgsl");
        embedded_asset!(app, "shaders/composite.wgsl");
        embedded_asset!(app, "shaders/mipmap.wgsl");
        // ---------------

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .insert_resource(self.settings.clone())
            .add_systems(
                ExtractSchedule,
                (
                    sdf::extract_occluder,
                    light::extract_lights,
                    size::extract_size,
                ),
            )
            .add_systems(
                Render,
                (
                    sdf::prepare_sdf_buffers,
                    size::prepare_bindgroup,
                    light::prepare_light_buffers,
                    config::prepare,        // todo: run on change
                    merge::prepare_uniform, // todo:run on change
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_render_graph_node::<ViewNodeRunner<node::LightNode>>(Core2d, node::LightNodeLabel)
            .add_render_graph_edge(Core2d, Node2d::EndMainPass, node::LightNodeLabel);
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<sdf::SdfPipeline>()
            .init_resource::<sdf::SdfBuffers>()
            .init_resource::<light::LightPipeline>()
            .init_resource::<light::LightBuffers>()
            .init_resource::<bounce::BouncePipeline>()
            .init_resource::<ComputedSizeBuffer>()
            .init_resource::<composite::CompositePipeline>()
            .init_resource::<config::ConfigBuffer>()
            .init_resource::<probe::ProbePipeline>()
            .init_resource::<merge::MergePipeline>()
            .init_resource::<merge::MergeUniforms>()
            .init_resource::<mipmap::MipMapPipeline>();
    }
}

// this is a temp fix for hot reloading while development, since
// embedded assets do not work with bevy's wgsl imports.
fn watch(mut shaders: ResMut<Assets<Shader>>) {
    let currrent_file = PathBuf::from(file!());
    let current_dir = currrent_file.parent().unwrap();

    watch_assset(
        &mut shaders,
        current_dir.join("shaders/raymarch.wgsl"),
        constant::RAYMARCH_SHADER,
    );
    watch_assset(
        &mut shaders,
        current_dir.join("shaders/common.wgsl"),
        constant::COMMON_SHADER,
    );
    watch_assset(
        &mut shaders,
        current_dir.join("shaders/shapes.wgsl"),
        constant::SHAPES_SHADER,
    );
}

#[allow(unused)]
fn watch_assset(shaders: &mut Assets<Shader>, path: PathBuf, handle: Handle<Shader>) {
    let Ok(meta) = std::fs::metadata(&path) else {
        return;
    };
    let systime = meta.modified().unwrap();
    let since = systime.elapsed().unwrap();

    if since > Duration::from_millis(50) {
        return;
    }

    let shader_content = std::fs::read_to_string(&path).unwrap();

    if let Some(shader) = shaders.get_mut(&handle) {
        info!("realoding shader: {path:?}");
        shader.source = Source::Wgsl(shader_content.into());
    }
}
