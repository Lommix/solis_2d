#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import solis_2d::common::{GiConfig,debug_merge0,debug_merge1, debug_voronoi, debug_sdf}

@group(0) @binding(0) var main_tex: texture_2d<f32>;
@group(0) @binding(1) var sdf_tex: texture_2d<f32>;
@group(0) @binding(2) var merge_tex_0: texture_2d<f32>;
@group(0) @binding(3) var merge_tex_1: texture_2d<f32>;
@group(0) @binding(4) var mipmap_tex: texture_2d<f32>;
@group(0) @binding(5) var normal_tex: texture_2d<f32>;
@group(0) @binding(6) var radiance_sampler: sampler;
@group(0) @binding(7) var point_sampler: sampler;
@group(0) @binding(8) var<uniform> cfg: GiConfig;

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{
	var out : vec4<f32>;

	let main_sample = textureSample(main_tex, point_sampler, in.uv);
	let sdf_sample = textureSample(sdf_tex, point_sampler,in.uv);

	let light = textureSample(mipmap_tex, radiance_sampler, in.uv);
	let edge_intensity = 1./abs(sdf_sample.a) * cfg.edge_highlight;
	let inside = sign(abs(max(sdf_sample.a,0.)));

	out = main_sample + light + light * edge_intensity - cfg.absorb;
	out *= cfg.modulate;

	out = mix(out, vec4(abs(sdf_sample.a / 20.)), debug_sdf(cfg));
	out = mix(out, vec4(sdf_sample.rgb, 1.), debug_voronoi(cfg));

	return max(out,vec4(0.));
}


fn bilinear(uv: vec2<f32>)->vec4<f32>{
	var out : vec4<f32>;
	return out;
}
