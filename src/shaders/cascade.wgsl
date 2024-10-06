#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import lommix_light::common::{debug_merge0, Probe, debug_merge1, GiConfig, ComputedSize, random, EPSILON }
#import lommix_light::raymarch::{raymarch_probe}
#import bevy_render::maths::{PI_2, HALF_PI}



@group(0) @binding(0) var sdf_tex: texture_2d<f32>;
@group(0) @binding(1) var lerp_sampler: sampler;
@group(0) @binding(2) var last_cascade: texture_2d<f32>;
@group(0) @binding(3) var<uniform> in_size: ComputedSize;
@group(0) @binding(4) var<uniform> in_cfg: GiConfig;
@group(0) @binding(5) var<uniform> in_probe: Probe;

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{



	let cascade_size = in_size.scaled;

	let coord			= floor(vec2<f32>(cascade_size) * in.uv);
	let sqr_angular		= pow(2.,f32(in_probe.cascade_index));
	let extent			= floor(vec2<f32>(cascade_size) / sqr_angular);
	let probe			= vec4(coord % extent, floor(coord / extent));
	let interval		= in_probe.cascade_interval * (1. - pow(4., f32(in_probe.cascade_index))/-4.);
	let linear			= vec2(f32(in_probe.base) * pow(2.0, f32(in_probe.cascade_index )));
	let limit			= (in_probe.cascade_interval * pow(4.0, f32(in_probe.cascade_index))) + length(linear * 2.0);
	let origin			= (probe.xy + 0.5) * linear;
	let angular			= sqr_angular * sqr_angular * 4.0;
	let index			= (probe.z + (probe.w * sqr_angular)) * 4.0;

	var out : vec4<f32>;
	for(var i = 0; i < 4; i++){
		let preavg = index + f32(i);
		let theta = (preavg + 0.5) * (PI_2 / angular);
		let delta = vec2(cos(theta), -sin(theta));
		let ray = origin + (delta * interval);
		let radiance = march(ray, delta, limit);

		out += merge(radiance, preavg, extent, probe.xy) * 0.2;
	}

	// if in_probe.cascade_index != 4 {
	// 	out = linear(out);
	// 	// out = vec4(0.);
	// }

	// out.g += probe.g * 0.1;
	// out.r += probe.r * 0.1;

	return out;
}


fn march(
	origin: vec2<f32>,
	delta: vec2<f32>,
	interval: f32,
) -> vec4<f32> {

	let scale = length(vec2<f32>(in_size.scaled));
	var rr = 0.;
	var dd : vec4<f32>;

	for(var i = 0; i < 32; i ++){

		let ray = ( origin + ( delta * rr ));
        dd = textureSample(sdf_tex,lerp_sampler, ray * 1. / scale);
		rr += scale * dd.a;

		if (rr >= interval ){
			break;
		}

		if dd.a < 0.001 {
			return vec4(dd.rgb, 0.0);
		}
	}

	return vec4(0.,0.,0.,1.);
}


fn raymarch(
	origin: vec2<f32>,
	direction: vec2<f32>,
	range: f32,
) -> vec4<f32> {
	var out : vec4<f32>;
	var travel_dist = 0.;
	var position = origin;
    let dimensions = vec2<f32>(textureDimensions(sdf_tex));

	out.a = 1.;

	for (var i = 0; i < 32; i ++ )
	{
        if (
            travel_dist >= range ||
            any(position >= dimensions) ||
            any(position < vec2<f32>(0.0))
        ) {
            break;
        }

        let coord = vec2<u32>(position);
        let sdf_sample = textureLoad(sdf_tex, coord, 0);
		let dist = sdf_sample.a;

		let rgb = sdf_sample.rgb;
		let intensity = (rgb.r + rgb.b + rgb.g);

		//follow ray to start position
        if dist < EPSILON {
			out = vec4(sdf_sample.rgb, 0.);
            break;
		}

		position += direction * dist;
		travel_dist += dist;
	}

	return out;
}


fn merge(
	radiance: vec4<f32>,
	index: f32,
	extent: vec2<f32>,
	probe:	vec2<f32>,
) -> vec4<f32> {

	if (radiance.a == 0.0 || in_probe.cascade_index >= in_probe.cascade_count - 1){
		return vec4(radiance.rgb, 1.0 - radiance.a);
	}

	let angularN1 = pow(2.0, floor(f32(in_probe.cascade_index) + 1.0));
	let extentN1 = floor(vec2<f32>( in_size.scaled )/ angularN1);
	var interpN1 = vec2(index % angularN1, floor(index / angularN1)) * extentN1;
	interpN1 += clamp((probe * 0.5) + 0.25, vec2(0.5), extentN1 - 0.5);


	let radianceN1 = textureSample(
		last_cascade,
		lerp_sampler,
		interpN1 * (1.0 / vec2<f32>(in_size.scaled)),
	);

	return radiance + radianceN1;
}

fn linear(in : vec4<f32>) -> vec4<f32>{
	let rgb = pow(in.rgb,vec3(1./2.2));
	return vec4(rgb,1.);
}
