#define_import_path lommix_light::raymarch

const MIN_DISTANCE: f32 = 0.001;
const MIN_DISTANCE_BOUNCE: f32 = 0.005;

struct RayResult{
	success: i32,
	steps: i32,
	current_pos: vec2<f32>,
	last_sample: vec4<f32>,
}

fn raymarch(
	origin: vec2<f32>,
	direction: vec2<f32>,
	sdf_tex: texture_2d<f32>,
	sdf_sampler: sampler,
	max_steps: i32,
) -> RayResult
{
	var result: RayResult;
	result.current_pos = origin;

	for (var i = 0; i < max_steps; i ++ )
	{
		result.steps ++;
		result.last_sample = textureSample(sdf_tex, sdf_sampler, result.current_pos);
		let current_distance = result.last_sample.a;

		// out of bounds
		if
			result.current_pos.x > 1. || result.current_pos.y > 1. ||
			result.current_pos.x < 0. || result.current_pos.y < 0.
		{
			break;
		}

		// is hit?
		if current_distance < MIN_DISTANCE {
			result.success = 1;
			break;
		}

		result.current_pos += direction * current_distance;
	}


	return result;
}


fn raymarch_bounce(
	origin: vec2<f32>,
	direction: vec2<f32>,
	sdf_tex: texture_2d<f32>,
	sdf_sampler: sampler,
	max_steps: i32,
) -> RayResult
{
	var result: RayResult;
	result.current_pos = origin;

	for (var i = 0; i < max_steps; i ++ )
	{
		result.steps ++;
		result.last_sample = textureSample(sdf_tex, sdf_sampler, result.current_pos);
		let current_distance = result.last_sample.a;

		// out of bounds
		if
			result.current_pos.x > 1. || result.current_pos.y > 1. ||
			result.current_pos.x < 0. || result.current_pos.y < 0.
		{
			break;
		}

		// is hit?
		if current_distance < MIN_DISTANCE_BOUNCE {
			result.success = 1;
			break;
		}

		result.current_pos += direction * current_distance;
	}


	return result;
}

fn raymarch_probe(
	origin: vec2<f32>,
	direction: vec2<f32>,
	offset: f32,
	max_dist: f32,
	sdf_tex: texture_2d<f32>,
	sdf_sampler: sampler,
	max_steps: i32,
	size: vec2<f32>,
) -> RayResult
{
	var result: RayResult;
	result.current_pos = origin + ( direction*offset )/size;
	var travel = 0.;

	for (var i = 0; i < max_steps; i ++ )
	{
		result.steps ++;
		result.last_sample = textureSample(sdf_tex, sdf_sampler, result.current_pos);
		let current_distance = result.last_sample.a;

		// out of bounds
		if
			result.current_pos.x > 1. || result.current_pos.y > 1. ||
			result.current_pos.x < 0. || result.current_pos.y < 0.
		{
			break;
		}

		// is hit?
		if current_distance < 0. {
			result.success = 1;
			break;
		}

		let to_next = direction * current_distance;
		travel += length(to_next);

		if travel > max_dist {
			break;
		}

		result.current_pos = result.current_pos + to_next/size;
	}


	return result;
}
