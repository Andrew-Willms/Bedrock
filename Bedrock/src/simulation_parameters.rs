#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SimulationParameters {
	pub(crate) delta_time: f32,
	pub(crate) _padding: [f32; 3],
}