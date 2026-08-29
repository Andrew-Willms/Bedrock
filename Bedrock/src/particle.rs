#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
	pub(crate) mass: f32,
	pub(crate) temperature: f32,
	pub(crate) position: [f32; 2],
	pub(crate) velocity: [f32; 2],
	pub(crate) neighbours: [u32; 8] // each element stores 2 16-bit indices
}