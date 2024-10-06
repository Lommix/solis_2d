#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import lommix_light::common::{ GiConfig, ComputedSize, random, PI, TAU }

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<storage> circle_occluder_buffer: CircleBuffer;
@group(0) @binding(2) var<storage> rect_occluder_buffer: RectBuffer;
@group(0) @binding(3) var<uniform> computed_size: ComputedSize;

struct CircleBuffer {
    count: u32,
    data:  array<Circle>,
}

struct RectBuffer {
    count: u32,
    data:  array<Rect>,
}

struct Circle{
	radius: f32,
	center: vec2<f32>,
	emit: vec3<f32>,
	intensity: f32,
}

struct Rect{
	half_extends: vec2<f32>,
	center: vec2<f32>,
	rotation: f32,
	emit: vec3<f32>,
	intensity: f32,
}

@fragment
fn fragment(in : FullscreenVertexOutput) -> @location(0) vec4<f32>{

	var dist = 1e+10;
	var emit : vec3<f32>;

	let size = vec2<f32>(computed_size.native);
	let frag_pos = vec2(size.x * in.uv.x,  size.y - size.y * in.uv.y);
	let ndc_pos = vec4<f32>((frag_pos.x / size.x) * 2.0 - 1.0,
                            (frag_pos.y / size.y) * 2.0 - 1.0,
                            0.0,
                            1.0);

	let world_position = (view.world_from_clip * ndc_pos ).xy;

	for(var i = 0; i < i32(circle_occluder_buffer.count); i ++ ){
		let circle = circle_occluder_buffer.data[i];
		let world_dist = world_circle(
			circle.center,
			world_position,
			circle.radius
		);
		emit = select(emit, circle.emit * circle.intensity, (dist > world_dist));
		dist = min(dist, world_dist);
	}

	for(var i = 0; i < i32(rect_occluder_buffer.count); i ++ ){
		let rect = rect_occluder_buffer.data[i];
		let world_dist = world_rect(
			world_position - rect.center,
			rect.half_extends,
			rect.rotation,
		);
		emit = select(emit, rect.emit * rect.intensity, (dist > world_dist));
		dist = min(dist, world_dist);
	}

	let scale = f32(computed_size.native.x)/f32(computed_size.scaled.x);
	return vec4(emit, dist / scale);
}

fn world_circle(
	center: vec2<f32>,
	sample: vec2<f32>,
	radius: f32,
) -> f32 {
	return length(center - sample) - radius;
}

fn world_rect(
	center: vec2<f32>,
	half_extends: vec2<f32>,
	angle: f32,
) -> f32{

    let cos_angle = cos(angle);
    let sin_angle = sin(angle);
    let rot_matrix = mat2x2<f32>(
        vec2(cos_angle, sin_angle),
        vec2(-sin_angle, cos_angle),
    );

    let edge_distance = abs(center * rot_matrix) - half_extends;
    let outside = length(max(edge_distance, vec2(0.)));
    let inside = min(max(edge_distance.x, edge_distance.y), 0.);
    return outside + inside;
}
