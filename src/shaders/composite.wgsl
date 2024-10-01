#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import lommix_light::common::{GiConfig,debug_merge, debug_probe, debug_voronoi, debug_bounce, debug_sdf, debug_light, ComputedSize}

@group(0) @binding(0) var main_tex: texture_2d<f32>;
@group(0) @binding(1) var main_sampler: sampler;

@group(0) @binding(2) var light_tex: texture_2d<f32>;
@group(0) @binding(3) var light_sampler: sampler;

@group(0) @binding(4) var sdf_tex: texture_2d<f32>;
@group(0) @binding(5) var sdf_sampler: sampler;

@group(0) @binding(6) var bounce_tex: texture_2d<f32>;
@group(0) @binding(7) var bounce_sampler: sampler;

@group(0) @binding(8) var probe_tex: texture_2d<f32>;
@group(0) @binding(9) var probe_sampler: sampler;

@group(0) @binding(10) var merge_tex: texture_2d<f32>;
@group(0) @binding(11) var merge_sampler: sampler;

@group(0) @binding(12) var<uniform> cfg: GiConfig;
@group(0) @binding(13) var<uniform> size: ComputedSize;

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{

	var out : vec4<f32>;

	let main_sample = textureSample(main_tex, main_sampler, in.uv);
	let sdf_sample = textureSample(sdf_tex,sdf_sampler,in.uv);
	let light_sample = textureSample(light_tex, light_sampler, in.uv);
	let bounce_sample = textureSample(bounce_tex, bounce_sampler, in.uv);
	let merge_sample = textureSample(merge_tex, merge_sampler, in.uv);


	let probe_0 = textureSample(probe_tex, probe_sampler, in.uv * vec2(0.25,1.) + vec2(0.25, 0.) * 0.);
	let probe_1 = textureSample(probe_tex, probe_sampler, in.uv * vec2(0.25,1.) + vec2(0.25, 0.) * 1.);
	let probe_2 = textureSample(probe_tex, probe_sampler, in.uv * vec2(0.25,1.) + vec2(0.25, 0.) * 2.);
	let probe_3 = textureSample(probe_tex, probe_sampler, in.uv * vec2(0.25,1.) + vec2(0.25, 0.) * 3.);

	// super simple ultra fast merge
	// let probe = mix(mix(probe_0,probe_1,0.5), mix(probe_2,probe_3,0.5), 0.5);

	// let s = textureSample(light_tex,light_sampler,in.uv);
	let s = sampleRadianceField(light_tex,2.,in.uv);
	out = mix(main_sample,s, min(sdf_sample.a,0.));

	// debug view
	out = mix(out, vec4(abs(sdf_sample.a/100.)), debug_sdf(cfg));
	out = mix(out, vec4(sdf_sample.rgb, 1.), debug_voronoi(cfg));
	out = mix(out, s, debug_light(cfg));
	out = mix(out, vec4(bounce_sample), debug_bounce(cfg));
	out = mix(out, probe_1, debug_probe(cfg));
	out = mix(out, merge_sample, debug_merge(cfg));
	// ----------


	return out;
}


fn sampleRadianceField(radianceField: texture_2d<f32>, distance : f32, uv: vec2<f32>) -> vec4<f32> {


    // Get the size of the texture
    let size = vec2<f32>(textureDimensions(radianceField));
    // Calculate the texel coordinates
    let texelCoords = floor(uv * size);

    // Get the integer part (top-left corner of the texel's 2x2 block)
    let i0 = texelCoords;
    let i1 = texelCoords + vec2<f32>(1.0, 0.0)*distance;
    let i2 = texelCoords + vec2<f32>(0.0, 1.0)*distance;
    let i3 = texelCoords + vec2<f32>(1.0, 1.0)*distance;

    // Sample the texture at each of the texel positions
    let sample0 = textureLoad(radianceField, vec2<i32>(i0), 0);
    let sample1 = textureLoad(radianceField, vec2<i32>(i1), 0);
    let sample2 = textureLoad(radianceField, vec2<i32>(i2), 0);
    let sample3 = textureLoad(radianceField, vec2<i32>(i3), 0);

    // Calculate the fractional part of the texel coordinates
    let fractCoords = fract(uv * size);

    // Interpolate between the samples
    let sample01 = mix(sample0, sample1, fractCoords.x);
    let sample23 = mix(sample2, sample3, fractCoords.x);
    let finalSample = mix(sample01, sample23, fractCoords.y);

    return finalSample;
}


fn lin_to_srgb(color: vec3<f32>) -> vec3<f32> {
   let x = color * 12.92;
   let y = 1.055 * pow(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(0.4166667)) - vec3<f32>(0.055);
   var clr = color;
   clr.x = select(x.x, y.x, (color.x < 0.0031308));
   clr.y = select(x.y, y.y, (color.y < 0.0031308));
   clr.z = select(x.z, y.z, (color.z < 0.0031308));
   return clr;
}


fn get_probe_tr() -> f32 {
	return 0.;
}
