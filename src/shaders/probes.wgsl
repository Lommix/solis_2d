#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import lommix_light::common::{ GiConfig,debug_probe, ComputedSize, random, PI, TAU,EPSILON }
#import lommix_light::raymarch::{raymarch_probe}


@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var sdf_tex: texture_2d<f32>;
@group(0) @binding(2) var sdf_sampler: sampler;
@group(0) @binding(3) var<uniform> size: ComputedSize;
@group(0) @binding(4) var<uniform> cfg: GiConfig;


@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{

	var out : vec4<f32>;

	/// probe the scene in n-cascades
	/// |---2x2|---4x4|-16x16|-32x32|
	/// |------|------|------|------|
	/// |--C0--|--C1--|--C2--|--C3--| ...

	let frag_coord			= floor(vec2<f32>(size.probe) * in.uv);
	let cascade_index		= floor(in.uv.x * 4.);
	let cascade_frag_coord	= vec2(frag_coord.x % f32(size.scaled.x),frag_coord.y);

	let probe_stride		= f32(cfg.probe_stride) * pow(2., cascade_index);
	let probe_coord			= floor(cascade_frag_coord / probe_stride);
	let ray_coord			= cascade_frag_coord % probe_stride;

	let ray_index			= ray_coord.x + ray_coord.y * probe_stride;
	let ray_count			= probe_stride * probe_stride;

    var angle				= (f32(ray_index) + 0.5) / f32(ray_count) * -TAU;
	let cascade_center_frag = cascade_frag_coord + probe_stride/2;
    let direction			= normalize(vec2<f32>(cos(angle), sin(angle)));

	let ray_length	= ( probe_stride* 5.) * (1. - pow(4., cascade_index)/-3);
	let ray_offset	= ( probe_stride* 5.) * pow(4., cascade_index);


	let probe_uv = ray_coord / vec2(probe_stride);

	out = ray_march(
		cascade_center_frag,
		direction,
		ray_length,
		ray_offset,
	);

	if debug_probe(cfg) > 0.{
		out.r += probe_uv.r * .2;
		out.g += probe_uv.g * .2;
	}

	return out;
}


fn ray_march(
	origin: vec2<f32>,
	direction: vec2<f32>,
	range : f32,
	offset: f32,
) -> vec4<f32> {
	var out : vec4<f32>;
	var travel_dist = 0.;
	var position = origin;
    let dimensions = vec2<f32>(textureDimensions(sdf_tex));

	for (var i = 0; i < 32; i ++ )
	{
        if (
            travel_dist >= range ||
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
			out = vec4(sdf_sample.rgb,1.);
            break;
		}

		position += direction * dist;
		travel_dist += dist;
	}

	return out;
}
