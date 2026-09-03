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
	output.position = vec4<f32>(
		LEFT_DISPLAY_EDGE + ((position.x / SIMULATION_WIDTH) * DISPLAY_HORIZONTAL_SPAN),
		BOTTOM_DISPLAY_EDGE + ((position.y / SIMULATION_HEIGHT) * DISPLAY_VERTICAL_SPAN),
		0.0,
		1.0
	);

	return output;
}

@fragment
fn main_neighbor_fragment_shader() -> @location(0) vec4<f32> {
	return vec4<f32>(0.2, 0.7, 1.0, 1.0);
}