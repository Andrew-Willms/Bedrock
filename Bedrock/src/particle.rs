use web_sys::console;



#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Particle {
	pub(crate) mass: f32,
	pub(crate) temperature: f32,
	pub(crate) position: [f32; 2],
	pub(crate) velocity: [f32; 2],
	pub(crate) neighbors: [u32; 8] // each element stores 2 16-bit indices  // todo: fix magic number
}

impl Particle {
	
	pub(crate) fn distance_squared(&self, other: &Particle) -> f32 {
		let delta_x = self.position[0] - other.position[0];
		let delta_y = self.position[1] - other.position[1];
		return delta_x * delta_x + delta_y * delta_y;
	}
	
}

pub(crate) fn set_particle_neighbors(particles: &mut Vec<Particle>) {

	// todo: fix magic number
	assert!(particles.len() >= 17, "There must be at least 17 particles (the particle itself and 16 neighbors).");
	assert!(particles.len() <= u16::MAX as usize, "There must be at most u16::MAX particles.");
	
	console::log_1(&"start neighbor finding".into());
	
	for i in 0..particles.len() {
		
		let particle = &particles[i];
		let mut neighbors = [NeighborInfo::new(0, f32::INFINITY); 16]; // TODO: fix magic number
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
		
		for k in 0..particle.neighbors.len() { // todo: fix magic number
			
			let neighbor_a = &neighbors[k * 2];
			let neighbor_b = &neighbors[(k * 2) + 1];
			
			assert!(neighbor_a.particle_index < u16::MAX as usize, "A particle index must be less than u16::MAX.");
			assert!(neighbor_b.particle_index < u16::MAX as usize, "A particle index must be less than u16::MAX.");
			
			let packed_indices = ((neighbor_a.particle_index as u32) << 16) | (neighbor_b.particle_index as u32);
			particle.neighbors[k] = packed_indices;
		}

		//console::log_1(&format!("{:#?}", particle).into());
	}
	
	console::log_1(&"neighbor finding complete".into());
}



#[derive(Clone, Copy)]
struct NeighborInfo {
	particle_index: usize,
	squared_distance: f32
}

impl NeighborInfo {

	pub fn new(particle_index: usize, squared_distance: f32) -> Self {
		return Self { particle_index, squared_distance };
	}

}

// TODO: consider switching back to vector so values aren't copied
fn get_farthest_neighbor_index(neighbors: &[NeighborInfo; 16]) -> usize { // TODO: fix magic number

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