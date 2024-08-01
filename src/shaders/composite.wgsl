#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import lommix_light::common::{GiConfig,debug_merge, debug_probe, debug_voronoi, debug_bounce, debug_sdf, debug_light, ComputedSize}

@group(0) @binding(0) var main_tex: texture_2d<f32>;
@group(0) @binding(1) var main_sampler: sampler;

@group(0) @binding(2) var light_tex: texture_2d<f32>;
@group(0) @binding(3) var light_sampler: sampler;

@group(0) @binding(4) var sdf_tex: texture_2d<f32>;
@group(0) @binding(5) var sdf_sampler: sampler;

@group(0) @binding(6) var bounce_tex: texture_2d<f32>;
@group(0) @binding(7) var bounce_sampler: sampler;

@group(0) @binding(8) var probe_tex: texture_2d<f32>;
@group(0) @binding(9) var probe_sampler: sampler;

@group(0) @binding(10) var merge_tex: texture_2d<f32>;
@group(0) @binding(11) var merge_sampler: sampler;

@group(0) @binding(12) var<uniform> cfg: GiConfig;
@group(0) @binding(13) var<uniform> size: ComputedSize;

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{

	var out : vec4<f32>;

	let main_sample = textureSample(main_tex, main_sampler, in.uv);
	let sdf_sample = textureSample(sdf_tex,sdf_sampler,in.uv);
	let light_sample = textureSample(light_tex, light_sampler, in.uv);
	let bounce_sample = textureSample(bounce_tex, bounce_sampler, in.uv);
	let merge_sample = textureSample(merge_tex, merge_sampler, in.uv);


	let probe_0 = textureSample(probe_tex, probe_sampler, in.uv * vec2(0.25,1.) + vec2(0.25, 0.) * 0.);
	let probe_1 = textureSample(probe_tex, probe_sampler, in.uv * vec2(0.25,1.) + vec2(0.25, 0.) * 1.);
	let probe_2 = textureSample(probe_tex, probe_sampler, in.uv * vec2(0.25,1.) + vec2(0.25, 0.) * 2.);
	let probe_3 = textureSample(probe_tex, probe_sampler, in.uv * vec2(0.25,1.) + vec2(0.25, 0.) * 3.);
	let probe = mix(mix(probe_0,probe_1,0.5), mix(probe_2,probe_3,0.5), 0.5);

	// debug view
	out = mix(out, vec4(abs(sdf_sample.a/100.)), debug_sdf(cfg));
	out = mix(out, vec4(sdf_sample.rgb, 1.), debug_voronoi(cfg));
	out = mix(out, vec4(light_sample), debug_light(cfg));
	out = mix(out, vec4(bounce_sample), debug_bounce(cfg));
	out = mix(out, probe_2, debug_probe(cfg));
	out = mix(out, merge_sample, debug_merge(cfg));
	// ----------

	return out;
}

fn lin_to_srgb(color: vec3<f32>) -> vec3<f32> {
   let x = color * 12.92;
   let y = 1.055 * pow(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(0.4166667)) - vec3<f32>(0.055);
   var clr = color;
   clr.x = select(x.x, y.x, (color.x < 0.0031308));
   clr.y = select(x.y, y.y, (color.y < 0.0031308));
   clr.z = select(x.z, y.z, (color.z < 0.0031308));
   return clr;
}


fn get_probe_tr() -> f32 {
	return 0.;
}
