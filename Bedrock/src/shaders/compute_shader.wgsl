struct Particle {
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

    if (particle.position.x < -1.0 || particle.position.x > 1.0) {
        particle.velocity.x = -particle.velocity.x;
    }

    if (particle.position.y < -1.0 || particle.position.y > 1.0) {
        particle.velocity.y = -particle.velocity.y;
    }

    particles_destinatino[index] = particle;
}