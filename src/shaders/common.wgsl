#define_import_path lommix_light::common

const PI:f32  = 3.14159;
const TAU:f32 = PI *2.;

struct GiConfig{
	probe_size: f32,
	scale: f32,
	flags: u32,

	cascade_count: u32,
	probe_stride: u32,
}


struct ComputedSize{
	native: vec2<f32>,
	scaled: vec2<f32>,
	//this can be calculated from
	// cascade count
	probe: vec2<f32>,
}

fn debug_voronoi(cfg: GiConfig) -> f32{
	return select(0.,1., (( cfg.flags & 0x1 )!= 0));
}

fn debug_sdf(cfg: GiConfig) -> f32{
	return select(0.,1., (( cfg.flags & 0x2 )!= 0));
}

fn debug_light(cfg: GiConfig) -> f32{
	return select(0.,1., ( cfg.flags & 0x4 )!= 0);
}

fn debug_bounce(cfg: GiConfig) -> f32{
	return select(0.,1., ( cfg.flags & 0x8 )!= 0);
}

fn debug_probe(cfg: GiConfig) -> f32{
	return select(0.,1., ( cfg.flags & 0x10 )!= 0);
}

fn debug_merge(cfg: GiConfig) -> f32{
	return select(0.,1., ( cfg.flags & 0x20 )!= 0);
}

fn debug_final(cfg: GiConfig) -> f32 {
	return select(0.,1., (( cfg.flags >> 5 ) & 0x1 )!= 0);
}

fn random(st : vec2<f32>) -> f32 {
   return fract(sin(dot(st.xy, vec2(12.9898,78.233))) * 43758.5453123);
}

