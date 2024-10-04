#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import lommix_light::common::{ GiConfig, ComputedSize, random, PI, TAU, EPSILON }
#import lommix_light::raymarch::{raymarch_probe}

// all cascades
@group(0) @binding(0) var sdf_tex: texture_2d<f32>;
@group(0) @binding(1) var last_cascade: texture_2d<f32>;
@group(0) @binding(2) var<uniform> size: ComputedSize;
@group(0) @binding(3) var<uniform> cfg: GiConfig;
@group(0) @binding(4) var<uniform> probe: Probe;

struct Probe {
    width: u32,
    start: f32,
    range: f32,
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
//  steps:
//	- calc current index
//	- sum avg line of last cadcade by index
//	- return last cascade avg + last merge result
// const RAY_INDEX_DIR = array(1, 0, 2, 3);

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{

	// first is 16x16
	let probe_base = f32(probe.width);

	// frag
	let texel_pos		= in.uv * vec2<f32>(size.scaled);
	let probe_pos		= floor(texel_pos / probe_base);
	let probe_center	= floor(probe_pos * probe_base + probe_base/2.);
	let probe_ray_count = probe_base * probe_base;
	let probe_coord		= floor(texel_pos%probe_base);
	let ray_index		= probe_coord.x + probe_coord.y * probe_base;
    let ray_angle		= (ray_index + 0.5) / probe_ray_count * TAU;
    let ray_dir			= normalize(vec2<f32>(cos(ray_angle), sin(ray_angle)));

	var color =  ray_march(
		probe_center ,
		ray_dir,
		probe.range,
		probe.start,
	);

	let probe_uv = probe_coord / vec2(probe_base);

	#ifdef MERGE
    if (color.a < EPSILON) {
		let last_cascade = merge(vec2<u32>(probe_pos), vec2<u32>(probe_coord), u32(ray_index));
		color += last_cascade * 0.5;
	}
	#endif

	return color;
}


fn ray_march(
	origin: vec2<f32>,
	direction: vec2<f32>,
	range: f32,
	offset: f32,
) -> vec4<f32> {
	var out : vec4<f32>;
	var travel_dist = 0.;
	var position = origin;
    let dimensions = vec2<f32>(textureDimensions(sdf_tex));

	for (var i = 0; i < 32; i ++ )
	{
        if (
            travel_dist >= ( range+offset ) ||
            any(position >= dimensions) ||
            any(position < vec2<f32>(0.0))
        ) {
            break;
        }

        let coord = vec2<u32>(round(position));
        let sdf_sample = textureLoad(sdf_tex, coord, 0);
		let dist = sdf_sample.a;

        if dist < EPSILON {
			let rgb = sdf_sample.rgb;
			if offset < travel_dist || length(rgb) > 0.1{
				out = vec4(rgb,1.);
			}
            break;
		}

		position += direction * dist;
		travel_dist += dist;
	}

	return out;
}

fn merge(probe_cell: vec2<u32>, probe_coord: vec2<u32>, ray_index: u32) -> vec4<f32> {
    let dimensions = textureDimensions(last_cascade);
    let prev_width = probe.width * 2;

    var TL = vec4<f32>(0.0);
    var TR = vec4<f32>(0.0);
    var BL = vec4<f32>(0.0);
    var BR = vec4<f32>(0.0);

    let probe_cell_i = vec2<i32>(probe_cell);
    let probe_correcetion_offset = probe_cell_i - probe_cell_i / 2 * 2;

    let prev_ray_index_start = ray_index * 4;
    for (var p: u32 = 0; p < 4; p++) {
        let prev_ray_index = prev_ray_index_start + p;

        let offset_coord = vec2<u32>(
            prev_ray_index % prev_width,
            prev_ray_index / prev_width,
        );

        TL += fetch_cascade(
            probe_cell_i,
            probe_correcetion_offset + vec2<i32>(-1, -1),
            offset_coord,
            dimensions,
            prev_width
        );
        TR += fetch_cascade(
            probe_cell_i,
            probe_correcetion_offset + vec2<i32>(0, -1),
            offset_coord,
            dimensions,
            prev_width
        );
        BL += fetch_cascade(
            probe_cell_i,
            probe_correcetion_offset + vec2<i32>(-1, 0),
            offset_coord,
            dimensions,
            prev_width
        );
        BR += fetch_cascade(
            probe_cell_i,
            probe_correcetion_offset + vec2<i32>(0, 0),
            offset_coord,
            dimensions,
            prev_width
        );
    }

    let weight = 0.75 - (
        vec2<f32>(probe_correcetion_offset) * 0.5
    );

    return mix(mix(TL, TR, weight.x), mix(BL, BR, weight.x), weight.y) * 0.25;
}


fn fetch_cascade(
    probe_cell: vec2<i32>,
    probe_offset: vec2<i32>,
    offset_coord: vec2<u32>,
    dimensions: vec2<u32>,
    prev_width: u32,
) -> vec4<f32> {
    var prev_probe_cell = probe_cell / 2 + probe_offset;
    prev_probe_cell = clamp(prev_probe_cell, vec2<i32>(0), vec2<i32>(dimensions / prev_width - 1));

    let prev_probe_coord = vec2<u32>(prev_probe_cell) * prev_width + offset_coord;
    return textureLoad(last_cascade, prev_probe_coord, 0);
}
