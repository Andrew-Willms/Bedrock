#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
	pub position: [f32; 2],
	pub velocity: [f32; 2]
}