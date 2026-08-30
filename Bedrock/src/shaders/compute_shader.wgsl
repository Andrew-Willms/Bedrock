override SIMULATION_WIDTH: f32 = 10.0; // in meters
override SIMULATION_HEIGHT: f32 = 10.0; // in meters

const BOUNCE_EFFICIENCY: f32 = 0.95;



struct Particle {
	mass: f32,
	temperature: f32,
	position: vec2<f32>,
	velocity: vec2<f32>,
	neighbors: array<u32, 8>
};

struct SimulationParams {
	delta_time: f32
};

fn t_impact(distance_to_wall: f32, initial_velocity: f32, acceleration: f32) -> f32 {

	// Initial_velocity is the velocity towards the wall.
	// Acceleration is the acceleration towards the wall.

	/*
		distance_to_wall = (initial_velocity * delta_t) + (0.5 * acceleration * delta_t ^ 2)
		distance_to_wall = (initial_velocity + (0.5 * acceleration)) * (delta_t + delta_t ^ 2)
		distance_to_wall / (initial_velocity + (0.5 * acceleration)) = delta_t * (1 + delta_t)

		When delta_t is zero distance_to_wall is also zero. This solution is trivial.
		The interesting solution occurs when (1 + delta_t) = 0.

		distance_to_wall / (initial_velocity + (0.5 * acceleration)) = 1 + delta_t
		distance_to_wall / (initial_velocity + (0.5 * acceleration)) - 1 = delta_t

		This gives us an equation for the delta_t at which the particle has traveled a certain distance.
		In particular, this can give us the time it takes for a particle with known initial_velocity and acceleration to
		reach a barrier off which it must bounce.
	*/

	return distance_to_wall / (initial_velocity + (0.5 * acceleration)) - 1;
}



@group(0) @binding(0)
var<storage, read> particles_source: array<Particle>;

@group(0) @binding(1)
var<storage, read_write> particles_destination: array<Particle>;

@group(0) @binding(2)
var<uniform> params: SimulationParams;

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {

	//// Initialization ////////////////////////////////////////////////////////////////////////////////////////////////

	let index = id.x;

	if (index >= arrayLength(&particles_source)) {
		return;
	}

	var particle: Particle = particles_source[index];

	//// First pass position update ////////////////////////////////////////////////////////////////////////////////////

	var force: vec2<f32> = vec2<f32>(0.0, -9.81 * particle.mass);
	let acceleration: vec2<f32> = force / particle.mass;

	var unchecked_velocity = particle.velocity + acceleration * params.delta_time;
	var unchecked_position =
		particle.position +
		particle.velocity * params.delta_time +
		0.5 * acceleration * params.delta_time * params.delta_time;
	
	//// Bounce of edges ///////////////////////////////////////////////////////////////////////////////////////////////

	if (unchecked_position.x < 0) {

		let t_impact: f32 = t_impact(particle.position.x, -particle.velocity.x, -acceleration.x);
		let t_remaining: f32 = params.delta_time - t_impact;

		let x_velocity_post_impact: f32 = -BOUNCE_EFFICIENCY * particle.velocity.x + acceleration.x * t_impact;

		unchecked_velocity.x = x_velocity_post_impact + acceleration.x * t_remaining;
		unchecked_position.x = x_velocity_post_impact * t_remaining + 0.5 * acceleration.x * t_remaining * t_remaining;

		// If the the particle is outside of bounds again assume it's making many very small bounces and approximate this to having settled.
		if (unchecked_position.x < 0) {
			unchecked_velocity.x = 0;
			unchecked_position.x = 0;
		}
	}

	if (unchecked_position.x > SIMULATION_WIDTH) {

		let t_impact: f32 = t_impact(particle.position.x - SIMULATION_WIDTH, particle.velocity.x, acceleration.x);
        let t_remaining: f32 = params.delta_time - t_impact;

        let x_velocity_post_impact: f32 = -BOUNCE_EFFICIENCY * particle.velocity.x + acceleration.x * t_impact;

        unchecked_velocity.x = x_velocity_post_impact + acceleration.x * t_remaining;
        unchecked_position.x = x_velocity_post_impact * t_remaining + 0.5 * acceleration.x * t_remaining * t_remaining;

        // If the the particle is outside of bounds again assume it's making many very small bounces and approximate this to having settled.
        if (unchecked_position.x > SIMULATION_WIDTH) {
            unchecked_velocity.x = SIMULATION_WIDTH;
            unchecked_position.x = SIMULATION_WIDTH;
        }
	}

	if (unchecked_position.y < 0) {

		let t_impact: f32 = t_impact(particle.position.y, -particle.velocity.y, -acceleration.y);
		let t_remaining: f32 = params.delta_time - t_impact;

		let y_velocity_post_impact: f32 = -BOUNCE_EFFICIENCY * particle.velocity.y + acceleration.y * t_impact;

		unchecked_velocity.y = y_velocity_post_impact + acceleration.y * t_remaining;
		unchecked_position.y = y_velocity_post_impact * t_remaining + 0.5 * acceleration.y * t_remaining * t_remaining;

		// If the the particle is outside of bounds again assume it's making many very small bounces and approximate this to having settled.
		if (unchecked_position.y < 0) {
			unchecked_velocity.y = 0;
			unchecked_position.y = 0;
		}
	}


	if (unchecked_position.y > SIMULATION_HEIGHT) {

		let t_impact: f32 = t_impact(particle.position.y - SIMULATION_HEIGHT, particle.velocity.y, acceleration.y);
        let t_remaining: f32 = params.delta_time - t_impact;

        let y_velocity_post_impact: f32 = -BOUNCE_EFFICIENCY * particle.velocity.y + acceleration.y * t_impact;

        unchecked_velocity.y = y_velocity_post_impact + acceleration.y * t_remaining;
        unchecked_position.y = y_velocity_post_impact * t_remaining + 0.5 * acceleration.y * t_remaining * t_remaining;

        // If the the particle is outside of bounds again assume it's making many very small bounces and approximate this to having settled.
        if (unchecked_position.y > SIMULATION_HEIGHT) {
            unchecked_velocity.y = SIMULATION_HEIGHT;
            unchecked_position.y = SIMULATION_HEIGHT;
        }
	}

	particle.position = unchecked_position;
	particle.velocity = unchecked_velocity;


	//// Calculate forces //////////////////////////////////////////////////////////////////////////////////////////////

	// Repulsion force between particles.
//	for (var i: i32 = 0; i < 8; i = i + 1) {
//
//		let neighbor_a_index: u32 = particle.neighbors[i] >> 16u;
//		let neighbor_b_index: u32 = particle.neighbors[i] & 0x0000FFFFu;
//
//		let neighbor_a: Particle = particles_source[neighbor_a_index];
//		let neighbor_b: Particle = particles_source[neighbor_b_index];
//
//		let vector_a: vec2<f32> = particle.position - neighbor_a.position;
//		let vector_b: vec2<f32> = particle.position - neighbor_b.position;
//
//		force += 0.001 / vector_a;
//		force += 0.001 / vector_b;
//	}

	particles_destination[index] = particle;
}