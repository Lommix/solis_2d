#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import lommix_light::common::{ GiConfig, ComputedSize, random, PI }
#import lommix_light::raymarch::{raymarch}

// the cascade0 texture
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> size: ComputedSize;


@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{
	var out : vec4<f32>;
	var count = 0;

    let size = vec2<f32>(textureDimensions(source_tex));
	let frag = vec2<i32>(size * in.uv);
	var offsets = array<vec2<i32>,4>(
		vec2(0,0),
		vec2(1,0),
		vec2(0,1),
		vec2(1,1),
	);

	for (var i = 0; i < 4; i ++){
		out += textureLoad(source_tex, frag + offsets[i],0);
	}

	return out / 4.;
}
