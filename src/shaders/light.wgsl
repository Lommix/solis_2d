#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import lommix_light::common::{ GiConfig, ComputedSize, random, PI }
#import lommix_light::raymarch::{raymarch}

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var sdf_tex: texture_2d<f32>;
@group(0) @binding(2) var sdf_sampler: sampler;
@group(0) @binding(3) var<storage> light_buffer: LightBuffer;
@group(0) @binding(4) var<uniform> computed_size: ComputedSize;
@group(0) @binding(5) var<uniform> cfg: GiConfig;


struct LightBuffer {
	count: u32,
	data: array<PointLight>,
}


struct PointLight{
	position: vec2<f32>,
	intensity: f32,
	range: f32,
}

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{
	let sdf = textureSample(sdf_tex, sdf_sampler, in.uv);

	let aspect_ratio = computed_size.native.y / computed_size.native.x;
	var light_level = 0.;

	var light : vec3<f32>;
	var hit_count = 0;

	let rand2pi = random(in.uv * 2.0);
	let golden_angle = PI * 0.7639320225;

	for (var i = 0; i < i32(cfg.sample_count); i++){

		let angle = rand2pi + golden_angle * f32(i);
		let dir = normalize(vec2(sin(angle), cos(angle)));

		let result = raymarch(
			in.uv,
			dir,
			sdf_tex,
			sdf_sampler,
			32,
		);

		if result.success == 1{
			light += textureSample(sdf_tex, sdf_sampler, result.current_pos).rgb;
			hit_count ++;
		}
	}

	return vec4(light/f32(hit_count), 1.);
}
