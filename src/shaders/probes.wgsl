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

	/// probe the scene in n-cascades
	/// |---2x2|---4x4|-16x16|-32x32|
	/// |------|------|------|------|
	/// |--C0--|--C1--|--C2--|--C3--| ...

	let frag_coord			= floor(vec2<f32>(size.probe) * in.uv);
	let cascade_index		= floor(in.uv.x * 4.);
	let cascade_frag_coord	= vec2(frag_coord.x % f32(size.scaled.x),frag_coord.y);

	let probe_stride		= f32(cfg.probe_stride) * pow(2., (cascade_index));
	let probe_coord			= floor(cascade_frag_coord / probe_stride);
	let ray_coord			= cascade_frag_coord % probe_stride;

	let ray_index			= ray_coord.x + ray_coord.y * probe_stride;
	let ray_count			= probe_stride * probe_stride;
	// let angle				= ray_index/ray_count * PI * 2.;

    var angle = (f32(ray_index) + 0.5) / f32(ray_count) * -TAU;
	// center frag coord of the probe
	let cascade_center_frag = cascade_frag_coord + vec2(probe_stride) * 0.5;
	// let direction = vec2(cos(angle),sin(angle));
    let direction = normalize(vec2<f32>(cos(angle), sin(angle)));
	// let direction = normalize(cascade_center_frag - cascade_frag_coord);

	let ray_length = cfg.probe_size * probe_stride * pow(4., ( cascade_index+1 ) * 2);
	let ray_offset = cfg.probe_size * probe_stride * pow(2., cascade_index * 2);

	let result = raymarch(
		cascade_center_frag + direction * ray_offset,
		direction,
		ray_length,
		sdf_tex,
		sdf_sampler,
		50,
	);


	out = select(out, result.last_sample, result.success == 1);
	out.a = f32(result.success) * sign((result.last_sample.r + result.last_sample.g + result.last_sample.b)/3. );

	// out.r = ((ray_coord)/vec2<f32>(size.scaled)).x;
	// out.g = ((ray_coord)/vec2<f32>(size.scaled)).y;
	return out;
}


struct RayResult{
	success: i32,
	steps: i32,
	current_pos: vec2<f32>,
	last_sample: vec4<f32>,
}

fn raymarch(
	origin: vec2<f32>,
	direction: vec2<f32>,
	max_dist: f32,
	sdf_tex: texture_2d<f32>,
	sdf_sampler: sampler,
	max_steps: i32,
) -> RayResult
{

	let size = vec2<f32>(textureDimensions(sdf_tex));

	var result: RayResult;
	result.current_pos = origin;// + direction * sqrt(max_dist);
	var travel = 0.;

	for (var i = 0; i < max_steps; i ++ )
	{
		// out of bounds
		if
			result.current_pos.x > size.x || result.current_pos.y > size.y ||
			result.current_pos.x < 0. || result.current_pos.y < 0.
		{
			break;
		}

		result.steps ++;
		result.last_sample = textureLoad(sdf_tex, vec2<i32>(result.current_pos), 0);

		let current_distance = result.last_sample.a;

		// is hit?
		if current_distance < 0.01 {
			result.success = 1;
			break;
		}

		let to_next = direction * current_distance;
		travel += current_distance;

		if travel > max_dist {
			break;
		}

		result.current_pos = result.current_pos + to_next;
	}

	return result;
}
