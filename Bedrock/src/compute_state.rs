use bytemuck::Zeroable;
use wasm_bindgen::JsValue;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, Buffer, BufferUsages, Device};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::particle::Particle;
use crate::simulation_parameters::SimulationParameters;



pub(crate) struct ComputeState {
	buffers: [Buffer; 2],
	bind_groups: [BindGroup; 2],
	current: usize,
	pub(crate) simulation_parameter_buffer: Buffer,
}



impl ComputeState {
	
	pub(crate) fn current_particle_buffer(&self) -> &Buffer {
		&self.buffers[self.current]
	}
	
	pub(crate) fn current_bind_group(&self) -> &BindGroup {
		&self.bind_groups[self.current]
	}
	
	pub(crate) fn swap_particle_buffer_and_bind_group(&mut self) {
		self.current = 1 - self.current;
	}
	
}

pub(crate) fn initialize_compute_state(
	device: &Device, particle_count: usize, compute_bind_group_layout: &BindGroupLayout)
	-> Result<ComputeState, JsValue> {
	
	let buffer_a = create_populated_particle_buffers(&device, particle_count);
	let buffer_b = create_empty_particle_buffers(&device, particle_count);
	let simulation_parameter_buffer = create_simulation_parameter_buffer(&device);
	
	return Ok(
		ComputeState {
			bind_groups: [
				device.create_bind_group(
					&BindGroupDescriptor {
						label: Some("Particles A to B"),
						layout: &compute_bind_group_layout,
						entries: &[
							BindGroupEntry {
								binding: 0,
								resource: buffer_a.as_entire_binding(),
							},
							BindGroupEntry {
								binding: 1,
								resource: buffer_b.as_entire_binding(),
							},
							BindGroupEntry {
								binding: 2,
								resource: simulation_parameter_buffer.as_entire_binding(),
							}
						],
					},
				),
				device.create_bind_group(
					&BindGroupDescriptor {
						label: Some("Particles B to A"),
						layout: &compute_bind_group_layout,
						entries: &[
							BindGroupEntry {
								binding: 0,
								resource: buffer_b.as_entire_binding(),
							},
							BindGroupEntry {
								binding: 1,
								resource: buffer_a.as_entire_binding(),
							},
							BindGroupEntry {
								binding: 2,
								resource: simulation_parameter_buffer.as_entire_binding(),
							},
						],
					},
				)
			],
			
			// Buffer must be set after bind_groups because the buffers are moved.
			buffers: [
				buffer_a,
				buffer_b
			],
			
			current: 0,
			simulation_parameter_buffer,
		}
	);
}

fn create_populated_particle_buffers(device: &Device, particle_count: usize) -> Buffer {
	
	let mut particles_a = Vec::with_capacity(particle_count);
	
	for i in 0..particle_count {
		particles_a.push(Particle {
			mass: 0.0,
			temperature: 0.0,
			position: [ 0.0, 0.0 ],
			velocity: random_2d(i as u32, -0.5, 0.5)
		});
	}
	
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
		delta_time: 0.0,
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
fn random_2d(seed: u32, a: f32, b: f32) -> [f32; 2] {
	
	fn hash(mut x: u32) -> u32 {
		x ^= x >> 16;
		x = x.wrapping_mul(0x7FEB_352D);
		x ^= x >> 15;
		x = x.wrapping_mul(0x846C_A68B);
		x ^= x >> 16;
		x
	}
	
	let h1 = hash(seed);
	let h2 = hash(seed.wrapping_add(0x9E37_79B9));
	let range = b - a;
	
	return [
		a + (h1 as f32 / u32::MAX as f32) * range,
		a + (h2 as f32 / u32::MAX as f32) * range,
	];
}