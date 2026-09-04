const HALF_NEIGHBOR_COUNT: u32 = 8; // Cannot be an override because its used as an array size.
const NEIGHBOR_COUNT: u32 = HALF_NEIGHBOR_COUNT * 2;

override SIMULATION_WIDTH: f32 = 10.0; // in meters
override SIMULATION_HEIGHT: f32  = 10.0; // in meters

const LEFT_DISPLAY_EDGE: f32  = -0.98; // in normalized canvas coordinates
const RIGHT_DISPLAY_EDGE: f32  = 0.98; // in normalized canvas coordinates
const TOP_DISPLAY_EDGE: f32  = 0.98; // in normalized canvas coordinates
const BOTTOM_DISPLAY_EDGE: f32  = -0.98; // in normalized canvas coordinates

const DISPLAY_HORIZONTAL_SPAN: f32 = RIGHT_DISPLAY_EDGE - LEFT_DISPLAY_EDGE;
const DISPLAY_VERTICAL_SPAN: f32 = TOP_DISPLAY_EDGE - BOTTOM_DISPLAY_EDGE;



struct Particle {
	mass: f32,
	temperature: f32,
	position: vec2<f32>,
	velocity: vec2<f32>,
	neighbors: array<u32, HALF_NEIGHBOR_COUNT>
};

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) @interpolate(flat) particle_index: u32,
};



@group(0) @binding(0)
var<storage, read> particles: array<Particle>;



@vertex
fn main_neighbor_vertex_shader(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {

	let line_index: u32 = vertex_index / 2u; // 2 verteces per line
	let particle_index: u32 = line_index / NEIGHBOR_COUNT; // NEIGHBOR_COUNT lines per particle

	let neighbor_slot: u32 = line_index % NEIGHBOR_COUNT;
	let particle: Particle = particles[particle_index];

	let neighbor_index: u32 = select(
		particles[particle_index].neighbors[neighbor_slot >> 1] >> 16u,
		particles[particle_index].neighbors[neighbor_slot >> 1] & 0x0000FFFFu,
		neighbor_slot % 2 == 1
	);

	var position: vec2<f32>;

	if (vertex_index % 2u == 0u) {
		position = particle.position;
	} else {
		position = particles[neighbor_index].position;
	}

	var output: VertexOutput;
	output.particle_index = particle_index;
	output.position = vec4<f32>(
		LEFT_DISPLAY_EDGE + ((position.x / SIMULATION_WIDTH) * DISPLAY_HORIZONTAL_SPAN),
		BOTTOM_DISPLAY_EDGE + ((position.y / SIMULATION_HEIGHT) * DISPLAY_VERTICAL_SPAN),
		0.0,
		1.0
	);

	return output;
}

@fragment
fn main_neighbor_fragment_shader(input: VertexOutput) -> @location(0) vec4<f32> {

	const MIN_COLOR: f32 = 0.4;
	const COLOR_RANGE: f32 = 0.6;
	const SEED_STEP: u32 = 0x9E3779B9u;
	const U32_MAX: f32 = 4294967295.0;

	let red_seed: u32 = input.particle_index;
	let green_seed: u32 = input.particle_index + SEED_STEP * 2;
	let blue_seed: u32 = input.particle_index + SEED_STEP * 3;

	var red_hash: u32 = red_seed ^ (red_seed >> 16u);
	red_hash *= 0x7FEB352Du;
    red_hash ^= red_hash >> 15u;
    red_hash *= 0x846CA68Bu;
    red_hash ^= red_hash >> 16u;

	var green_hash: u32 = green_seed ^ (green_seed >> 16u);
	green_hash *= 0x7FEB352Du;
    green_hash ^= green_hash >> 15u;
    green_hash *= 0x846CA68Bu;
    green_hash ^= green_hash >> 16u;

    var blue_hash: u32 = blue_seed ^ (blue_seed >> 16u);
    blue_hash *= 0x7FEB352Du;
    blue_hash ^= blue_hash >> 15u;
    blue_hash *= 0x846CA68Bu;
    blue_hash ^= blue_hash >> 16u;

	let red_value: f32 = MIN_COLOR + (f32(red_hash) / U32_MAX) * COLOR_RANGE;
	let green_value: f32 = MIN_COLOR + (f32(green_hash) / U32_MAX) * COLOR_RANGE;
	let blue_value: f32 = MIN_COLOR + (f32(blue_hash) / U32_MAX) * COLOR_RANGE;

	return vec4<f32>(red_value, green_value, blue_value, 0.5);
	//return vec4<f32>(0.2, 0.7, 1.0, 1.0); // Neon aqua
}