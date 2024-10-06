#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import lommix_light::common::{GiConfig,debug_merge0,debug_merge1, debug_voronoi, debug_bounce, debug_sdf, debug_light, ComputedSize}

@group(0) @binding(0) var main_tex: texture_2d<f32>;
@group(0) @binding(1) var light_tex: texture_2d<f32>;
@group(0) @binding(2) var sdf_tex: texture_2d<f32>;
@group(0) @binding(3) var merge_tex_0: texture_2d<f32>;
@group(0) @binding(4) var merge_tex_1: texture_2d<f32>;

@group(0) @binding(5) var point_sampler: sampler;
@group(0) @binding(6) var linear_sampler: sampler;

@group(0) @binding(7) var<uniform> cfg: GiConfig;
@group(0) @binding(8) var<uniform> size: ComputedSize;

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{

	var out : vec4<f32>;

	let main_sample = textureSample(main_tex, point_sampler, in.uv);
	let sdf_sample = textureSample(sdf_tex,point_sampler,in.uv);
	let light_sample = textureSample(light_tex, point_sampler, in.uv);

	let merge_sample_1 = textureSample(merge_tex_1, point_sampler, in.uv);
	let merge_sample_0 = textureSample(merge_tex_0, point_sampler, in.uv);

	// super simple ultra fast merge
	// let probe = mix(mix(probe_0,probe_1,0.5), mix(probe_2,probe_3,0.5), 0.5);
	var s = sampleRadianceField(merge_tex_0, 1. ,in.uv);
	let intensity = (sdf_sample.r + sdf_sample.g + sdf_sample.b)/3.;
	let not_inside = sign(max(sdf_sample.a,0.));
	out = main_sample + s * max(not_inside, sign(intensity));


	// debug view
	out = mix(out, vec4(abs(sdf_sample.a / 20.)), debug_sdf(cfg));
	out = mix(out, vec4(sdf_sample.rgb, 1.), debug_voronoi(cfg));
	out = mix(out, s, debug_light(cfg));
	// out = mix(out, probe_0, debug_probe(cfg));
	out = mix(out, merge_sample_0, debug_merge0(cfg));
	out = mix(out, merge_sample_1, debug_merge1(cfg));
	// ----------
	return out;
}


fn sampleRadianceField(radianceField: texture_2d<f32>, distance: f32, uv: vec2<f32>) -> vec4<f32> {
    // Get the size of the texture
    let size = vec2<f32>(textureDimensions(radianceField));

    // Calculate the texel coordinates with proper scaling
    let texelCoords = floor(uv * size);

    // Get the integer part (top-left corner of the texel's 2x2 block)
    let i0 = texelCoords;
    let i1 = texelCoords + vec2<f32>(1.0, 0.0) * distance;
    let i2 = texelCoords + vec2<f32>(0.0, 1.0) * distance;
    let i3 = texelCoords + vec2<f32>(1.0, 1.0) * distance;

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

