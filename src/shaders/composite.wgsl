#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import lommix_light::common::{GiConfig,debug_merge0,debug_merge1, debug_voronoi, debug_sdf}

@group(0) @binding(0) var main_tex: texture_2d<f32>;
@group(0) @binding(1) var sdf_tex: texture_2d<f32>;
@group(0) @binding(2) var merge_tex_0: texture_2d<f32>;
@group(0) @binding(3) var merge_tex_1: texture_2d<f32>;
@group(0) @binding(4) var radiance_sampler: sampler;
@group(0) @binding(5) var point_sampler: sampler;
@group(0) @binding(6) var<uniform> cfg: GiConfig;

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{
	var out : vec4<f32>;

	let main_sample = textureSample(main_tex, point_sampler, in.uv);
	let sdf_sample = textureSample(sdf_tex, point_sampler,in.uv);
	let merge_sample_1 = textureSample(merge_tex_1, point_sampler, in.uv);
	let merge_sample_0 = textureSample(merge_tex_0, point_sampler, in.uv);

	let light = textureSample(merge_tex_0, radiance_sampler, in.uv);

	let edge_intensity = 1./abs(sdf_sample.a) * 8.;
	let inside = sign(abs(max(sdf_sample.a,0.)));

	out = main_sample + light * max(edge_intensity , inside);

	out = mix(out, vec4(abs(sdf_sample.a / 20.)), debug_sdf(cfg));
	out = mix(out, vec4(sdf_sample.rgb, 1.), debug_voronoi(cfg));
	out = mix(out, merge_sample_0, debug_merge0(cfg));
	out = mix(out, merge_sample_1, debug_merge1(cfg));

	return out;
}
