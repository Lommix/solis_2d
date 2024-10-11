use bevy::{
    asset::{embedded_asset, load_internal_asset},
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        render_graph::{RenderGraphApp, ViewNodeRunner},
        render_resource::Source,
        view::{check_visibility, VisibilitySystems},
        Render, RenderApp, RenderSet,
    },
    time::common_conditions::on_timer,
};
use sdf::Emitter;
use std::{path::PathBuf, time::Duration};

mod constant;
mod node;
mod radiance;
mod sdf;
mod view;

pub mod prelude {
    pub use super::sdf::{Emitter, Occluder, SdfShape};
    pub use super::view::{GiFlags, RadianceCameraBundle, NormalTarget, RadianceConfig, RadianceDebug};
    pub use super::LightPlugin;
}

#[derive(Default)]
pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<view::RadianceConfig>::default(),
            ExtractComponentPlugin::<view::RadianceDebug>::default(),
            ExtractComponentPlugin::<view::NormalTarget>::default(),
        ));

        // adds some hot reloading for dev
        #[cfg(debug_assertions)]
        app.add_systems(Last, watch.run_if(on_timer(Duration::from_millis(50))));

        app.add_systems(
            PostUpdate,
            check_visibility::<With<Emitter>>.in_set(VisibilitySystems::CheckVisibility),
        );
        // ---------------
        // fix later
        // bevy's wgsl definition do not work with embedded assets
        load_internal_asset!(
            app,
            constant::COMMON_SHADER,
            "shaders/common.wgsl",
            Shader::from_wgsl
        );

        embedded_asset!(app, "shaders/sdf.wgsl");
        embedded_asset!(app, "shaders/composite.wgsl");
        embedded_asset!(app, "shaders/cascade.wgsl");
        embedded_asset!(app, "shaders/mipmap.wgsl");
        // ---------------

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(ExtractSchedule, sdf::extract_emitter)
            .add_systems(
                Render,
                (
                    sdf::prepare_sdf_buffers,
                    view::prepare_config,
                    view::prepare_textures,
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
            .init_resource::<radiance::RadiancePipeline>();
    }
}

// this is a temp fix for hot reloading while development, since
// embedded assets do not work with bevy's WGSL imports.
fn watch(mut shaders: ResMut<Assets<Shader>>) {
    let currrent_file = PathBuf::from(file!());
    let current_dir = currrent_file.parent().unwrap();

    watch_assset(
        &mut shaders,
        current_dir.join("shaders/common.wgsl"),
        constant::COMMON_SHADER,
    );
}

#[allow(unused)]
fn watch_assset(shaders: &mut Assets<Shader>, path: PathBuf, handle: Handle<Shader>) {
    let Ok(meta) = std::fs::metadata(&path) else {
        return;
    };
    let systime = meta.modified().unwrap();
    let since = systime.elapsed().unwrap();

    // stupid, but works
    if since > Duration::from_millis(50) {
        return;
    }

    let shader_content = std::fs::read_to_string(&path).unwrap();

    if let Some(shader) = shaders.get_mut(&handle) {
        info!("realoding shader: {path:?}");
        shader.source = Source::Wgsl(shader_content.into());
    }
}
