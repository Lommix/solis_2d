#define_import_path lommix_light::shapes
#import bevy_render::view::View

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

// ----------------------------------------------
// circle
// ----------------------------------------------

fn circle(
	circle: Circle,
	sample: vec2<f32>,
	screen_width: f32,
	aspect: f32,
	view: View
) -> f32 {
	let circle_center_ndc = world_to_ndc(circle.center , view.view_proj) * vec2(-1.,1.);
	let circle_center_ndc_offset = (sample * 2. - 1.) + circle_center_ndc;
	let circle_radius_ndc = circle.radius / screen_width;
	return sd_circle(circle_center_ndc_offset, circle_radius_ndc, aspect);
}

fn sd_circle(point: vec2<f32>, radius: f32, aspect_ratio: f32) -> f32 {
    let normalized_point = point / vec2<f32>(aspect_ratio, 1.0);
	return length(normalized_point) - radius/aspect_ratio;
}


// ----------------------------------------------
// rect
// ----------------------------------------------

fn rect(
	rect: Rect,
	sample: vec2<f32>,
	aspect: f32,
	view: View
) -> f32 {
	let rect_center_ndc = world_to_ndc(rect.center , view.view_proj) * vec2(-1.,1.);
	let rect_center_ndc_offset = (sample * 2. - 1.) + rect_center_ndc;
	let rect_extends_ndc = local_to_ndc(rect.half_extends, view.view_proj);
	return sd_rect_orianted(
		rect_center_ndc_offset,
		rect_extends_ndc.xy,
		rect.rotation,
		aspect
	);
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


// ----------------------------------------------
// util
// ----------------------------------------------

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
