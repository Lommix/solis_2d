#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import lommix_light::common::{debug_merge0, Probe, debug_merge1, GiConfig, ComputedSize, random, EPSILON }
#import lommix_light::raymarch::{raymarch_probe}
#import bevy_render::maths::{PI_2, HALF_PI}

@group(0) @binding(0) var sdf_tex: texture_2d<f32>;
@group(0) @binding(1) var last_cascade: texture_2d<f32>;
@group(0) @binding(2) var normal_tex: texture_2d<f32>;
@group(0) @binding(3) var rad_sampler: sampler;
@group(0) @binding(4) var<uniform> in_cfg: GiConfig;
@group(0) @binding(5) var<uniform> in_probe: Probe;

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{

	let cascade_size	= in_cfg.scaled / in_cfg.probe_base;
	let coord			= floor(vec2<f32>(cascade_size) * in.uv);
	let sqr_angular		= pow(2.,f32(in_probe.cascade_index));
	let extent			= floor(vec2<f32>(cascade_size) / sqr_angular);
	let probe			= vec4(coord % extent, floor(coord / extent));
	let linear			= vec2(f32(in_cfg.probe_base) * pow(2.0, f32(in_probe.cascade_index )));
	let interval		= in_cfg.interval * (1. - pow(4., f32(in_probe.cascade_index)))/-3.;
	let limit			= (in_cfg.interval * pow(4., f32(in_probe.cascade_index)));
	let origin			= (probe.xy + .5) * linear;
	let angular			= sqr_angular * sqr_angular * 4.0;
	let index			= (probe.z + (probe.w * sqr_angular)) * 4.0;

	let normal_sample = textureSample(normal_tex, rad_sampler, origin/vec2<f32>( in_cfg.scaled ));
	let normal = normalize(normal_sample.rgb * 2. - 1.);

	var out : vec4<f32>;
	for(var i = 0; i < 4; i++){
		let preavg = index + f32(i);
		let theta = (preavg + 0.5) * (PI_2 / angular);
		let delta = vec2(cos(theta), -sin(theta));
		let ray = origin + (delta * interval);

		let normal_dot = max(0.,dot(vec3(-delta,in_cfg.light_z),normal));
		var radiance = march(ray, delta, limit);

		if ( in_cfg.flags >> 5 & 1 ) > 0 {
			radiance *= normal_dot;
		}

		out += merge(radiance, preavg, extent, probe.xy) * 0.25;
	}


	return out;
}


fn march(
	origin: vec2<f32>,
	delta: vec2<f32>,
	interval: f32,
) -> vec4<f32> {

	var dst_traveled= 0.;
	var sample : vec4<f32>;

	for(var i = 0; i < 16; i ++){
		let ray = ( origin + ( delta * dst_traveled ));

		let uv = vec2<f32>(ray) / vec2<f32>(textureDimensions(sdf_tex));
		if uv.x < 0. || uv.y < 0. || uv.x > 1. || uv.y > 1. {
			return vec4(0.,0.,0.,1.);
		}

        sample = textureSample(sdf_tex, rad_sampler, uv);
		dst_traveled += sample.a;

		if (dst_traveled >= interval){
			break;
		}

		if sample.a < 0.1 {
			return vec4(sample.rgb, 0.0);
		}
	}

	return vec4(0.,0.,0.,1.);
}

fn merge(
	radiance: vec4<f32>,
	index: f32,
	extent: vec2<f32>,
	probe:	vec2<f32>,
) -> vec4<f32> {

	let size = in_cfg.scaled / in_cfg.probe_base;

	if (radiance.a == 0.0 || in_probe.cascade_index >= in_cfg.cascade_count - 1){
		return vec4(radiance.rgb, 1.0 - radiance.a);
	}

	let angularN1 = pow(2.0, floor(f32(in_probe.cascade_index) + 1.0));
	let extentN1 = floor(vec2<f32>( size )/ angularN1);
	var interpN1 = vec2(index % angularN1, floor(index / angularN1)) * extentN1;
	interpN1 += clamp((probe * 0.5) + 0.25, vec2(0.5), extentN1 - 0.5);

	let radianceN1 = textureSample(
		last_cascade,
		rad_sampler,
		interpN1 * (1.0 / vec2<f32>(size)),
	);

	return radiance + radianceN1;
}
