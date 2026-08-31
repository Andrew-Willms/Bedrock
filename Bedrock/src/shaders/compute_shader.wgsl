override SIMULATION_WIDTH: f32 = 10.0; // in meters
override SIMULATION_HEIGHT: f32 = 10.0; // in meters

const BOUNCE_EFFICIENCY: f32 = 0.95;
const F32_MIN_FINITE_VALUE: f32 = -3.402823466e+38f; // wgsl has no NaN literals



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

	//// First pass position update ////////////////////////////////////////////////////////////////////////////////////

	var force: vec2<f32> = vec2<f32>(0.0, -9.81 * particle.mass);

	// If the particle is on the floor cancel gravity.
	if (particle.position.y == 0 && particle.velocity.y == 0) {
		force = vec2<f32>(0.0, 0);
	}

	let acceleration: vec2<f32> = force / particle.mass;

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