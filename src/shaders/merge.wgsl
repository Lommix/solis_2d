#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import lommix_light::common::{ GiConfig, ComputedSize, random, PI, TAU }
#import lommix_light::raymarch::{raymarch_probe}


// all cascades
@group(0) @binding(0) var cascades_tex: texture_2d<f32>;
@group(0) @binding(1) var cascades_sampler: sampler;

// last merged cascade
@group(0) @binding(2) var last_merge_tex: texture_2d<f32>;
@group(0) @binding(3) var last_merge_sampler: sampler;

@group(0) @binding(4) var<uniform> size: ComputedSize;
@group(0) @binding(5) var<uniform> cfg: GiConfig;
@group(0) @binding(6) var<uniform> merge_cfg: MergeConfig;

struct MergeConfig {
	iteration: u32,
	target_size: vec2<f32>,
}


// current		last cascade sample
// ---------	-----------------
// |[1]| 0 |	|0 ..
// ---------	-----------------
// | 2 | 3 |	|1 ..
// ---------	-----------------
//				|2 ..
//				-----------------
//				|3 ..
//				-----------------
//
//	counter clockwise rays
//
//  steps:
//	- calc current index
//	- sum avg line of last cadcade by index
//	- return last cascade avg + last merge result
// const RAY_INDEX_DIR = array(1, 0, 2, 3);

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{

	// first is 16x16
	let probe_stride = f32(cfg.probe_stride) * pow(2., f32(4 - merge_cfg.iteration));

	// swap 0 1 to make it counter clockwise
	var positions = array<i32,4>(1,0,2,3);
	let frag_pos = floor(merge_cfg.target_size * in.uv);

	let cascade_probe_pos = floor(frag_pos/2);
	let corner_pos = frag_pos%2;

	// counter clockwise index
	let corner_index = positions[i32(corner_pos.x) + i32(corner_pos.y * 2)];

	// probe corner
	var sum = sample_corner(
		vec2<i32>(cascade_probe_pos),
		i32(probe_stride),
		corner_index,
		i32(3 - merge_cfg.iteration),
	);

	if merge_cfg.iteration > 0 {
		let last_merge = textureSample(last_merge_tex, last_merge_sampler, in.uv);
		sum = mix(sum , last_merge, 0.5);
	}

	return sum;
}


fn sample_corner(
	probe_coord: vec2<i32>,
	probe_stride: i32,
	corner_index: i32,
	cascade_index: i32,
) -> vec4<f32>{
	var sum : vec4<f32>;
	var count = 0;

	let xoffset = i32(textureDimensions(cascades_tex).x)/4 * cascade_index;
	let size = vec2<i32>(textureDimensions(cascades_tex));


	let yoffset =  corner_index * probe_stride/4;
	let corner_size = probe_stride * probe_stride/2;

	for (var i=0; i < corner_size; i ++){

		let x = (i%probe_stride) + xoffset;
		let y = (i/probe_stride) + yoffset;

		let probe_start		= probe_coord * probe_stride;
		let sample_coord	= probe_start + vec2(x,y);


		if sample_coord.x < 0 || sample_coord.y < 0 || sample_coord.y > size.y || sample_coord.x > size.x {
			continue;
		}

		let sample = textureLoad(cascades_tex,sample_coord,0);
		count += 1;
		sum += sample;

	}

	return sum/f32(count);
}
