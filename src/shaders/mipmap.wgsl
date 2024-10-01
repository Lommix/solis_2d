#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import lommix_light::common::{ GiConfig, ComputedSize, random, PI }
#import lommix_light::raymarch::{raymarch}

// the cascade0 texture
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> size: ComputedSize;


@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{
	var out : vec4<f32>;
	out = sampleRadianceField(source_tex, source_sampler, in.uv);
	return out;
}

fn sampleRadianceField(radianceField: texture_2d<f32>, s: sampler, uv: vec2<f32>) -> vec4<f32> {
    // Get the size of the texture
    let size = vec2<f32>(textureDimensions(radianceField));
    // Calculate the texel coordinates
    let texelCoords = uv * size;

    // Get the integer part (top-left corner of the texel's 2x2 block)
    let i0 = floor(texelCoords);
    let i1 = i0 + vec2<f32>(1.0, 0.0);
    let i2 = i0 + vec2<f32>(0.0, 1.0);
    let i3 = i0 + vec2<f32>(1.0, 1.0);

	var sum : vec4<f32>;
    sum += textureSample(radianceField, s, i0 / size);
	sum += textureSample(radianceField, s, i1 / size);
    sum += textureSample(radianceField, s, i2 / size);
    sum += textureSample(radianceField, s, i3 / size);

    return sum/4.;
}
