#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<storage> circle_occluder_buffer: CircleBuffer;
@group(0) @binding(2) var<storage> rect_occluder_buffer: RectBuffer;
@group(0) @binding(3) var<uniform> computed_size: ComputedSize;

struct ComputedSize{
	native: vec2<f32>,
	scaled: vec2<f32>,
}

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

	// let aspect = computed_size.native.x / computed_size.native.y;


	// var clip_pos = vec4<f32>(( in.uv * 2.0 - 1.0 ) * vec2(1.,-1.), 0.0, 1.0);
	// clip_pos.z = -1.0; // Set the z component to -1 for perspective projection (near plane)
	//
	// // Transform the clip position to world space using the view_from_clip matrix
	// var world_pos = view.view_from_clip * clip_pos;
	// world_pos /= world_pos.w; // Perspective divide
	//
	// let world_position = world_pos.xy;

	let world_position = view.world_position.xy + ( in.uv - 0.5 ) * computed_size.native * 0.923  * vec2(1.,-1.);

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

	// scale
	let factor = length( computed_size.native/computed_size.scaled );
	return vec4(emit, dist * (1./factor));
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

    let edge_distance = abs(( center ) * rot_matrix) - half_extends;
    let outside = length(max(edge_distance, vec2(0.)));
    let inside = min(max(edge_distance.x, edge_distance.y), 0.);
    return outside + inside;
}


fn world_to_ndc(world_position: vec2<f32>, view_projection: mat4x4<f32>) -> vec2<f32> {
    return (view_projection * vec4<f32>(world_position, 0.0, 1.0)).xy;
}

fn ndc_to_screen(ndc: vec2<f32>, screen_size: vec2<f32>) -> vec2<f32> {
    let screen_position: vec2<f32> = (ndc + 1.0) * 0.5 * screen_size;
    return vec2(screen_position.x, (screen_size.y - screen_position.y));
}

fn world_to_screen(
    world_position: vec2<f32>,
    screen_size: vec2<f32>,
    view_projection: mat4x4<f32>
) -> vec2<f32> {
    return ndc_to_screen(world_to_ndc(world_position, view_projection), screen_size);
}
