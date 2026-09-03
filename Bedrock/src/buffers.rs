use bytemuck::Zeroable;
use wgpu::{Buffer, BufferUsages, Device};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::particle::{set_particle_neighbors, Particle};
use crate::simulation_parameters::{SimulationParameters, SIMULATION_HEIGHT, SIMULATION_WIDTH, SIMULATION_TIME_STEP};



pub(crate) struct Buffers {
	pub(crate) particles_a: Buffer,
	pub(crate) particles_b: Buffer,
	pub(crate) parameters: Buffer
}

impl Buffers {
	
	pub(crate) fn new(device: &Device, particle_count: usize) -> Buffers {
		let buffer_a = create_populated_particle_buffers(&device, particle_count);
		let buffer_b = create_empty_particle_buffers(&device, particle_count);
		let simulation_parameter_buffer = create_simulation_parameter_buffer(&device);
		
		return Buffers {
			particles_a: buffer_a,
			particles_b: buffer_b,
			parameters: simulation_parameter_buffer,
		};
	}
	
}

fn create_populated_particle_buffers(device: &Device, particle_count: usize) -> Buffer {
	
	let mut particles_a = Vec::with_capacity(particle_count);
	
	for i in 0..particle_count {
		particles_a.push(Particle {
			mass: 1.0,
			temperature: 0.0,
			position: [
				random(i as u32, 0, 0.0, SIMULATION_WIDTH),
				random(i as u32, 1, 0.0, SIMULATION_HEIGHT)
			],
			velocity: [
				random(i as u32, 2, -0.5, 0.5),
				random(i as u32, 3, -0.5, 0.5),
			],
			neighbors: [0; 8]
		});
	}
	
	set_particle_neighbors(&mut particles_a);
	
	return device.create_buffer_init(
		&BufferInitDescriptor {
			label: Some("Particle Buffer A"),
			contents: bytemuck::cast_slice(&particles_a),
			usage: BufferUsages::VERTEX | BufferUsages::STORAGE | BufferUsages::COPY_DST,
		},
	);
}

fn create_empty_particle_buffers(device: &Device, particle_count: usize) -> Buffer {
	
	let particles_b = vec![Particle::zeroed(); particle_count];
	
	return device.create_buffer_init(
		&BufferInitDescriptor {
			label: Some("Particle Buffer B"),
			contents: bytemuck::cast_slice(&particles_b),
			usage: BufferUsages::VERTEX | BufferUsages::STORAGE | BufferUsages::COPY_DST,
		},
	);
}

fn create_simulation_parameter_buffer(device: &Device) -> Buffer {
	
	let simulation_params = SimulationParameters {
		delta_time: SIMULATION_TIME_STEP,
		_padding: [0.0; 3],
	};
	
	return device.create_buffer_init(
		&BufferInitDescriptor {
			label: Some("Simulation Parameters"),
			contents: bytemuck::bytes_of(&simulation_params),
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
		},
	);
}



/// A small hashing algorithm with no dependencies that produces 2-element arrays where each element
/// is an approximately random value between a and b.
#[inline]
fn random(seed: u32, sub_seed: u32, min: f32, max: f32) -> f32 {
	
	fn hash(mut x: u32) -> u32 {
		x ^= x >> 16;
		x = x.wrapping_mul(0x7FEB_352D);
		x ^= x >> 15;
		x = x.wrapping_mul(0x846C_A68B);
		x ^= x >> 16;
		return x;
	}
	
	const STEP: u32 = 0x9E37_79B9;
	let range = max - min;
	let hash_value = hash(seed.wrapping_add(STEP.wrapping_mul(sub_seed)));
	
	return min + (hash_value as f32 / u32::MAX as f32) * range;
}