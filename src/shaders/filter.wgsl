#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import lommix_light::common::{ GiConfig, ComputedSize, random, PI }
#import lommix_light::raymarch::{raymarch}

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var sdf_tex: texture_2d<f32>;
@group(0) @binding(2) var sdf_sampler: sampler;
@group(0) @binding(4) var<uniform> computed_size: ComputedSize;
@group(0) @binding(5) var<uniform> cfg: GiConfig;


@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{
	var out : vec4<f32>;
	return out;
}
