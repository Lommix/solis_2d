#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import lommix_light::common::{ GiConfig, ComputedSize, random, PI }
#import lommix_light::raymarch::{raymarch}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var<uniform> probe: Probe;


struct Probe {
    width: u32,
    start: f32,
    range: f32,
}

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{
    let base_coord = vec2<u32>(in.position.xy);
    let probe_cell = base_coord * probe.width;
    let ray_count = probe.width * 2;

    var sum = vec4<f32>(0.0);

    for (var y: u32 = 0; y < probe.width; y++) {
        for (var x: u32 = 0; x < probe.width; x++) {
            sum += textureLoad(source_tex, probe_cell + vec2<u32>(x, y), 0);
        }
    }

    return sum / f32(ray_count);
}
