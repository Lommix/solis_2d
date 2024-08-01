#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import lommix_light::common::{ GiConfig, ComputedSize, random, PI }
#import lommix_light::raymarch::raymarch_bounce;

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var sdf_tex: texture_2d<f32>;
@group(0) @binding(2) var sdf_sampler: sampler;
@group(0) @binding(3) var light_tex: texture_2d<f32>;
@group(0) @binding(4) var light_sampler: sampler;
@group(0) @binding(5) var<uniform> computed_size: ComputedSize;
@group(0) @binding(6) var<uniform> cfg: GiConfig;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {

	let rand2pi = random(in.uv * 2.0);
	let golden_angle = PI * 0.7639320225;

	var light_sample : vec3<f32>;
	var hit_count = 0;

	for (var i = 0; i < i32(cfg.sample_count); i++){

		let angle = rand2pi + golden_angle * f32(i);
		let dir = normalize(vec2(sin(angle), cos(angle)));

		let result = raymarch_bounce(
			in.uv,
			dir,
			sdf_tex,
			sdf_sampler,
			20,
		);


		let last = length(result.last_sample.rgb);
		if last < 0.1 {
			light_sample += textureSample(light_tex, light_sampler, result.current_pos).rgb;
			hit_count ++;
		}
	}

	let bounce_light = light_sample/f32(hit_count);
	return vec4(bounce_light, 1.);
}
