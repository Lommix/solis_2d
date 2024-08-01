#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import lommix_light::common::{ GiConfig, ComputedSize, random, PI, TAU }
#import lommix_light::raymarch::{raymarch_probe}


@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var sdf_tex: texture_2d<f32>;
@group(0) @binding(2) var sdf_sampler: sampler;
@group(0) @binding(3) var<uniform> size: ComputedSize;
@group(0) @binding(4) var<uniform> cfg: GiConfig;


@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{

	var out : vec4<f32>;

	let probe_uv = in.uv;
	let frag_coord = floor(in.uv * size.probe);
	let cascade_size = size.probe / vec2(4.,1.);
	let cascade_index = floor(in.uv.x * 4.);
	let cascade_uv = fract(in.uv * vec2(4.,1.));
	let cascade_coord = floor(cascade_uv * cascade_size);

	//@todo:cleanup

	let probe_stride = 2. * (1 + cascade_index);
	let ray_count = pow(probe_stride,2.);

	let probe_coord = floor(cascade_coord / probe_stride);
	let probe_uv_size = vec2(probe_stride) / cascade_size;
	let probe_uv_center = probe_uv_size * probe_coord + probe_uv_size/2.;

	let ray_coord = cascade_coord % probe_stride;
	let ray_index = ray_coord.y * probe_stride + ray_coord.x;

	let ray_dir = TAU * ( ray_index + 0.5 )/ray_count;
	let ray_length = pow(4., cascade_index) * probe_stride;

	let result = raymarch_probe(
		probe_uv_center,
		vec2(cos(ray_dir), sin(ray_dir)),
		cascade_index * ( probe_stride + 1 ),
		ray_length * cfg.probe_size * 100.,
		sdf_tex,
		sdf_sampler,
		20,
		size.native,
	);

	out = select(out, result.last_sample, result.success == 1);
	return out;
}
