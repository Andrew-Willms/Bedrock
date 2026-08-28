use wgpu::{Buffer, Device};
use wgpu::util::DeviceExt;



#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
	pub position: [f32; 2],
	pub velocity: [f32; 2]
}



pub(crate) fn create_particle_buffer(device: &Device, particle_count: usize) -> Buffer {
	
	let mut particles = Vec::with_capacity(particle_count);
	
	for i in 0..particle_count {
		particles.push(Particle {
			position: [ 0.0, 0.0 ],
			velocity: random_2d(i as u32, -0.5, 0.5)
		});
	}
	
	return device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Particle Buffer"),
			contents: bytemuck::cast_slice(&particles),
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
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