override SIMULATION_WIDTH: f32 = 10.0; // in meters
override SIMULATION_HEIGHT: f32 = 10.0; // in meters

const HALF_NEIGHBOR_COUNT: u32 = 8;
const HALF_NEIGHBOR_COUNT_RECIPROCAL: f32 = 0.125; // optimization for "i / HALF_NEIGHBOR_COUNT"
const BOUNCE_EFFICIENCY: f32 = 0.95;
const F32_MIN_FINITE_VALUE: f32 = -3.402823466e+38f; // wgsl has no NaN literals



struct Particle {
	mass: f32,
	temperature: f32,
	position: vec2<f32>,
	velocity: vec2<f32>,
	neighbors: array<u32, HALF_NEIGHBOR_COUNT>
};

struct SimulationParams {
	delta_time: f32
};



fn t_impact(d_initial: f32, v_initial: f32, acceleration: f32) -> f32 {

	// Handle <= 0 in a single if branch to hopefully improve performance.
	if (d_initial <= 0.0) {
		return select(0.0, F32_MIN_FINITE_VALUE, d_initial < 0);
	}

	// Not a quadratic.
	if (acceleration == 0.0) {

		if (v_initial == 0.0) {
			return F32_MIN_FINITE_VALUE;
		}

		let t_impact: f32 = -d_initial / v_initial;
		return select(F32_MIN_FINITE_VALUE, t_impact, t_impact > 0.0);
	}

	// 2.0 is used here instead of 4.0 because the the kinematic equation is d(t) = d_i + v_i*t + 0.5*a*d^2
	// Since acceleration and d_initial are non-zero at this point the discriminant is also non-zero.
	let discriminant: f32 = v_initial * v_initial - 2.0 * acceleration * d_initial;

	// No real roots.
	if (discriminant < 0.0) {
		return F32_MIN_FINITE_VALUE;
	}

	let sqrt_discriminant: f32 = sqrt(discriminant);

	// Since the discriminant is non-zero q is also non-zero.
	let q: f32 = -0.5 * (v_initial + select(-sqrt_discriminant, sqrt_discriminant, v_initial >= 0.0));

	let t_impact_1 = 2.0 * q / acceleration;
	let t_impact_2 = d_initial / q;

	if (t_impact_1 > 0.0 && t_impact_2 > 0.0) {
		return min(t_impact_1, t_impact_2);
	}

	if (t_impact_1 > 0.0) {
		return t_impact_1;
	}

	if (t_impact_2 > 0.0) {
		return t_impact_2;
	}

	return F32_MIN_FINITE_VALUE;
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

	//// Calculate acceleration ////////////////////////////////////////////////////////////////////////////////////////

	// Set the initial force on the particle to be the force of gravity if it is not on the floor.
	var force: vec2<f32> = select(
		vec2<f32>(0.0, -9.81 * particle.mass),
		vec2<f32>(0.0, 0.0),
		particle.position.y == 0.0 && particle.velocity.y == 0.0
	);

	// Repulsion force between particles.
	for (var i: u32 = 0; i < HALF_NEIGHBOR_COUNT; i = i + 1) {

		let neighbor_a_index: u32 = particle.neighbors[i] >> 16u;
		let neighbor_b_index: u32 = particle.neighbors[i] & 0x0000FFFFu;

		let neighbor_a: Particle = particles_source[neighbor_a_index];
		let neighbor_b: Particle = particles_source[neighbor_b_index];

		let neighbor_a_delta = particle.position - neighbor_a.position;
		let neighbor_b_delta = particle.position - neighbor_b.position;

		let neighbor_a_distance = length(neighbor_a_delta);
		let neighbor_b_distance = length(neighbor_b_delta);

		let neighbor_a_force_magnitude: f32 = 0.25;
		let neighbor_b_force_magnitude: f32 = 0.25;

		// This is an explanation of the following select statements.
		// If the neighbor distance is non-zero, normalize the delta and multiply it by the force magnitude.
		// If the neighbor distance is zero, invent a vector and multiply it by the force magnitude.
		// The vector must be non-zero it will create a non-finite force.
		// It is desirable that the vector has approximately unit magnitude since the equation expects a direction vector.
		// It is desirable that the vector be unique so that the two overlapping particles don't move in the same direction
		// and recreate the problem in the next frame. The chosen formula is unique as long as the two particles aren't
		// in the same place in eachother's neighbor array.
		// Finally, it is desirable that computing the vector be no slower than computing the default case.

		force += select(
			(neighbor_a_delta / neighbor_a_distance) * neighbor_a_force_magnitude,
			vec2<f32>(0.5, f32(i) * HALF_NEIGHBOR_COUNT_RECIPROCAL) * neighbor_a_force_magnitude,
			neighbor_a_distance == 0);

		force += select(
            (neighbor_b_delta / neighbor_b_distance) * neighbor_b_force_magnitude,
            vec2<f32>(-0.5, f32(i) * HALF_NEIGHBOR_COUNT_RECIPROCAL) * neighbor_b_force_magnitude,
            neighbor_b_distance == 0);
	}

	// TODO: update neighbors

//	if (force.y >= 0) {
//		particles_destination[index].position = particle.position;
//		particles_destination[index].velocity = vec2<f32>(0.0, 0.0);
//		force = vec2<f32>(0.0, 0.0);
//		return;
//	}

	let acceleration: vec2<f32> = force / particle.mass;

	//// First pass position update ////////////////////////////////////////////////////////////////////////////////////

	var unchecked_velocity = particle.velocity + acceleration * params.delta_time;
	var unchecked_position =
		particle.position +
		particle.velocity * params.delta_time +
		0.5 * acceleration * params.delta_time * params.delta_time;
	
	//// Bounce of edges ///////////////////////////////////////////////////////////////////////////////////////////////

	// Check left wall
	if (unchecked_position.x < 0) {

		let t_impact: f32 = t_impact(particle.position.x, particle.velocity.x, acceleration.x);
		let t_remaining: f32 = params.delta_time - t_impact;

		// Error state
		if (t_impact == F32_MIN_FINITE_VALUE || t_remaining < 0) {
			particles_destination[index].position = vec2<f32>(5.0, 8.0);
			particles_destination[index].velocity = vec2<f32>(0.0, 0.0);
			return;
		}

		let x_velocity_post_impact: f32 = -BOUNCE_EFFICIENCY * (particle.velocity.x + acceleration.x * t_impact);

		unchecked_velocity.x = x_velocity_post_impact + acceleration.x * t_remaining;
		unchecked_position.x = x_velocity_post_impact * t_remaining + 0.5 * acceleration.x * t_remaining * t_remaining;

		// If the the particle is outside of bounds again assume it's making many very small bounces and approximate this to having settled.
		if (unchecked_position.x < 0) {
			unchecked_position.x = 0;
			unchecked_velocity.x = 0;
		}

	// Check right wall
	} else if (unchecked_position.x > SIMULATION_WIDTH) {

		let t_impact: f32 = t_impact(SIMULATION_WIDTH - particle.position.x, -particle.velocity.x, -acceleration.x);
		let t_remaining: f32 = params.delta_time - t_impact;

		// Error state
		if (t_impact == F32_MIN_FINITE_VALUE || t_remaining < 0) {
			particles_destination[index].position = vec2<f32>(5.0, 8.0);
			particles_destination[index].velocity = vec2<f32>(0.0, 0.0);
			return;
		}

		let x_velocity_post_impact: f32 = -BOUNCE_EFFICIENCY * (particle.velocity.x + acceleration.x * t_impact);

		unchecked_velocity.x = x_velocity_post_impact + acceleration.x * t_remaining;
		unchecked_position.x = SIMULATION_WIDTH + x_velocity_post_impact * t_remaining + 0.5 * acceleration.x * t_remaining * t_remaining;

		// If the the particle is outside of bounds again assume it's making many very small bounces and approximate this to having settled.
		if (unchecked_position.x > SIMULATION_WIDTH) {
			unchecked_position.x = SIMULATION_WIDTH;
			unchecked_velocity.x = 0;
		}
	}

	// Check floor
	if (unchecked_position.y < 0) {

		let t_impact: f32 = t_impact(particle.position.y, particle.velocity.y, acceleration.y);
		let t_remaining: f32 = params.delta_time - t_impact;

		// Error state
		if (t_impact == F32_MIN_FINITE_VALUE || t_remaining < 0) {
			particles_destination[index].position = vec2<f32>(5.0, 8.0);
			particles_destination[index].velocity = vec2<f32>(0.0, 0.0);
			return;
		}

		let y_velocity_post_impact: f32 = -BOUNCE_EFFICIENCY * (particle.velocity.y + acceleration.y * t_impact);

		unchecked_velocity.y = y_velocity_post_impact + acceleration.y * t_remaining;
		unchecked_position.y = y_velocity_post_impact * t_remaining + 0.5 * acceleration.y * t_remaining * t_remaining;

		// If the the particle is outside of bounds again assume it's making many very small bounces and approximate this to having settled.
		if (unchecked_position.y < 0) {
			unchecked_position.y = 0;
			unchecked_velocity.y = 0;
		}

	// Check ceiling
	} else if (unchecked_position.y > SIMULATION_HEIGHT) {

		let t_impact: f32 = t_impact(SIMULATION_HEIGHT - particle.position.y, -particle.velocity.y, -acceleration.y);
		let t_remaining: f32 = params.delta_time - t_impact;

		let y_velocity_post_impact: f32 = -BOUNCE_EFFICIENCY * (particle.velocity.y + acceleration.y * t_impact);

		// Error state
		if (t_impact == F32_MIN_FINITE_VALUE || t_remaining < 0) {
			particles_destination[index].position = vec2<f32>(5.0, 8.0);
			particles_destination[index].velocity = vec2<f32>(0.0, 0.0);
			return;
		}

		unchecked_velocity.y = y_velocity_post_impact + acceleration.y * t_remaining;
		unchecked_position.y = SIMULATION_HEIGHT + y_velocity_post_impact * t_remaining + 0.5 * acceleration.y * t_remaining * t_remaining;

		// If the the particle is outside of bounds again assume it's making many very small bounces and approximate this to having settled.
		if (unchecked_position.y > SIMULATION_HEIGHT) {
			unchecked_position.y = SIMULATION_HEIGHT;
			unchecked_velocity.y = 0;
		}
	}

	particle.position = unchecked_position;
	particle.velocity = unchecked_velocity;


	//// Update particle position //////////////////////////////////////////////////////////////////////////////////////W

	particles_destination[index] = particle;
}