#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<storage> circle_occluder_buffer: CircleBuffer;
@group(0) @binding(2) var<storage> rect_occluder_buffer: RectBuffer;
@group(0) @binding(3) var<uniform> computed_size: ComputedSize;

struct ComputedSize{
	native: vec2<f32>,
	scaled: vec2<f32>,
	factor: f32,
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
	let aspect_ratio = computed_size.native.y / computed_size.native.x;

	var dist = 1e+10;
	var emit : vec3<f32>;

	let world_position = view.world_position.xy +
	floor(in.uv * computed_size.native - computed_size.native/2. )
	* vec2(1.,-1.)
	;

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
			world_position-rect.center,
			rect.half_extends,
			world_position,
			rect.rotation,
		);
		emit = select(emit, rect.emit * rect.intensity, (dist > world_dist));
		dist = min(dist, world_dist);
	}

	return vec4(emit, dist);
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
	sample: vec2<f32>,
	angle: f32,
) -> f32{

    let cos_angle = cos(angle);
    let sin_angle = sin(angle);
    let rot_matrix = mat2x2<f32>(
        vec2(cos_angle, -sin_angle),
        vec2(sin_angle, cos_angle),
    );

    let edge_distance = abs(center * rot_matrix) - half_extends;
    let outside = length(max(edge_distance, vec2(0.)));
    let inside = min(max(edge_distance.x, edge_distance.y), 0.);
    return outside + inside;
}


fn circle(
	circle: Circle,
	sample: vec2<f32>,
	screen_width: f32,
	aspect: f32,
	view: View
) -> f32 {
	let circle_center_ndc = world_to_ndc(circle.center , view.clip_from_world) * vec2(-1.,1.);
	let circle_center_ndc_offset = (sample * 2. - 1.) + circle_center_ndc;
	let circle_radius_ndc = circle.radius / screen_width;
	return sd_circle(circle_center_ndc_offset, circle_radius_ndc, aspect);
}

fn rect(
	rect: Rect,
	sample: vec2<f32>,
	aspect: f32,
	view: View
) -> f32 {
	let rect_center_ndc = world_to_ndc(rect.center , view.clip_from_world) * vec2(-1.,1.);
	let rect_center_ndc_offset = (sample * 2. - 1.) + rect_center_ndc;
	let rect_extends_ndc = local_to_ndc(rect.half_extends, view.clip_from_world);
	return sd_rect_orianted(
		rect_center_ndc_offset,
		rect_extends_ndc.xy,
		rect.rotation,
		aspect
	);
}


fn sd_circle(point: vec2<f32>, radius: f32, aspect_ratio: f32) -> f32 {
    let normalized_point = point / vec2<f32>(aspect_ratio, 1.0);
	return length(normalized_point) - radius/aspect_ratio;
}


fn sd_rect_orianted(center: vec2<f32>, half_extends: vec2<f32>, rotation: f32, aspect_ratio : f32) -> f32
{
    let cos_angle = cos(rotation);
    let sin_angle = sin(rotation);
    let rot_matrix = mat2x2<f32>(
        vec2(cos_angle, -sin_angle),
        vec2(sin_angle, cos_angle),
    );

    let normalized_center = center / vec2<f32>(aspect_ratio, 1.0);
    let normalized_extends = half_extends / vec2<f32>(aspect_ratio, 1.0);

    let edge_distance = abs(normalized_center * rot_matrix) - normalized_extends;
    let outside = length(max(edge_distance, vec2(0.)));
    let inside = min(max(edge_distance.x, edge_distance.y), 0.);
    return outside + inside;
}

fn local_to_ndc(local_position: vec2<f32>, view_projection: mat4x4<f32>) -> vec2<f32> {
    let scale_rotation_matrix = mat2x2<f32>(
        view_projection[0].xy, // First column (scale and rotation for X)
        view_projection[1].xy  // Second column (scale and rotation for Y)
    );
    let ndc_position = scale_rotation_matrix * local_position;
    return ndc_position;
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


fn sdf_uv_to_world(uv_in: vec2<f32>, inverse_view_proj: mat4x4<f32>, sdf_scale: vec2<f32>) -> vec2<f32> {
    let y = 1.0 - uv_in.y;
    let uv = vec2<f32>(uv_in.x, y);
    let ndc_sdf = (uv * 2.0) - 1.0;
    let ndc = ndc_sdf * sdf_scale;
    return (inverse_view_proj * vec4<f32>(ndc, 0.0, 1.0)).xy;
}
