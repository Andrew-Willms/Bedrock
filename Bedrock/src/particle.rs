use wgpu::{Buffer, Device};
use wgpu::util::DeviceExt;




#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
	pub position: [f32; 2],
}



pub(crate) fn create_particle_buffer(device: &Device, particle_count: usize) -> Buffer {
	
	let mut particles = Vec::with_capacity(particle_count);
	let grid_dimensions = (particle_count as f32).sqrt().ceil() as usize;
	
	for i in 0..particle_count {
		let x = (i % grid_dimensions) as f32;
		let y = (i / grid_dimensions) as f32;
		
		particles.push(Particle {
			position: [
				x / 20.0 - 1.0,
				y / 20.0 - 1.0,
			],
		});
	}
	
	return device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Particle Buffer"),
			contents: bytemuck::cast_slice(&particles),
			usage: wgpu::BufferUsages::VERTEX,
		},
	);
}