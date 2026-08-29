struct Particle {
    mass: f32,
    temperature: f32,
    position: vec2<f32>,
    velocity: vec2<f32>,
};

struct SimulationParams {
    delta_time: f32,
};

@group(0) @binding(0)
var<storage, read> particles_source: array<Particle>;

@group(0) @binding(1)
var<storage, read_write> particles_destinatino: array<Particle>;

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

    if (particle.position.x < -0.98) {
        particle.position.x = -0.98;
        particle.velocity.x = -0.95 * particle.velocity.x;
    }

    if (particle.position.x > 0.98) {
        particle.position.x = 0.98;
        particle.velocity.x = -0.95 * particle.velocity.x;
    }

    if (particle.position.y < -0.98) {
        particle.position.y = -0.98;
        particle.velocity.y = -0.95 * particle.velocity.y;
    }

    if (particle.position.y > 0.98) {
        particle.position.y = 0.98;
        particle.velocity.y = -0.95 * particle.velocity.y;
    }

    particle.velocity.y -= 0.00981;

    particles_destinatino[index] = particle;
}