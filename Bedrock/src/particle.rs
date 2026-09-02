//use web_sys::console;



//type ParticleAddress = u16;
type PackedParticleAddress = u32;

const PARTICLE_INDEX_PACKING_BIT_SHIFT: usize = 16;
const MAX_PARTICLE_INDEX: usize = u16::MAX as usize;
const MAX_PARTICLE_COUNT: usize = MAX_PARTICLE_INDEX + 1;

const NEIGHBOR_ARRAY_SIZE_U32: usize = 8;
const NEIGHBOR_ARRAY_SIZE: usize = NEIGHBOR_ARRAY_SIZE_U32 * 2;



#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Particle {
	pub(crate) mass: f32,
	pub(crate) temperature: f32,
	pub(crate) position: [f32; 2],
	pub(crate) velocity: [f32; 2],
	pub(crate) neighbors: [PackedParticleAddress; NEIGHBOR_ARRAY_SIZE_U32] // each element stores 2 16-bit indices
}

impl Particle {
	
	pub(crate) fn distance_squared(&self, other: &Particle) -> f32 {
		let delta_x = self.position[0] - other.position[0];
		let delta_y = self.position[1] - other.position[1];
		return delta_x * delta_x + delta_y * delta_y;
	}
	
}



#[derive(Clone, Copy)]
struct NeighborInfo {
	index: usize,
	squared_distance: f32
}

impl NeighborInfo {

	pub fn new(index: usize, squared_distance: f32) -> Self {
		return Self { index, squared_distance };
	}

}



pub(crate) fn set_particle_neighbors(particles: &mut Vec<Particle>) {

	assert!(
		particles.len() >= NEIGHBOR_ARRAY_SIZE + 1,
		"There must be at least {} particles (the particle itself and {} neighbors).",
		NEIGHBOR_ARRAY_SIZE + 1,
		NEIGHBOR_ARRAY_SIZE
	);

	assert!(
		particles.len() <= MAX_PARTICLE_COUNT,
		"There must be at most {} particles.",
		MAX_PARTICLE_COUNT
	);
	
	//console::log_1(&"start neighbor finding".into());
	
	for i in 0..particles.len() {
		
		let particle = &particles[i];
		let mut neighbors = [NeighborInfo::new(0, f32::INFINITY); NEIGHBOR_ARRAY_SIZE];
		let mut farthest_neighbor_index: usize = 0;
		
		for j in 0..particles.len() {
			
			if i == j {
				continue;
			}
			
			let distance2_to_current = particle.distance_squared(&particles[j]);

			// Use <= so that the dummy values are replaced even if all other particles are infinite distance away.
			if distance2_to_current <= neighbors[farthest_neighbor_index].squared_distance {
				neighbors[farthest_neighbor_index] = NeighborInfo::new(j, distance2_to_current);
				farthest_neighbor_index = get_farthest_neighbor_index(&neighbors);
			}
		}
		
		let particle = &mut particles[i]; // upgrade to mutable reference
		
		for k in 0..particle.neighbors.len() {
			
			let neighbor_a = &neighbors[k * 2];
			let neighbor_b = &neighbors[(k * 2) + 1];
			
			assert!(neighbor_a.index <= MAX_PARTICLE_INDEX, "A particle index must be less than {}.", MAX_PARTICLE_INDEX);
			assert!(neighbor_b.index <= MAX_PARTICLE_INDEX, "A particle index must be less than {}.", MAX_PARTICLE_INDEX);
			
			let packed_indices =
				((neighbor_a.index as PackedParticleAddress) << PARTICLE_INDEX_PACKING_BIT_SHIFT)
					| (neighbor_b.index as PackedParticleAddress);

			particle.neighbors[k] = packed_indices;
		}

		//console::log_1(&format!("{:#?}", particle).into());
	}
	
	//console::log_1(&"neighbor finding complete".into());
}

fn get_farthest_neighbor_index(neighbors: &[NeighborInfo; NEIGHBOR_ARRAY_SIZE]) -> usize {

	let mut farthest_neighbor_index = 0;
	let mut farthest_neighbor_distance2 = f32::NEG_INFINITY;

	for i in 0..neighbors.len() {

		if neighbors[i].squared_distance > farthest_neighbor_distance2 {
			farthest_neighbor_index = i;
			farthest_neighbor_distance2 = neighbors[i].squared_distance;
		}
	}
	
	return farthest_neighbor_index;
}