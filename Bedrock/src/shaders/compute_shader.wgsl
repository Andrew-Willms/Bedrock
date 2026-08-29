override SIMULATION_WIDTH: f32 = 10.0; // in meters
override SIMULATION_HEIGHT: f32  = 10.0; // in meters

struct Particle {
    mass: f32,
    temperature: f32,
    position: vec2<f32>,
    velocity: vec2<f32>,
    neighbours: array<u32, 8>
};

struct SimulationParams {
    delta_time: f32
};

@group(0) @binding(0)
var<storage, read> particles_source: array<Particle>;

@group(0) @binding(1)
var<storage, read_write> particles_destination: array<Particle>;

@group(0) @binding(2)
var<uniform> params: SimulationParams;

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {

    let index = id.x;

    if (index >= arrayLength(&particles_source)) {
        return;
    }

    var particle = particles_source[index];
    particle.position += particle.velocity * params.delta_time;

    if (particle.position.x < 0) {
        particle.position.x = -particle.position.x;
        particle.velocity.x = -0.95 * particle.velocity.x;
    }

    if (particle.position.x > SIMULATION_WIDTH) {
        particle.position.x = SIMULATION_WIDTH - (particle.position.x - SIMULATION_WIDTH);
        particle.velocity.x = -0.95 * particle.velocity.x;
    }

    if (particle.position.y < 0) {
        particle.position.y = -particle.position.y;
        particle.velocity.y = -0.95 * particle.velocity.y;
    }

    if (particle.position.y > SIMULATION_HEIGHT) {
        particle.position.y = SIMULATION_HEIGHT - (particle.position.y - SIMULATION_HEIGHT);;
        particle.velocity.y = -0.95 * particle.velocity.y;
    }

    let force: vec2<f32> = vec2<f32>(0.0, -9.81 * particle.mass);
    particle.velocity += force / particle.mass * params.delta_time;

    particles_destination[index] = particle;
}