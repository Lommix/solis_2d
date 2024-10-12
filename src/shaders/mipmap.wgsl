#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import solis_2d::common::{GiConfig,debug_merge0,debug_merge1, debug_voronoi, debug_sdf}

@group(0) @binding(0) var cascade0: texture_2d<f32>;
@group(0) @binding(1) var<uniform> in_cfg: GiConfig;

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{
	var out : vec4<f32>;

	let size = vec2<f32>(textureDimensions(cascade0));
	let frag = in.uv * size;
	let cell = vec2<i32>(frag/f32(in_cfg.probe_base));
	let probe_size = in_cfg.probe_base * in_cfg.probe_base;

	for (var i = 0; i < i32(probe_size); i++){
		let offset = vec2(i % i32(in_cfg.probe_base), i / i32(in_cfg.probe_base));
		out += textureLoad(cascade0, vec2<i32>(cell * i32(in_cfg.probe_base)) + offset, 0);
	}

	return out/f32(probe_size);
}
