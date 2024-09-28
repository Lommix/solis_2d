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

@group(0) @binding(4) var<uniform> computed_size: ComputedSize;
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
	var out : vec4<f32>;


	// swap 0 1 to make it counter clockwise
	var positions = array<i32,4>(1,0,2,3);

	let frag_pos = merge_cfg.target_size * in.uv;
	let cascade_pos = frag_pos/2;
	let probe_pos = frag_pos%2;
	let corner_index = positions[i32(probe_pos.x + probe_pos.y * 2)];

	let probe_stride = pow(2.,f32(3 - merge_cfg.iteration));
	let probe_pixel_uv_size = 1./computed_size.scaled;
	let segment_size =  probe_stride * probe_stride;

	var sum : vec4<f32>;
	var count = 0;

	let start_uv = probe_pixel_uv_size * vec2<f32>(cascade_pos) * probe_stride;

	var cascade_uv = start_uv * vec2(0.25,1.) + vec2(0.25,0.) * f32(3 - merge_cfg.iteration);

	cascade_uv.y += probe_pixel_uv_size.y * ( probe_stride / 4. ) * f32(corner_index);


	for	(var i = 0; i < i32(segment_size); i++){

		let offset = vec2(
			probe_pixel_uv_size.x * ( f32(i) % probe_stride ) * 0.25,
			probe_pixel_uv_size.y * ( f32(i) / probe_stride ),
		);

		sum += textureSample(cascades_tex,cascades_sampler, cascade_uv + offset);
		count += 1;
	}

	sum = sum /f32(count);


	if merge_cfg.iteration > 0 {
		let last_merge = textureSample(last_merge_tex,last_merge_sampler,in.uv);
		sum = ( sum + last_merge ) / 2.;
	}

	return sum;
}


fn test(uv: vec2<f32>){

	// let target_pixel_pos = merge_cfg.target_size * uv;
	// let last_cascade_pos = i32( target_pixel_pos ) / 4;
	// let last_cascade_corner = i32(target_pixel_pos) % 4;
	//
	// let avarage_corner = probe_corner_avg(
	// 	last_cascade_pos,
	// 	last_cascade_corner,
	// 	merge_cfg.iteration,
	// );
}


// calculates the avrage color of a probe corner
fn probe_corner_avg(
	position: vec2<i32>,
	corner: vec2<i32>,
	iteration: u32,
	size: vec2<f32>,
) -> vec4<f32>{
	var out : vec4<f32>;


	// let probe_size = 8; // todo: Probe size of the last cascade
	// let offset = corner_to_offset(corner, probe_size);

	// let uv_position = position / probe_chunk_size;
	//
	// let corner_pixel_count = ( probe_size * probe_size ) / 4;
	//
	// for ( var i = 0; i < corner_pixel_count; i ++ ) {
	//
	// 	let pixel_position = offset + i;
	//
	// 	let uv_position =  pixel_position / probe_tex_size;
	//
	// }

	return out;
}


fn offset_to_corner(offset: u32) -> vec2<f32>{
	var corner : vec2<f32>;
	return corner;
}

fn corner_to_offset(corner: vec2<f32>, probe_size: u32) -> i32{
	return i32(corner.x);
}

// order is const, offset changes
// x y coord of prev cascade + which corner
// represent corner by offset
// order: TOPRIGHT->TOPLEFT->BOTTOMLEFT->BOTTOMRIGHT
// offset step = pixel/4
fn sum_corner(
	x: u32,
	y: u32,
	corner_index: i32,
	size: i32,
) -> vec4<f32>{
	var out : vec4<f32>;
	let corner_offset_step = f32(size * size / 4);
	let offset = corner_offset_step * f32(corner_index);
	return out;
}

// which corner am i
