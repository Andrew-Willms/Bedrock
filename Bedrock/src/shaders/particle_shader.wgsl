override SIMULATION_WIDTH: f32 = 10.0; // in meters
override SIMULATION_HEIGHT: f32  = 10.0; // in meters

const LEFT_DISPLAY_EDGE: f32  = -0.98; // in normalized canvas coordinates
const RIGHT_DISPLAY_EDGE: f32  = 0.98; // in normalized canvas coordinates
const TOP_DISPLAY_EDGE: f32  = 0.98; // in normalized canvas coordinates
const BOTTOM_DISPLAY_EDGE: f32  = -0.98; // in normalized canvas coordinates

const DISPLAY_HORIZONTAL_SPAN: f32 = RIGHT_DISPLAY_EDGE - LEFT_DISPLAY_EDGE;
const DISPLAY_VERTICAL_SPAN: f32 = TOP_DISPLAY_EDGE - BOTTOM_DISPLAY_EDGE;



struct VertexInput {
	@location(0) position: vec2<f32>,
};

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
};



@vertex
fn main_particle_vertex_shader(input: VertexInput) -> VertexOutput {

	var output: VertexOutput;

	output.position = vec4<f32>(
		LEFT_DISPLAY_EDGE + ((input.position.x / SIMULATION_WIDTH) * DISPLAY_HORIZONTAL_SPAN),
		BOTTOM_DISPLAY_EDGE + ((input.position.y / SIMULATION_HEIGHT) * DISPLAY_VERTICAL_SPAN),
		0.0,
		1.0
	);

	return output;
}

@fragment
fn main_particle_fragment_shader() -> @location(0) vec4<f32> {
	return vec4<f32>(1.0, 1.0, 1.0, 1.0); // white
	//return vec4<f32>(0.133, 0.545, 0.133, 1.0); // forest green
}