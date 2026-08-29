use web_sys::console;



#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
	pub(crate) mass: f32,
	pub(crate) temperature: f32,
	pub(crate) position: [f32; 2],
	pub(crate) velocity: [f32; 2],
	pub(crate) neighbors: [u32; 8] // each element stores 2 16-bit indices
}

impl Particle {
	
	pub(crate) fn distance(&self, other: &Particle) -> f32 {
		let delta_x = self.position[0] - other.position[0];
		let delta_y = self.position[1] - other.position[1];
		(delta_x * delta_x + delta_y * delta_y).sqrt()
	}
	
}

pub(crate) fn set_particle_neighbors(particles: &mut Vec<Particle>) {
	
	assert!(particles.len() >= 16, "There must be at least 16 particles.");
	assert!(particles.len() <= u16::MAX as usize, "There must be at most u16::MAX particles.");
	
	console::log_1(&"start neighbor finding".into());
	
	for i in 0..particles.len() {
		
		let particle = &particles[i];
		let mut neighbors = vec![NeighborInfo::new(0, f32::INFINITY); 16];
		let mut farthest_neighbor = NeighborInfo::new(0, f32::INFINITY);
		
		for j in 0..particles.len() {
			
			if i == j {
				continue;
			}
			
			let distance_to_current = particle.distance(&particles[j]);
			
			if distance_to_current < farthest_neighbor.distance {
				neighbors[farthest_neighbor.index] = NeighborInfo::new(j, distance_to_current);
				farthest_neighbor = get_farthest_neighbor(&neighbors);
			}
		}
		
		let particle = &mut particles[i]; // upgrade to mutable reference
		
		for k in 0..particle.neighbors.len() {
			
			let neighbor_a = &neighbors[k * 2];
			let neighbor_b = &neighbors[(k * 2) + 1];
			
			assert!(neighbor_a.index < u16::MAX as usize, "A particle index must be less than u16::MAX.");
			assert!(neighbor_b.index < u16::MAX as usize, "A particle index must be less than u16::MAX.");
			
			let packed_indices = ((neighbor_a.index as u32) << 16) | (neighbor_b.index as u32);
			particle.neighbors[k] = packed_indices;
		}
	}
	
	console::log_1(&"neighbor finding complete".into());
}



#[derive(Clone, Copy)]
struct NeighborInfo {
	index: usize,
	distance: f32
}

impl NeighborInfo {

	pub fn new(index: usize, distance: f32) -> Self {
		return Self { index, distance };
	}

}

fn get_farthest_neighbor(neighbors: &Vec<NeighborInfo>) -> NeighborInfo {

	let mut farthest_neighbor = NeighborInfo::new(0, f32::NEG_INFINITY);
	
	for neighbor in neighbors {
		
		if neighbor.distance > farthest_neighbor.distance {
			farthest_neighbor = *neighbor;
		}
	}
	
	return farthest_neighbor;
}