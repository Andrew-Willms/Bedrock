override SIMULATION_WIDTH: f32 = 10.0; // in meters
override SIMULATION_HEIGHT: f32 = 10.0; // in meters

const BOUNCE_EFFICIENCY: f32 = 0.95;
const F32_MIN_FINITE_VALUE: f32 = -3.402823466e+38f; // wgsl has no nan literals


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

/// This function returns the time at which a particle contacts a wall.
/// This function may return a negative result if the particle contacted the wall in the past.
/// If the particle will never contact the wall this function returns F32_MIN_FINITE_VALUE.
/// For particles that contact the wall twice, the contact time closest to ___ will be returned <--- figure this out
/// parameters:
///  - d_initial: The distance bewteen the particle to the wall at t = 0. This must be positive.
///  - v_initial: The velocity of the particle at t = 0. Positive velocity is away from the wall.
///  - acceleration: The acceleration of the particle. Positive acceleration is away from the wall.
fn t_impact(d_initial: f32, v_initial: f32, acceleration: f32) -> f32 {

	// This function is based on the assumption the particle has not yet contacted the wall.
	if (d_initial < 0) {
		return F32_MIN_FINITE_VALUE;
	}

	// The particle is already in contact with the wall.
	if (d_initial == 0) {
		return 0;
	}

	// The particle is stationary (and not already touching the wall) and will not impact the wall.
	if (v_initial == 0 && acceleration == 0) {
		return F32_MIN_FINITE_VALUE;
	}

	// This is a valid, linear case.
	if (acceleration == 0) {
		return d_initial / v_initial;
	}

	// The particle is at the vertex, not touching the wall, and will not get any closer to the wall.
	if (v_initial == 0 && acceleration > 0) {
		return F32_MIN_FINITE_VALUE;
	}

	// The particle is at the vertex and is accelerating towards the wall.
	if (v_initial == 0) {

		// Guaranteed non-Nan since acceleration != 0;
		// Guaranteed non-zero since d_initial is a special case handled above.
		return sqrt(d_initial / acceleration);
	}

	/*
		---- Simple form -----------------------------------------------------------------------------------------------

		Consider the abstract following quadratic equation:
			d = t^2

		For this equation, the following ratio exists:
			t_2^2	 d_2
			-----  =  ---
			t_1^2	 d_1

		Using this ratio the following equation for t_2 can be derived.
			t_2 = sqrt( d_2 / d_1 ) * t_1

		---- Full form -------------------------------------------------------------------------------------------------

		Now sonsider the equation
			d = a * (t - t_v)^2 + d_v
		where
		  - d: the distance to the wall
		  - t: the time, 0 being the beginning of the current simulation step
		  - a: the acceration of the particle
		  - t_v: the time at which the particle is at its vertex
		  - d_v: the distance from the particle to the wall when the particle is at its vertex

		For this equation, a similar ratio exists:
			(t_2 - t_v)^2	 (d_2 - d_v) / a	 d_2 - d_v
			-------------  =  ---------------  =  ---------
			(t_1 - t_v)^2	 (d_1 - d_v) / a	 d_1 - d_v

		Using this ratio the following equation for t_2 can be derived.
					   / d_2 - d_v \
			t_2 = sqrt|  ---------  | * (t_1 - t_v) + t_v
					   \ d_1 - d_v /

		If we have values for t_v and d_v, we can plug the following values into the above equation
		  - (t_1, d_1) = (0, d_initial)
			  - The time and the particle's distance to the wall at the start of the simulation time step.
		  - (t_2, d_2) = (t_w, 0)
			  - The time and the particle's distance to the wall when the particle contacts the wall.
		then we can determine the time, t_w, at which the particle hits the wall.

		What's more is that this process allows us to calculate t_w without solving the quadratic formula.

		---- Calculate t_v ---------------------------------------------------------------------------------------------

		It is relatively intuitive to calculate t_v. Since we know both the acceleration of the particle and its
		velocity at the beginning of the simulation time step, we can divide the velocity by the acceleration to
		determine the amount of time required for the particle to accelerate or decelerate to the vertex. This is given
		by the following equation:
			t_v = -v_i / a
			  - where v_i is the velocity of the particle at the beginning of the simulation time step

		This works if the particle has passed its vertex or if it is approaching its vertex. If it has already passed
		its vertex, t_v will be negative (before the start of the current simulation step). If it is approaching its
		vertex, t_v will be positive (after the start of the current simulation step).

		---- Calculate d_v ---------------------------------------------------------------------------------------------

		Once we have calculated t_v it is also simple to calculate d_v. The particle's distance to the wall at the
		vertex is

			d_v = d_i + (v_i * t_v) + (0.5 * a * t_v^2)

		This works in all cases:

		  - At t_0 the particle is traveling and accelerating towards the wall.
			  - d_i: positive (always)
			  - v_i: negative (the particle is traveling towards the wall at t_0)
			  - t_v: negative (the particle has already passed its vertex)
			  - a: negative (the particle is accelerating towards the wall at t_0)
			  - This results in:
				  - The first term is positive. This is always true since the particle never starts behind the wall.
				  - The second term is positive since it is the product of two negative numbers. This term is
				    effectively a nevative time multiplied by a negative velocity. This term being larger results in
				    the particle being farther from the wall at the vertex.
				  - The third term is nevative since it is the product of three negative terms and one positive term.
				    This term being more negative results in the particle being closer to the wall at the vertex.
				    The third term is always smaller than the second term in this case.
				  - The overall result is that d_v is a positive number larger than d_i. This makes sense because, in
				    this case the particle is farthest from the wall at the vertex and has already passed the vertex.

		  - At t_0 the partcile is traveling away from the wall, decelerating towards its vertex, and will reverse
			direction and hit the wall before the end of the simulation time step.
			  - d_i: positive (always)
			  - v_i: positive (the particle is traveling away from the wall at t_0)
			  - t_v: positive (the particle has yet to reach its vertex)
			  - a: negative (the particle is accelerating towards the wall at t_0)
			  - This results in:
				  - The first term is positive. This is always true since the particle never starts behind the wall.
				  - The second term is positive since it is the product of two positive numbers. This term is
					effectively a possitive time multiplied by a possitive velocity. This term being larger results in
					the particle being farther from the wall at the vertex.
				  - The third term is negative since it is the product of three positive terms and one negative term.
				    This term being more negative results in the particle being closer to the wall at the vertex.
				    The third term is always smaller than the second term in this case.
				  - The overall result is that d_v is a positive number larger than d_i. This makes sense, because in
					this case the particle is farthest from the wall at the vertex and is approaching the vertex.

		  - At t_0 the partlce is traveling towards the wall and decelerating, but will hit the wall before it reverses
		    direction.
			  - d_i: positive (always)
			  - v_i: negative (the particle is traveling towards the wall at t_0)
			  - t_v: positive (the particle has yet to reach its vertex)
			  - a: positive (the particle is accelerating away from the wall at t_0)
			  - This results in:
				  - The first term is positive. This is always true since the particle never starts behind the wall.
				  - The second term is negative since it is the product of a positive and negative term. This term is
					effectively a possitive time multiplied by a negative velocity. This term being more negative
					results in the particle being farther past the wall at the vertex.
				  - The third term is positive since it is the product of four positive terms. This term being larger
				    results in the particle being less far past the wall at the vertex. The third term is always smaller
				    than the second term in this case.
				  - The overall result is that d_v is a negative number. This makes sense, because in this case the
				    particle travels past the wall and is on the other side of it at the vertex.
	*/

	// Guaranteed non-zero sicne v_initial = 0 is a special case handled above.
	// Guarnateed non-Nan since acceleration = 0 is a special case handled above.
	let t_vertex: f32 = -v_initial / acceleration;

	// Guaranteed d_vertex != d_initial.
	// Logically, d_vertex = d_initial at the vertex (v_initial = 0) or if the parabola is infinitely flat
	// (v_initial = 0 and acceleration = 0). All of these special cases are handled above.
	// This may be zero but that case is handled immediately below.
	let d_vertex: f32 = d_initial + (v_initial * t_vertex) + (0.5 * acceleration * t_vertex * t_vertex);
	if (d_vertex == 0) {
		return t_vertex;
	}

	// Guaranteed non-zero since d_vertex != 0.
	// Guaranteed non-Nan since d_initial != d_vertex.
	// Guaranteed non-one. For this to be one, d_initial must equal 0, which is a special case taken care of above.
	let descriminant: f32 = -d_vertex / (d_initial - d_vertex);

	// if the descriminant is < 0 the parabolic trajectory does not touch the wall
	if (descriminant < 0) {
		return F32_MIN_FINITE_VALUE;
	}

	// Guaranteed non-Nan since descriminant > 0.
	// Guaranteed non-zero since descriminant != 0, descriminant != 1, t_vertex != 0.
	return sqrt(descriminant) * -t_vertex + t_vertex;
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
		if (t_impact == F32_MIN_FINITE_VALUE) {
			particles_destination[index].position = vec2<f32>(5.0, 8.0);
			particles_destination[index].velocity = vec2<f32>(0.0, 0.0);
			return;
		}

		let x_velocity_post_impact: f32 = -BOUNCE_EFFICIENCY * (particle.velocity.x + acceleration.x * t_impact);

		unchecked_velocity.x = x_velocity_post_impact + acceleration.x * t_remaining;
		unchecked_position.x = x_velocity_post_impact * t_remaining + 0.5 * acceleration.x * t_remaining * t_remaining;

		// If the the particle is outside of bounds again assume it's making many very small bounces and approximate this to having settled.
		if (unchecked_position.x < 0) {
			unchecked_velocity.x = 0;
			unchecked_position.x = 0;
		}
	}

	// Check right wall
	if (unchecked_position.x > SIMULATION_WIDTH) {

		let t_impact: f32 = t_impact(SIMULATION_WIDTH - particle.position.x, -particle.velocity.x, -acceleration.x);
		let t_remaining: f32 = params.delta_time - t_impact;

		// Error state
		if (t_impact == F32_MIN_FINITE_VALUE) {
			particles_destination[index].position = vec2<f32>(5.0, 8.0);
			particles_destination[index].velocity = vec2<f32>(0.0, 0.0);
			return;
		}

		let x_velocity_post_impact: f32 = -BOUNCE_EFFICIENCY * (particle.velocity.x + acceleration.x * t_impact);

		unchecked_velocity.x = x_velocity_post_impact + acceleration.x * t_remaining;
		unchecked_position.x = SIMULATION_WIDTH + x_velocity_post_impact * t_remaining + 0.5 * acceleration.x * t_remaining * t_remaining;

		// If the the particle is outside of bounds again assume it's making many very small bounces and approximate this to having settled.
		if (unchecked_position.x > SIMULATION_WIDTH) {
			unchecked_velocity.x = 0;
			unchecked_position.x = SIMULATION_WIDTH;
		}
	}

	// Check floor
	if (unchecked_position.y < 0) {

		let t_impact: f32 = t_impact(particle.position.y, particle.velocity.y, acceleration.y);
		let t_remaining: f32 = params.delta_time - t_impact;

		// Error state
		if (t_impact == F32_MIN_FINITE_VALUE) {
			particles_destination[index].position = vec2<f32>(5.0, 8.0);
			particles_destination[index].velocity = vec2<f32>(0.0, 0.0);
			return;
		}

		let y_velocity_post_impact: f32 = -BOUNCE_EFFICIENCY * (particle.velocity.y + acceleration.y * t_impact);

		unchecked_velocity.y = y_velocity_post_impact + acceleration.y * t_remaining;
		unchecked_position.y = y_velocity_post_impact * t_remaining + 0.5 * acceleration.y * t_remaining * t_remaining;

		// If the the particle is outside of bounds again assume it's making many very small bounces and approximate this to having settled.
		if (unchecked_position.y < 0) {
			unchecked_velocity.y = 0;
			unchecked_position.y = 0;
		}
	}

	// Check ceiling
	if (unchecked_position.y > SIMULATION_HEIGHT) {

		let t_impact: f32 = t_impact(SIMULATION_HEIGHT - particle.position.y, particle.velocity.y, acceleration.y);
		let t_remaining: f32 = params.delta_time - t_impact;

		let y_velocity_post_impact: f32 = -BOUNCE_EFFICIENCY * (particle.velocity.y + acceleration.y * t_impact);

		// Error state
		if (t_impact == F32_MIN_FINITE_VALUE) {
			particles_destination[index].position = vec2<f32>(5.0, 8.0);
			particles_destination[index].velocity = vec2<f32>(0.0, 0.0);
			return;
		}

		unchecked_velocity.y = y_velocity_post_impact + acceleration.y * t_remaining;
		unchecked_position.y = SIMULATION_HEIGHT + y_velocity_post_impact * t_remaining + 0.5 * acceleration.y * t_remaining * t_remaining;

		// If the the particle is outside of bounds again assume it's making many very small bounces and approximate this to having settled.
		if (unchecked_position.y > SIMULATION_HEIGHT) {
			unchecked_velocity.y = 0;
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