use bevy::{
    asset::{embedded_asset, load_internal_asset},
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        render_graph::{RenderGraphApp, ViewNodeRunner},
        render_resource::Source,
        Render, RenderApp, RenderSet,
    },
    time::common_conditions::on_timer,
};
use std::{path::PathBuf, time::Duration};

mod camera;
mod cascade;
mod constant;
mod node;
mod sdf;

pub mod prelude {
    pub use super::camera::{
        GiFlags, Light2dCameraTag, RadianceCameraBundle, RadianceConfig, RadianceDebug,
    };
    pub use super::sdf::{Emitter, Occluder, SdfShape};
    pub use super::LightPlugin;
}

#[derive(Default)]
pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        #[rustfmt::skip]
        app
            .add_plugins((
                ExtractComponentPlugin::<camera::Light2dCameraTag>::default(),
                ExtractComponentPlugin::<camera::RadianceConfig>::default(),
                ExtractComponentPlugin::<camera::RadianceDebug>::default(),
            ));

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

        embedded_asset!(app, "shaders/sdf.wgsl");
        embedded_asset!(app, "shaders/composite.wgsl");
        embedded_asset!(app, "shaders/cascade.wgsl");
        // ---------------

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            // .insert_resource(self.settings.clone())
            .add_systems(ExtractSchedule, sdf::extract_occluder)
            .add_systems(
                Render,
                (
                    sdf::prepare_sdf_buffers,
                    camera::prepare_config,
                    camera::prepare_textures,
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
            .init_resource::<cascade::CascadePipeline>();
    }
}

// this is a temp fix for hot reloading while development, since
// embedded assets do not work with bevy's wgsl imports.
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

    if since > Duration::from_millis(50) {
        return;
    }

    let shader_content = std::fs::read_to_string(&path).unwrap();

    if let Some(shader) = shaders.get_mut(&handle) {
        info!("realoding shader: {path:?}");
        shader.source = Source::Wgsl(shader_content.into());
    }
}
